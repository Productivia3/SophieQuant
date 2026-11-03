//! engine/datastore.rs — persistencia SQLite (WAL) + importador CSV.
//! El schema se aplica con PRAGMA user_version (sin refinery para menos fricción).

use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use super::types::Bar;

pub const SCHEMA: &str = include_str!("../../schema.sql");

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(conn)
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(&format!("pragma table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }
    drop(stmt);
    conn.execute(
        &format!("alter table {table} add column {column} {definition}"),
        [],
    )?;
    Ok(())
}

/// Aplica el schema y migra bases de datos existentes de forma idempotente.
pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    for (table, column, definition) in [
        ("symbols", "contract_size", "real not null default 100000"),
        ("symbols", "swap_long", "real not null default 0"),
        ("symbols", "swap_short", "real not null default 0"),
        ("symbols", "spread_asia", "real"),
        ("symbols", "spread_london", "real"),
        ("symbols", "spread_ny", "real"),
        ("backtest_runs", "oos_frac", "real"),
        ("strategies", "robust_score", "real"),
        ("strategies", "robust_detail", "jsonb"),
    ] {
        ensure_column(conn, table, column, definition)?;
    }
    // tabla de analisis personalizados (Custom Analysis)
    conn.execute_batch(crate::engine::analysis::SCHEMA)?;
    Ok(())
}


/// ── Resample de datasets ────────────────────────────────────────────

/// Minutos por temporalidad (None para tick, que no se resamplea).
pub fn tf_minutes(tf: &str) -> Option<i64> {
    Some(match tf {
        "m1" => 1,
        "m5" => 5,
        "m15" => 15,
        "m30" => 30,
        "h1" => 60,
        "h4" => 240,
        "d1" => 1440,
        "w1" => 10080,
        "mn1" => 43200,
        _ => return None,
    })
}

/// Temporalidades a las que se puede subir desde `from_tf`.
pub fn tf_targets(from_tf: &str) -> Vec<&'static str> {
    let todos = ["m1", "m5", "m15", "m30", "h1", "h4", "d1", "w1", "mn1"];
    let f = match tf_minutes(from_tf) { Some(m) => m, None => return vec![] };
    todos
        .iter()
        .copied()
        .filter(|t| tf_minutes(t).unwrap_or(0) > f)
        .collect()
}

/// Agrega barras a una temporalidad mayor.
///
/// Los cubos van alineados al epoch (UTC), asi que una hora son 60 barras m1
/// exactas. `open` es la primera, `high` el maximo, `low` el minimo, `close` la
/// ultima y `volume` la suma.
pub fn resample(bars: &[Bar], from_tf: &str, to_tf: &str) -> Result<Vec<Bar>, String> {
    let f = tf_minutes(from_tf).ok_or(format!("'{from_tf}' no se puede resamplear"))?;
    let t = tf_minutes(to_tf).ok_or(format!("'{to_tf}' no se puede resamplear"))?;
    if t < f {
        return Err("no se puede bajar de temporalidad: se perderia informacion".into());
    }
    if t == f {
        return Ok(bars.to_vec());
    }
    if bars.is_empty() {
        return Ok(vec![]);
    }
    let secs = t * 60;
    let mut out: Vec<Bar> = Vec::with_capacity(bars.len() / (t / f) as usize + 1);
    let mut cur: Option<Bar> = None;
    for b in bars {
        let bucket = (b.time / secs) * secs;
        match cur.as_mut() {
            Some(c) if c.time == bucket => {
                c.high = c.high.max(b.high);
                c.low = c.low.min(b.low);
                c.close = b.close;
                c.volume += b.volume;
            }
            _ => {
                if let Some(c) = cur.take() {
                    out.push(c);
                }
                cur = Some(Bar {
                    time: bucket,
                    open: b.open,
                    high: b.high,
                    low: b.low,
                    close: b.close,
                    volume: b.volume,
                });
            }
        }
    }
    if let Some(c) = cur {
        out.push(c);
    }
    Ok(out)
}

