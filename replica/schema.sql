-- QuantForge — primera migración (refinery V1)
-- SQLite (WAL). App de escritorio local-first: sin multiusuario, sin RLS.
-- Proceso único dueño del fichero; acceso exclusivo vía capa core::db (Rust).

-- ── Datos de mercado ────────────────────────────────────────────────

create table if not exists symbols (
  id integer primary key,
  name text not null unique,            -- EURUSD, AAPL, ES...
  asset_class text not null check (asset_class in ('forex','cfd','stock','future','crypto')),
  exchange text,
  tick_size real not null default 0.0001,
  pip_size real not null default 0.0001,
  commission real not null default 0
);

create table if not exists data_files (
  id integer primary key,
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null               -- tick|m1|m5|m15|m30|h1|h4|d1|w1|mn
    check (timeframe in ('tick','m1','m5','m15','m30','h1','h4','d1','w1','mn')),
  storage_kind text not null
    check (storage_kind in ('local_qfd','imported_csv','dukascopy','yahoo')),
  compressed_bytes integer not null default 0,
  n_bars integer not null default 0,
  date_from integer,                    -- unix epoch UTC
  date_to integer,
  status text not null default 'downloading'
    check (status in ('downloading','ready','corrupt')),
  created_at integer not null default (unixepoch())
);
create index if not exists idx_data_files_symbol on data_files(symbol_id, timeframe);
-- Constraint duro: no duplicar la misma fuente para el mismo símbolo/TF
create unique index if not exists uq_data_files_source
  on data_files(symbol_id, timeframe, storage_kind);

-- ── Estrategias ─────────────────────────────────────────────────────

create table if not exists strategy_templates (
  id integer primary key,
  name text not null,
  source text not null default 'user' check (source in ('user','built_in')),
  logic jsonb not null,                 -- árbol de bloques con placeholders aleatorios
  created_at integer not null default (unixepoch())
);

create table if not exists strategies (
  id integer primary key,
  name text not null,
  created_at integer not null default (unixepoch()),
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null,
  multi_charts jsonb not null default '[]',  -- gráficos de entrada adicionales (multi-market / multi-TF)
  template_id integer references strategy_templates(id) on delete set null,
  origin text not null check (origin in ('builder','improver','optimizer','import')),
  logic jsonb not null,                 -- árbol de bloques: entry/exit/stop/MM/filtros
  params jsonb not null default '{}',
  status text not null default 'active' check (status in ('active','dirty','archived')),
  problems jsonb not null default '[]',
  robust_score real,                      -- score de robustez 0-100 (ver engine/robust.rs)
  robust_detail jsonb                    -- componentes del score (wfe, mc, spp, perfil)
);
create index if not exists idx_strategies_bank on strategies(symbol_id, timeframe, status);

-- ── Builds y backtests ──────────────────────────────────────────────

create table if not exists builds (
  id integer primary key,
  kind text not null check (kind in ('builder','retester','improver','optimizer','monte_carlo','custom_project')),
  config jsonb not null,
  status text not null default 'queued'
    check (status in ('queued','running','paused','done','error','cancelled')),
  n_passed integer not null default 0,
  n_total integer not null default 0,
  started_at integer,
  finished_at integer,
  parent_build_id integer references builds(id) on delete cascade
);
create index if not exists idx_builds_status on builds(status, started_at);

create table if not exists backtest_runs (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  data_file_id integer not null references data_files(id) on delete cascade,
  engine_cfg jsonb not null,            -- spread, slippage, saldo inicial, position sizing, precisión
  build_id integer references builds(id) on delete cascade,
  -- métricas planas para ordenar/filtrar el bank rápido (NO en json):
  net_profit real,  gross_profit real,  profit_factor real,  win_rate real,
  max_dd real,      rdd real,           sharpe real,         sortino real,
  z_score real,     r_squared real,     n_trades integer,    avg_trade real,
  wfe real,
  equity jsonb,                         -- serie de equity
  oos_frac real,                       -- fraccion de la serie que es out-of-sample (corte walk-forward)
  status text not null default 'running'
    check (status in ('running','done','error','cancelled'))
);
create index if not exists idx_bt_strategy on backtest_runs(strategy_id, status);
-- Bank: sorting por métricas (hot path)
create index if not exists idx_bt_np on backtest_runs(net_profit desc, profit_factor desc);
create index if not exists idx_bt_build on backtest_runs(build_id);
-- Constraint duro: una única corrida activa por (strategy, dataset)
create unique index if not exists uq_bt_running
  on backtest_runs(strategy_id, data_file_id) where status = 'running';

-- ── Robustez ────────────────────────────────────────────────────────

create table if not exists robustness_runs (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  test_type text not null check (test_type in
    ('monte_carlo','wf_optimization','wf_matrix','spp','optimization_profile','oos_check','mc_portfolio')),
  cfg jsonb not null,
  verdict text not null default 'pending' check (verdict in ('pending','running','pass','fail')),
  details jsonb,
  created_at integer not null default (unixepoch())
);
create index if not exists idx_rob_str on robustness_runs(strategy_id, test_type, verdict);

-- ── Custom projects (workflows) ─────────────────────────────────────

create table if not exists custom_projects (
  id integer primary key,
  name text not null,
  steps jsonb not null,                 -- [{kind:'build',cfg}, {kind:'test'}, ...]
  created_at integer not null default (unixepoch())
);

-- ── Portfolio Master ────────────────────────────────────────────────

create table if not exists portfolios (
  id integer primary key,
  name text not null,
  created_at integer not null default (unixepoch()),
  weighting_model text not null default 'equal_vol'
    check (weighting_model in ('equal_vol','risk_parity','equal_weight','custom')),
  combination text not null default 'OR'
);

create table if not exists portfolio_members (
  portfolio_id integer not null references portfolios(id) on delete cascade,
  strategy_id integer not null references strategies(id) on delete cascade,
  weight real not null default 1,
  position integer,
  primary key (portfolio_id, strategy_id)
);
create index if not exists idx_pm_strategy on portfolio_members(strategy_id);

create table if not exists portfolio_runs (
  id integer primary key,
  portfolio_id integer not null references portfolios(id) on delete cascade,
  cfg jsonb not null,
  metrics jsonb,
  equity jsonb,
  corr_matrix jsonb,
  created_at integer not null default (unixepoch())
);

-- ── Export a plataformas ────────────────────────────────────────────

create table if not exists exports (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  platform text not null check (platform in
    ('mql4','mql5','ninjatrader','tradestation','multicharts','jforex')),
  file_name text,
  generated_at integer not null default (unixepoch()),
  checksum text
);
create index if not exists idx_exports_str on exports(strategy_id, platform);

-- ── Ajustes de la app ───────────────────────────────────────────────

create table if not exists app_settings (
  key text primary key,
  value jsonb not null
);