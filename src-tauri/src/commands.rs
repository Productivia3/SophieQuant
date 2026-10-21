//! commands.rs — comandos Tauri. Contrato exacto de replica/architecture.md,
//! consumido por src/lib/api.ts (camelCase en Payloads con serde).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{params as q, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::engine::datastore::{self, Qfn};
use crate::engine::genetic::{self, BuildConfig, BuildReporter};
use crate::engine::rules::Template;
use crate::engine::types::{Bar, EngineCfg};
use crate::export::{self, Platform};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: String,
}

pub static CANCEL_BUILD: AtomicBool = AtomicBool::new(false);

fn now_ts() -> i64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) }

// ── helpers DB ──────────────────────────────────────────────────────

pub fn seed_symbols(db: &Connection) -> rusqlite::Result<()> {
    let have: Option<i64> = db
        .query_row("select count(*) from symbols", [], |r| r.get(0)).ok();
    if have.unwrap_or(0) > 0 { return Ok(()) }
    db.execute_batch(r#"
      insert into symbols (name, asset_class, exchange, tick_size, pip_size, commission) values
        ('EURUSD','forex',null,0.00001,0.0001,0.0),
        ('GBPUSD','forex',null,0.00001,0.0001,0.0),
        ('USDJPY','forex',null,0.001,0.01,0.0);"#)?;
    Ok(())
}

pub fn seed_templates(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(r#"
      insert or ignore into strategy_templates (id, name, source, logic) values
        (1,'Cruce de medias','built_in','{"template":"ma_cross"}'),
        (2,'Reversión osciladora','built_in','{"template":"rsi_reversion"}'),
        (3,'Ruptura con filtro de tendencia','built_in','{"template":"atr_breakout"}');"#)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRow { pub id: i64, pub name: String, pub asset_class: String, pub pip_size: f64 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFileRow {
    pub id: i64, pub symbol_id: i64, pub timeframe: String,
    pub storage_kind: String, pub compressed_bytes: i64, pub n_bars: i64,
    pub date_from: Option<i64>, pub date_to: Option<i64>, pub status: String, pub created_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRow {
    pub id: i64, pub name: String, pub source: String,
}

// ── data manager ────────────────────────────────────────────────────

/// Metricas por las que se puede filtrar y pesos del score de robustez.
/// La UI los pide aqui: no hay lista de metricas duplicada en el front.
#[tauri::command]
pub fn list_build_metrics() -> Result<serde_json::Value, String> {
    let metrics: Vec<serde_json::Value> = crate::engine::robust::METRICAS
        .iter()
        .map(|(key, label, _)| serde_json::json!({ "key": key, "label": label }))
        .collect();
    let pesos: Vec<serde_json::Value> = crate::engine::robust::PESOS_AVANZADO
        .iter()
        .map(|(key, peso, desc)| serde_json::json!({ "key": key, "peso": peso, "desc": desc }))
        .collect();
    Ok(serde_json::json!({ "metrics": metrics, "pesos": pesos }))
}

#[tauri::command]
pub fn list_symbols(state: State<AppState>) -> Result<Vec<SymbolRow>, String> {
    let db = state.db.lock();
    seed_symbols(&db).map_err(|e| e.to_string())?;
    let mut st = db.prepare("select id, name, asset_class, pip_size from symbols order by name").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        Ok(SymbolRow { id: r.get(0)?, name: r.get(1)?, asset_class: r.get(2)?, pip_size: r.get(3)? })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn list_data_files(state: State<AppState>) -> Result<Vec<DataFileRow>, String> {
    let db = state.db.lock();
    let mut st = db.prepare(
        "select id, symbol_id, timeframe, storage_kind, compressed_bytes, n_bars, date_from, date_to, status, created_at
         from data_files order by id").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        Ok(DataFileRow {
            id: r.get(0)?, symbol_id: r.get(1)?, timeframe: r.get(2)?,
            storage_kind: r.get(3)?, compressed_bytes: r.get(4)?, n_bars: r.get(5)?,
            date_from: r.get(6)?, date_to: r.get(7)?, status: r.get(8)?, created_at: r.get(9)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn import_data_csv(
    app: AppHandle, state: State<AppState>, symbol_id: i64, timeframe: String, path: String,
) -> Result<DataFileRow, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).ok();
    let db = state.db.lock();
    import_csv_registered(&db, &dir, symbol_id, &timeframe, std::path::Path::new(&path))
}

/// Importa un CSV de mercado: lo lee, lo limpia, lo guarda como QFN y lo registra
/// en `data_files`. Funciona con cualquier simbolo y temporalidad.
///
/// Separada del comando para poder testearla sin Tauri.
pub fn import_csv_registered(
    db: &Connection,
    app_dir: &std::path::Path,
    symbol_id: i64,
    timeframe: &str,
    path: &std::path::Path,
) -> Result<DataFileRow, String> {
    let (bars, report) = datastore::import_csv(path)?;
    let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("dataset").to_string();
    let qfn_path = app_dir.join(format!("{file_name}_{timeframe}.qfn"));
    Qfn::write(&qfn_path, &bars).map_err(|e| e.to_string())?;

    db.execute(
        "insert into data_files (symbol_id, timeframe, storage_kind, file_path, compressed_bytes, n_bars, date_from, date_to, status, created_at)
         values (?1, ?2, 'imported_csv', ?3, ?4, ?5, ?6, ?7, 'ready', ?8)
         on conflict do update set file_path = excluded.file_path,
            compressed_bytes = excluded.compressed_bytes, n_bars = excluded.n_bars,
            date_from = excluded.date_from, date_to = excluded.date_to, status = 'ready'",
        q![symbol_id, timeframe, qfn_path.to_string_lossy(),
           std::fs::metadata(&qfn_path).map(|m| m.len() as i64).unwrap_or(0),
           report.n_bars, report.date_from, report.date_to, now_ts()],
    ).map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    Ok(DataFileRow {
        id, symbol_id, timeframe: timeframe.to_string(), storage_kind: "imported_csv".into(),
        compressed_bytes: 0, n_bars: report.n_bars,
        date_from: Some(report.date_from), date_to: Some(report.date_to),
        status: "ready".into(), created_at: now_ts(),
    })
}

#[tauri::command]
pub fn list_templates(state: State<AppState>) -> Result<Vec<TemplateRow>, String> {
    let db = state.db.lock();
    seed_templates(&db).map_err(|e| e.to_string())?;
    let mut st = db.prepare("select id, name, source from strategy_templates order by id").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| Ok(TemplateRow { id: r.get(0)?, name: r.get(1)?, source: r.get(2)? }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

// ── builds ──────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BuildEvent {
    build_id: i64,
    level: &'static str,
    message: String,
    passed: u32,
    total: u32,
    progress: f64,
}

struct TauriReporter {
    app: AppHandle,
    build_id: i64,
}
impl BuildReporter for TauriReporter {
    fn report(&self, level: &str, msg: &str, passed: u32, total: u32, progress: f64) {
        let l = match level { "err" => "error", "warn" => "warn", _ => "info" };
        let _ = self.app.emit("build-event", BuildEvent {
            build_id: self.build_id, level: l,
            message: msg.to_string(), passed, total, progress,
        });
    }
    fn cancelled(&self) -> bool { CANCEL_BUILD.load(Ordering::Relaxed) }
    fn paused(&self) -> bool { PAUSE_BUILD.load(Ordering::Relaxed) }
}

#[tauri::command]
pub fn start_build(app: AppHandle, state: State<AppState>, config: BuildConfig) -> Result<i64, String> {
    // el dataset debe existir para el símbolo/TF
    let (dataset_path, data_file_id) = {
        let db = state.db.lock();
        let row: Option<(String, i64)> = db
            .query_row(
                "select file_path, id from data_files where symbol_id = ?1 and timeframe = ?2 and status = 'ready'
                 order by n_bars desc limit 1",
                q![config.symbol_id, config.timeframe],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional().map_err(|e| e.to_string())?;
        row.ok_or_else(|| "no hay datos para este símbolo/TF — importa un CSV en Datos".to_string())?
    };

    let bars: Vec<Bar> = Qfn::read(std::path::Path::new(&dataset_path)).map_err(|e| e.to_string())?;
    if bars.len() < 200 {
        return Err(format!("dataset demasiado pequeño ({} barras); importa al menos 200", bars.len()));
    }

    // validar los filtros antes de generar: una metrica o un operador que no
    // existen no pueden fallar en silencio durante la busqueda
    for f in &config.filters {
        if !crate::engine::robust::metric_exists(&f.metric) {
            return Err(format!(
                "metrica desconocida: '{}'. Disponibles: {}",
                f.metric,
                crate::engine::robust::METRICAS
                    .iter()
                    .map(|(k, _, _)| *k)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !matches!(f.op.as_str(), ">=" | "<=" | ">" | "<") {
            return Err(format!("operador desconocido: '{}' (usa >=, <=, > o <)", f.op));
        }
    }

    CANCEL_BUILD.store(false, Ordering::Relaxed);

    let db = state.db.lock();
    db.execute(
        "insert into builds (kind, config, status, n_total, started_at) values ('builder', ?1, 'running', ?2, ?3)",
        q![serde_json::to_string(&config).unwrap_or_default(), config.n_target, now_ts()],
    ).map_err(|e| e.to_string())?;
    let build_id = db.last_insert_rowid();
    drop(db);

    let db_path = state.db_path.clone();

    let app2 = app.clone();
    let total = config.n_target;
    std::thread::spawn(move || {
        let conn = match open_thread_db(&db_path) {
            Ok(c) => c,
            Err(e) => {
                let _ = app2.emit("build-event", BuildEvent { build_id, level: "err", message: e.to_string(), passed: 0, total, progress: 0.0 });
                return;
            }
        };
        let engine_cfg = EngineCfg::default();
        let reporter = TauriReporter { app: app2.clone(), build_id };

        let results = genetic::build(&bars, &engine_cfg, &config, &reporter);

        // persistir estrategias pasadas
        let ins_strategy = |th: &Connection, s: &genetic::GenStrat| -> Result<i64, rusqlite::Error> {
            th.execute(
                "insert into strategies (name, symbol_id, timeframe, template_id, origin, logic, params,
                   status, robust_score, robust_detail)
                 values (?1, ?2, ?3, ?4, 'builder', ?5, ?6, 'active', ?7, ?8)",
                q![s.name, config.symbol_id, config.timeframe, config.template_id,
                   serde_json::json!({"template": format!("{:?}", s.tmpl)}).to_string(),
                   serde_json::json!({"p": s.params}).to_string(),
                   s.robust.score,
                   serde_json::json!({
                       "wfe": s.robust.wfe,
                       "mcSurvival": s.robust.mc_survival,
                       "spp": s.robust.spp,
                       "perfil": s.robust.perfil,
                       "score": s.robust.score,
                       "avanzado": s.robust.avanzado,
                   }).to_string()],
            )?;
            Ok(th.last_insert_rowid())
        };

        let mut passed_n = 0u32;
        for s in &results {
            if reporter.cancelled() { break }
            match ins_strategy(&conn, s).and_then(|sid| {
                conn.execute(
                    "insert into backtest_runs (strategy_id, data_file_id, engine_cfg, build_id,
                       net_profit, gross_profit, profit_factor, win_rate, max_dd, rdd, sharpe,
                       sortino, z_score, r_squared, n_trades, avg_trade, wfe, equity, oos_frac, status)
                     values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0, ?17, ?18, 'done')",
                    q![sid, data_file_id,
                       serde_json::to_string(&engine_cfg).unwrap_or_default(), build_id,
                       s.result.metrics.net_profit, s.result.metrics.gross_profit,
                       s.result.metrics.profit_factor, s.result.metrics.win_rate,
                       s.result.metrics.max_dd, s.result.metrics.rdd, s.result.metrics.sharpe,
                       s.result.metrics.sortino, s.result.metrics.z_score, s.result.metrics.r_squared,
                       s.result.metrics.n_trades as i64, s.result.metrics.avg_trade,
                       serde_json::to_string(&s.result.equity).unwrap_or_default(),
                       ops::oos_frac_of(&bars, &s.result.trades)],
                ).map(|_| sid)
            }) {
                Ok(_) => {
                    passed_n += 1;
                    // el log dice por que se ha quedado: score de robustez y por que
                    let _ = reporter.report(
                        "info",
                        &format!(
                            "PASSED #{}: {} · PF {} · DD {} % · robustez {}",
                            passed_n,
                            s.name,
                            round2f(s.result.metrics.profit_factor),
                            round2f(s.result.metrics.max_dd),
                            round2f(s.robust.score),
                        ),
                        passed_n,
                        total,
                        1.0,
                    );
                }
                Err(e) => { let _ = reporter.report("err", &format!("db error: {e}"), passed_n, total, 1.0); }
            }
        }
        let final_status = if reporter.cancelled() { "cancelled" } else { "done" };
        let _ = reporter.report("info", &format!("build finalizado — {passed_n} estrategias pasadas"),
            passed_n, total, 1.0);
        let _ = conn.execute("update builds set status = ?1, finished_at = ?2 where id = ?3",
            q![final_status, now_ts(), build_id]);
    });

    Ok(build_id)
}

fn open_thread_db(path: &str) -> rusqlite::Result<Connection> {
    let c = datastore::open(std::path::Path::new(path))?;
    datastore::init_schema(&c)?;
    Ok(c)
}

#[tauri::command]
pub fn list_builds(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock();
    let mut st = db.prepare(
        "select id, kind, status, n_passed, n_total, started_at, finished_at from builds order by id desc").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?, "kind": r.get::<_, String>(1)?,
            "status": r.get::<_, String>(2)?, "nPassed": r.get::<_, i64>(3)?,
            "nTotal": r.get::<_, i64>(4)?, "startedAt": r.get::<_, Option<i64>>(5)?,
            "finishedAt": r.get::<_, Option<i64>>(6)?,
        }))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn cancel_build() -> Result<(), String> {
    CANCEL_BUILD.store(true, Ordering::Relaxed);
    Ok(())
}

// ── bank ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_bank(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock();
    let mut st = db.prepare(
        "select s.id, s.name, sy.name, s.timeframe, s.status,
                b.net_profit, b.profit_factor, b.win_rate, b.max_dd, b.n_trades, b.sharpe, b.equity, b.oos_frac,
                s.robust_score
         from strategies s
         join symbols sy on sy.id = s.symbol_id
         left join backtest_runs b on b.strategy_id = s.id and b.id = (
            select id from backtest_runs where strategy_id = s.id order by id desc limit 1)
         order by (b.net_profit is null), b.net_profit desc").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        let equity_raw: Option<String> = r.get(11).ok();
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "name": r.get::<_, String>(1)?,
            "symbol": r.get::<_, String>(2)?,
            "timeframe": r.get::<_, String>(3)?,
            "status": r.get::<_, String>(4)?,
            "netProfit": r.get::<_, Option<f64>>(5)?,
            "profitFactor": r.get::<_, Option<f64>>(6)?,
            "winRate": r.get::<_, Option<f64>>(7)?,
            "maxDD": r.get::<_, Option<f64>>(8)?,
            "nTrades": r.get::<_, Option<i64>>(9)?,
            "sharpe": r.get::<_, Option<f64>>(10)?,
            "robustScore": r.get::<_, Option<f64>>(13)?,
            "lastRun": {
                "equity": equity_raw.and_then(|e| serde_json::from_str::<Vec<f64>>(&e).ok()),
            "oosFrac": r.get::<_, Option<f64>>(17)?,
                "oosFrac": r.get::<_, Option<f64>>(12)?,
                "status": "done",
            },
        }))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn get_strategy(state: State<AppState>, id: i64) -> Result<Option<serde_json::Value>, String> {
    let db = state.db.lock();
    let mut st = db.prepare(
        "select s.id, s.name, sy.name, s.timeframe, s.status, s.template_id,
                b.net_profit, b.profit_factor, b.win_rate, b.max_dd, b.sharpe, b.n_trades,
                b.r_squared, b.z_score, b.avg_trade, b.rdd, b.equity, b.oos_frac,
                s.params, s.logic
         from strategies s
         join symbols sy on sy.id = s.symbol_id
         left join backtest_runs b on b.strategy_id = s.id
             and b.id = (select id from backtest_runs where strategy_id = s.id order by id desc limit 1)
         where s.id = ?1").map_err(|e| e.to_string())?;
    let row = st.query_row(q![id], |r| {
        let equity_raw: Option<String> = r.get(16).ok();
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "name": r.get::<_, String>(1)?,
            "symbol": r.get::<_, String>(2)?,
            "timeframe": r.get::<_, String>(3)?,
            "status": r.get::<_, String>(4)?,
            "templateId": r.get::<_, Option<i64>>(5)?,
            "params": r.get::<_, Option<String>>(18)?.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(serde_json::Value::Null),
            "logic": r.get::<_, Option<String>>(19)?.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(serde_json::Value::Null),
            "metrics": {
                "netProfit": r.get::<_, Option<f64>>(6)?,
                "profitFactor": r.get::<_, Option<f64>>(7)?,
                "winRate": r.get::<_, Option<f64>>(8)?,
                "maxDD": r.get::<_, Option<f64>>(9)?,
                "sharpe": r.get::<_, Option<f64>>(10)?,
                "nTrades": r.get::<_, Option<i64>>(11)?,
                "rSquared": r.get::<_, Option<f64>>(12)?,
                "zScore": r.get::<_, Option<f64>>(13)?,
                "avgTrade": r.get::<_, Option<f64>>(14)?,
                "rdd": r.get::<_, Option<f64>>(15)?,
            },
            "lastRun": {
                "equity": equity_raw.and_then(|e| serde_json::from_str::<Vec<f64>>(&e).ok()),
                "status": "done",
            },
        }))
    }).optional().map_err(|e| e.to_string())?;
    Ok(row)
}