/// Formato binario propio (.qfn): header "QFN1" + u32 count + [i64 time + 5×f64 OHLCV]
pub struct Qfn;
impl Qfn {
    pub fn write(path: &Path, bars: &[Bar]) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
        f.write_all(b"QFN1")?;
        f.write_all(&(bars.len() as u32).to_le_bytes())?;
        for b in bars {
            f.write_all(&b.time.to_le_bytes())?;
            f.write_all(&b.open.to_le_bytes())?;
            f.write_all(&b.high.to_le_bytes())?;
            f.write_all(&b.low.to_le_bytes())?;
            f.write_all(&b.close.to_le_bytes())?;
            f.write_all(&b.volume.to_le_bytes())?;
        }
        f.flush()
    }

    pub fn read(path: &Path) -> std::io::Result<Vec<Bar>> {
        let data = std::fs::read(path)?;
        if data.len() < 8 || &data[0..4] != b"QFN1" {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "no es un fichero QFN válido"));
        }
        let n = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let mut bars = Vec::with_capacity(n);
        let mut off = 8usize;
        for _ in 0..n {
            if off + 8 + 5 * 8 > data.len() { break }
            let read_f64 = |o: &mut usize| -> f64 {
                let v = f64::from_le_bytes(data[*o..*o + 8].try_into().unwrap());
                *o += 8;
                v
            };
            let time = i64::from_le_bytes(data[off..off + 8].try_into().unwrap());
            off += 8;
            let open = read_f64(&mut off);
            let high = read_f64(&mut off);
            let low = read_f64(&mut off);
            let close = read_f64(&mut off);
            let volume = read_f64(&mut off);
            bars.push(Bar { time, open, high, low, close, volume });
        }
        Ok(bars)
    }
}

/// Importa CSV separados por coma, punto y coma o tab; acepta timestamps simples
/// y columnas de fecha/hora separadas con OHLC identificado por cabecera.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub n_bars: i64,
    pub skipped: i64,
    pub date_from: i64,
    pub date_to: i64,
    /// Diagnostico de calidad del fichero: huecos, timestamps raro, precios imposibles.
    pub avisos: Vec<String>,
    /// Numero de huecos detectados (barras ausentes entre la primera y la ultima).
    pub huecos: i64,
    /// Mayor hueco consecutivo de barras.
    pub hueco_max: i64,
    /// Barras con volumen cero.
    pub sin_volumen: i64,
    /// Filas con OHLC incoherente (close fuera de [low, high] o high < low).
    pub precios_incoherentes: i64,
    /// Timestamps repetidos.
    pub timestamps_repetidos: i64,
}

