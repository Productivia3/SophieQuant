// Tipos del dominio — espejo exacto del schema.sql / architecture.md
// Cuando el núcleo Rust (Tauri) esté conectado, estas estructuras son los DTO de los commands.

export type AssetClass = 'forex' | 'cfd' | 'stock' | 'future' | 'crypto'
export type Timeframe = 'tick' | 'm1' | 'm5' | 'm15' | 'm30' | 'h1' | 'h4' | 'd1' | 'w1' | 'mn'
export type DataStatus = 'downloading' | 'ready' | 'corrupt'
export type StrategyOrigin = 'builder' | 'improver' | 'optimizer' | 'import'
export type BuildKind = 'builder' | 'retester' | 'improver' | 'optimizer' | 'monte_carlo' | 'custom_project'
export type BuildStatus = 'queued' | 'running' | 'paused' | 'done' | 'error' | 'cancelled'
export type RobustnessTest =
  | 'monte_carlo' | 'wf_optimization' | 'wf_matrix'
  | 'spp' | 'optimization_profile' | 'oos_check' | 'mc_portfolio'
export type Verdict = 'pending' | 'running' | 'pass' | 'fail'
export type Platform = 'mql4' | 'mql5' | 'ninjatrader' | 'tradestation' | 'multicharts' | 'jforex'

export interface SymbolRec {
  id: number
  name: string
  assetClass: AssetClass
  exchange?: string
  tickSize: number
  pipSize: number
  commission: number
}

export interface DataFileRec {
  id: number
  symbolId: number
  timeframe: Timeframe
  storageKind: 'local_qfd' | 'imported_csv' | 'dukascopy' | 'yahoo'
  compressedBytes: number
  nBars: number
  dateFrom: number | null
  dateTo: number | null
  status: DataStatus
  createdAt: number
}

export interface Metrics {
  netProfit: number
  grossProfit: number
  profitFactor: number
  winRate: number
  maxDD: number
  rdd: number
  sharpe: number
  sortino: number
  zScore: number
  rSquared: number
  nTrades: number
  avgTrade: number
  wfe: number
  /** Sharpe/Sortino anualizados (base diaria, comparable con MT5). */
  sharpeAnual?: number
  sortinoAnual?: number
  /** MAE/MFE medios en % del precio de entrada. */
  maePct?: number
  mfePct?: number
}

export interface BacktestRunRec {
  id: number
  strategyId: number
  dataFileId: number
  engineCfg: Record<string, unknown>
  buildId: number | null
  metrics: Metrics | null
  equity: number[] | null
  /** Fraccion de la serie que es out-of-sample (corte walk-forward). */
  oosFrac?: number | null
  status: 'running' | 'done' | 'error' | 'cancelled'
}

export interface StrategyRec {
  id: number
  name: string
  createdAt: number
  symbolId: number
  timeframe: Timeframe
  multiCharts: Array<{ symbolId: number; timeframe: Timeframe }>
  templateId: number | null
  origin: StrategyOrigin
  logic: unknown
  params: Record<string, unknown>
  status: 'active' | 'dirty' | 'archived'
  /** Score de robustez 0-100 asignado por el generador. */
  robustScore?: number | null
  problems: string[]
  symbol?: string     // presente cuando viene del núcleo Rust
  lastRun?: BacktestRunRec | null
}

export interface BuildRec {
  id: number
  kind: BuildKind
  config: Record<string, unknown>
  status: BuildStatus
  nPassed: number
  nTotal: number
  startedAt: number | null
  finishedAt: number | null
  parentBuildId: number | null
}

export interface TemplateRec {
  id: number
  name: string
  source: 'user' | 'built_in'
  logic: unknown
  createdAt: number
}

export interface WorkstepEvent {
  buildId: number
  level: 'info' | 'warn' | 'error'
  message: string
  ts: number
  progress?: number // 0..1
}