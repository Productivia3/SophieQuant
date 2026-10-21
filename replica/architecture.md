# Architecture: QuantForge (a rebuild of StrategyQuant X's core features)

Recon leído: `replica/recon.md` (24 pantallas, 11 flujos, 10 entidades) + `replica/features.csv` (51 features, 20 must).

## Stack

Caso **local-first desktop** (el original es una app de escritorio con datos pesados y motor de cómputo local). No aplica el default "Next.js + Postgres gestionado": los datos de tick son GB por símbolo y el motor corre en la máquina del usuario. Swap justificado:

| layer | choice | why |
| --- | --- | --- |
| shell de escritorio | **Tauri 2** | más ligero que Electron (~10 MB), dijo el scope; integrate con WebView del SO |
| UI | **React 18 + TypeScript + Vite** | ecosistema sólido para dashboards/tablas densas; grid virtualizado con TanStack Table + Virtual |
| styling | **Tailwind + design tokens de replica-design** | tokens JSON compartidos con skill de diseño |
| gráficos | **lightweight-charts (equity/candles) + TypeScript propio para heatmap/3D** | lib abierta financiera, sin vendor lock-in |
| núcleo de cómputo | **Rust (crate `quantforge-core`)** | el motor de backtest con precisión de tick y "miles de backtests/seg" exige compilado nativo; Rust da C-speed + memory safety |
| paralelismo | **rayon (work-stealing), N = physical cores** | generación genética paralela sin locks; mismas técnicas que el original implementa en Java |
| base de datos | **SQLite (WAL) vía rusqlite, un fichero por proyecto** | local, fiable, sin servidor; los banks de millones de backtests se guardan filtrados con índices por métrica |
| storage de series | **binario comprimido propio (QF dat) + Arrow/Feather para export** | menos overhead que GBs de JSON; los datos de tick no van a la DB, van a ficheros |
| migraciones | **refinery (embedded)** | migraciones versionadas dentro del binario Tauri |
| jobs largos | **threads Rust + cancel token, canal de eventos → front (Tauri event bus)** | el workflow como pipeline encadenado con outputs entre pasos |
| licencias/pagos (después, si se vende) | **Keygen o Lemon Squeezy license API** | decidir en Fase 4 con replica-launch |
| hosting de marketing/web | **Vercel (Next.js)** | solo la landing/lista del skill replica-launch |
| i18n | **Format.js (ES/EN primero)** | el original vende en 8 idiomas; arrancamos 2 |
| tests e2e | **Playwright sobre la ventana (driver localStorage bridge) + Tauri WebDriver** | replica-test necesita click-through |

**Una sola DB local por proyecto, sin microservicios, sin nube obligatoria.** El clon es más simple que el original: StrateSyQuant usa Java Swing + H2; nosotros Tauri+Rust+SQLite.

## Schema

SQLite (WAL mode). Tables: 14. Access rules: proceso único dueño del fichero — acceso exclusivo vía capa `core::db` (no RLS: no hay multiusuario local); la unicidad y las exclusiones se garantizan con índices y constraints en la propia DB.