pub fn import_csv(path: &Path) -> Result<(Vec<Bar>, ImportReport), String> {
    let txt = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut bars: Vec<Bar> = Vec::with_capacity(100_000);
    let mut skipped = 0i64;
    let first_line = txt
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'));
    let delimiter = first_line.map(detect_csv_delimiter).unwrap_or(',');
    let decimal_comma = delimiter == ';';
    let columns = first_line
        .map(|line| csv_fields(line, delimiter))
        .map(|fields| csv_columns(&fields))
        .transpose()?
        .unwrap_or_default();

    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue }
        if columns.header && first_line == Some(line) { continue }
        let parts = csv_fields(line, delimiter);
        if parts.len() <= columns.max_index() { skipped += 1; continue }

        // timestamp roto = fila individual mala: se descarta, no aborta el import
        let timestamp = columns.timestamp(&parts);
        let time = match parse_time(&timestamp) {
            Ok(t) => t,
            Err(_) => { skipped += 1; continue }
        };
        let Some(open) = parse_csv_number(&parts[columns.open], decimal_comma) else {
            skipped += 1;
            continue;
        };
        let Some(high) = parse_csv_number(&parts[columns.high], decimal_comma) else {
            skipped += 1;
            continue;
        };
        let Some(low) = parse_csv_number(&parts[columns.low], decimal_comma) else {
            skipped += 1;
            continue;
        };
        let Some(close) = parse_csv_number(&parts[columns.close], decimal_comma) else {
            skipped += 1;
            continue;
        };
        let volume = columns
            .volume
            .and_then(|index| parse_csv_number(&parts[index], decimal_comma))
            .unwrap_or(0.0);
        if !(high >= low && high >= open && high >= close && low <= open && low <= close) {
            skipped += 1;
            continue;
        }
        bars.push(Bar { time, open, high, low, close, volume });
    }
    // el motor asume orden temporal estricto: un CSV con formatos mezclados
    // (ISO + epoch) puede llegarnos desordenado
    bars.sort_by_key(|b| b.time);
    if bars.len() < 2 {
        let delimiter_name = match delimiter {
            '\t' => "tabulador",
            ';' => "punto y coma",
            _ => "coma",
        };
        return Err(format!(
            "CSV con solo {} barras útiles ({} descartadas). Se detectó separador por {delimiter_name}; se esperan columnas de fecha/hora y OHLC.",
            bars.len(), skipped
        ));
    }
    // ── diagnostico de calidad ───────────────────────────────────────
    // Se comprueba el fichero *antes* de darlo por bueno: huecos, timestamps
    // repetidos, volumen cero y precios incoherentes. No aborta la importacion
    // (los datos son utilizables) pero avisa de lo que puede explicar
    // diferencias entre el backtest y lo que ves en el broker.
    let mut huecos: i64 = 0;
    let mut hueco_max: i64 = 0;
    let mut sin_volumen: i64 = 0;
    let mut precios_incoherentes: i64 = 0;
    let mut timestamps_repetidos: i64 = 0;
    let mut pasos: Vec<i64> = Vec::new();
    for (i, b) in bars.iter().enumerate() {
        if b.volume <= 0.0 { sin_volumen += 1; }
        if b.high < b.low || b.close > b.high || b.close < b.low || b.open > b.high || b.open < b.low {
            precios_incoherentes += 1;
        }
        if i > 0 {
            let dt = b.time - bars[i - 1].time;
            if dt == 0 { timestamps_repetidos += 1; }
            else if dt > 0 {
                // el paso esperado se deduce del primer intervalo util
                let esperado = *pasos.first().unwrap_or(&dt);
                if esperado > 0 && dt > esperado {
                    let faltan = (dt / esperado) - 1;
                    huecos += faltan;
                    if faltan > hueco_max { hueco_max = faltan; }
                }
                if pasos.is_empty() { pasos.push(dt); }
            }
        }
    }
    let mut avisos: Vec<String> = Vec::new();
    if skipped > 0 {
        avisos.push(format!("{skipped} filas descartadas por estar mal formadas o tener datos invalidos"));
    }
    if timestamps_repetidos > 0 {
        avisos.push(format!("{timestamps_repetidos} barras con el mismo timestamp"));
    }
    if precios_incoherentes > 0 {
        avisos.push(format!("{precios_incoherentes} barras con precios incoherentes (close fuera de [low, high])"));
    }
    if hueco_max > 0 {
        avisos.push(format!("huecos de datos: {huecos} barras ausentes (el mayor, {hueco_max} barras)"));
    }
    if sin_volumen > 0 {
        avisos.push(format!("{sin_volumen} barras sin volumen"));
    }
    if avisos.is_empty() {
        avisos.push("sin avisos: el fichero es limpio".to_string());
    }

    let report = ImportReport {
        n_bars: bars.len() as i64,
        skipped,
        avisos,
        huecos,
        hueco_max,
        sin_volumen,
        precios_incoherentes,
        timestamps_repetidos,
        date_from: bars[0].time,
        date_to: bars[bars.len() - 1].time,
    };
    Ok((bars, report))
}