// ── export ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_strategy(state: State<AppState>, strategy_id: i64, platform: String) -> Result<String, String> {
    let plat = Platform::from_str(&platform).ok_or("plataforma no soportada en V1")?;
    let db = state.db.lock();
    let (name, template_id, params_json): (String, Option<i64>, Option<String>) = db
        .query_row("select name, template_id, params from strategies where id = ?1", q![strategy_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        }).optional().map_err(|e| e.to_string())?
        .ok_or("estrategia no encontrada")?;
    let params: [f64; 3] = params_json
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .and_then(|v| v.get("p").and_then(|x| serde_json::from_value::<Vec<f64>>(x.clone()).ok()))
        .and_then(|v| <[f64; 3]>::try_from(v).ok())
        .unwrap_or([20.0, 50.0, 0.0]);
    let tmpl = Template::from_id(template_id.unwrap_or(1)).unwrap_or(Template::MaCross);
    let code = export::strategy_code(plat, &name, tmpl, params);

    let db2 = state.db.lock();
    db2.execute(
        "insert into exports (strategy_id, platform, file_name) values (?1, ?2, ?3)",
        q![strategy_id, format!("{platform}"), format!("{name}.{platform}")],
    ).map_err(|e| e.to_string())?;
    drop(db2);
    Ok(code)}

