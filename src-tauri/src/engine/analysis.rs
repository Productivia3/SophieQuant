//! engine/analysis.rs — enlace con scripts externos (Custom Analysis).
//!
//! El contrato es simple y estable: la app escribe un JSON de entrada con la
//! estrategia y los datos, ejecuta el script que el usuario elija con
//! `python <script> --input <json> --output <json>` y lee el JSON de salida.
//!
//! Decisiones de seguridad (es una app local, pero aun asi):
//!   - se invoca el interprete **sin shell** (Command::new), sin concatenar
//!     cadenas de shell;
//!   - el script lo elige el usuario (selector de ficheros), no la app;
//!   - hay un tiempo limite: si el script se cuelga, se le mata;
//!   - la salida se valida antes de trustarla: solo se leen campos concretos.

use rusqlite::{params as q, Connection};
use serde_json::{json, Value};

use super::ops::StrategyCore;

/// Version del contrato. Si cambia, el contrato deja de ser compatible.
pub const CONTRATO_VERSION: u32 = 1;

/// Tabla donde quedan los analisis personalizados.
pub const SCHEMA: &str = "
create table if not exists custom_analyses (
  id integer primary key autoincrement,
  strategy_id integer not null references strategies(id) on delete cascade,
  script text not null default '',
  verdict text not null default 'pending' check (verdict in ('pending','pass','warn','fail')),
  label text not null default '',
  input jsonb not null default '{}',
  output jsonb not null default '{}',
  created_at integer not null default (unixepoch())
);
create index if not exists idx_custom_analyses on custom_analyses(strategy_id, id desc);
";

/// Prepara el JSON de entrada para el script.
///
/// Incluye la estrategia (regla y parametros), el rango temporal y la ruta del
/// CSV con las barras, para que el script no tenga que reinventar el formato.
pub fn contract(
    core: &StrategyCore,
    csv_path: &std::path::Path,
    n_bars: usize,
) -> Value {
    json!({
        "contrato": "sophiequant.custom-analysis",
        "version": CONTRATO_VERSION,
        "estrategia": {
            "id": core.id,
            "nombre": core.name,
            "regla": core.template.label(),
            "plantilla": format!("{:?}", core.template),
            "params": core.params,
        },
        "datos": {
            "csv": csv_path.to_string_lossy(),
            "formato": "time,open,high,low,close,volume",
            "barras": n_bars,
        },
        "salida_esperada": {
            "veredicto": "pass | warn | fail",
            "etiqueta": "texto corto que resuma el analisis",
            "metricas": { "clave": "valor numerico" },
            "notas": "texto opcional con el razonamiento",
        },
    })
}

/// Normaliza la salida del script: solo campos conocidos y con tipos correctos.
pub fn normalize_output(raw: &Value) -> Value {
    // el veredicto se acepta en cualquier combinacion de mayusculas
    let bruto = raw.get("veredicto").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let verdict = match bruto.as_str() {
        "pass" | "passed" | "ok" | "true" => "pass",
        "warn" | "warning" => "warn",
        "fail" | "failed" | "ko" | "false" => "fail",
        _ => "pending",
    };
    let mut metricas = serde_json::Map::new();
    if let Some(obj) = raw.get("metricas").and_then(|m| m.as_object()) {
        for (k, v) in obj {
            if let Some(n) = v.as_f64() {
                metricas.insert(k.clone(), json!((n * 1000.0).round() / 1000.0));
            }
        }
    }
    json!({
        "veredicto": verdict,
        "etiqueta": raw.get("etiqueta").and_then(|v| v.as_str()).unwrap_or("analisis personalizado").to_string(),
        "metricas": serde_json::Value::Object(metricas),
        "notas": raw.get("notas").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    })
}

/// Guarda un analisis personalizado en la base de datos.
pub fn store(
    db: &Connection,
    strategy_id: i64,
    script: &str,
    out: &Value,
    input: &Value,
) -> Result<i64, String> {
    db.execute(
        "insert into custom_analyses (strategy_id, script, verdict, label, input, output)
         values (?1, ?2, ?3, ?4, ?5, ?6)",
        q![
            strategy_id,
            script,
            out["veredicto"].as_str().unwrap_or("pending"),
            out["etiqueta"].as_str().unwrap_or(""),
            input.to_string(),
            out.to_string(),
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

/// Historial de analisis personalizados de una estrategia (o de todas).
pub fn list(db: &Connection, strategy_id: Option<i64>, limit: usize) -> Result<Vec<Value>, String> {
    let lim = limit.clamp(1, 200) as i64;
    let mut sql = String::from(
        "select id, strategy_id, script, verdict, label, output, created_at
         from custom_analyses where 1=1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(sid) = strategy_id {
        sql.push_str(" and strategy_id = ?");
        args.push(Box::new(sid));
    }
    sql.push_str(" order by id desc limit ?");
    args.push(Box::new(lim));
    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(refs.as_slice(), |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "strategyId": r.get::<_, i64>(1)?,
                "script": r.get::<_, String>(2)?,
                "verdict": r.get::<_, String>(3)?,
                "label": r.get::<_, String>(4)?,
                "output": r.get::<_, Option<String>>(5)?
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or(Value::Null),
                "createdAt": r.get::<_, i64>(6)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_salida_se_normaliza_y_se_descarta_lo_que_no_se_sabe() {
        let raw = json!({
            "veredicto": "PASSED",
            "etiqueta": "el setup aguanta",
            "metricas": { "sharpe": 1.23456, "texto": "no es numero" },
            "notas": "probado en 3 periodos",
            "campo_raro": "se ignora",
        });
        let n = normalize_output(&raw);
        assert_eq!(n["veredicto"], "pass");
        assert_eq!(n["etiqueta"], "el setup aguanta");
        assert_eq!(n["metricas"]["sharpe"], 1.235);
        assert!(n["metricas"].get("texto").is_none(), "solo metricas numericas");
        assert!(n.get("campo_raro").is_none(), "los campos desconocidos no se copian");
    }

    #[test]
    fn un_veredicto_desconocido_queda_pendiente() {
        let n = normalize_output(&json!({ "veredicto": "inventado" }));
        assert_eq!(n["veredicto"], "pending");
    }

    #[test]
    fn el_analisis_se_guarda_y_se_relee() {
        let c = Connection::open_in_memory().unwrap();
        super::super::datastore::init_schema(&c).unwrap();
        c.execute_batch(SCHEMA).unwrap();
        c.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = c.last_insert_rowid();
        c.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, status)
             values ('S1', ?1, 'm1', 'builder', '{}', 'active')",
            rusqlite::params![sym],
        )
        .unwrap();
        let sid = c.last_insert_rowid();

        let out = normalize_output(&json!({ "veredicto": "pass", "etiqueta": "ok" }));
        let id = store(&c, sid, "mi_script.py", &out, &json!({ "contrato": 1 })).unwrap();
        assert!(id > 0);
        let rows = list(&c, Some(sid), 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["verdict"], "pass");
        assert_eq!(rows[0]["script"], "mi_script.py");
    }
}