#[derive(Clone, Copy)]
struct CsvColumns {
    header: bool,
    timestamp: Option<usize>,
    date: Option<usize>,
    time: Option<usize>,
    open: usize,
    high: usize,
    low: usize,
    close: usize,
    volume: Option<usize>,
}

impl Default for CsvColumns {
    fn default() -> Self {
        Self {
            header: false,
            timestamp: Some(0),
            date: None,
            time: None,
            open: 1,
            high: 2,
            low: 3,
            close: 4,
            volume: Some(5),
        }
    }
}

impl CsvColumns {
    fn max_index(self) -> usize {
        [
            self.timestamp,
            self.date,
            self.time,
            Some(self.open),
            Some(self.high),
            Some(self.low),
            Some(self.close),
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0)
    }

    fn timestamp(self, fields: &[String]) -> String {
        if let Some(date) = self.date {
            return self.time.map_or_else(
                || fields[date].clone(),
                |time| format!("{} {}", fields[date], fields[time]),
            );
        }
        fields[self.timestamp.expect("CSV timestamp column is configured")].clone()
    }
}

fn detect_csv_delimiter(line: &str) -> char {
    [',', ';', '\t']
        .into_iter()
        .max_by_key(|delimiter| line.matches(*delimiter).count())
        .filter(|delimiter| line.contains(*delimiter))
        .unwrap_or(',')
}

fn csv_fields(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|field| field.trim().trim_matches('"').trim().to_string())
        .collect()
}

fn csv_header_name(field: &str) -> String {
    field
        .trim_start_matches('\u{feff}')
        .trim()
        .trim_matches('"')
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_ascii_lowercase()
}

fn csv_columns(fields: &[String]) -> Result<CsvColumns, String> {
    let names = fields.iter().map(|field| csv_header_name(field)).collect::<Vec<_>>();
    let find = |aliases: &[&str]| names.iter().position(|name| aliases.contains(&name.as_str()));
    let open = find(&["open"]);
    let high = find(&["high"]);
    let low = find(&["low"]);
    let close = find(&["close"]);
    let header_count = [open, high, low, close].into_iter().flatten().count();
    if header_count == 0 {
        if fields.len() >= 6
            && parse_time(fields[0].trim()).is_ok()
            && fields[1].contains(':')
        {
            return Ok(CsvColumns {
                header: false,
                timestamp: None,
                date: Some(0),
                time: Some(1),
                open: 2,
                high: 3,
                low: 4,
                close: 5,
                volume: Some(6).filter(|index| fields.len() > *index),
            });
        }
        return Ok(CsvColumns::default());
    }
    if header_count != 4 {
        return Err("CSV con cabecera detectada: deben existir las columnas open, high, low y close".into());
    }

    let date = find(&["date"]);
    let time = find(&["time"]);
    let timestamp = find(&["datetime", "timestamp", "date_time"])
        .or_else(|| if date.is_none() { time } else { None });
    if date.is_none() && timestamp.is_none() {
        return Err("CSV con cabecera detectada: falta la columna time, datetime o date".into());
    }

    Ok(CsvColumns {
        header: true,
        timestamp,
        date,
        time: if date.is_some() { time } else { None },
        open: open.expect("cabecera open detectada"),
        high: high.expect("cabecera high detectada"),
        low: low.expect("cabecera low detectada"),
        close: close.expect("cabecera close detectada"),
        volume: find(&["volume", "tickvol", "tick_volume", "vol"]),
    })
}

fn parse_csv_number(value: &str, decimal_comma: bool) -> Option<f64> {
    let value = if decimal_comma {
        value.replace(',', ".")
    } else {
        value.to_string()
    };
    value.parse::<f64>().ok()
}