```sql
-- 🔍 ver replica/schema.sql (primera migración refinery)
create table symbols (
  id integer primary key,
  name text not null unique,            -- EURUSD, AAPL...
  asset_class text not null check (asset_class in ('forex','cfd','stock','future','crypto')),
  exchange text, tick_size real, commission real
);
create table data_files (
  id integer primary key,
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null,              -- 'tick','m1','m5','m15','m30','h1','h4','d1','w1','mn'
  storage_kind text not null check (storage_kind in ('local_qfd','imported_csv','dukascopy','yahoo'))
      -- sources: dukascopy local download, yahoo public, csv import, quantmanager
  compressed_bytes integer, n_bars integer,
  date_from text, date_to text,          -- ISO days (UTC)
  status text not null default 'ready' check (status in ('downloading','ready','corrupt')),
  created_at integer not null default (unixepoch())
);
create index on data_files(symbol_id, timeframe);

create table strategies (
  id integer primary key,
  name text not null, created_at integer not null default (unixepoch()),
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null,
  multi_tf jsonb default '[]',           -- charts de entrada adicionales (multi market/TF)
  template_id integer references strategy_templates(id) on delete set null,
  origin text not null check (origin in ('builder','improver','optimizer','import')),
  logic jsonb not null,                  -- árbol de bloques (entry/exit/stop/MM/filters)
  params jsonb default '{}',             -- parámetros concretos de la lógica
  status text not null default 'active' check (status in ('active','dirty','archived')),
  problems jsonb default '[]'            -- auto-detected issues (peak-trading, degenerate...)
);
create index on strategies(symbol_id, timeframe, status);
-- ⚠ hotspot: el jsonb de logic hace el bank lento con millones de rows; para sorting por métrica
-- se usan las columnas generadas planas de backtest_runs (net_profit, profit_factor, ...) con índice.

create table strategy_templates (
  id integer primary key, name text not null, source text default 'user',
  logic jsonb not null,                  -- árbol con placeholders aleatorios (tipo "RandomSymbolRule")
  created_at integer not null default (unixepoch())
);

create table backtest_runs (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  data_file_id integer not null references data_files(id) on delete cascade,
  engine_config jsonb not null,          -- spread, slippage, initial balance, position sizing, test precision
  build_id integer references builds(id) on delete cascade,
  -- métricas planas para índices y sorting de bank (NO json):
  net_profit real, gross_profit real, profit_factor real, win_rate real,
  max_dd real, rdd real, sharpe real, sortino real, z_score real, r_squared real,
  n_trades integer, avg_trade real, wfe real,
  equity jsonb,                          -- series equity[]
  status text not null check (status in ('running','done','error','cancelled'))
);
create index on backtest_runs(strategy_id, status);
create index on backtest_runs(net_profit desc, profit_factor desc);  -- bank sorting
create index on backtest_runs(build_id);

create table robustness_runs (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  test_type text not null check (test_type in ('monte_carlo','wf_optimization','wf_matrix','spp','optimization_profile','oos_check','mc_portfolio')),
  config jsonb not null,
  verdict text not null default 'pending' check (verdict in ('pending','running','pass','fail')),
  details jsonb,                          -- per-trial results, clusters...
  created_at integer not null default (unixepoch())
);
create index on robustness_runs(strategy_id, test_type, verdict);

create table builds (
  id integer primary key,
  kind text not null check (kind in ('builder','retester','improver','optimizer','monte_carlo','custom_project')),
  config jsonb not null,                 -- filtros, plantilla, datos, n strategias target...
  status text not null default 'queued' check (status in ('queued','running','paused','done','error','cancelled')),
  n_passed integer default 0, n_total integer default 0,
  started_at integer, finished_at integer,
  parent_build_id integer references builds(id) on delete cascade  -- pasos custom project hijos
);

create table custom_projects (
  id integer primary key, name text not null,
  steps jsonb not null,                  -- [{kind:'build',config:...},{kind:'test'...}]
  created_at integer not null default (unixepoch())
);

create table portfolios (
  id integer primary key, name text not null, created_at integer,
  weighting_model text default 'equal_vol' check (weighting_model in ('equal_vol','risk_paridad','equal_weight','custom')),
  combination text default 'OR'           -- cuándo entra cada estrategia
);
create table portfolio_members (
  portfolio_id integer not null references portfolios(id) on delete cascade,
  strategy_id integer not null references strategies(id) on delete cascade,
  weight real default 1, position text,   -- ranking en la cartera
  primary key (portfolio_id, strategy_id)
);
create index on portfolio_members(strategy_id);

create table portfolio_runs (  -- backtest conjunto de cartera
  id integer primary key, portfolio_id integer not null references portfolios(id) on delete cascade,
  config jsonb, metrics jsonb, equity jsonb, corr_matrix jsonb
);

create table exports (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  platform text not null check (platform in ('mql4','mql5','ninjatrader','tradestation','multicharts','jforex')),
  file_name text, type text, generated_at integer, checksum text
);
create index on exports(strategy_id, platform);

create table app_settings (key text primary key, value jsonb);  -- licencia, dirs, tema, workers, proxy
```

### Constraints duros del recon (en DB, no en UI)

1. **No canibalizar resultados de banco**: un `backtest_run` activo por (strategy, data_file) se garantiza vía `partial unique index` (status='running').
2. **Datos no solapados por símbolo/TF**: en descarga, rangos ya cubiertos → merge automático a un único `data_files` continuo índice único `(symbol_id, timeframe, storage kind)`.
3. **Millones de rows**: paginación por scroll + sorting server-side (core queries Rust a SQLite) — el front nunca pull el bank completo.
4. **fsync WAL + checkpoint** tras cada batch de rewritable para que un crash no pierda el bank.

## API

Todo local vía **Tauri commands** (el front llama Rust). Nada de network salvo descargadores públicos declarados.

