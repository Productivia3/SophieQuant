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
      insert into symbols (name, asset_class, exchange, tick_size, pip_size, commission,
                           contract_size, swap_long, swap_short,
                           spread_asia, spread_london, spread_ny) values
        ('EURUSD','forex',null,0.00001,0.0001,0.0,100000,0.0,0.0, 0.7, 1.0, 1.3),
        ('GBPUSD','forex',null,0.00001,0.0001,0.0,100000,0.0,0.0, 0.9, 1.2, 1.5),
        ('USDJPY','forex',null,0.001,0.01,0.0,100000,0.0,0.0, 0.7, 1.0, 1.3);"#)?;
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

/// Configuracion del motor a partir de un **simbolo** (para la build, que aun
/// no tiene estrategia).
fn cfg_for_symbol_id(db: &Connection, symbol_id: i64) -> EngineCfg {
    let mut cfg = EngineCfg::default();
    let fila: Option<(f64, f64, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>)> = db
        .query_row(
            "select tick_size, pip_size, commission, contract_size, swap_long, swap_short,
                    spread_asia, spread_london, spread_ny
             from symbols where id = ?1",
            q![symbol_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
        )
        .optional()
        .unwrap_or(None);
    if let Some((tick, pip, comision, contrato, swap_l, swap_s, asia, london, ny)) = fila {
        cfg.tick_size = tick;
        cfg.pip_size = pip;
        cfg.commission_per_lot = comision;
        cfg.contract_size = contrato;
        cfg.swap_long = swap_l;
        cfg.swap_short = swap_s;
        cfg.spread_asia = asia;
        cfg.spread_london = london;
        cfg.spread_ny = ny;
    }
    cfg
}

/// Configuracion del motor **de la estrategia**, tomada de su simbolo.
///
/// Antes el motor usaba siempre `EngineCfg::default()`: pip 0,0001 y 1 lote =
/// 100 000 unidades para todo. Eso hacia que en USDJPY el spread se aplicara
/// 100 veces menor de lo real y que en futuros o acciones el P&L fuera erroneo.
/// Ahora se leen los metadatos del simbolo (tick, pip, comision, contract size,
/// swaps y spreads por sesion) y se combinan con lo que si es de la estrategia
/// (balance, lotes, gestion de salida).
fn cfg_for_symbol(db: &Connection, strategy_id: i64) -> EngineCfg {
    let mut cfg = EngineCfg::default();
    // lo guardado en la ultima corrida (balance, lotes, stop/objetivo)
    let engine_guardado: Option<String> = db
        .query_row(
            "select engine_cfg from backtest_runs where strategy_id = ?1 order by id desc limit 1",
            q![strategy_id],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
        .unwrap_or(None)
        .flatten();
    if let Some(s) = engine_guardado {
        if let Ok(guardado) = serde_json::from_str::<EngineCfg>(&s) {
            cfg.initial_balance = guardado.initial_balance;
            cfg.fixed_lots = guardado.fixed_lots;
            cfg.slippage = guardado.slippage;
            cfg.sl_atr_mult = guardado.sl_atr_mult;
            cfg.tp_atr_mult = guardado.tp_atr_mult;
        }
    }
    // y ahora los datos de mercado del simbolo
    let sym: Option<i64> = db
        .query_row(
            "select symbol_id from strategies where id = ?1",
            q![strategy_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    if let Some(sym) = sym {
        let fila: Option<(f64, f64, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>)> = db
            .query_row(
                "select tick_size, pip_size, commission, contract_size, swap_long, swap_short,
                        spread_asia, spread_london, spread_ny
                 from symbols where id = ?1",
                q![sym],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
            )
            .optional()
            .unwrap_or(None);
        if let Some((tick, pip, comision, contrato, swap_l, swap_s, asia, london, ny)) = fila {
            cfg.tick_size = tick;
            cfg.pip_size = pip;
            cfg.commission_per_lot = comision;
            cfg.contract_size = contrato;
            cfg.swap_long = swap_l;
            cfg.swap_short = swap_s;
            cfg.spread_asia = asia;
            cfg.spread_london = london;
            cfg.spread_ny = ny;
            // si no hay spreads por sesion, el spread guardado manda
            if asia.is_none() && london.is_none() && ny.is_none() {
                // se mantiene el spread de la configuracion guardada
            }
        }
    }
    cfg
}

/// Metadatos de mercado de un simbolo (los que el motor usa de verdad).
#[tauri::command]
pub fn get_symbol_meta(state: State<AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut stmt = db
        .prepare(
            "select id, name, asset_class, tick_size, pip_size, commission, contract_size,
                    swap_long, swap_short, spread_asia, spread_london, spread_ny
             from symbols order by name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "assetClass": r.get::<_, String>(2)?,
                "tickSize": r.get::<_, f64>(3)?,
                "pipSize": r.get::<_, f64>(4)?,
                "commission": r.get::<_, f64>(5)?,
                "contractSize": r.get::<_, f64>(6)?,
                "swapLong": r.get::<_, f64>(7)?,
                "swapShort": r.get::<_, f64>(8)?,
                "spreadAsia": r.get::<_, Option<f64>>(9)?,
                "spreadLondon": r.get::<_, Option<f64>>(10)?,
                "spreadNy": r.get::<_, Option<f64>>(11)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(serde_json::json!({ "symbols": out }))
}

/// Actualiza los metadatos de mercado de un simbolo.
///
/// Es imprescindible poder editarlos: los swaps del broker cambian cada semana
/// y el spread depende de la sesion y del momento.
#[tauri::command]
pub fn update_symbol_meta(
    state: State<AppState>,
    symbol_id: i64,
    tick_size: Option<f64>,
    pip_size: Option<f64>,
    commission: Option<f64>,
    contract_size: Option<f64>,
    swap_long: Option<f64>,
    swap_short: Option<f64>,
    spread_asia: Option<f64>,
    spread_london: Option<f64>,
    spread_ny: Option<f64>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    db.execute(
        "update symbols set
            tick_size = coalesce(?2, tick_size),
            pip_size = coalesce(?3, pip_size),
            commission = coalesce(?4, commission),
            contract_size = coalesce(?5, contract_size),
            swap_long = coalesce(?6, swap_long),
            swap_short = coalesce(?7, swap_short),
            spread_asia = ?8,
            spread_london = ?9,
            spread_ny = ?10
         where id = ?1",
        q![symbol_id, tick_size, pip_size, commission, contract_size, swap_long, swap_short,
           spread_asia, spread_london, spread_ny],
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
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
) -> Result<serde_json::Value, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).ok();
    let db = state.db.lock();
    let (dataset, calidad) =
        import_csv_registered(&db, &dir, symbol_id, &timeframe, std::path::Path::new(&path))?;
    Ok(serde_json::json!({ "dataset": dataset, "calidad": calidad }))
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
) -> Result<(DataFileRow, serde_json::Value), String> {
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
    let calidad = serde_json::json!({
        "nBars": report.n_bars,
        "skipped": report.skipped,
        "dateFrom": report.date_from,
        "dateTo": report.date_to,
        "avisos": report.avisos,
        "huecos": report.huecos,
        "huecoMax": report.hueco_max,
        "sinVolumen": report.sin_volumen,
        "preciosIncoherentes": report.precios_incoherentes,
        "timestampsRepetidos": report.timestamps_repetidos,
        "limpio": report.avisos.len() == 1 && report.avisos[0].starts_with("sin avisos"),
    });
    Ok((
        DataFileRow {
            id, symbol_id, timeframe: timeframe.to_string(), storage_kind: "imported_csv".into(),
            compressed_bytes: 0, n_bars: report.n_bars,
            date_from: Some(report.date_from), date_to: Some(report.date_to),
            status: "ready".into(), created_at: now_ts(),
        },
        calidad,
    ))
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
    db_path: String,
}
impl BuildReporter for TauriReporter {
    fn report(&self, level: &str, msg: &str, passed: u32, total: u32, progress: f64) {
        let progress_db = open_thread_db(&self.db_path)
            .and_then(|db| persist_build_progress(&db, self.build_id, passed, total));
        if let Err(e) = progress_db {
            let _ = self.app.emit("build-event", BuildEvent {
                build_id: self.build_id,
                level: "error",
                message: format!("no se pudo guardar el progreso del build: {e}"),
                passed,
                total,
                progress,
            });
        }
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

    // la build se mide con los parametros REALES del simbolo (pip, contrato,
    // spread por sesion, swaps): antes se usaba el de por defecto para todos
    let engine_cfg = {
        let db = state.db.lock();
        cfg_for_symbol_id(&db, config.symbol_id)
    };

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
        let reporter = TauriReporter { app: app2.clone(), build_id, db_path: db_path.clone() };
        let conn = match open_thread_db(&db_path) {
            Ok(c) => c,
            Err(e) => {
                let message = format!("No se pudo iniciar el motor del build: {e}");
                if let Ok(db) = Connection::open(&db_path) {
                    if let Err(update_error) = finish_build(&db, build_id, "error", 0, total, &message) {
                        reporter.report(
                            "err",
                            &format!("{message}; tampoco se pudo persistir el error: {update_error}"),
                            0,
                            total,
                            1.0,
                        );
                        return;
                    }
                }
                reporter.report("err", &message, 0, total, 1.0);
                return;
            }
        };

        let outcome = genetic::build(&bars, &engine_cfg, &config, &reporter);

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
                       "calificado": s.robust.calificado,
                       "datasetId": data_file_id,
                   }).to_string()],
            )?;
            Ok(th.last_insert_rowid())
        };

        let mut passed_n = 0u32;
        for s in &outcome.strategies {
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
        let final_message = format!(
            "{}\nGuardadas: {passed_n} de {} estrategias.",
            outcome.summary, config.n_target
        );
        if let Err(e) = finish_build(&conn, build_id, final_status, passed_n, total, &final_message) {
            reporter.report("err", &format!("no se pudo guardar el resultado final del build: {e}"), passed_n, total, 1.0);
        }
        reporter.report("info", &final_message, passed_n, total, 1.0);
    });

    Ok(build_id)
}