fn parse_time(s: &str) -> Result<i64, String> {
    if let Ok(v) = s.parse::<i64>() {
        // segundos o milisegundos: si es enorme es ms
        return Ok(if v > 4_000_000_000 { v / 1000 } else { v });
    }
    // ISO-like: 2024-01-02 09:00:00 | 2024-01-02T09:00 | 2024.01.02, 09:00 (MT4) | 2024/01/02
    let t = s.replace('T', " ").replace('.', "-").replace('/', "-");
    let (date, time) = match t.split_once(' ') {
        Some((d, h)) => (d, h),
        None => (t.as_str(), "00:00:00"),
    };
    let dp: Vec<&str> = date.split('-').collect();
    let tp: Vec<&str> = time.split(':').collect();
    if dp.len() < 3 || dp.len() > 3 {
        return Err(format!("timestamp inválido: '{s}'"));
    }
    // Se aceptan fechas y-m-d y d-m-y; la hora permite formatos parciales.
    let parsed = (
        dp[0].parse::<i64>(),
        dp[1].parse::<i64>(),
        dp[2].parse::<i64>(),
    );
    let (y, mo, d) = match parsed {
        (Ok(first), Ok(mo), Ok(last)) if first >= 1000 => (first, mo, last),
        (Ok(day), Ok(mo), Ok(year)) if year >= 1000 => (year, mo, day),
        _ => return Err(format!("timestamp inválido: '{s}'")),
    };
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return Err(format!("timestamp inválido: '{s}'"));
    }
    let h = tp.first().and_then(|p| p.parse::<i64>().ok()).unwrap_or(0);
    let mi = tp.get(1).and_then(|p| p.parse::<i64>().ok()).unwrap_or(0);
    let se = tp.get(2).and_then(|p| p.parse::<i64>().ok()).unwrap_or(0);
    // días desde epoch (algoritmo exacto de Hinnant, bisiestos incluidos)
    let days = days_from_civil(y, mo, d);
    Ok(days * 86_400 + h * 3600 + mi * 60 + se)
}

// Howard Hinnant's days_from_civil
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

