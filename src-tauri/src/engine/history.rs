//! engine/history.rs — historial de análisis.
//!
//! Cada retesteo, refinamiento, optimización, matriz WF, Monte Carlo, SPP o
//! perfil de optimización queda registrado con su veredicto y sus métricas.
//! Eso permite lo que el producto promete: **comparar el antes y el después**
//! de cambiar una regla, en vez de mirar un estado que se pierde al cerrar.

use rusqlite::{params as q, Connection, OptionalExtension};
use serde_json::{json, Value};

/// Tipos de análisis que se registran.
pub const KINDS: [&str; 7] = [
    "retest",
    "improver",
    "optimizer",
    "wf_matrix",
    "monte_carlo",
    "spp",
    "opt_profile",
];

/// Etiquetas legibles para la interfaz.
pub fn kind_label(kind: &str) -> &'static str {
    match kind {
        "retest" => "Retesteo",
        "improver" => "Refinador",
        "optimizer" => "Optimizador",
        "wf_matrix" => "Matriz WF",
        "monte_carlo" => "Monte Carlo",
        "spp" => "Permutaciones (SPP)",
        "opt_profile" => "Perfil de optimización",
        _ => "Análisis",
    }
}

/// Registra una corrida de análisis.
///
/// `metrics` son las métricas planas que se comparan entre corridas (net, PF,
/// DD...); `params` la configuración usada; `detail` el resultado completo.
pub fn record(
    db: &Connection,
    strategy_id: i64,
    kind: &str,
    verdict: &str,
    label: &str,
    metrics: &Value,
    params: &Value,
    detail: &Value,
) -> Result<i64, String> {
    db.execute(
        "insert into analysis_runs (strategy_id, kind, verdict, label, metrics, params, detail)
         values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        q![
            strategy_id,
            kind,
            verdict,
            label,
            metrics.to_string(),
            params.to_string(),
            detail.to_string(),
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

/// Lista el historial, del más reciente al más antiguo.
///
/// Sin filtros devuelve todo; con `strategy_id` o `kind` se acota.
pub fn list(
    db: &Connection,
    strategy_id: Option<i64>,
    kind: Option<&str>,
    limit: usize,
) -> Result<Vec<Value>, String> {
    let mut sql = String::from(
        "select a.id, a.strategy_id, s.name, a.kind, a.verdict, a.label,
                a.metrics, a.params, a.detail, a.created_at
         from analysis_runs a
         join strategies s on s.id = a.strategy_id
         where 1=1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(sid) = strategy_id {
        sql.push_str(" and a.strategy_id = ?");
        args.push(Box::new(sid));
    }
    if let Some(k) = kind {
        sql.push_str(" and a.kind = ?");
        args.push(Box::new(k.to_string()));
    }
    sql.push_str(" order by a.id desc limit ?");
    args.push(Box::new(limit as i64));

    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(refs.as_slice(), |r| {
            let metrics_raw: Option<String> = r.get(6)?;
            let params_raw: Option<String> = r.get(7)?;
            let detail_raw: Option<String> = r.get(8)?;
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "strategyId": r.get::<_, i64>(1)?,
                "nombre": r.get::<_, String>(2)?,
                "kind": r.get::<_, String>(3)?,
                "kindLabel": kind_label(&r.get::<_, String>(3)?),
                "verdict": r.get::<_, String>(4)?,
                "label": r.get::<_, String>(5)?,
                "metrics": metrics_raw
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or(Value::Null),
                "params": params_raw
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or(Value::Null),
                "detail": detail_raw
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or(Value::Null),
                "createdAt": r.get::<_, i64>(9)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// Borra una entrada del historial.
pub fn delete(db: &Connection, id: i64) -> Result<bool, String> {
    let n = db
        .execute("delete from analysis_runs where id = ?1", q![id])
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

/// Comparación entre dos corridas del mismo tipo.
///
/// Devuelve, para cada métrica presente en ambas, la diferencia (la nueva menos
/// la vieja). Las métricas en las que "más es mejor" se interpretan con su
/// signo; el resto se informa tal cual.
pub fn compare(a: &Value, b: &Value) -> Value {
    let ma = a.get("metrics").and_then(|m| m.as_object());
    let mb = b.get("metrics").and_then(|m| m.as_object());
    let (ma, mb) = match (ma, mb) {
        (Some(x), Some(y)) => (x, y),
        _ => return json!({ "metricas": [] }),
    };
    let mut deltas: Vec<Value> = Vec::new();
    let mut keys: Vec<&String> = ma.keys().collect();
    keys.sort();
    for k in keys {
        let (Some(va), Some(vb)) = (ma.get(k).and_then(|v| v.as_f64()), mb.get(k).and_then(|v| v.as_f64())) else {
            continue;
        };
        deltas.push(json!({
            "metrica": k,
            "antes": vb,
            "despues": va,
            "delta": (va - vb).round_2(),
        }));
    }
    json!({ "metricas": deltas })
}

/// Redondeo a 2 decimales sin depender de una dependencia extra.
trait Round2 {
    fn round_2(self) -> f64;
}
impl Round2 for f64 {
    fn round_2(self) -> f64 {
        (self * 100.0).round() / 100.0
    }
}

/// Corridas de una estrategia y un tipo, de la mas nueva a la mas antigua.
pub fn previous_of(
    db: &Connection,
    strategy_id: i64,
    kind: &str,
    before_id: i64,
) -> Result<Option<Value>, String> {
    let row: Option<String> = db
        .query_row(
            "select metrics from analysis_runs
             where strategy_id = ?1 and kind = ?2 and id < ?3
             order by id desc limit 1",
            q![strategy_id, kind, before_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(row.and_then(|s| serde_json::from_str::<Value>(&s).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&c).unwrap();
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
        c
    }

    #[test]
    fn registra_y_relee_el_historial() {
        let c = db();
        let sid: i64 = c.query_row("select id from strategies", [], |r| r.get(0)).unwrap();
        let id = record(
            &c, sid, "retest", "pass", "EURUSD h1",
            &json!({ "netProfit": 100.0, "profitFactor": 1.4 }),
            &json!({ "simbolo": "EURUSD" }),
            &json!({ "ok": true }),
        )
        .unwrap();
        assert!(id > 0);

        let rows = list(&c, Some(sid), Some("retest"), 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["kind"], "retest");
        assert_eq!(rows[0]["kindLabel"], "Retesteo");
        assert_eq!(rows[0]["verdict"], "pass");
        assert_eq!(rows[0]["metrics"]["profitFactor"], 1.4);
        assert_eq!(rows[0]["params"]["simbolo"], "EURUSD");
    }

    #[test]
    fn el_historial_va_de_mas_reciente_a_mas_antiguo_y_se_filtra() {
        let c = db();
        let sid: i64 = c.query_row("select id from strategies", [], |r| r.get(0)).unwrap();
        for i in 0..3 {
            record(&c, sid, "retest", "pass", &format!("m{i}"),
                &json!({ "netProfit": i as f64 }), &json!({}), &json!({})).unwrap();
        }
        record(&c, sid, "spp", "warn", "spp", &json!({}), &json!({}), &json!({})).unwrap();

        let todos = list(&c, None, None, 10).unwrap();
        assert_eq!(todos.len(), 4);
        // el más reciente primero
        assert_eq!(todos[0]["kind"], "spp");

        let solo_retest = list(&c, Some(sid), Some("retest"), 10).unwrap();
        assert_eq!(solo_retest.len(), 3);
        assert_eq!(solo_retest[0]["label"], "m2");
    }

    #[test]
    fn la_comparacion_muestra_las_diferencias_entre_corridas() {
        let antes = json!({ "metrics": { "netProfit": 100.0, "profitFactor": 1.2 } });
        let despues = json!({ "metrics": { "netProfit": 150.0, "profitFactor": 1.1 } });
        let c = compare(&despues, &antes);
        let ms = c["metricas"].as_array().unwrap();
        assert_eq!(ms.len(), 2);
        let np = ms.iter().find(|m| m["metrica"] == "netProfit").unwrap();
        assert_eq!(np["antes"], 100.0);
        assert_eq!(np["despues"], 150.0);
        assert_eq!(np["delta"], 50.0);
    }


    /// Flujo completo: registrar dos corridas del mismo tipo y ver el delta.
    #[test]
    fn el_delta_entre_dos_corridas_muestra_el_cambio() {
        let c = db();
        let sid: i64 = c.query_row("select id from strategies", [], |r| r.get(0)).unwrap();
        record(&c, sid, "retest", "pass", "EURUSD h1",
            &json!({ "netProfit": 100.0, "profitFactor": 1.2 }), &json!({}), &json!({})).unwrap();
        record(&c, sid, "retest", "warn", "EURUSD h1",
            &json!({ "netProfit": 150.0, "profitFactor": 1.35 }), &json!({}), &json!({})).unwrap();

        let rows = list(&c, Some(sid), Some("retest"), 10).unwrap();
        assert_eq!(rows.len(), 2);
        // la mas reciente primero
        let nueva = &rows[0];
        let anterior = &rows[1];
        let cmp = compare(nueva, anterior);
        let ms = cmp["metricas"].as_array().unwrap();
        let pf = ms.iter().find(|m| m["metrica"] == "profitFactor").unwrap();
        assert_eq!(pf["antes"], 1.2);
        assert_eq!(pf["despues"], 1.35);
        assert!((pf["delta"].as_f64().unwrap() - 0.15).abs() < 1e-9);

        // y la corrida anterior se localiza para comparar
        let prev = previous_of(&c, sid, "retest", nueva["id"].as_i64().unwrap()).unwrap();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap()["profitFactor"], 1.2);
    }

    #[test]
    fn se_puede_borrar_una_entrada() {
        let c = db();
        let sid: i64 = c.query_row("select id from strategies", [], |r| r.get(0)).unwrap();
        let id = record(&c, sid, "spp", "pass", "x", &json!({}), &json!({}), &json!({})).unwrap();
        assert!(delete(&c, id).unwrap());
        assert!(list(&c, None, None, 10).unwrap().is_empty());
    }
}