fn open_thread_db(path: &str) -> rusqlite::Result<Connection> {
    let c = datastore::open(std::path::Path::new(path))?;
    datastore::init_schema(&c)?;
    Ok(c)
}

fn persist_build_progress(
    db: &Connection,
    build_id: i64,
    passed: u32,
    total: u32,
) -> rusqlite::Result<()> {
    db.execute(
        "update builds set n_passed = ?1, n_total = ?2 where id = ?3",
        q![passed, total, build_id],
    )?;
    Ok(())
}

fn finish_build(
    db: &Connection,
    build_id: i64,
    status: &str,
    passed: u32,
    total: u32,
    summary: &str,
) -> rusqlite::Result<()> {
    db.execute(
        "update builds set status = ?1, n_passed = ?2, n_total = ?3, finished_at = ?4, result_summary = ?5 where id = ?6",
        q![status, passed, total, now_ts(), summary, build_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn list_builds(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db = state.db.lock();
    let mut st = db.prepare(
        "select id, kind, status, n_passed, n_total, started_at, finished_at, result_summary from builds order by id desc").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?, "kind": r.get::<_, String>(1)?,
            "status": r.get::<_, String>(2)?, "nPassed": r.get::<_, i64>(3)?,
            "nTotal": r.get::<_, i64>(4)?, "startedAt": r.get::<_, Option<i64>>(5)?,
            "finishedAt": r.get::<_, Option<i64>>(6)?,
            "resultSummary": r.get::<_, Option<String>>(7)?,
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
    list_bank_rows(&db)
}

fn list_bank_rows(db: &Connection) -> Result<Vec<serde_json::Value>, String> {
    let mut st = db.prepare(
        "select s.id, s.name, sy.name, s.timeframe, s.status,
                b.net_profit, b.profit_factor, b.win_rate, b.max_dd, b.n_trades, b.sharpe, b.equity, b.oos_frac,
                s.robust_score, s.robust_detail, b.data_file_id
         from strategies s
         join symbols sy on sy.id = s.symbol_id
         left join backtest_runs b on b.strategy_id = s.id and b.id = (
            select id from backtest_runs where strategy_id = s.id order by id desc limit 1)
         order by (b.net_profit is null), b.net_profit desc").map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| {
        let equity_raw: Option<String> = r.get(11).ok();
        let robust_detail: Option<String> = r.get(14)?;
        let run_dataset_id: Option<i64> = r.get(15)?;
        let robust_qualified = robust_detail
            .and_then(|detail| serde_json::from_str::<serde_json::Value>(&detail).ok())
            .map(|detail| {
                detail["calificado"].as_bool() == Some(true)
                    && run_dataset_id.is_some()
                    && detail["datasetId"].as_i64() == run_dataset_id
            })
            .unwrap_or(false);
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
            "robustScore": if robust_qualified { r.get::<_, Option<f64>>(13)? } else { None },
            "lastRun": {
                "equity": equity_raw.and_then(|e| serde_json::from_str::<Vec<f64>>(&e).ok()),
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
pub fn export_strategy(
    state: State<AppState>,
    strategy_id: i64,
    platform: String,
) -> Result<serde_json::Value, String> {
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
    // la configuracion real con la que se corrio el backtest: los supuestos del
    // export tienen que ser los de verdad, no unos genericos
    let cfg_raw: Option<String> = db
        .query_row(
            "select engine_cfg from backtest_runs where strategy_id = ?1 order by id desc limit 1",
            q![strategy_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let cfg: EngineCfg = cfg_raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let verificacion = export::verification(tmpl, params, &cfg);

    drop(db);
    let db2 = state.db.lock();
    db2.execute(
        "insert into exports (strategy_id, platform, file_name) values (?1, ?2, ?3)",
        q![strategy_id, format!("{platform}"), format!("{name}.{platform}")],
    ).map_err(|e| e.to_string())?;
    drop(db2);
    Ok(serde_json::json!({ "code": code, "verificacion": verificacion }))
}

// ── settings mínimos ─────────────────────────────────────────────────

/// Resamplea un dataset a otra temporalidad mayor y lo registra como dataset nuevo.
///
/// Con un solo historico m1 se pueden sacar m5, m15, h1, d1... sin volver a
/// descargar nada.
#[tauri::command]
pub fn resample_dataset(
    app: AppHandle,
    state: State<AppState>,
    data_file_id: i64,
    target_tf: String,
) -> Result<DataFileRow, String> {
    let (symbol_id, source_tf, path) = {
        let db = state.db.lock();
        let row: (i64, String, String) = db
            .query_row(
                "select symbol_id, timeframe, file_path from data_files where id = ?1",
                q![data_file_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("dataset {data_file_id} no encontrado"))?;
        row
    };
    let bars = datastore::Qfn::read(std::path::Path::new(&path))
        .map_err(|e| format!("no se pudo leer el dataset: {e}"))?;
    let resampleados = datastore::resample(&bars, &source_tf, &target_tf)?;
    if resampleados.is_empty() {
        return Err("el resample no ha producido barras".into());
    }
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).ok();
    let nombre = format!("resample_{}_{}", data_file_id, target_tf);
    let qfn_path = dir.join(format!("{nombre}.qfn"));
    datastore::Qfn::write(&qfn_path, &resampleados).map_err(|e| e.to_string())?;

    let date_from = resampleados.first().map(|b| b.time).unwrap_or(0);
    let date_to = resampleados.last().map(|b| b.time).unwrap_or(0);
    let db = state.db.lock();
    db.execute(
        "insert into data_files (symbol_id, timeframe, storage_kind, file_path, compressed_bytes, n_bars, date_from, date_to, status, created_at)
         values (?1, ?2, 'local_qfd', ?3, ?4, ?5, ?6, ?7, 'ready', ?8)
         on conflict do update set file_path = excluded.file_path,
            compressed_bytes = excluded.compressed_bytes, n_bars = excluded.n_bars,
            date_from = excluded.date_from, date_to = excluded.date_to, status = 'ready'",
        q![symbol_id, target_tf, qfn_path.to_string_lossy(),
           std::fs::metadata(&qfn_path).map(|m| m.len() as i64).unwrap_or(0),
           resampleados.len() as i64, date_from, date_to, now_ts()],
    )
    .map_err(|e| e.to_string())?;
    let id = db.last_insert_rowid();
    drop(db);
    Ok(DataFileRow {
        id, symbol_id, timeframe: target_tf, storage_kind: "local_qfd".into(),
        compressed_bytes: std::fs::metadata(&qfn_path).map(|m| m.len() as i64).unwrap_or(0),
        n_bars: resampleados.len() as i64,
        date_from: Some(date_from), date_to: Some(date_to),
        status: "ready".into(), created_at: now_ts(),
    })
}

/// Temporalidades a las que se puede resamplear un dataset.
#[tauri::command]
pub fn list_resample_targets(timeframe: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "targets": datastore::tf_targets(&timeframe),
        "origen": timeframe,
    }))
}

// ── Custom Analysis: enlace con scripts de Python ──────────────────

/// Ejecuta un script de Python sobre los datos de una estrategia.
///
/// El script lo elige el usuario. Se invoca SIN shell (no se concatenan
/// comandos) y con un tiempo limite; si se cuelga, se le mata. La salida se
/// valida antes de trustarla (solo campos conocidos).
#[tauri::command(async)]
pub fn run_custom_analysis(
    app: AppHandle,
    state: State<AppState>,
    strategy_id: i64,
    script_path: String,
) -> Result<serde_json::Value, String> {
    let script = std::path::Path::new(&script_path);
    if !script.exists() {
        return Err(format!("no encuentro el script '{}'", script_path));
    }
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let work = dir.join("custom_analysis");
    std::fs::create_dir_all(&work).ok();

    // ── entrada: estrategia + CSV con las barras ───────────────────
    let (core, bars_path) = {
        let db = state.db.lock();
        let core = load_core(&db, strategy_id)?;
        let path: String = db
            .query_row(
                "select df.file_path from backtest_runs b
                 join data_files df on df.id = b.data_file_id
                 where b.strategy_id = ?1 order by b.id desc limit 1",
                q![strategy_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        (core, path)
    };
    // el CSV de barras lo lee el script: se exporta desde el QFN
    let csv_path = work.join(format!("strategy_{}_bars.csv", strategy_id));
    if !bars_path.is_empty() {
        if let Ok(bars) = datastore::Qfn::read(std::path::Path::new(&bars_path)) {
            let mut s = String::from("time,open,high,low,close,volume\n");
            for b in &bars {
                s.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    b.time, b.open, b.high, b.low, b.close, b.volume
                ));
            }
            std::fs::write(&csv_path, s).map_err(|e| e.to_string())?;
        }
    }
    let input_path = work.join(format!("strategy_{}_input.json", strategy_id));
    let output_path = work.join(format!("strategy_{}_output.json", strategy_id));
    let contrato = crate::engine::analysis::contract(&core, &csv_path, 0);
    std::fs::write(&input_path, contrato.to_string()).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&output_path);

    // ── ejecucion: python <script> --input <json> --output <json> ──
    let mut cmd = std::process::Command::new("python");
    cmd.arg(script)
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .current_dir(&work)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = cmd.spawn().map_err(|e| format!("no se pudo lanzar python: {e}"))?;

    // espera con tiempo limite (60 s)
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let salida = child.wait_with_output().ok();
                    let err = salida
                        .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
                        .unwrap_or_default();
                    let primera = err.trim().lines().next().unwrap_or("").to_string();
                    let detalle = if primera.is_empty() {
                        String::new()
                    } else {
                        format!(": {primera}")
                    };
                    return Err(format!("el script termino con error{detalle}"));
                }
                break;
            }
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    let _ = child.kill();
                    return Err("el script no ha terminado en 60 s: se ha detenido".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(120));
            }
            Err(e) => return Err(format!("error esperando al script: {e}")),
        }
    }

    // ── salida ─────────────────────────────────────────────────────
    let raw_txt = std::fs::read_to_string(&output_path).map_err(|e| {
        format!("el script no escribio el fichero de salida ({e}): debe crear --output con un JSON")
    })?;
    let raw: serde_json::Value = serde_json::from_str(&raw_txt)
        .map_err(|e| format!("la salida del script no es un JSON valido: {e}"))?;
    let out = crate::engine::analysis::normalize_output(&raw);

    let db = state.db.lock();
    let id = crate::engine::analysis::store(&db, strategy_id, &script_path, &out, &contrato)?;
    drop(db);
    Ok(serde_json::json!({
        "id": id,
        "resultado": out,
        "input": contrato,
        "script": script_path,
    }))
}