// ── tests de la tanda de QA bloqueada por falta de toolchain ──────────
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn init_schema_migra_simbolos_anteriores_sin_perder_datos() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "create table symbols (
                id integer primary key,
                name text not null unique,
                asset_class text not null,
                exchange text,
                tick_size real not null default 0.00001,
                pip_size real not null default 0.0001,
                commission real not null default 0
            );
            insert into symbols (id, name, asset_class, tick_size, pip_size, commission)
            values (7, 'EURUSD', 'forex', 0.00001, 0.0001, 2.5);",
        )
        .unwrap();

        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap();

        let (name, commission, contract_size, swap_long, spread_asia): (String, f64, f64, f64, Option<f64>) =
            conn.query_row(
                "select name, commission, contract_size, swap_long, spread_asia
                 from symbols where id = 7",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(name, "EURUSD");
        assert_eq!(commission, 2.5);
        assert_eq!(contract_size, 100_000.0);
        assert_eq!(swap_long, 0.0);
        assert_eq!(spread_asia, None);
    }

    /// Los veredictos se guardan y se releen: es la cache que pinta la tarjeta.
    #[test]
    fn veredictos_se_guardan_y_se_releen() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();
        conn.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, status)
             values ('S1', ?1, 'm1', 'builder', '{}', 'active')",
            rusqlite::params![sym],
        )
        .unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute(
            "insert into strategy_verdicts (strategy_id, check_key, estado, detalle, valor)
             values (?1, 'walkForward', 'pass', 'WFE 62.0 %', 62.0)",
            rusqlite::params![sid],
        )
        .unwrap();
        let (estado, detalle, valor): (String, String, Option<f64>) = conn
            .query_row(
                "select estado, detalle, valor from strategy_verdicts
                 where strategy_id = ?1 and check_key = 'walkForward' order by id desc limit 1",
                rusqlite::params![sid],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(estado, "pass");
        assert_eq!(detalle, "WFE 62.0 %");
        assert_eq!(valor, Some(62.0));
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("qf_test_{}_{name}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }


    /// El resample agrega bien: OHLC correcto, volumen sumado y cubos alineados.
    #[test]
    fn el_resample_agrega_correctamente() {
        // 12 barras de 1 minuto, con inicio alineado al epoch (multiplo de 5 min)
        let base = 1_700_000_100i64;
        let bars: Vec<Bar> = (0..12)
            .map(|i| Bar {
                time: base + i * 60,
                open: 1.0 + i as f64 * 0.01,
                high: 1.5 + i as f64 * 0.01,
                low: 0.5 + i as f64 * 0.01,
                close: 1.2 + i as f64 * 0.01,
                volume: 10.0,
            })
            .collect();
        let m5 = resample(&bars, "m1", "m5").unwrap();
        // 12 barras de 1 min -> 3 cubos de 5 min (5 + 5 + 2)
        assert_eq!(m5.len(), 3, "cubos: {}", m5.len());
        // el primer cubo: open de la primera, close de la ultima, volumen sumado
        assert_eq!(m5[0].open, bars[0].open);
        assert_eq!(m5[0].close, bars[4].close);
        assert_eq!(m5[0].high, bars.iter().take(5).map(|b| b.high).fold(f64::MIN, f64::max));
        assert_eq!(m5[0].low, bars.iter().take(5).map(|b| b.low).fold(f64::MAX, f64::min));
        assert_eq!(m5[0].volume, 50.0);
        // los cubos estan alineados al epoch
        assert_eq!(m5[0].time % (5 * 60), 0);
    }

    /// Bajar de temporalidad se rechaza: se perderia informacion.
    #[test]
    fn no_se_puede_bajar_de_temporalidad() {
        let bars: Vec<Bar> = (0..5)
            .map(|i| Bar {
                time: 1_700_000_000 + i * 3600,
                open: 1.0, high: 1.1, low: 0.9, close: 1.0, volume: 1.0,
            })
            .collect();
        assert!(resample(&bars, "h1", "m1").is_err());
        // y a la misma temporalidad devuelve lo mismo
        let igual = resample(&bars, "h1", "h1").unwrap();
        assert_eq!(igual.len(), bars.len());
    }

    /// La lista de destinos posibles respeta el origen.
    #[test]
    fn los_destinos_de_resample_son_mayores_que_el_origen() {
        assert_eq!(tf_targets("m1"), vec!["m5", "m15", "m30", "h1", "h4", "d1", "w1", "mn1"]);
        assert_eq!(tf_targets("h1"), vec!["h4", "d1", "w1", "mn1"]);
        assert!(tf_targets("tick").is_empty());
    }

    /// CSV con: ruido (filas basura), fila con OHLC inválido, gaps de tiempo,
    /// timestamps mezclados (ISO y ms) — debe descartar lo roto y quedar lo sano.
    #[test]
    fn import_csv_ruido_gaps_y_timestamps_mixtos() {
        let p = temp("import.csv");
        let csv = "\
# comentario
time,open,high,low,close,volume
2024-01-02 09:00:00,1.1000,1.1020,1.0990,1.1015,100
2024-01-02 09:01:00,1.1015,1.1030,1.1010,1.1025,
basura sin comas
2024-01-02 09:02:00,1.1025,1.1000,1.1010,1.1020,50
1704787380000,1.1020,1.1040,1.1015,1.1035,80
2024-01-02 09:04:00,1.1035,1.1045,1.1025,1.1040,60
no-es-numero,1.1,1.1,1.1,1.1,1
";
        std::fs::write(&p, csv).unwrap();
        let (bars, rep) = import_csv(&p).expect("import ok");
        std::fs::remove_file(&p).ok();
        // filas malas: basura (1) + no-es-numero (1) + OHLC inválido (high < open)
        assert_eq!(rep.skipped, 3);
        assert_eq!(bars.len(), 4);
        // 2024-01-02 09:00:00 UTC = 1704186000
        assert_eq!(bars[0].time, 1704186000);
        // tras el sort: [09:00, 09:01, 09:04, epoch-ms]
        assert_eq!(bars[2].time, 1704186240);
        assert_eq!(rep.date_from, 1704186000);
        // la fila en ms queda última al ordenar y define el fin del rango
        assert_eq!(bars[3].time, 1704787380);
        assert_eq!(rep.date_to, 1704787380);
        // gaps de tiempo NO descartan barras (solo fechas raras o inválidas)
        assert_ne!(bars[2].time - bars[1].time, 60, "el gap de minutos existe");
    }

    #[test]
    fn import_csv_acepta_punto_y_coma_decimales_y_fecha_hora_separadas() {
        let p = temp("semicolon.csv");
        let csv = "\
Date;Time;Open;High;Low;Close;Tickvol;Vol;Spread
02.01.2024;09:00:00;1,1000;1,1020;1,0990;1,1015;100;0;2
02.01.2024;09:01:00;1,1015;1,1030;1,1010;1,1025;120;0;2
";
        std::fs::write(&p, csv).unwrap();

        let (bars, report) = import_csv(&p).expect("import semicolon");
        std::fs::remove_file(&p).ok();

        assert_eq!(report.skipped, 0);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].time, 1704186000);
        assert!((bars[0].open - 1.1).abs() < 1e-9);
        assert_eq!(bars[0].volume, 100.0);
    }

    #[test]
    fn import_csv_acepta_tab_y_cabecera_mt5_con_bom() {
        let p = temp("tab.csv");
        let csv = "\
\u{feff}<DATE>\t<TIME>\t<OPEN>\t<HIGH>\t<LOW>\t<CLOSE>\t<TICKVOL>\t<VOL>\t<SPREAD>
2024.01.02\t09:00:00\t1.1000\t1.1020\t1.0990\t1.1015\t100\t0\t2
2024.01.02\t09:01:00\t1.1015\t1.1030\t1.1010\t1.1025\t120\t0\t2
";
        std::fs::write(&p, csv).unwrap();

        let (bars, report) = import_csv(&p).expect("import tab");
        std::fs::remove_file(&p).ok();

        assert_eq!(report.skipped, 0);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].time, 1704186000);
        assert_eq!(bars[1].time, 1704186060);
        assert_eq!(bars[0].volume, 100.0);
    }

    #[test]
    fn import_csv_rechaza_menos_de_2_barras_utiles() {
        let p = temp("tiny.csv");
        std::fs::write(&p, "time,open,high,low,close\n2024-01-02 09:00:00,1.1,1.1,1.1,1.1\n").unwrap();
        let err = import_csv(&p).unwrap_err();
        std::fs::remove_file(&p).ok();
        assert!(err.contains("barras"), "el error menciona barras: {err}");
    }

    #[test]
    fn qfn_roundtrip_byte_a_byte() {
        let p = temp("roundtrip.qfn");
        let bars = vec![
            Bar { time: 1704781200, open: 1.1, high: 1.2, low: 1.05, close: 1.15, volume: 100.0 },
            Bar { time: 1704781260, open: 1.15, high: 1.25, low: 1.10, close: 1.20, volume: 120.0 },
            Bar { time: 1704781320, open: 1.20, high: 1.21, low: 1.14, close: 1.18, volume: 90.0 },
        ];
        Qfn::write(&p, &bars).unwrap();
        let read_back = Qfn::read(&p).expect("lectura QFN");
        std::fs::remove_file(&p).ok();
        assert_eq!(read_back.len(), 3);
        for (a, b) in bars.iter().zip(read_back.iter()) {
            assert_eq!(a.time, b.time);
            assert_eq!(a.open, b.open);
            assert_eq!(a.high, b.high);
            assert_eq!(a.low, b.low);
            assert_eq!(a.close, b.close);
            assert_eq!(a.volume, b.volume);
        }
    }
}
