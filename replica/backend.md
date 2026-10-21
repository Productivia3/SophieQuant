# replica/backend.md — Backend de QuantForge (edición escritorio)

> QuantForge es una aplicación **desktop (Tauri 2)** del núcleo de un generador de
> estrategias. El "backend" no es un servidor web: es el **núcleo Rust + SQLite
> local** expuesto como commands Tauri, con la capa híbrida `src/lib/api.ts`
> (real-first: `invoke` si hay Tauri, seed fake si corre en navegador para QA).

## Estado

- Motor Rust completo y real: backtest long-only sin lookahead, generador
  genético determinista, importación CSV robusta, almacenamiento QFN binario,
  exportadores MQL4/MQL5.
- **NUEVO (backend real)**: 16 commands de robustez/cartera en `engine/ops.rs` +
  `commands.rs`, todos sobre datos reales (equities persistidas en `backtest_runs`
  y re-ejecuciones del motor sobre el dataset QFN guardado en disco):
  `run_retest` (OOS 30% final), `run_improver` (4 variantes re-ejecutadas),
  `run_optimizer` + `run_wfm` (grid p1×p2 con WFE; WF = train 60% / eval 40%),
  `run_opt_profile` (perfil de Pardo con forma meseta/unimodal/puntual),
  `run_mc_suite` (9 tests MC: shuffle/skip/duplicate/slippage/dd/params…),
  `run_spp` (6 variantes de señales con el mismo bucle de cartera),
  `get_correlation_matrix` (Pearson sobre retornos de equity),
  `get_portfolio_equity` / `get_portfolio_results` / `run_portfolio_mc`
  (bootstrap 500 trials, ruina = −30%),
  `run_project` (orquestador de pasos reales: build/monte_carlo/spp/optimizer/save),
  `get_strategy_chart` (velas muestreadas + trades recomputados),
  `pause_build` / `resume_build` (bandera atómica en el hilo del generador),
  `export_dataset` (CSV regenerado o copia QFN a `AppData/exports/`).
- 21/21 tests unitarios Rust en verde; 77/77 e2e; build 283 KB gzip 84 KB.

## Base de datos

- [x] Schema de 13 tablas (`src-tauri/schema.sql`) aplicado en arranque con
  `create table if not exists` y `@step` de migración manual documentado.
- [x] Índices y FK con `on delete cascade`. Claves foráneas activadas.
- [x] día WAL activado (`journal_mode=WAL`).
- Seed: símbolos (EURUSD/GBPUSD/USDJPY) y 3 plantillas built-in sembradas en
  arranque si la DB está vacía; en navegador, seed fake de 18 estrategias para QA.
- Accesos: **usuario único local** — el fichero SQLite vive en el AppData del
  usuario y no se abre a la red. No aplica RLS ni roles (sin servidor); la única
  "regla de acceso" es a nivel de proceso: la DB solo la tocan los commands del
  proceso (mutex `AppState`).

## Autenticación / pagos / email

- **Fuera de alcance V1 por diseño de producto**: la referencia vende licencias
  de escritorio, sin cuentas cloud en-app. Documentado en recon.md.
  - Autenticación: N/A (no hay servicio multiusuario).
  - Pagos: se resolverán en la web de venta en `/replica-launch` +
    `/replica-deploy` (licencias, no suscripciones in-app).
  - Email: no hay curación de cuentas; N/A hasta que exista portal de licencias.

## Seguridad (checklist adaptado a escritorio)

- [x] Sin secretos: no hay API keys en el binario; los datos son locales.
- [x] Validación de entrada en los commands (import CSV valida timestamps, OHLC
  y descarta filas corruptas — BUG-006; `Platform::from_str` valida alias).
- [x] Sin datos de usuario en URLs ni logs (los builds emiten progreso por
  canal de eventos, no contenido sensible).
- [x] **Dependencias Rust auditadas**: `cargo audit --deny warnings` ejecutado
  el 21-10-2026 sobre las 437 dependencias del `Cargo.lock`: **0
  vulnerabilidades**. Los 2 avisos informativos (RUSTSEC-2024-0429 `glib` y
  RUSTSEC-2024-0370 `proc-macro-error`) vienen **solo del stack GTK/Linux** de
  Tauri, no se compilan en la entrega de Windows y la version corregida de
  `glib` es incompatible con el arbol de dependencias de Tauri 2.12 (verificado:
  `cargo update -p glib --precise 0.20.0` falla). Registro y comando de
  referencia en `src-tauri/audit.toml`.
- [x] Configuración protegida: `app_settings` solo se toca vía `update_settings`.

## Huecos conocidos (para fases siguientes)

1. ~~**importDatasetFile sigue fake en escritorio**~~ — **cerrado** (21-10):
   instalado `tauri-plugin-dialog` 2.8.3 (Rust + JS), con permisos en
   `src-tauri/capabilities/default.json`. El boton «Importar CSV» abre el
   selector nativo, elige simbolo y temporalidad de destino y llama al command
   real `import_data_csv`, que limpia el CSV, lo guarda como QFN y lo registra
   en `data_files`. La importacion se extrajo a `import_csv_registered` para
   poder testearla: 2 tests de integracion con un CSV real de 500 barras.
2. **downloadData (Dukascopy) diferido** por decisión del usuario (V1 = CSV
   propio o import; el fake simula la descarga con progreso en cámara lenta).
3. ~~**Builders de resultados de análisis no persisten**~~ — **cerrado**
   (21-10): los siete análisis (retesteo, refinador, optimizador, matriz WF,
   Monte Carlo, SPP y perfil) se registran en la tabla `analysis_runs` con su
   veredicto y sus métricas, y el historial compara cada corrida con la
   anterior del mismo tipo (módulo `engine/history.rs`).
4. ~~`cargo fix` para limpiar warnings de codigo muerto (8) + `cargo audit`~~ —
   **cerrado** (21-10): los 6 avisos de `dead_code` se resolvieron de raiz (no
   con `allow`): dos helpers pasaron a usarse de verdad, tres se eliminaron y uno
   se sustituyo por el nombre de la estrategia en los mensajes de error. El
   `cargo audit` da 0 vulnerabilidades.
5. Portfolio Composer con ponderaciones (solo media equiponderada) y el ajuste
   ATS/ATM — coulés de `features.csv`, se decide tras `/replica-entrepreneur`.

## Estado

Las tareas de calidad que quedaban abiertas estan cerradas: el import de ficheros
usa el **selector nativo real** (plugin de dialogo) y el **`cargo audit` da 0
vulnerabilidades**. El producto minimo es ejecutable en local.

## Proximo paso

Producto minimo ejecutable en local cerrado. La evolucion continua por la lista
de ampliaciones (multi-simbolo en el retesteo, mas indicadores, tick, ponderaciones
del Portfolio Composer, ATM).
