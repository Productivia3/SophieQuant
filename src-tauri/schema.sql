-- schema.sql del núcleo Rust (idempotente, se ejecuta con PRAGMA user_version como guard)
-- subconjunto V1 del schema completo de replica/schema.sql

create table if not exists symbols (
  id integer primary key,
  name text not null unique,
  asset_class text not null check (asset_class in ('forex','cfd','stock','future','crypto')),
  exchange text,
  tick_size real not null default 0.00001,
  pip_size real not null default 0.0001,
  commission real not null default 0,
  contract_size real not null default 100000,   -- unidades por lote (100000 forex, 50 ES, 1 acciones)
  swap_long real not null default 0,            -- dinero por lote y noche (swap del broker)
  swap_short real not null default 0,
  spread_asia real,                            -- spread en pips por sesion (null = plano)
  spread_london real,
  spread_ny real
);

create table if not exists data_files (
  id integer primary key,
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null check (timeframe in ('tick','m1','m5','m15','m30','h1','h4','d1','w1','mn')),
  storage_kind text not null check (storage_kind in ('local_qfd','imported_csv','dukascopy','yahoo')),
  file_path text not null,
  compressed_bytes integer not null default 0,
  n_bars integer not null default 0,
  date_from integer, date_to integer,
  status text not null default 'downloading' check (status in ('downloading','ready','corrupt')),
  created_at integer not null default (unixepoch())
);
create index if not exists idx_data_files_symbol on data_files(symbol_id, timeframe);
create unique index if not exists uq_data_files_source on data_files(symbol_id, timeframe, storage_kind);

create table if not exists strategy_templates (
  id integer primary key,
  name text not null,
  source text not null default 'user' check (source in ('user','built_in')),
  logic jsonb not null,
  created_at integer not null default (unixepoch())
);

create table if not exists strategies (
  id integer primary key,
  name text not null,
  created_at integer not null default (unixepoch()),
  symbol_id integer not null references symbols(id) on delete cascade,
  timeframe text not null,
  multi_charts jsonb not null default '[]',
  template_id integer references strategy_templates(id) on delete set null,
  origin text not null check (origin in ('builder','improver','optimizer','import')),
  logic jsonb not null,
  params jsonb not null default '{}',
  status text not null default 'active' check (status in ('active','dirty','archived')),
  problems jsonb not null default '[]',
  robust_score real,                      -- score de robustez 0-100 (ver engine/robust.rs)
  robust_detail jsonb                    -- componentes del score (wfe, mc, spp, perfil)
);
create index if not exists idx_strategies_bank on strategies(symbol_id, timeframe, status);

create table if not exists builds (
  id integer primary key,
  kind text not null check (kind in ('builder','retester','improver','optimizer','monte_carlo','custom_project')),
  config jsonb not null,
  status text not null default 'queued' check (status in ('queued','running','paused','done','error','cancelled')),
  n_passed integer not null default 0, n_total integer not null default 0,
  started_at integer, finished_at integer,
  parent_build_id integer references builds(id) on delete cascade
);

create table if not exists backtest_runs (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  data_file_id integer not null references data_files(id) on delete cascade,
  engine_cfg jsonb not null,
  build_id integer references builds(id) on delete cascade,
  net_profit real, gross_profit real, profit_factor real, win_rate real,
  max_dd real, rdd real, sharpe real, sortino real, z_score real, r_squared real,
  n_trades integer, avg_trade real, wfe real,
  equity jsonb,
  oos_frac real,
  status text not null default 'running' check (status in ('running','done','error','cancelled'))
);
create index if not exists idx_bt_strategy on backtest_runs(strategy_id, status);
create index if not exists idx_bt_np on backtest_runs(net_profit desc, profit_factor desc);
create index if not exists idx_bt_build on backtest_runs(build_id);

create table if not exists exports (
  id integer primary key,
  strategy_id integer not null references strategies(id) on delete cascade,
  platform text not null check (platform in ('mql4','mql5','ninjatrader','tradestation','multicharts','jforex')),
  file_name text,
  generated_at integer not null default (unixepoch()),
  checksum text
);

create table if not exists app_settings (
  key text primary key,
  value jsonb not null
);

-- Veredictos consolidados por estrategia (tarjeta de robustez).
-- Un registro por (estrategia, prueba): el ultimo insertado es el vigente.
create table if not exists strategy_verdicts (
  id integer primary key autoincrement,
  strategy_id integer not null references strategies(id) on delete cascade,
  check_key text not null check (check_key in ('walkForward','monteCarlo','spp','perfil')),
  estado text not null check (estado in ('pass','warn','fail','pending')),
  detalle text not null default '',
  valor real,
  method_version integer not null default 0,
  calculado_en integer not null default (unixepoch())
);
create index if not exists idx_verdicts_strategy
  on strategy_verdicts(strategy_id, check_key, id desc);

-- Historial de analisis: cada retesteo, refinamiento, optimizacion, matriz WF,
-- Monte Carlo, SPP o perfil queda registrado con su veredicto y sus metricas.
-- Sirve para comparar el antes y el despues de cambiar una regla.
create table if not exists analysis_runs (
  id integer primary key autoincrement,
  strategy_id integer not null references strategies(id) on delete cascade,
  kind text not null check (kind in
    ('retest','improver','optimizer','wf_matrix','monte_carlo','spp','opt_profile')),
  verdict text not null default 'pending' check (verdict in ('pending','running','pass','warn','fail')),
  label text not null default '',
  metrics jsonb,                -- metricas planas del analisis (net, PF, DD...)
  params jsonb not null default '{}',   -- configuracion usada (mercado, filtros, pasos...)
  detail jsonb,                -- resultado completo del comando
  created_at integer not null default (unixepoch())
);
create index if not exists idx_analysis_strategy on analysis_runs(strategy_id, kind, id desc);
create index if not exists idx_analysis_kind on analysis_runs(kind, id desc);

-- Corridas de cartera: modelo de ponderacion, pesos y metricas resultantes.
create table if not exists portfolio_runs (
  id integer primary key autoincrement,
  model text not null check (model in ('equal_weight','equal_vol','risk_parity','custom')),
  strategy_ids jsonb not null,
  weights jsonb not null,
  metrics jsonb not null,
  detail jsonb,
  created_at integer not null default (unixepoch())
);
create index if not exists idx_portfolio_runs on portfolio_runs(id desc);