| command | does | who | input | output | flow |
| --- | --- | --- | --- | --- | --- |
| `list_symbols` | símbolos con data disponible | front | - | Symbol[] | S02, S20 |
| `download_data` | descarga tick/m1 de Dukascopy o stocks Yahoo, resample, save .qfd | S20 | Symbol, TF, rango | progress events | F09 |
| `import_data_csv` | importa CSV a dataset | front | MetaFile | DataFile | F09 |
| `list_data_files` | datasets por symbol | front | - | DataFile[] | S20 |
| `convert_data` | resample tick→m1→..., fix gaps | front | DataFile, TF target | DataFile | F09 |
| `create_template` | plantilla con placeholders | front | TemplateSave | Template | F01 |
| `list_templates` | - | front | Template[] | S02 |
| `start_build` | lanza build (builder/retester/improver/optimizer) async | front | BuildConfig | buildId | F01,F03 |
| `pause/resume/cancel_build` | control | front | buildId | - | S03 |
| `on_build_event` (event) | ticks de progreso, log lines, passed/failed counters | core | eventos | front | S03 |
| `list_bank` | bank paginado, sort cols, filtros por métrica | front | FilterSpec | Strategy[] | S04 |
| `get_strategy` | detalle completo con métricas del backtest last | front | id | StrategyDetail | F02 |
| `strategy_chart` | velas + indicadores + trades de la mejor corrida | front | strategyId, rango | SeriesChart | S06 |
| `edit_strategy` | modifica lógica/params vaidación | front | id, patch | Strategy | F02 |
| `save_as_template` | convierte estrategia en plantilla | front | strategyId | StrategyTemplate | F01 |
| `start_improve` | improver de entrada/salida (ML) | front | strategyId, parts | buildId | F03 |
| `start_optimize` | simple o walk-forward | front | strategyId, paramRanges, mode | buildId | F04 |
| `get_optimization_3d` | superficie 3D de matrix walk-forward | front | buildId | 3DPoints | S11 |
| `run_once_monteCarlo` | run MC sobre una estrategia (fuera de build) | front | strategyId, cfg | RobustnessResult | S13 |
| `start_wf_matrix` / `get_wf_matrix` | clustering run | front | strategyId, cfg | Clusters | S12 |
| `start_spp` / `start_optimization_profile` | robustez singles | front | strategyId, cfg | Robustness | S14, S15 |
| `get_robustness_runs` | historial robustez de estrategia | front | strategyId | Robustness[] | S05 |
| `create_custom_project` / `get` / `list_custom_projects` | CRUD workflow | front | Config | Project | F06 |
| `run_custom_project` | lanza pipeline encadeado de steps | front | projectId | buildId | F06, F07 |
| `create_portfolio` / `update_portfolio` | construir cartera con modelo de ponderación | front | PortfolioConfig | Portfolio | F08 |
| `run_portfolio_backtest` | test conjunto + matriz de correlación | front | portfolioId, cfg | PortfolioRun | S19 |
| `search_uncorrelated_candidates` | algoritmo para añadir estrategia no correlacionada | front | portfolioId, n | Strategy[] | S19 |
| `export_strategy` | genera código fuente (MQL4/5/NT/...) | front | strategyId, platform, opts | code | F08 |
| `save_export_file` | guarda fichero export | front | path | ok | F08 |
| `brand_sweep_check` | anti-copy scan de la skill replica-brand | launch | - | lista | antes de release |
| `project_crud` | decisión de proyecto (crear/abrir/backup) | front | - | - | S01 |
| `get_settings` / `update_settings` | preferencias | front | - | Settings | S23 |

**Fuera de la app (public, propios)**:
- Dukascopy.js: descarga de histórico forex/CFDs, publicDownload.
- Yahoo Finance FY (crunching): descarga stocks EOD.
- Nada de APIs de StrategyQuant. Las reglas de sintaxis de cada plataforma (MT/Ninja/TS): se implementan con las SDKs y docs públicas de cada una — escritas por nosotros.

**Jobs (internos, sin cron porque es local):**
- `bank_wal_checkpoint`: auto cada 5 min mientras un build corre (WAL checkpoint).
- `data_lifecycle`: limpieza de tick intermedios > N días a petición.
- `crash_recovery`: en arranque, builds marcados running sin latido → cancelled con log.

## The parts that bite