// ── settings mínimos ─────────────────────────────────────────────────

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let rows: std::collections::HashMap<String, serde_json::Value> = {
        let mut st = db.prepare("select key, value from app_settings").map_err(|e| e.to_string())?;
        let m = st.query_map([], |r| {
            let k: String = r.get(0)?;
            let v: String = r.get(1)?;
            Ok((k, serde_json::from_str::<serde_json::Value>(&v).unwrap_or(serde_json::Value::Null)))
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();
        m
    };
    Ok(serde_json::to_value(rows).unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, key: String, value: serde_json::Value) -> Result<(), String> {
    let db = state.db.lock();
    db.execute(
        "insert into app_settings (key, value) values (?1, ?2)
         on conflict(key) do update set value = excluded.value",
        q![key, value.to_string()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// uso de Arc importado para poder extender con state compartido en Fase 2
#[allow(dead_code)]
type _KeepArc = Arc<Mutex<Connection>>;

// ── pausa / reanudación de builds ─────────────────────────────────────

pub static PAUSE_BUILD: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn pause_build(state: State<AppState>, id: i64) -> Result<(), String> {
    PAUSE_BUILD.store(true, Ordering::Relaxed);
    let db = state.db.lock();
    db.execute("update builds set status = 'paused' where id = ?1", q![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn resume_build(state: State<AppState>, id: i64) -> Result<(), String> {
    PAUSE_BUILD.store(false, Ordering::Relaxed);
    let db = state.db.lock();
    db.execute("update builds set status = 'running' where id = ?1 and status = 'paused'", q![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── robustez / cartera: commands sobre el motor real (engine::ops) ────

use crate::engine::ops;

fn load_core(db: &Connection, strategy_id: i64) -> Result<ops::StrategyCore, String> {
    ops::load_strategy(db, strategy_id)?
        .ok_or_else(|| format!("estrategia {strategy_id} no encontrada"))
}

// ── Veredictos consolidados (tarjeta de robustez) ──────────────────
//
// Cuatro pruebas por estrategia: walk-forward, Monte Carlo (9 tests), SPP y
// perfil de optimización. Se calculan con el motor real, se guardan en
// `strategy_verdicts` y se recuperan sin recalcular.

/// Lee los veredictos ya calculados (cache) para las estrategias pedidas.
#[tauri::command]
pub fn get_strategy_verdicts(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for id in strategy_ids {
        let name: Option<String> = db
            .query_row("select name from strategies where id = ?1", q![id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let mut checks = serde_json::Map::new();
        let mut calculado_en: Option<i64> = None;
        for key in ["walkForward", "monteCarlo", "spp", "perfil"] {
            let row: Option<(String, String, Option<f64>, i64)> = db
                .query_row(
                    "select estado, detalle, valor, calculado_en from strategy_verdicts
                     where strategy_id = ?1 and check_key = ?2 order by id desc limit 1",
                    q![id, key],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            match row {
                Some((estado, detalle, valor, ts)) => {
                    calculado_en = Some(ts);
                    checks.insert(key.to_string(), serde_json::json!({
                        "estado": estado, "detalle": detalle, "valor": valor,
                    }));
                }
                None => {
                    checks.insert(key.to_string(), serde_json::json!({
                        "estado": "pending", "detalle": "sin calcular", "valor": serde_json::Value::Null,
                    }));
                }
            }
        }
        let checks = serde_json::Value::Object(checks);
        rows.push(serde_json::json!({
            "strategyId": id,
            "nombre": name.unwrap_or_else(|| format!("#{id}")),
            "checks": checks,
            "overall": crate::engine::verdicts::overall(&checks),
            "calculadoEn": calculado_en,
        }));
    }
    Ok(serde_json::json!({ "rows": rows }))
}

/// Calcula (y guarda) los cuatro veredictos de las estrategias pedidas.
#[tauri::command(async)]
pub fn compute_strategy_verdicts(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let cfg = EngineCfg::default();
    let mut guard = state.db.lock();
    let db = &mut *guard;
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for id in strategy_ids {
        let name: Option<String> = db
            .query_row("select name from strategies where id = ?1", q![id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let checks = match ops::load_strategy(db, id) {
            Ok(Some(core)) => crate::engine::verdicts::for_strategy(&core, &cfg),
            Ok(None) => return Err(format!("estrategia {id} no encontrada")),
            Err(e) => {
                let p = serde_json::json!({
                    "estado": "pending", "detalle": e, "valor": serde_json::Value::Null,
                });
                serde_json::json!({
                    "walkForward": p, "monteCarlo": p, "spp": p, "perfil": p,
                })
            }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        for key in ["walkForward", "monteCarlo", "spp", "perfil"] {
            let c = &checks[key];
            db.execute(
                "insert into strategy_verdicts (strategy_id, check_key, estado, detalle, valor, calculado_en)
                 values (?1, ?2, ?3, ?4, ?5, ?6)",
                q![
                    id,
                    key,
                    c["estado"].as_str().unwrap_or("pending"),
                    c["detalle"].as_str().unwrap_or(""),
                    c["valor"].as_f64(),
                    now,
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        rows.push(serde_json::json!({
            "strategyId": id,
            "nombre": name.unwrap_or_else(|| format!("#{id}")),
            "checks": checks,
            "overall": crate::engine::verdicts::overall(&checks),
            "calculadoEn": now,
        }));
    }
    drop(guard);
    Ok(serde_json::json!({ "rows": rows }))
}


// ── Editor sin código: guardar de verdad ───────────────────────────
//
// El editor expone exactamente las reglas que el motor ejecuta. Al guardar:
// 1) se persiste la regla (plantilla + parametros + logico),
// 2) se vuelve a correr el backtest sobre el dataset de la estrategia,
// 3) se guarda la corrida nueva (metricas, equity y corte out-of-sample).
// Asi lo que se ve en pantalla es el resultado de la regla guardada.

#[tauri::command(async)]
pub fn save_strategy_logic(
    state: State<AppState>,
    strategy_id: i64,
    template_id: i64,
    params: Vec<f64>,
    logic: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if params.len() != 3 {
        return Err("la regla necesita exactamente 3 parametros".into());
    }
    let template = Template::from_id(template_id)
        .ok_or_else(|| format!("plantilla {template_id} no soportada por el motor"))?;
    let p: [f64; 3] = [params[0], params[1], params[2]];

    let (dataset_path, data_file_id, engine_cfg) = {
        let db = state.db.lock();
        let (dataset_path, cfg_raw, data_file_id): (String, String, i64) = {
            db.query_row(
                "select df.file_path, b.engine_cfg, b.data_file_id
                 from strategies s
                 join backtest_runs b on b.id = (
                    select id from backtest_runs where strategy_id = s.id order by id desc limit 1)
                 join data_files df on df.id = b.data_file_id
                 where s.id = ?1",
                q![strategy_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or(format!(
                "estrategia {strategy_id} sin dataset asociado: importa el CSV antes de editar"
            ))?
        };
        let cfg: EngineCfg = serde_json::from_str(&cfg_raw).unwrap_or_default();
        (dataset_path, data_file_id, cfg)
    };

    let db = state.db.lock();
    ops::save_logic(
        &db,
        std::path::Path::new(&dataset_path),
        data_file_id,
        &engine_cfg,
        strategy_id,
        template,
        p,
        &logic,
    )
}

#[tauri::command(async)]
pub fn run_retest(state: State<AppState>, strategy_ids: Vec<i64>, symbol_id: i64, timeframe: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let cfg = EngineCfg::default();
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for id in strategy_ids {
        let orig = ops::stored_metrics(&db, id);
        match load_core(&db, id).and_then(|c| ops::retest_one(&c, orig.as_ref(), &cfg)) {
            Ok(mut v) => {
                v["strategyId"] = id.into();
                rows.push(v);
            }
            Err(e) => rows.push(serde_json::json!({
                "strategyId": id, "original": orig.as_ref().map(ops::metrics_json),
                "retest": serde_json::Value::Null, "verdict": "fail", "note": e,
            })),
        }
    }
    drop(db);
    Ok(serde_json::json!({ "symbolId": symbol_id, "timeframe": timeframe, "rows": rows }))
}

#[tauri::command(async)]
pub fn run_improver(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::improver(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    drop(db);
    Ok(v)
}

#[tauri::command(async)]
pub fn run_optimizer(state: State<AppState>, strategy_id: i64, param_a: String, param_b: String, steps: i64, kind: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let g = ops::run_grid(&core, &EngineCfg::default(), steps as usize, &kind)?;
    drop(db);
    Ok(serde_json::json!({
        "strategyId": strategy_id, "kind": kind,
        "paramA": param_a, "paramB": param_b,
        "aValues": g["aValues"], "bValues": g["bValues"],
        "cells": g["cells"], "best": g["best"],
    }))
}

#[tauri::command(async)]
pub fn run_wfm(state: State<AppState>, strategy_id: i64, a: String, b: String, steps: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let g = ops::run_grid(&core, &EngineCfg::default(), steps as usize, "wf")?;
    drop(db);
    Ok(serde_json::json!({
        "strategyId": strategy_id, "paramA": a, "paramB": b,
        "aValues": g["aValues"], "bValues": g["bValues"], "cells": g["cells"],
    }))
}

#[tauri::command(async)]
pub fn run_spp(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::spp(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    drop(db);
    Ok(v)
}

#[tauri::command(async)]
pub fn run_mc_suite(state: State<AppState>, strategy_id: i64, tests: Vec<String>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::mc_suite(&core, &EngineCfg::default(), &tests, 100)?;
    v["strategyId"] = strategy_id.into();
    drop(db);
    Ok(v)
}

#[tauri::command]
pub fn get_correlation_matrix(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut ids: Vec<i64> = Vec::new();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    for id in strategy_ids {
        if let Some(eq) = ops::stored_equity(&db, id) {
            ids.push(id);
            seqs.push(eq);
        }
    }
    let m = ops::corr_matrix(&seqs);
    drop(db);
    Ok(serde_json::json!({ "ids": ids, "values": m }))
}

#[tauri::command]
pub fn get_portfolio_equity(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut members = Vec::new();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    for id in strategy_ids {
        if let Some(eq) = ops::stored_equity(&db, id) {
            members.push(serde_json::json!({ "strategyId": id, "equity": eq }));
            seqs.push(eq);
        }
    }
    let combined = ops::mean_equity(&seqs);
    drop(db);
    Ok(serde_json::json!({ "members": members, "combined": combined }))
}

#[tauri::command]
pub fn get_portfolio_results(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut ids: Vec<i64> = Vec::new();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    for id in &strategy_ids {
        if let Some(eq) = ops::stored_equity(&db, *id) {
            ids.push(*id);
            seqs.push(eq);
        }
    }
    let matrix = ops::corr_matrix(&seqs);
    let n = ids.len();
    let mut members: Vec<serde_json::Value> = Vec::new();
    for (k, id) in ids.iter().enumerate() {
        let m = ops::stored_metrics(&db, *id);
        // correlación media de esta estrategia con el resto (excluye la diagonal)
        let avg_corr = if n > 1 {
            let off: Vec<f64> = matrix[k].iter().enumerate()
                .filter(|(i, _)| *i != k).map(|(_, v)| *v).collect();
            (ops::mean(&off)).max(0.0)
        } else { 0.0 };
        members.push(serde_json::json!({
            "strategyId": id,
            "metrics": m.as_ref().map(ops::metrics_json),
            "correlation": avg_corr,
            "weight": (100.0 / n.max(1) as f64),
            "verdict": if avg_corr >= 0.75 { "warn" } else { "pass" },
        }));
    }
    let combined = ops::mean_equity(&seqs);
    let combined_dd = ops::max_dd_pct(&combined);
    let max_member_dd = seqs.iter().map(|s| ops::max_dd_pct(s)).fold(0.0f64, f64::max);
    let combined_net: f64 = members.iter()
        .filter_map(|mv| mv["metrics"].as_object().and_then(|mo| mo.get("netProfit").and_then(|x| x.as_f64())))
        .sum();
    let sharpe_c = ops::sharpe(&ops::returns_of(&combined));
    drop(db);
    Ok(serde_json::json!({
        "members": members,
        "combinedEquity": combined,
        "totals": {
            "netProfit": combined_net,
            "maxDD": combined_dd,
            "sharpe": sharpe_c,
            "diversification": if max_member_dd > 0.001 { (1.0 - combined_dd / max_member_dd).max(0.0) } else { 0.0 },
        },
    }))
}

#[tauri::command]
pub fn run_portfolio_mc(state: State<AppState>, strategy_ids: Vec<i64>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    for id in strategy_ids {
        if let Some(eq) = ops::stored_equity(&db, id) { seqs.push(eq); }
    }
    drop(db);
    // retornos de la equity combinada (la media usa longitud común mínima)
    let combined = ops::mean_equity(&seqs);
    let rets = if combined.len() > 2 { ops::returns_of(&combined) } else {
        // sin equities combinables: alimenta con los retornos disponibles, aplanados
        seqs.iter().flat_map(|s| ops::returns_of(s)).collect::<Vec<f64>>()
    };
    Ok(ops::bootstrap_mc(&rets, 500, 4242))
}

#[tauri::command(async)]
pub fn run_project(state: State<AppState>, steps: Vec<String>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    // estrategias más recientes (hasta 8) para los pasos de análisis
    let ids: Vec<i64> = {
        let mut st = db.prepare(
            "select id from strategies order by id desc limit 8").map_err(|e| e.to_string())?;
        let v = st.query_map([], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
        v
    };
    let cfg = EngineCfg::default();
    let mut progressed: Vec<serde_json::Value> = Vec::new();
    let mut failed = false;
    for s in &steps {
        if failed { break }
        let status = match s.as_str() {
            "build" => if ids.is_empty() { Err("sin estrategias generadas todavía".into()) } else { Ok(()) },
            "monte_carlo" => if ids.is_empty() { Err("sin estrategias".into()) } else {
                let mut seqs = Vec::new();
                for id in &ids {
                    if let Some(eq) = ops::stored_equity(&db, *id) { seqs.push(eq); }
                }
                let combined = ops::mean_equity(&seqs);
                let rets = if combined.len() > 2 { ops::returns_of(&combined) } else { seqs.iter().flat_map(|x| ops::returns_of(x)).collect() };
                let _ = ops::bootstrap_mc(&rets, 500, 4242);
                Ok(())
            },
            "spp" => ids.first().ok_or("sin estrategias".to_string()).and_then(|id| {
                let core = load_core(&db, *id)?;
                ops::spp(&core, &cfg).map(|_| ())
            }),
            "optimizer" => ids.first().ok_or("sin estrategias".to_string()).and_then(|id| {
                let core = load_core(&db, *id)?;
                ops::run_grid(&core, &cfg, 5, "wf").map(|_| ())
            }),
            "save" => Ok(()), // el bank ya persiste en SQLite de forma continua
            other => Err(format!("paso desconocido: {other}")),
        };
        match status {
            Ok(()) => progressed.push(serde_json::json!({ "step": s, "status": "done" })),
            Err(e) => {
                progressed.push(serde_json::json!({ "step": s, "status": "error", "note": e }));
                failed = true;
            }
        }
    }
    drop(db);
    Ok(serde_json::json!({ "steps": progressed, "finishedAt": now_ms() }))
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[tauri::command(async)]
pub fn get_strategy_chart(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    drop(db);
    let out = crate::engine::backtest::run(&core.bars, core.template, core.params, &EngineCfg::default());
    // velas muestreadas (hasta 400) y trades mapeados al índice de vela muestreada
    let step = (core.bars.len() / 400).max(1);
    let sampled: Vec<&Bar> = core.bars.iter().step_by(step).collect();
    let candles: Vec<serde_json::Value> = sampled.iter().map(|b| serde_json::json!({ "o": b.open, "h": b.high, "l": b.low, "c": b.close })).collect();
    let n = sampled.len();
    let trades: Vec<serde_json::Value> = out.trades.iter().filter_map(|t| {
        // índice de la vela muestreada más cercana al cierre del trade
        let target = t.close_time;
        let idx = sampled.iter().position(|b| b.time >= target).unwrap_or(n.saturating_sub(1));
        Some(serde_json::json!({ "idx": idx, "dir": "buy", "entry": round2f(t.open_price), "exit": round2f(t.close_price) }))
    }).collect();
    // corte out-of-sample: la misma fraccion que usa la comprobacion walk-forward
    let oos_from = crate::engine::verdicts::oos_fraction();
    Ok(serde_json::json!({
        "candles": candles, "trades": trades,
        "oosFrom": oos_from,
        "oosLabel": format!("Fuera de muestra (ultimo {:.0} %)", oos_from * 100.0),
    }))
}

fn round2f(v: f64) -> f64 { (v * 10000.0).round() / 10000.0 }

#[tauri::command]
pub fn export_dataset(app: AppHandle, state: State<AppState>, data_file_id: i64, format: String) -> Result<serde_json::Value, String> {
    let (path, sym, tf): (String, String, String) = {
        let db = state.db.lock();
        db.query_row(
            "select df.file_path, sy.name, df.timeframe from data_files df join symbols sy on sy.id = df.symbol_id where df.id = ?1",
            q![data_file_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).optional().map_err(|e| e.to_string())?
        .ok_or(format!("dataset {data_file_id} no encontrado"))?
    };
    let src = std::path::Path::new(&path);
    let out_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("exports");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let name = if format == "csv" {
        // regenerar CSV legible desde el QFN fuente
        let bars = Qfn::read(src).map_err(|e| e.to_string())?;
        let name = format!("{sym}_{tf}.csv");
        let mut csv = String::with_capacity(bars.len() * 48);
        csv.push_str("time,open,high,low,close,volume\n");
        for b in &bars {
            csv.push_str(&format!("{},{},{},{},{},{}\n", b.time, b.open, b.high, b.low, b.close, b.volume));
        }
        std::fs::write(out_dir.join(&name), csv).map_err(|e| e.to_string())?;
        name
    } else {
        let name = format!("{sym}_{tf}.qfn");
        std::fs::copy(src, out_dir.join(&name)).map_err(|e| e.to_string())?;
        name
    };
    let bytes = std::fs::metadata(out_dir.join(&name)).map(|m| m.len() as i64).unwrap_or(0);
    Ok(serde_json::json!({ "fileName": name, "bytes": bytes }))
}

#[tauri::command(async)]
pub fn run_opt_profile(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::opt_profile(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    drop(db);
    Ok(v)
}

// ── tests ─────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// CSV de mercado en formato simple (time,open,high,low,close,volume).
    fn csv_de_ejemplo(n: usize) -> String {
        let mut s = String::from("time,open,high,low,close,volume\n");
        for i in 0..n {
            let base = 1.1000 + (i as f64) * 0.00001;
            s.push_str(&format!(
                "{},{},{},{},{},100\n",
                1_700_000_000 + (i as i64) * 60,
                base, base * 1.001, base * 0.999, base * 1.0005
            ));
        }
        s
    }

    /// Importar un CSV de verdad: lee, limpia, guarda el QFN y registra el dataset.
    #[test]
    fn importar_csv_registra_el_dataset_y_guarda_el_qfn() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();

        let dir = std::env::temp_dir().join(format!("qf_import_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("eurusd.csv");
        std::fs::write(&csv, csv_de_ejemplo(500)).unwrap();

        let row = import_csv_registered(&conn, &dir, sym, "m1", &csv).expect("la importacion debe funcionar");

        assert_eq!(row.symbol_id, sym);
        assert_eq!(row.timeframe, "m1");
        assert_eq!(row.storage_kind, "imported_csv");
        assert_eq!(row.n_bars, 500, "debe registrar todas las barras");
        assert_eq!(row.status, "ready");

        // el QFN queda escrito en disco
        let qfn = dir.join("eurusd_m1.qfn");
        assert!(qfn.exists(), "falta el fichero QFN");
        let bars = crate::engine::datastore::Qfn::read(&qfn).unwrap();
        assert_eq!(bars.len(), 500);

        // y el dataset esta en la base de datos
        let (n, kind): (i64, String) = conn
            .query_row(
                "select count(*), max(storage_kind) from data_files where symbol_id = ?1",
                q![sym],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(kind, "imported_csv");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Un CSV que no existe debe fallar con mensaje claro (no inventarse datos).
    #[test]
    fn importar_un_fichero_inexistente_falla() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        let dir = std::env::temp_dir().join(format!("qf_import_mal_{}", std::process::id()));
        let err = import_csv_registered(
            &conn, &dir, 1, "m1",
            std::path::Path::new("no_existe_este_fichero.csv"),
        );
        assert!(err.is_err(), "debe fallar si el fichero no existe");
    }
}