/// Historial de analisis personalizados.
#[tauri::command]
pub fn list_custom_analyses(
    state: State<AppState>,
    strategy_id: Option<i64>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let rows = crate::engine::analysis::list(&db, strategy_id, limit.unwrap_or(50))?;
    Ok(serde_json::json!({ "rows": rows }))
}

/// Escribe un CSV con las barras de una estrategia (para scripts o para mirar).
#[tauri::command]
pub fn export_strategy_csv(
    app: AppHandle,
    state: State<AppState>,
    strategy_id: i64,
) -> Result<String, String> {
    let path: String = {
        let db = state.db.lock();
        db.query_row(
            "select df.file_path from backtest_runs b
             join data_files df on df.id = b.data_file_id
             where b.strategy_id = ?1 order by b.id desc limit 1",
            q![strategy_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_default()
    };
    if path.is_empty() {
        return Err("esta estrategia no tiene dataset cargado".into());
    }
    let bars = datastore::Qfn::read(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("exports");
    std::fs::create_dir_all(&dir).ok();
    let nombre: String = {
        let db = state.db.lock();
        db.query_row("select name from strategies where id = ?1", q![strategy_id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| format!("estrategia_{strategy_id}"))
    };
    let out = dir.join(format!("{nombre}_bars.csv"));
    let mut s = String::from("time,open,high,low,close,volume\n");
    for b in &bars {
        s.push_str(&format!("{},{},{},{},{},{}\n", b.time, b.open, b.high, b.low, b.close, b.volume));
    }
    std::fs::write(&out, s).map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().to_string())
}

/// Sugiere que estrategias meter en la cartera, con el motivo de cada descarte.
#[tauri::command]
pub fn suggest_portfolio(
    state: State<AppState>,
    candidate_ids: Vec<i64>,
    max_correlation: Option<f64>,
    max_members: Option<usize>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let mut ids: Vec<i64> = Vec::new();
    let mut nombres: Vec<(i64, String)> = Vec::new();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    let mut metricas: Vec<Option<crate::engine::types::Metrics>> = Vec::new();
    for id in &candidate_ids {
        let nombre: String = db
            .query_row("select name from strategies where id = ?1", q![*id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| format!("#{id}"));
        if let Some(eq) = ops::stored_equity(&db, *id) {
            ids.push(*id);
            nombres.push((*id, nombre));
            seqs.push(eq);
            metricas.push(ops::stored_metrics(&db, *id));
        }
    }
    let max_corr = max_correlation.unwrap_or(0.7).clamp(0.0, 1.0);
    let max_mem = max_members.unwrap_or(8).clamp(2, 12);
    Ok(ops::suggest_portfolio(&ids, &nombres, &seqs, &metricas, max_corr, max_mem))
}

/// Perfil de riesgo de una estrategia: MAE/MFE, expectativa y capital diario.
#[tauri::command]
pub fn analyze_strategy_risk(
    state: State<AppState>,
    strategy_id: i64,
) -> Result<serde_json::Value, String> {
    let (core, bars_path, cfg) = {
        let db = state.db.lock();
        let core = load_core(&db, strategy_id)?;
        let path: String = db
            .query_row(
                "select df.file_path from backtest_runs b
                 join data_files df on df.id = b.data_file_id
                 where b.strategy_id = ?1 order by b.id desc limit 1",
                q![strategy_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        // la configuracion real del simbolo (pip, contrato, swap, spread por sesion)
        let cfg = cfg_for_symbol(&db, strategy_id);
        (core, path, cfg)
    };
    if bars_path.is_empty() {
        return Err("esta estrategia no tiene dataset cargado".into());
    }
    let bars = datastore::Qfn::read(std::path::Path::new(&bars_path)).map_err(|e| e.to_string())?;
    ops::risk_profile(&bars, core.template, core.params, &cfg)
}

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
// Cuatro pruebas por estrategia: walk-forward, Monte Carlo (6 tests), SPP y
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
                     where strategy_id = ?1 and check_key = ?2 and method_version = ?3
                     order by id desc limit 1",
                    q![id, key, crate::engine::verdicts::ROBUSTNESS_VERSION],
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
                        "estado": "pending", "detalle": "método actualizado: vuelve a calcular para validar con los criterios estrictos", "valor": serde_json::Value::Null,
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
    let mut guard = state.db.lock();
    let db = &mut *guard;
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for id in strategy_ids {
        // cada estrategia con la configuracion de SU simbolo
        let cfg = cfg_for_symbol(db, id);
        let name: Option<String> = db
            .query_row("select name from strategies where id = ?1", q![id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let robust_detail: Option<String> = db
            .query_row("select robust_detail from strategies where id = ?1", q![id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let run_dataset_id: Option<i64> = db
            .query_row(
                "select data_file_id from backtest_runs where strategy_id = ?1 order by id desc limit 1",
                q![id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();
        let searched_on_is = robust_detail
            .and_then(|detail| serde_json::from_str::<serde_json::Value>(&detail).ok())
            .map(|detail| {
                detail["calificado"].as_bool() == Some(true)
                    && run_dataset_id.is_some()
                    && detail["datasetId"].as_i64() == run_dataset_id
            })
            .unwrap_or(false);
        let checks = match ops::load_strategy(db, id) {
            Ok(Some(core)) => crate::engine::verdicts::for_strategy(&core, &cfg, searched_on_is),
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
                "insert into strategy_verdicts (strategy_id, check_key, estado, detalle, valor, calculado_en, method_version)
                 values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                q![
                    id,
                    key,
                    c["estado"].as_str().unwrap_or("pending"),
                    c["detalle"].as_str().unwrap_or(""),
                    c["valor"].as_f64(),
                    now,
                    crate::engine::verdicts::ROBUSTNESS_VERSION,
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


// ── Historial de analisis ─────────────────────────────────────────────
//
// Cada analisis deja registro con su veredicto y sus metricas, para poder
// comparar el antes y el despues de cambiar una regla.

/// Registra una corrida (si falla, no interrumpe el analisis: el historial es
/// informativo, el resultado del comando es lo importante).
fn record_run(
    db: &Connection,
    strategy_id: i64,
    kind: &str,
    verdict: &str,
    label: &str,
    metrics: serde_json::Value,
    params: serde_json::Value,
    detail: &serde_json::Value,
) {
    let _ = crate::engine::history::record(db, strategy_id, kind, verdict, label, &metrics, &params, detail);
}

/// Extrae un conjunto plano de metricas de un objeto de resultado.
fn pick_metrics(src: &serde_json::Value, keys: &[&str]) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for k in keys {
        if let Some(v) = src.get(*k) {
            out.insert((*k).to_string(), v.clone());
        }
    }
    serde_json::Value::Object(out)
}

#[tauri::command]
pub fn list_analysis_runs(
    state: State<AppState>,
    strategy_id: Option<i64>,
    kind: Option<String>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let lim = limit.unwrap_or(200).clamp(1, 1000);
    let rows = crate::engine::history::list(&db, strategy_id, kind.as_deref(), lim)?;

    // anade a cada fila el delta frente a la corrida anterior del mismo tipo
    let mut out: Vec<serde_json::Value> = Vec::new();
    for r in rows {
        let sid = r["strategyId"].as_i64().unwrap_or(0);
        let k = r["kind"].as_str().unwrap_or("");
        let id = r["id"].as_i64().unwrap_or(0);
        let mut row = r.clone();
        match crate::engine::history::previous_of(&db, sid, k, id) {
            Ok(Some(prev)) => {
                let cmp = crate::engine::history::compare(&row, &serde_json::json!({ "metrics": prev }));
                row["comparacion"] = cmp["metricas"].clone();
                row["hayPrevia"] = serde_json::Value::Bool(true);
            }
            _ => {
                row["comparacion"] = serde_json::Value::Array(vec![]);
                row["hayPrevia"] = serde_json::Value::Bool(false);
            }
        }
        out.push(row);
    }
    Ok(serde_json::json!({
        "rows": out,
        "kinds": crate::engine::history::KINDS
            .iter()
            .map(|k| serde_json::json!({ "key": k, "label": crate::engine::history::kind_label(k) }))
            .collect::<Vec<_>>(),
    }))
}

#[tauri::command]
pub fn delete_analysis_run(state: State<AppState>, id: i64) -> Result<bool, String> {
    let db = state.db.lock();
    crate::engine::history::delete(&db, id)
}

#[tauri::command(async)]
pub fn run_retest(
    state: State<AppState>,
    strategy_ids: Vec<i64>,
    symbol_ids: Vec<i64>,
    timeframe: String,
    periodos: Option<usize>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock();

    // datos de cada mercado destino (si no hay dataset, se marca "sin datos")
    let mut mercados: Vec<(i64, String, Option<Vec<Bar>>)> = Vec::new();
    for sid in &symbol_ids {
        let path: Option<String> = db
            .query_row(
                "select file_path from data_files
                 where symbol_id = ?1 and timeframe = ?2 and status = 'ready'
                 order by n_bars desc limit 1",
                q![sid, timeframe],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let nombre: String = db
            .query_row("select name from symbols where id = ?1", q![sid], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| format!("#{sid}"));
        let bars = match path {
            Some(p) => Qfn::read(std::path::Path::new(&p)).ok(),
            None => None,
        };
        mercados.push((*sid, nombre, bars));
    }

    let mut rows: Vec<serde_json::Value> = Vec::new();
    for id in strategy_ids {
        let orig = ops::stored_metrics(&db, id);
        let core = load_core(&db, id);
        // cada estrategia se mide con la configuracion de SU simbolo
        let cfg = cfg_for_symbol(&db, id);

        // 1) out-of-sample dentro de su propio mercado, en k periodos
        let k = periodos.unwrap_or(1).clamp(1, 12);
        let origen = match &core {
            Ok(c) => match ops::retest_periods(&c.bars, c.template, c.params, &cfg, k) {
                Ok(v) => {
                    // el primer periodo OOS es el que se muestra como "retest"
                    let primer = v["periodos"].as_array().and_then(|ps| ps.first().cloned());
                    serde_json::json!({
                        "original": orig.as_ref().map(ops::metrics_json),
                        "retest": primer.as_ref().map(|p| p["metrics"].clone()).unwrap_or(serde_json::Value::Null),
                        "verdict": primer.as_ref().map(|p| p["verdict"].clone()).unwrap_or(serde_json::json!("fail")),
                        "note": primer.as_ref().map(|p| p["nota"].clone()).unwrap_or(serde_json::json!("")),
                        "periodos": v["periodos"].clone(),
                        "consistencia": v["consistencia"].clone(),
                    })
                }
                Err(e) => serde_json::json!({
                    "original": orig.as_ref().map(ops::metrics_json),
                    "retest": serde_json::Value::Null, "verdict": "fail", "note": e,
                    "periodos": serde_json::Value::Array(vec![]),
                    "consistencia": serde_json::json!({
                        "verdict": "fail", "detalle": "no se pudo evaluar", "pasados": 0, "total": k,
                    }),
                }),
            },
            Err(e) => serde_json::json!({
                "original": orig.as_ref().map(ops::metrics_json),
                "retest": serde_json::Value::Null, "verdict": "fail", "note": e,
            }),
        };

        // 2) la misma regla en cada mercado destino
        let mut celdas: Vec<serde_json::Value> = Vec::new();
        let mut veredictos: Vec<(String, String)> = Vec::new();
        for (sid, nombre, bars) in &mercados {
            let celda = match (core.as_ref(), bars) {
                (Ok(c), Some(b)) => match ops::retest_on_market(b, c.template, c.params, &cfg) {
                    Ok(v) => serde_json::json!({
                        "symbolId": sid, "simbolo": nombre,
                        "verdict": v["verdict"], "nota": v["nota"],
                        "retest": v["retest"], "nBars": v["nBars"],
                    }),
                    Err(e) => serde_json::json!({
                        "symbolId": sid, "simbolo": nombre, "verdict": "fail",
                        "nota": e, "retest": serde_json::Value::Null,
                    }),
                },
                (_, None) => serde_json::json!({
                    "symbolId": sid, "simbolo": nombre, "verdict": "sin datos",
                    "nota": format!("sin datos de {nombre} en {timeframe}"),
                    "retest": serde_json::Value::Null,
                }),
                (Err(e), _) => serde_json::json!({
                    "symbolId": sid, "simbolo": nombre, "verdict": "fail",
                    "nota": e.to_string(), "retest": serde_json::Value::Null,
                }),
            };
            veredictos.push((nombre.clone(), celda["verdict"].as_str().unwrap_or("fail").to_string()));
            celdas.push(celda);
        }

        let resumen = ops::verdict_mercados(&veredictos);
        let m = pick_metrics(&origen["retest"], &["netProfit", "profitFactor", "maxDD", "nTrades"]);
        let mut metricas = m.as_object().cloned().unwrap_or_default();
        metricas.insert("mercados".into(), serde_json::json!(resumen["mercadosConDatos"]));
        metricas.insert("sobrevive".into(), serde_json::json!(resumen["pass"]));
        let mercados_nombre: Vec<String> = mercados.iter().map(|(_, n, _)| n.clone()).collect();
        record_run(
            &db, id, "retest",
            resumen["verdict"].as_str().unwrap_or("pending"),
            &format!("{} · {}", mercados_nombre.join(", "), timeframe),
            serde_json::Value::Object(metricas),
            serde_json::json!({ "simbolos": mercados_nombre, "timeframe": timeframe }),
            &serde_json::json!({ "origen": origen, "mercados": celdas, "resumen": resumen }),
        );
        rows.push(serde_json::json!({
            "strategyId": id,
            "origen": origen,
            "mercados": celdas,
            "resumen": resumen,
        }));
    }
    drop(db);
    Ok(serde_json::json!({
        "timeframe": timeframe,
        "mercados": mercados.iter().map(|(id, n, b)| serde_json::json!({
            "symbolId": id, "simbolo": n, "conDatos": b.is_some(),
        })).collect::<Vec<_>>(),
        "rows": rows,
    }))
}

#[tauri::command(async)]
pub fn run_improver(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::improver(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    // historial: la mejor variante y cuanto mejora
    let mejor = v["variants"]
        .as_array()
        .and_then(|vs| vs.iter().max_by(|a, b| {
            a["improvement"].as_f64().unwrap_or(f64::MIN)
                .partial_cmp(&b["improvement"].as_f64().unwrap_or(f64::MIN))
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
        .cloned();
    let improvement = mejor.as_ref().and_then(|m| m["improvement"].as_f64()).unwrap_or(0.0);
    let verdict = if improvement > 0.0 { "pass" } else { "warn" };
    let metricas = pick_metrics(
        mejor.as_ref().map(|m| &m["metrics"]).unwrap_or(&serde_json::Value::Null),
        &["netProfit", "profitFactor", "maxDD", "nTrades"],
    );
    let mut metricas = metricas.as_object().cloned().unwrap_or_default();
    metricas.insert("mejora".into(), serde_json::json!((improvement * 100.0).round() / 100.0));
    record_run(
        &db, strategy_id, "improver", verdict,
        v["bestVariant"].as_str().unwrap_or("sin mejora"),
        serde_json::Value::Object(metricas),
        serde_json::json!({}),
        &v,
    );
    drop(db);
    Ok(v)
}

#[tauri::command(async)]
pub fn run_optimizer(state: State<AppState>, strategy_id: i64, param_a: String, param_b: String, steps: i64, kind: String) -> Result<serde_json::Value, String> {
    // (el registro del historial se anade al final de la funcion)
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let g = ops::run_grid(&core, &EngineCfg::default(), steps as usize, &kind)?;
    let names = core.template.param_names();
    let param_a = if names[0] == "—" { param_a } else { names[0].to_string() };
    let param_b = if names[1] == "—" { param_b } else { names[1].to_string() };
    // historial: la mejor combinacion de la rejilla
    let best = g["best"].clone();
    let score = if kind == "wf" {
        best["wfe"].as_f64().unwrap_or(0.0)
    } else {
        best["pf"].as_f64().unwrap_or(0.0)
    };
    let (verdict, metrics) = if kind == "wf" {
        let verdict = if score >= 50.0 { "pass" } else if score >= 35.0 { "warn" } else { "fail" };
        (verdict, pick_metrics(&best, &["wfe", "maxDD", "a", "b"]))
    } else {
        let verdict = if score >= 1.15 { "pass" } else if score >= 0.95 { "warn" } else { "fail" };
        (verdict, pick_metrics(&best, &["pf", "maxDD", "a", "b"]))
    };
    record_run(
        &db, strategy_id, "optimizer", verdict,
        &format!("{} = {} · {} = {}", param_a, best["a"], param_b, best["b"]),
        metrics,
        serde_json::json!({ "paramA": param_a, "paramB": param_b, "steps": steps, "kind": kind }),
        &serde_json::json!({ "aValues": g["aValues"], "bValues": g["bValues"], "best": best }),
    );
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
    let cfg = cfg_for_symbol(&db, strategy_id);
    let g = ops::run_grid(&core, &cfg, steps as usize, "wf")?;
    // historial: la mejor celda de la matriz y su veredicto
    let best = g["best"].clone();
    let verdict = best["verdict"].as_str().unwrap_or("pending").to_string();
    let metricas = pick_metrics(&best, &["wfe", "a", "b"]);

    let cut = (core.bars.len() * 6 / 10).max(60);
    let params = [
        best["a"].as_f64().ok_or("la mejor celda WF no contiene el parámetro A")?.max(2.0),
        best["b"].as_f64().ok_or("la mejor celda WF no contiene el parámetro B")?.max(2.0),
        core.params[2],
    ];
    let is = crate::engine::backtest::run(&core.bars[..cut], core.template, params, &cfg);
    let oos_bars = &core.bars[cut..];
    let oos = crate::engine::backtest::run(oos_bars, core.template, params, &cfg);
    let oos_years = match (oos_bars.first(), oos_bars.last()) {
        (Some(first), Some(last)) => {
            (last.time.saturating_sub(first.time) as f64 / 31_557_600.0).max(0.0)
        }
        _ => 0.0,
    };
    let (peak_is, _) = equity_peak_and_max_drawdown(
        is.trades.iter().map(|trade| trade.profit),
        cfg.initial_balance,
    );
    let (_, max_dd_oos) = equity_peak_and_max_drawdown(
        oos.trades.iter().map(|trade| trade.profit),
        cfg.initial_balance,
    );
    let oos_trades = oos.trades.len();
    let (insufficient_oos, stopped) =
        wfm_validation_flags(oos_trades, oos_years, peak_is, max_dd_oos);

    record_run(
        &db, strategy_id, "wf_matrix", &verdict,
        &format!("{} x {}", a, b),
        metricas,
        serde_json::json!({ "paramA": a, "paramB": b, "steps": steps }),
        &serde_json::json!({ "aValues": g["aValues"], "bValues": g["bValues"], "best": best }),
    );
    drop(db);
    Ok(serde_json::json!({
        "strategyId": strategy_id, "paramA": a, "paramB": b,
        "aValues": g["aValues"], "bValues": g["bValues"], "cells": g["cells"],
        "insufficientOos": insufficient_oos,
        "oosTradesApprox": oos_trades,
        "oosYearsApprox": (oos_years * 10.0).round() / 10.0,
        "stopped": stopped,
    }))
}

fn equity_peak_and_max_drawdown(
    profits: impl Iterator<Item = f64>,
    initial_balance: f64,
) -> (f64, f64) {
    let mut balance = initial_balance;
    let mut peak = initial_balance;
    let mut max_drawdown = 0.0_f64;
    for profit in profits {
        balance += profit;
        peak = peak.max(balance);
        max_drawdown = max_drawdown.max(peak - balance);
    }
    (peak, max_drawdown)
}

fn wfm_validation_flags(
    oos_trades: usize,
    oos_years: f64,
    peak_is: f64,
    max_dd_oos: f64,
) -> (bool, bool) {
    let insufficient_oos = oos_trades < 30 || oos_years < 1.0;
    let stopped = peak_is > 0.0 && max_dd_oos / peak_is > 0.8;
    (insufficient_oos, stopped)
}

#[tauri::command(async)]
pub fn run_spp(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::spp(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    // historial: el impacto maximo de las permutaciones
    let peor = v["results"]
        .as_array()
        .map(|rs| rs.iter().filter_map(|r| r["impactPct"].as_f64()).fold(0.0_f64, f64::max))
        .unwrap_or(0.0);
    let verdict = v["overall"].as_str().unwrap_or("pending");
    record_run(
        &db, strategy_id, "spp", verdict,
        &format!("impacto maximo {peor:.1} %"),
        serde_json::json!({ "impactoMax": (peor * 10.0).round() / 10.0 }),
        serde_json::json!({}),
        &v,
    );
    drop(db);
    Ok(v)
}

#[tauri::command]
pub async fn run_mc_suite(
    state: State<'_, AppState>,
    strategy_id: i64,
    tests: Vec<String>,
) -> Result<serde_json::Value, String> {
    let core = {
        let db = state.db.lock();
        load_core(&db, strategy_id)?
    };
    let tests_for_history = tests.clone();
    let mut v = tauri::async_runtime::spawn_blocking(move || {
        ops::mc_suite(&core, &EngineCfg::default(), &tests, 100)
    })
    .await
    .map_err(|e| format!("Falló la tarea de Monte Carlo: {e}"))??;
    v["strategyId"] = strategy_id.into();
    // historial: cuantas pruebas se superaron y la tasa media
    let res = v["results"].as_array().cloned().unwrap_or_default();
    let total = res.len();
    let pasadas = res.iter().filter(|r| r["passed"] == true).count();
    let media = if total == 0 {
        0.0
    } else {
        res.iter().filter_map(|r| r["passRate"].as_f64()).sum::<f64>() / total as f64
    };
    let verdict = if total > 0 && pasadas == total {
        "pass"
    } else if pasadas > 0 {
        "warn"
    } else {
        "fail"
    };
    let db = state.db.lock();
    record_run(
        &db, strategy_id, "monte_carlo", verdict,
        &format!("{pasadas}/{total} pruebas superadas"),
        serde_json::json!({
            "pruebas": total,
            "superadas": pasadas,
            "tasaMedia": (media * 100.0).round() / 100.0,
        }),
        serde_json::json!({ "tests": tests_for_history }),
        &v,
    );
    drop(db);
    Ok(v)
}

// ── Cartera: ponderacion y persistencia ────────────────────────────

/// Calculo de cartera con un modelo de ponderacion.
///
/// Devuelve por miembro: volatilidad, peso, peso en % y contribucion al riesgo;
/// y el resumen de la cartera (volatilidad, drawdown maximo, beneficio neto).
fn portfolio_calc(
    db: &Connection,
    strategy_ids: &[i64],
    model: &str,
    custom: &[f64],
) -> serde_json::Value {
    let mut ids: Vec<i64> = Vec::new();
    let mut seqs: Vec<Vec<f64>> = Vec::new();
    for id in strategy_ids {
        if let Some(eq) = ops::stored_equity(db, *id) {
            ids.push(*id);
            seqs.push(eq);
        }
    }
    let n = ids.len();
    if n == 0 {
        return serde_json::json!({ "model": model, "miembros": [], "combinada": serde_json::Value::Null });
    }
    let matrix = ops::corr_matrix(&seqs);
    let w = ops::weights(model, &seqs, custom);
    let rc = ops::risk_contributions(&seqs, &w);
    let combinada = ops::weighted_equity(&seqs, &w);
    let dd = ops::max_dd_pct(&combinada);
    let vol = ops::volatility(&combinada);

    let mut miembros: Vec<serde_json::Value> = Vec::new();
    for (k, id) in ids.iter().enumerate() {
        let m = ops::stored_metrics(db, *id);
        let nombre: String = db
            .query_row("select name from strategies where id = ?1", q![*id], |r| r.get(0))
            .unwrap_or_else(|_| format!("#{id}"));
        let avg_corr = if n > 1 {
            let off: Vec<f64> = matrix[k].iter().enumerate()
                .filter(|(i, _)| *i != k).map(|(_, v)| *v).collect();
            (ops::mean(&off)).max(0.0)
        } else { 0.0 };
        miembros.push(serde_json::json!({
            "strategyId": id,
            "nombre": nombre,
            "metrics": m.as_ref().map(ops::metrics_json),
            "correlation": avg_corr,
            "volatility": (ops::volatility(&seqs[k]) * 100.0 * 100.0).round() / 100.0,
            "weight": (w[k] * 1000.0).round() / 1000.0,
            "weightPct": (w[k] * 100.0 * 10.0).round() / 10.0,
            "riskContribution": (rc[k] * 10.0).round() / 10.0,
        }));
    }
    // metricas de la cartera combinada
    let net = if let Some(first) = seqs.first() {
        let ini = first.first().copied().unwrap_or(0.0);
        let fin = combinada.last().copied().unwrap_or(0.0);
        ((fin - ini) * 100.0).round() / 100.0
    } else { 0.0 };
    serde_json::json!({
        "model": model,
        "modelLabel": ops::model_label(model),
        "miembros": miembros,
        "combinada": {
            "equity": combinada,
            "maxDD": (dd * 10.0).round() / 10.0,
            "netProfit": net,
            "volatility": (vol * 100.0 * 100.0).round() / 100.0,
        },
        "portfolioVol": (vol * 100.0 * 100.0).round() / 100.0,
    })
}

/// Calcula la cartera con el modelo pedido y **guarda la corrida**.
#[tauri::command]
pub fn run_portfolio_weighting(
    state: State<AppState>,
    strategy_ids: Vec<i64>,
    model: Option<String>,
    custom_weights: Option<Vec<f64>>,
) -> Result<serde_json::Value, String> {
    let m = model.unwrap_or_else(|| "equal_vol".to_string());
    if !ops::MODELS.contains(&m.as_str()) {
        return Err(format!("modelo de ponderacion desconocido: '{m}'"));
    }
    let custom = custom_weights.unwrap_or_default();
    let db = state.db.lock();
    let calc = portfolio_calc(&db, &strategy_ids, &m, &custom);
    let ids: Vec<i64> = calc["miembros"]
        .as_array()
        .map(|ms| ms.iter().filter_map(|x| x["strategyId"].as_i64()).collect())
        .unwrap_or_default();
    let pesos: Vec<f64> = calc["miembros"]
        .as_array()
        .map(|ms| ms.iter().filter_map(|x| x["weight"].as_f64()).collect())
        .unwrap_or_default();
    let now = now_ts();
    db.execute(
        "insert into portfolio_runs (model, strategy_ids, weights, metrics, detail, created_at)
         values (?1, ?2, ?3, ?4, ?5, ?6)",
        q![
            m,
            serde_json::to_string(&ids).unwrap_or_default(),
            serde_json::to_string(&pesos).unwrap_or_default(),
            serde_json::to_string(&calc["combinada"]).unwrap_or_default(),
            serde_json::to_string(&calc["miembros"]).unwrap_or_default(),
            now,
        ],
    )
    .map_err(|e| e.to_string())?;
    let run_id = db.last_insert_rowid();
    drop(db);
    Ok(serde_json::json!({ "runId": run_id, "resultado": calc }))
}

/// Historial de corridas de cartera.
#[tauri::command]
pub fn list_portfolio_runs(state: State<AppState>, limit: Option<usize>) -> Result<serde_json::Value, String> {
    let db = state.db.lock();
    let lim = limit.unwrap_or(50).clamp(1, 500);
    let mut stmt = db.prepare(
        "select id, model, strategy_ids, weights, metrics, created_at
         from portfolio_runs order by id desc limit ?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(q![lim as i64], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "model": r.get::<_, String>(1)?,
            "modelLabel": ops::model_label(&r.get::<_, String>(1)?),
            "strategyIds": r.get::<_, Option<String>>(2)?.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(serde_json::Value::Null),
            "weights": r.get::<_, Option<String>>(3)?.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(serde_json::Value::Null),
            "metrics": r.get::<_, Option<String>>(4)?.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(serde_json::Value::Null),
            "createdAt": r.get::<_, i64>(5)?,
        }))
    }).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows { out.push(r.map_err(|e| e.to_string())?); }
    Ok(serde_json::json!({ "rows": out }))
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
    let (core, cfg) = {
        let db = state.db.lock();
        let core = load_core(&db, strategy_id)?;
        // la configuracion real del simbolo (pip, contrato, swap, spread por sesion)
        let cfg = cfg_for_symbol(&db, strategy_id);
        (core, cfg)
    };
    let out = crate::engine::backtest::run(&core.bars, core.template, core.params, &cfg);
    // velas muestreadas (hasta 400) y trades mapeados al índice de vela muestreada
    let step = (core.bars.len() / 400).max(1);
    let sampled: Vec<&Bar> = core.bars.iter().step_by(step).collect();
    let candles: Vec<serde_json::Value> = sampled.iter().map(|b| serde_json::json!({ "o": b.open, "h": b.high, "l": b.low, "c": b.close })).collect();

    // ── overlays: medias calculadas sobre TODAS las barras y muestreadas
    //    con el mismo paso que las velas, para que queden alineadas ──
    let closes: Vec<f64> = core.bars.iter().map(|b| b.close).collect();
    let mut overlays: Vec<serde_json::Value> = Vec::new();
    for (key, label, serie) in [
        ("ema20", "EMA 20", super::engine::indicators::ema(&closes, 20)),
        ("ema50", "EMA 50", super::engine::indicators::ema(&closes, 50)),
        ("sma200", "SMA 200", super::engine::indicators::sma(&closes, 200)),
    ] {
        if core.bars.len() < 50 { continue; }
        let puntos: Vec<Option<f64>> = serie
            .iter()
            .step_by(step)
            .take(sampled.len())
            .map(|v| if v.is_finite() { Some(round2f(*v)) } else { None })
            .collect();
        if puntos.iter().any(|p| p.is_some()) {
            overlays.push(serde_json::json!({ "key": key, "label": label, "points": puntos }));
        }
    }
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
        "candles": candles, "trades": trades, "overlays": overlays,
        "oosFrom": oos_from,
        "oosLabel": format!("Fuera de muestra (ultimo {:.0} %)", oos_from * 100.0),
    }))
}

fn round2f(v: f64) -> f64 { (v * 10000.0).round() / 10000.0 }

fn export_dataset_file(src: &std::path::Path, destination: &std::path::Path, format: &str) -> Result<i64, String> {
    match format {
        "csv" => {
            use std::io::Write;
            let bars = Qfn::read(src).map_err(|e| e.to_string())?;
            let file = std::fs::File::create(destination).map_err(|e| e.to_string())?;
            let mut writer = std::io::BufWriter::new(file);
            writeln!(writer, "time,open,high,low,close,volume").map_err(|e| e.to_string())?;
            for bar in &bars {
                writeln!(writer, "{},{},{},{},{},{}", bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume)
                    .map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
        }
        "qfd" => {
            std::fs::copy(src, destination).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("formato de exportación no admitido: {format}")),
    }
    let bytes = std::fs::metadata(destination).map_err(|e| e.to_string())?.len();
    i64::try_from(bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_dataset(state: State<AppState>, data_file_id: i64, format: String, destination_path: String) -> Result<serde_json::Value, String> {
    let path: String = {
        let db = state.db.lock();
        db.query_row(
            "select df.file_path from data_files df where df.id = ?1",
            q![data_file_id],
            |r| r.get(0),
        ).optional().map_err(|e| e.to_string())?
        .ok_or(format!("dataset {data_file_id} no encontrado"))?
    };
    let src = std::path::Path::new(&path);
    let destination = std::path::Path::new(&destination_path);
    let bytes = export_dataset_file(src, destination, &format)?;
    let file_name = destination.file_name()
        .and_then(|name| name.to_str())
        .ok_or("la ruta de destino no contiene un nombre de archivo válido")?;
    Ok(serde_json::json!({
        "fileName": file_name,
        "filePath": destination_path,
        "bytes": bytes,
    }))
}

#[tauri::command(async)]
pub fn run_opt_profile(state: State<AppState>, strategy_id: i64) -> Result<serde_json::Value, String> {
    // (el registro del historial se anade al final de la funcion)
    let db = state.db.lock();
    let core = load_core(&db, strategy_id)?;
    let mut v = ops::opt_profile(&core, &EngineCfg::default())?;
    v["strategyId"] = strategy_id.into();
    // historial: la forma del perfil y el mejor parametro
    let shape = v["shape"].as_str().unwrap_or("puntual");
    let verdict = match shape {
        "meseta" => "pass",
        "unimodal" => "warn",
        _ => "fail",
    };
    record_run(
        &db, strategy_id, "opt_profile", verdict,
        &format!("{} · mejor {}", v["param"].as_str().unwrap_or("p1"), v["bestParam"]),
        serde_json::json!({
            "bestParam": v["bestParam"],
            "stableFrom": v["stableFrom"],
            "stableTo": v["stableTo"],
        }),
        serde_json::json!({}),
        &v,
    );
    drop(db);
    Ok(v)
}

// ── tests ─────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_dataset_escribe_el_fichero_seleccionado_y_confirma_su_tamano() {
        let dir = std::env::temp_dir().join(format!("qf_export_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("source.qfn");
        let csv = dir.join("selected.csv");
        let qfd = dir.join("selected.qfd");
        let bars = vec![
            Bar { time: 1_700_000_000, open: 1.1, high: 1.2, low: 1.0, close: 1.15, volume: 100.0 },
            Bar { time: 1_700_000_060, open: 1.15, high: 1.25, low: 1.1, close: 1.2, volume: 120.0 },
        ];
        Qfn::write(&src, &bars).unwrap();

        let csv_bytes = export_dataset_file(&src, &csv, "csv").unwrap();
        let csv_text = std::fs::read_to_string(&csv).unwrap();
        assert!(csv_text.starts_with("time,open,high,low,close,volume\n"));
        assert!(csv_text.contains("1700000000,1.1,1.2,1,1.15,100"));
        assert_eq!(csv_bytes as u64, std::fs::metadata(&csv).unwrap().len());

        let qfd_bytes = export_dataset_file(&src, &qfd, "qfd").unwrap();
        assert_eq!(Qfn::read(&qfd).unwrap().len(), bars.len());
        assert_eq!(qfd_bytes as u64, std::fs::metadata(&qfd).unwrap().len());
        assert!(export_dataset_file(&src, &dir.join("bad.bin"), "bin").is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn los_flags_wf_usan_trades_duracion_y_drawdown_reales() {
        assert_eq!(wfm_validation_flags(29, 1.0, 20_000.0, 0.0), (true, false));
        assert_eq!(wfm_validation_flags(30, 0.9, 20_000.0, 0.0), (true, false));
        assert_eq!(wfm_validation_flags(30, 1.0, 20_000.0, 16_000.0), (false, false));
        assert_eq!(wfm_validation_flags(30, 1.0, 20_000.0, 16_001.0), (false, true));
    }

    #[test]
    fn drawdown_wf_compara_el_deterioro_oos_con_el_peak_is() {
        let (peak, drawdown) =
            equity_peak_and_max_drawdown([10_000.0, -17_000.0].into_iter(), 10_000.0);
        assert_eq!(peak, 20_000.0);
        assert_eq!(drawdown, 17_000.0);
    }

    #[test]
    fn listar_banco_lee_columnas_existentes_y_devuelve_estrategias() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        conn.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, robust_score, robust_detail)
             values ('QA strategy', 1, 'm1', 'builder', '{}', 95, '{\"score\":95}')",
            [],
        )
        .unwrap();

        let bank = list_bank_rows(&conn).unwrap();

        assert_eq!(bank.len(), 1);
        assert_eq!(bank[0]["name"], "QA strategy");
        assert_eq!(bank[0]["lastRun"]["oosFrac"], serde_json::Value::Null);
        assert!(bank[0]["robustScore"].is_null(), "el score antiguo no debe presentarse como calificado");

        conn.execute(
            "insert into data_files (id, symbol_id, timeframe, storage_kind, file_path, status)
             values (1, 1, 'm1', 'imported_csv', 'unused.qfn', 'ready')",
            [],
        ).unwrap();
        conn.execute(
            "insert into backtest_runs (strategy_id, data_file_id, engine_cfg)
             values (1, 1, '{}')",
            [],
        ).unwrap();
        conn.execute(
            "update strategies set robust_detail = '{\"calificado\":true,\"datasetId\":1}' where id = 1",
            [],
        ).unwrap();
        let bank = list_bank_rows(&conn).unwrap();
        assert_eq!(bank[0]["robustScore"], 95.0);
    }

    #[test]
    fn persistir_progreso_actualiza_las_pasadas_del_build() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into builds (kind, config, status, n_total) values ('builder', '{}', 'running', 1)",
            [],
        )
        .unwrap();
        let build_id = conn.last_insert_rowid();

        persist_build_progress(&conn, build_id, 7, 100).unwrap();

        let progress: (i64, i64) = conn
            .query_row(
                "select n_passed, n_total from builds where id = ?1",
                rusqlite::params![build_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(progress, (7, 100));
    }

    #[test]
    fn finalizar_build_persiste_estado_y_resumen_diagnostico() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into builds (kind, config, status, n_total) values ('builder', '{}', 'running', 10)",
            [],
        )
        .unwrap();
        let build_id = conn.last_insert_rowid();

        finish_build(
            &conn,
            build_id,
            "done",
            0,
            10,
            "No se alcanzó el objetivo.\nFiltro Profit factor >= 1000000000: 480 candidatos descartados.",
        )
        .unwrap();

        let (status, passed, total, summary): (String, i64, i64, Option<String>) = conn.query_row(
            "select status, n_passed, n_total, result_summary from builds where id = ?1",
            rusqlite::params![build_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).unwrap();
        assert_eq!(status, "done");
        assert_eq!((passed, total), (0, 10));
        assert!(summary.unwrap().contains("480 candidatos descartados"));
    }

    /// La configuracion del motor sale de los metadatos del simbolo, no de los
    /// valores por defecto (que son de Forex USD-quote).
    #[test]
    fn la_cfg_del_motor_toma_los_datos_del_simbolo() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class, tick_size, pip_size, commission,
                                  contract_size, swap_long, swap_short,
                                  spread_asia, spread_london, spread_ny)
             values ('USDJPY', 'forex', 0.001, 0.01, 3.5, 100000, 1.2, -0.8, 0.7, 1.0, 1.3)",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();
        conn.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, params, status)
             values ('S1', ?1, 'm1', 'builder', '{}', '{\"p\":[20,50,0]}', 'active')",
            rusqlite::params![sym],
        )
        .unwrap();
        let sid = conn.last_insert_rowid();

        let cfg = cfg_for_symbol(&conn, sid);
        assert_eq!(cfg.pip_size, 0.01, "el pip debe ser el del simbolo");
        assert_eq!(cfg.tick_size, 0.001);
        assert_eq!(cfg.commission_per_lot, 3.5);
        assert_eq!(cfg.contract_size, 100_000.0);
        assert_eq!(cfg.swap_long, 1.2);
        assert_eq!(cfg.swap_short, -0.8);
        assert_eq!(cfg.spread_asia, Some(0.7));
        assert_eq!(cfg.spread_ny, Some(1.3));
        // y el spread de esa sesion es el del simbolo, no el de por defecto
        let medianoche = 1_699_920_000i64;
        assert_eq!(cfg.spread_pips(medianoche + 9 * 3_600), 1.0);
    }

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

        let (row, calidad) = import_csv_registered(&conn, &dir, sym, "m1", &csv)
            .expect("la importacion debe funcionar");

        assert_eq!(row.symbol_id, sym);
        assert_eq!(row.timeframe, "m1");
        assert_eq!(row.storage_kind, "imported_csv");
        assert_eq!(row.n_bars, 500, "debe registrar todas las barras");
        assert_eq!(row.status, "ready");
        // el diagnostico dice que el fichero esta limpio
        assert_eq!(calidad["limpio"], true);
        assert_eq!(calidad["nBars"], 500);

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


    /// El diagnostico de calidad detecta huecos, volumen cero y precios imposibles.
    #[test]
    fn el_diagnostico_avisa_de_huecos_y_precios_incoherentes() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();

        let dir = std::env::temp_dir().join(format!("qf_calidad_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("roto.csv");
        // 210 filas, un hueco de una barra, un volumen cero y un close imposible
        let mut s = String::from("time,open,high,low,close,volume\n");
        for i in 0..210i64 {
            if i == 120 { continue; }
            let t = 1_700_000_000 + i * 60;
            if i == 50 {
                // precio imposible: se descarta
                s.push_str(&format!("{t},1.1,1.2,1.0,9.9,100\n"));
            } else if i == 60 {
                // sin volumen: se acepta pero se avisa
                s.push_str(&format!("{t},1.1,1.2,1.0,1.15,0\n"));
            } else {
                s.push_str(&format!("{t},1.1,1.2,1.0,1.15,100\n"));
            }
        }
        std::fs::write(&csv, s).unwrap();

        let (row, calidad) = import_csv_registered(&conn, &dir, sym, "m1", &csv).unwrap();
        // 210 filas: se descarta la de precio imposible (209) y falta una por el hueco (208)
        assert_eq!(row.n_bars, 208);
        // el hueco se detecta comparando el paso esperado
        assert_eq!(calidad["limpio"], false);
        // la fila imposible se descarta y se avisa
        assert!(calidad["skipped"].as_i64().unwrap() >= 1, "filas descartadas: {calidad}");
        // el hueco se detecta
        assert!(calidad["huecos"].as_i64().unwrap() >= 1, "huecos: {calidad}");
        // y el volumen cero se avisa
        assert!(calidad["sinVolumen"].as_i64().unwrap() >= 1);
        let avisos = calidad["avisos"].as_array().unwrap();
        assert!(avisos.len() >= 2, "avisos: {avisos:?}");

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