- **Mutación tick-local/native numeric**: todos los cálculos de price en **Rust f64** con convención de pip según asset; nunca JS number para el engine. El front solo presenta strings formateados.
- **Time zones/DST**: datos UTC siempre; timestamps as unix seconds; broker-server-vs-UTC offset configurable por symbol (los backtests MT calculan in server time — el clon igual).
- **Idempotencia**: `start_build` withhalt config hash → si idéntica config y no cancelada, devuelve mismo buildId en vez de duplicar huecos. Webhooks: no hay (escritorio).
- **Races**: SQLite en WAL desde un único proceso propietario del archivo; el front nunca hago SQL directo. Builds cancelables → solo si build row lo permite; builder loop revisa `should_cancel()` por chunk de work.
- **Rate limits externos**: Yahoo Finance / Dukascopy blockan bursts; implementamos backoff en descargadores y cola serializada por símbolo. Sus términos: cumplimos.
- **Ficheros gigantes (tick GB)**: storage binario por año+mes; nunca carga en memoria el rango completo salvo build; backtester usa cursor streaming (mmap + ranges).
- **Memoria**: limitar workers y deduplicar el bank — guardamos las mejores N estrategias por métrica y comprimimos stats; el backtest completo solo de supervivientes (el original hace lo mismo con su filtro).
- **Search**: sobre bank, filtros SQL (index); sobre símbolos: header file local.
- **Realtime**: no en v1 (live trading no está en scope del VP core del clon; import/export son la puente con MT).
- **Offline**: la app es 100% offline salvo descargadores.
- **Auto package updates**: activo Tauri updater desde release Hosting GitHub con firma, desactivable.
- **GDPR/tenant**: N/A desktop; datos del usuario quedan en su máquina. Logout de licencia limPIA.
- **Consistencia de exporters**: cada plataforma tiene sus quirks (MT5 netting vs hedging). Cada export se valida con golden files por plataforma como snapshot test (lo usa replica-test).

## Build order

Basado en features.csv (20 must, 22 should, 9 could):

1. **Vertical slice (proves the stack, ~2 weeks)**
   - S01 shell (sidebar+project) + S20 data download Dukascopy (EURUSD tick 5y) → merge data
   - S02 build config mínima (2 plantillas fijadas rsi/ma entry-exit) → S03 run con rayon → S04 bank
   - S05 resultados + equity chart + métricas_core
   - S22 export MQL5 set completo
   - Tables: symbols, data_files, strategies, builds, backtest_runs + settings
   - Routes: data Download/Import, start_build, list_bank, get_strategy, export_strategy
2. **Must-haves restantes por área**
   - backtester/completes: multi-symbol/multi-TF, métricas full, detect problems
   - builder: plantillas placeholders completo, aleatorio+evolución, multi-market primary
   - blocks: catálogo indicadores estándar + 4/6 entrada/salida + velas patterns
   - filtering/filtering avanzado por métrica
   - databankousands rows + sorting/indexes
   - robustness musts: Monte Carlo (subset 5 de 9), WF Matrix clusterizado
   - workflows: custom projects con steps básicos (build→test→filter)
   - portfolio: composer + correlación
   - platform: empaquetar Windows/macOS/Linux builds con updater
3. **Should-haves**: SPP, Optimization Profile, OOS múltiple, improver, optimizer WF + 3D, export restante (NT/TS/MC/JForex con golden tests), gestores datos (resample/fix), QuantAnalyzer dashboard básico, QuantDataManager separada, chart con trades
4. **Could-haves**: fuzzy logic, snippet custom blocks, custom analysis Python hooks, Monte Carlo cartera, MM sim, ATM, gráficos 3D advanced, Portfolio "adjust to existing" y, licencias
5. **Fixes de replica-entrepreneur**: una vez corro la skill de reviews para saber QUÉ e extra dimensiones de precio (per-seat...) y qué usuarios más odian del original priorizar.

## Los 3 riskiest parts

1. **Motor backtest tick con suficiente velocidad** — mitigación: benchmark continuo contra baseline propio, no prometer "miles/seg" hasta que los benchmarks lo digan.
2. **Exporters de código idéntico a la sintaxis de cada plataforma** — riesgo alto de bugs silenciosos; mitigación: golden-file tests por plataforma + docs públicas MT5/Ninja de referencia.
3. **Suite de robustez matemática** — SPP, Optimization Profile, WF clustering: riesgo de mala interpretación estadística. Implementar cada test con papers propios (Pardo et al) y validar contra examples de manual público.

---

**Resumen**: Tauri 2 + React + Rust core + SQLite local. 14 tablas, ~35 Tauri commands. Riesgos: motor tick rápido, exporters 5 plataformas con golden tests, suite robustez validada. Siguiente paso: **`/replica-design`** para tokens (tema oscuro financiero) y specs de grid+charts.
