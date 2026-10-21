//! SophieQuant — núcleo Tauri. Contrato de commands: replica/architecture.md.
//! El build real corre en un hilo aparte con su propia conexión SQLite;
//! los progreso se emiten por el canal "build-event" hacia la UI.

// En release la app es una aplicacion de ventana: sin consola detras.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod engine;
mod export;
use parking_lot::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use commands::AppState;
fn main() {
    tauri::Builder::default()
        // plugin de dialogo: selector de ficheros real (importar CSV, guardar exportaciones)
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // fichero de DB en AppData — un proyecto → una DB
            let dir = app.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&dir).ok();
            let db_path = dir.join("sophiequant.db");
            let conn = Connection::open(&db_path).expect("abrir SQLite");
            conn.pragma_update(None, "journal_mode", "WAL").ok();
            engine::datastore::init_schema(&conn).expect("init schema");
            commands::seed_symbols(&conn).ok();
            commands::seed_templates(&conn).ok();
            app.manage(AppState {
                db: Mutex::new(conn),
                db_path: db_path.to_string_lossy().to_string(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_symbols,
            commands::list_build_metrics,
            commands::list_data_files,
            commands::import_data_csv,
            commands::list_templates,
            commands::start_build,
            commands::list_builds,
            commands::cancel_build,
            commands::list_bank,
            commands::get_strategy,
            commands::export_strategy,
            commands::get_settings,
            commands::update_settings,
            commands::pause_build,
            commands::resume_build,
            commands::run_retest,
            commands::run_improver,
            commands::run_optimizer,
            commands::run_wfm,
            commands::run_spp,
            commands::run_mc_suite,
            commands::get_correlation_matrix,
            commands::get_portfolio_equity,
            commands::get_portfolio_results,
            commands::run_portfolio_mc,
            commands::run_project,
            commands::get_strategy_chart,
            commands::export_dataset,
            commands::run_opt_profile,
            commands::get_strategy_verdicts,
            commands::compute_strategy_verdicts,
            commands::save_strategy_logic,
            commands::list_analysis_runs,
            commands::run_portfolio_weighting,
            commands::list_portfolio_runs,
            commands::delete_analysis_run,
        ])
        .run(tauri::generate_context!())
        .expect("error al arrancar SophieQuant");
}