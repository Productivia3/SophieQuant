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

/// Aplica el schema una vez (idempotente: `if not exists` en todo).
pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    // Migraciones idempotentes: SQLite no admite "add column if not exists",
    // asi que ignoramos el error cuando la columna ya existe.
    let _ = conn.execute("alter table backtest_runs add column oos_frac real", []);
    let _ = conn.execute("alter table strategies add column robust_score real", []);
    let _ = conn.execute("alter table strategies add column robust_detail jsonb", []);
    Ok(())
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

/// Import CSV: formato `time,open,high,low,close[,volume]` (time = unix epoch o ISO básico).
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
    for (ln, line) in txt.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue }
        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        if parts.len() < 5 { skipped += 1; continue }
        // cabecera habitual (time,open,high,...): se ignora sin contarla como ruido
        if parts[0].eq_ignore_ascii_case("time") { continue }
        let parse_f = |s: &str, what: &str| -> Result<f64, String> {
            s.parse::<f64>().map_err(|_| format!("línea {}: {} inválido ('{s}')", ln + 1, what))
        };
        // timestamp roto = fila individual mala: se descarta, no aborta el import
        let time = match parse_time(parts[0]) {
            Ok(t) => t,
            Err(_) => { skipped += 1; continue }
        };
        let (Ok(open), Ok(high), Ok(low), Ok(close)) = (
            parse_f(parts[1], "open"), parse_f(parts[2], "high"),
            parse_f(parts[3], "low"), parse_f(parts[4], "close"),
        ) else {
            skipped += 1;
            continue;
        };
        let volume = parts.get(5).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
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
        return Err(format!("CSV con solo {} barras útiles ({} descartadas)", bars.len(), skipped));
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
    // se exige fecha completa (y-m-d); la parte de hora permite formatos parciales
    let (y, mo, d) = match (dp[0].parse::<i64>(), dp[1].parse::<i64>(), dp[2].parse::<i64>()) {
        (Ok(y), Ok(mo), Ok(d)) => (y, mo, d),
        _ => return Err(format!("timestamp inválido: '{s}'")),
    };
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
