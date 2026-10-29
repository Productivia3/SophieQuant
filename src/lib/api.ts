// Capa de datos — firmas idénticas a los Tauri commands de replica/architecture.md.
// Híbrido: cuando corre dentro de la app Tauri (window.__TAURI__), los commands
// reales de Rust responden; fuera (navegador), cae a la implementación fake con seed.
// Contrato en `src/lib/types.ts` y `replica/architecture.md`.

import type {
  BuildRec, BuildKind, DataFileRec, Metrics, Platform,
  StrategyRec, SymbolRec, TemplateRec, BacktestRunRec, WorkstepEvent, Timeframe,
} from './types'

// ── seed ───────────────────────────────────────────────────────────

const now = Date.now()
const DAY = 86_400_000

const symbolsSeed: SymbolRec[] = [
  { id: 1, name: 'EURUSD', assetClass: 'forex', tickSize: 0.00001, pipSize: 0.0001, commission: 0 },
  { id: 2, name: 'GBPUSD', assetClass: 'forex', tickSize: 0.00001, pipSize: 0.0001, commission: 0 },
  { id: 3, name: 'USDJPY', assetClass: 'forex', tickSize: 0.001, pipSize: 0.01, commission: 0 },
  { id: 4, name: 'AAPL', assetClass: 'stock', exchange: 'NASDAQ', tickSize: 0.01, pipSize: 0.01, commission: 1 },
  { id: 5, name: 'ES', assetClass: 'future', exchange: 'CME', tickSize: 0.25, pipSize: 0.25, commission: 2.1 },
]

const dataFilesSeed: DataFileRec[] = [
  { id: 1, symbolId: 1, timeframe: 'tick', storageKind: 'dukascopy', compressedBytes: 842_000_000, nBars: 24_600_000, dateFrom: now - 5 * 365 * DAY, dateTo: now, status: 'ready', createdAt: now - 40 * DAY },
  { id: 2, symbolId: 1, timeframe: 'm1', storageKind: 'local_qfd', compressedBytes: 12_300_000, nBars: 1_890_000, dateFrom: now - 5 * 365 * DAY, dateTo: now, status: 'ready', createdAt: now - 39 * DAY },
  { id: 3, symbolId: 2, timeframe: 'm1', storageKind: 'dukascopy', compressedBytes: 9_900_000, nBars: 1_700_000, dateFrom: now - 3 * 365 * DAY, dateTo: now, status: 'ready', createdAt: now - 20 * DAY },
  { id: 4, symbolId: 2, timeframe: 'h1', storageKind: 'local_qfd', compressedBytes: 320_000, nBars: 26_000, dateFrom: now - 3 * 365 * DAY, dateTo: now, status: 'ready', createdAt: now - 20 * DAY },
]

// PRNG determinístico (mulberry32): dados estables entre recargas
function mulberry32(seed: number) {
  return function () {
    seed |= 0; seed = (seed + 0x6D2B79F5) | 0
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}
const rnd = mulberry32(20261020)

function synthStrategy(id: number, symbolId: number, tf: Timeframe): StrategyRec {
  const names = ['MA cross + RSI filter', 'BB reversion (fuzzy 60%)', 'Breakout ATR-stop',
    'Donchian trend, SLMR exit', 'Stoch divergence v2', 'MACD scalp H1', 'Channel pullback']
  const m = mkMetrics()
  return {
    id, name: `S${String(id).padStart(3, '0')} ${names[id % names.length]}`,
    createdAt: now - Math.floor(rnd() * 30) * DAY,
    symbolId, timeframe: tf, multiCharts: rnd() > 0.8 ? [{ symbolId: symbolId + 1, timeframe: 'h4' }] : [],
    templateId: null, origin: 'builder',
    logic: { entry: 'placeholder', exit: 'placeholder', blocks: 3 + (id % 5) },
    params: { period: 10 + (id % 20), threshold: +(rnd() * 2).toFixed(2) },
    status: rnd() > 0.95 ? 'dirty' : 'active',
    problems: rnd() > 0.9 ? ['Pico de operativa en oct 2023'] : [],
    lastRun: {
      id, strategyId: id, dataFileId: tf === 'h1' ? 4 : 2, engineCfg: { spread: 1, slippage: 1 },
      buildId: 1, metrics: m, equity: equitySeries(m), status: 'done',
    },
  }
}

function mkMetrics(): Metrics {
  const pf = +(0.9 + rnd() * 1.6).toFixed(3)
  return {
    netProfit: +(rnd() * 40_000 - 5_000).toFixed(2),
    grossProfit: +(rnd() * 90_000).toFixed(2),
    profitFactor: pf,
    winRate: +(0.42 + rnd() * 0.4).toFixed(3),
    maxDD: +(rnd() * 25 + 2).toFixed(2),
    rdd: +(rnd() * 270 + 10).toFixed(1),
    sharpe: +(rnd() * 3.4 - 0.4).toFixed(2),
    sortino: +(rnd() * 4 - 0.2).toFixed(2),
    zScore: +(rnd() * 4 - 2).toFixed(2),
    rSquared: +rnd().toFixed(3),
    nTrades: 40 + Math.floor(rnd() * 900),
    avgTrade: +(rnd() * 240 - 40).toFixed(2),
    wfe: +(rnd() * 100).toFixed(1),
  }
}

function equitySeries(m: Metrics): number[] {
  // random walk con drift según netProfit — solo para los gráficos fake
  const n = 500
  const out = [0]
  const drift = m.netProfit / n / 100
  for (let i = 1; i < n; i++) out.push(out[i - 1] + drift * 100 + (rnd() - 0.5) * 8)
  return out
}

const strategiesSeed: StrategyRec[] = Array.from({ length: 18 }, (_, i) =>
  synthStrategy(i + 1, i % 3 + 1, i % 2 ? 'h1' : 'm1'))

const templatesSeed: TemplateRec[] = [
  { id: 1, name: 'Base MA cross', source: 'built_in', logic: { kind: 'ma_cross' }, createdAt: now },
  { id: 2, name: 'Reversión osciladora', source: 'built_in', logic: { kind: 'osc_reversion' }, createdAt: now },
  { id: 3, name: 'Breakout con filtro de tendencia', source: 'built_in', logic: { kind: 'breakout_trend' }, createdAt: now },
]

// ── eventos de build (fake publisher) ──────────────────────────────

type BuildListener = (e: WorkstepEvent) => void
const listeners = new Set<BuildListener>()
export function onBuildEvent(fn: BuildListener): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}
function emit(e: WorkstepEvent) { listeners.forEach(l => l(e)) }

// ── implementación fake ────────────────────────────────────────────

const appSettings: AppSettings = {
  dataDir: 'C:\\SophieQuantData',
  proxy: '',
  theme: 'dark',
  language: 'es',
  autosave: true,
  engineThreads: 4,
}

const db = {
  symbols: symbolsSeed,
  dataFiles: dataFilesSeed,
  strategies: strategiesSeed,
  templates: templatesSeed,
  builds: [] as BuildRec[],
  runs: strategiesSeed.map(s => s.lastRun) as BacktestRunRec[],
  /** Historial de analisis (solo en el fake: en escritorio va en la base de datos). */
  analysisRuns: [] as AnalysisRun[],
}

const delay = (ms = 220) => new Promise(r => setTimeout(r, ms))

// ── puente Tauri: invoke real si existe la API global; si no, undefined ──
type TauriGlobal = {
  __TAURI__?: {
    core?: { invoke?: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T> }
  }
}
function isTauri(): boolean {
  return typeof window !== 'undefined' && typeof (window as unknown as TauriGlobal).__TAURI__ === 'object'
}
async function real<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  if (!isTauri()) return undefined
  try {
    const t = (window as unknown as TauriGlobal).__TAURI__
    const inv = t?.core?.invoke
    if (!inv) return undefined
    return await inv<T>(cmd, args)
  } catch (err) {
    console.warn(`[sophiequant] command ${cmd} falló — fallback a fake:`, err)
    return undefined
  }
}

/// invoke que NO se traga los errores: para acciones donde caer en un fake
/// silencioso haria creer al usuario que algo se ha guardado o importado.
async function realStrict<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  if (!isTauri()) return undefined
  const t = (window as unknown as TauriGlobal).__TAURI__
  const inv = t?.core?.invoke
  if (!inv) return undefined
  return await inv<T>(cmd, args)   // el error sube hasta la UI
}

// Normalización del shape de Rust → shape del front
function normBankRow(r: Record<string, unknown>): StrategyRec {
  // list_bank devuelve { lastRun: {...} }; get_strategy devuelve metrics/equity en la raiz
  const last = (r.lastRun ?? {
    equity: (r.equity as number[] | null | undefined) ?? null,
    oosFrac: (r.oosFrac as number | null | undefined) ?? null,
  }) as { equity?: number[] | null; status?: string; oosFrac?: number | null }
  const m = (r.metrics ?? null) as Metrics | null
  return {
    id: r.id as number,
    name: r.name as string,
    createdAt: 0,
    symbolId: 0,
    timeframe: (r.timeframe ?? 'm1') as Timeframe,
    multiCharts: [],
    templateId: null,
    origin: 'builder',
    logic: null,
    params: {},
    status: (r.status as StrategyRec['status']) ?? 'active',
    robustScore: (r.robustScore as number | null | undefined) ?? null,
    problems: [],
    lastRun: {
      id: r.id as number, strategyId: r.id as number, dataFileId: 0,
      engineCfg: {}, buildId: null,
      metrics: m,
      equity: last.equity ?? null,
      oosFrac: last.oosFrac ?? null,
      status: (last.status as 'done') ?? 'done',
    },
    symbol: r.symbol as string | undefined,
  }
}

export const api = {
  // data manager
  async listSymbols(): Promise<SymbolRec[]> {
    const r = await real<SymbolRec[]>('list_symbols')
    return r ?? db.symbols
  },
  async listDataFiles(): Promise<DataFileRec[]> {
    const r = await real<DataFileRec[]>('list_data_files')
    if (r) return r.map((d: any) => ({ ...d, symbolId: d.symbolId ?? d.symbolId }))
    return [...db.dataFiles]
  },
  async downloadData(symbolId: number, tf: Timeframe, from: number, to: number, onProgress?: (p: number) => void): Promise<DataFileRec> {
    // fake: simulación de descarga con progreso
    await delay(150)
    for (let p = 0.05; p <= 1; p += 0.1) { await delay(140); onProgress?.(Math.min(p, 1)) }
    const df: DataFileRec = {
      id: db.dataFiles.length + 1, symbolId, timeframe: tf, storageKind: 'dukascopy',
      compressedBytes: Math.floor(50e6 + Math.random() * 400e6), nBars: Math.floor(2e6 + Math.random() * 20e6),
      dateFrom: from, dateTo: to, status: 'ready', createdAt: Date.now(),
    }
    db.dataFiles.push(df)
    return df
  },

  // templates
  async listTemplates(): Promise<TemplateRec[]> {
    const r = await real<TemplateRec[]>('list_templates')
    return r ?? db.templates
  },

  // builds
  async startBuild(cfg: {
    kind: BuildKind
    symbolId: number
    timeframe: Timeframe
    nTarget: number
    templateId?: number
    filters?: MetricFilter[]
    minTrades?: number
    porRobustez?: boolean
  }): Promise<number> {
    const r = await real<number>('start_build', {
      kind: 'builder',
      symbolId: cfg.symbolId,
      timeframe: cfg.timeframe,
      templateId: cfg.templateId ?? 1,
      nTarget: cfg.nTarget,
      filters: cfg.filters ?? [
        { metric: 'profitFactor', op: '>=', value: 1.15 },
        { metric: 'maxDD', op: '<=', value: 35 },
      ],
      minTrades: cfg.minTrades ?? 20,
      porRobustez: cfg.porRobustez ?? true,
      monteCarlo: true,
      nIterations: 4000,
    })
    if (r != null) return r
    await delay(120)
    const id = db.builds.length + 1
    const b: BuildRec = { id, kind: cfg.kind, config: cfg as unknown as Record<string, unknown>, status: 'running', nPassed: 0, nTotal: cfg.nTarget, startedAt: Date.now(), finishedAt: null, parentBuildId: null }
    db.builds.push(b)
    // simulación: emite progreso y crea "estrategias" poco a poco
    ;(async () => {
      try {
        for (let i = 0; i < cfg.nTarget * 40; i++) {
          if (b.status !== 'running') break
          if (i % 10 === 0) {
            emit({ buildId: id, level: 'info', ts: Date.now(), message: `iteración ${i} — pasadas ${b.nPassed}/${cfg.nTarget}`, progress: i / (cfg.nTarget * 40) })
          }
          if (rnd() > 0.985 && b.nPassed < cfg.nTarget) {
            b.nPassed++
            const s = synthStrategy(db.strategies.length + 1, cfg.symbolId, cfg.timeframe)
            db.strategies.push(s)
            emit({ buildId: id, level: 'info', ts: Date.now(), message: `PASSED #${b.nPassed}: ${s.name} (PF ${s.lastRun?.metrics?.profitFactor})` })
          }
        }
        b.status = b.nPassed >= cfg.nTarget ? 'done' : 'done'
        b.finishedAt = Date.now()
        emit({ buildId: id, level: 'info', ts: Date.now(), message: `build completado — ${b.nPassed} estrategias pasaron los filtros`, progress: 1 })
      } catch (err) {
        b.status = 'error'
        emit({ buildId: id, level: 'error', ts: Date.now(), message: String(err) })
      }
    })()
    return id
  },
  async pauseBuild(id: number): Promise<void> { const ran = await real('pause_build', { id }); if (ran !== undefined) return; const b = db.builds.find(x => x.id === id); if (b) b.status = 'paused'; await delay(60) },
  async resumeBuild(id: number): Promise<void> { const ran = await real('resume_build', { id }); if (ran !== undefined) return; const b = db.builds.find(x => x.id === id); if (b) { b.status = 'running'; await delay(60); } },
  async cancelBuild(id: number): Promise<void> { await real('cancel_build'); const b = db.builds.find(x => x.id === id); if (b) { b.status = 'cancelled'; b.finishedAt = Date.now(); await delay(60) } },
  async listBuilds(): Promise<BuildRec[]> {
    const r = await real<BuildRec[]>('list_builds')
    return r ?? [...db.builds].reverse()
  },

  // bank
  async listBank(filter: { symbolId?: number }): Promise<StrategyRec[]> {
    const r = await real<StrategyRec[]>('list_bank')
    if (r) {
      const rows = r.map(x => normBankRow(x as unknown as Record<string, unknown>))
      return rows.filter(s => !filter?.symbolId || s.symbolId === filter.symbolId)
    }
    await delay(120)
    return db.strategies
      .filter(s => !filter?.symbolId || s.symbolId === filter.symbolId)
      .sort((a, b) => (b.lastRun?.metrics?.netProfit ?? -1e9) - (a.lastRun?.metrics?.netProfit ?? -1e9))
  },
  async getStrategy(id: number): Promise<StrategyRec | null> {
    const r = await real<Record<string, unknown>>('get_strategy', { id })
    if (r) return normBankRow(r)
    await delay(90)
    return db.strategies.find(s => s.id === id) ?? null
  },

  // robustez (fake)
  async runMonteCarlo(strategyId: number, cfg: Record<string, unknown> = {}): Promise<Verdict2> {
    void cfg
    const suite = await real<{ results?: Array<{ passed: boolean; passRate: number }> }>('run_mc_suite', { strategyId, tests: ['shuffle'] })
    if (suite) {
      const r0 = suite.results?.[0]
      return { testType: 'monte_carlo', verdict: r0?.passed ? 'pass' : 'fail', details: { trials: 100, passed: r0?.passRate ?? 0 } }
    }
    await delay(400)
    const s = db.strategies.find(x => x.id === strategyId)
    const v = s && s.lastRun?.metrics?.profitFactor != null && s.lastRun.metrics.profitFactor > 1.15 ? 'pass' : 'fail'
    return { testType: 'monte_carlo', verdict: v, details: { trials: 1000, passed: v === 'pass' ? 0.92 : 0.31 } }
  },

  // Monte Carlo multi-test (fake): corre los tipos seleccionados y devuelve veredicto por test
  async runMonteCarloSuite(strategyId: number, tests: string[]): Promise<McSuiteResult> {
    const r = await real<McSuiteResult['results']>('run_mc_suite', { strategyId, tests })
    if (r) return { strategyId, results: r }
    await delay(600)
    // (el registro en el historial lo hace el metodo real; aqui solo el fake)
    const s = db.strategies.find(x => x.id === strategyId)
    const base = s?.lastRun?.metrics?.profitFactor ?? 1
    return {
      strategyId,
      results: tests.map((t, i) => {
        const p = Math.max(0.02, Math.min(0.99, 0.95 - i * 0.11 - (base > 1.4 ? 0.1 : 0.35) + rnd() * 0.15))
        return {
          test: t,
          passed: p >= 0.5,
          passRate: +p.toFixed(2),
          percentiles: {
            p5: +(base * (0.35 + rnd() * 0.2)).toFixed(2),
            p50: +(base * (0.75 + rnd() * 0.15)).toFixed(2),
            p95: +(base * (0.95 + rnd() * 0.2)).toFixed(2),
          },
        }
      }),
    }
  },

  // WFMatrix (fake): heat map de combos (paramA × paramB) con clusters estables
  async runWfMatrix(strategyId: number, opts: { a: string; b: string; steps: number }): Promise<WfMatrixResult> {
    const r = await real<WfMatrixResult>('run_wfm', { strategyId, a: opts.a, b: opts.b, steps: opts.steps })
    if (r) return r
    await delay(700)
    const N = Math.max(3, Math.min(9, opts.steps))
    const aValues = Array.from({ length: N }, (_, i) => 5 + i * 5)   // p.ej. periodo 5..45
    const bValues = Array.from({ length: N }, (_, i) => +(0.5 + i * 0.25).toFixed(2)) // umbral
    const cells: WfMatrixResult['cells'] = []
    for (let i = 0; i < N; i++) {
      for (let j = 0; j < N; j++) {
        const stableZone = i >= 1 && i <= 4 && j >= 2 && j <= 4 // zona "estable" fake
        const wfe = stableZone ? 45 + rnd() * 30 : rnd() * 45
        cells.push({
          a: aValues[i], b: bValues[j],
          wfe: +wfe.toFixed(1),
          verdict: wfe >= 50 ? 'pass' : wfe >= 35 ? 'warn' : 'fail',
        })
      }
    }
    return {
      strategyId, paramA: opts.a, paramB: opts.b, aValues, bValues, cells,
      // El fake no puede acreditar suficiente histórico para emitir un veredicto positivo.
      insufficientOos: true,
      oosTradesApprox: 0,
      oosYearsApprox: 0,
      stopped: false,
    }
  },

  // candles + trades (fake, determinista por estrategia)
  async getStrategyChart(strategyId: number): Promise<ChartSeries> {
    const fetched = await real<ChartSeries>('get_strategy_chart', { strategyId })
    if (fetched) return fetched
    await delay(200)
    const r = mulberry32(1000 + strategyId)
    const candles: ChartSeries['candles'] = []
    let price = 1.0850
    for (let i = 0; i < 180; i++) {
      const o = price
      const drift = (r() - 0.48) * 0.0012
      const c = o + drift
      const hi = Math.max(o, c) + r() * 0.0008
      const lo = Math.min(o, c) - r() * 0.0008
      candles.push({ o, h: hi, l: lo, c })
      price = c
    }
    const trades: ChartSeries['trades'] = []
    for (let i = 20; i < 170; i += 14 + Math.floor(r() * 12)) {
      const dir = r() > 0.5 ? 'buy' : 'sell'
      trades.push({ idx: i, dir, entry: candles[i].c, exit: candles[Math.min(i + 8, 179)].c })
    }
    // mismo corte out-of-sample que el motor real (60 % in-sample)
    // overlays de ejemplo: medias sobre los cierres
    const closes = candles.map(c => c.c)
    const media = (period: number) => closes.map((_, i) => {
      if (i < period - 1) return null
      const trozo = closes.slice(i - period + 1, i + 1)
      return Math.round((trozo.reduce((a, b) => a + b, 0) / period) * 10000) / 10000
    })
    return {
      candles, trades,
      overlays: [
        { key: 'ema20', label: 'EMA 20', points: media(20) },
        { key: 'ema50', label: 'EMA 50', points: media(50) },
      ],
      oosFrom: 0.4,
      oosLabel: 'Fuera de muestra (ultimo 40 %)',
    }
  },

  // ── Fase 3: retest · improver · optimizer · SPP · portfolio · projects (fake) ──

  async runRetest(
    strategyIds: number[], symbolIds: number[], timeframe: Timeframe, periodos = 1,
  ): Promise<RetestResult> {
    const r = await real<RetestResult>('run_retest', { strategyIds, symbolIds, timeframe, periodos })
    if (r) return r
    await delay(900)
    // fake: cada mercado se comporta de forma distinta (a veces sin datos)
    return {
      timeframe,
      mercados: symbolIds.map(id => ({
        symbolId: id,
        simbolo: db.symbols.find(s => s.id === id)?.name ?? `#${id}`,
        // misma regla que el panel: hay dataset de ese simbolo en esa temporalidad
        conDatos: db.dataFiles.some(d =>
          d.symbolId === id && d.timeframe === timeframe && d.status === 'ready'),
      })),
      rows: strategyIds.map(id => {
        const base = db.strategies.find(s => s.id === id)?.lastRun?.metrics
        const mk = (escala: number): Metrics => ({
          ...(base ?? ({} as Metrics)),
          netProfit: (base?.netProfit ?? 0) * escala,
          profitFactor: Math.max(0.3, (base?.profitFactor ?? 1) * (0.8 + rnd() * 0.45)),
          maxDD: Math.min(95, (base?.maxDD ?? 10) * (1.1 + rnd() * 0.9)),
          nTrades: Math.max(1, Math.round((base?.nTrades ?? 10) * (0.6 + rnd() * 0.8))),
        })
        const ver = (m: Metrics): RetestMercado['verdict'] =>
          m.profitFactor >= 1.15 && m.netProfit > 0 ? 'pass' : m.profitFactor >= 0.95 ? 'warn' : 'fail'
        const mercados: RetestMercado[] = symbolIds.map(sid => {
          const conDatos = db.dataFiles.some(d =>
            d.symbolId === sid && d.timeframe === timeframe && d.status === 'ready')
          if (!conDatos) {
            return { symbolId: sid, simbolo: db.symbols.find(s => s.id === sid)?.name ?? `#${sid}`,
              verdict: 'sin datos' as const, nota: 'sin datos en este mercado', retest: null }
          }
          const m = mk(0.55 + rnd() * 0.8)
          return { symbolId: sid, simbolo: db.symbols.find(s => s.id === sid)?.name ?? `#${sid}`,
            verdict: ver(m), nota: '', retest: m }
        })
        const conDatos = mercados.filter(x => x.verdict !== 'sin datos')
        const pass = conDatos.filter(x => x.verdict === 'pass').length
        const fail = conDatos.filter(x => x.verdict === 'fail').length
        const warn = conDatos.filter(x => x.verdict === 'warn').length
        const resumen: RetestResumen = conDatos.length === 0
          ? { verdict: 'pending', detalle: 'ningun mercado con datos', mercadosConDatos: 0, pass: 0, warn: 0, fail: 0 }
          : fail === 0 && warn === 0
            ? { verdict: 'pass', detalle: `sobrevive en ${pass} de ${conDatos.length}`, mercadosConDatos: conDatos.length, pass, warn, fail }
            : pass > 0
              ? { verdict: 'warn', detalle: `sobrevive en ${pass} de ${conDatos.length}`, mercadosConDatos: conDatos.length, pass, warn, fail }
              : { verdict: 'fail', detalle: `falla en ${fail + warn} de ${conDatos.length}`, mercadosConDatos: conDatos.length, pass, warn, fail }
        this._fakeAnalysis(id, 'retest', resumen.verdict, `mock · ${mercados.length} mercados`,
          { ...(base ? { netProfit: base.netProfit, profitFactor: base.profitFactor, maxDD: base.maxDD } : {}) })
        return {
          strategyId: id,
          origen: { strategyId: id, original: base ?? null, retest: base ?? null, verdict: 'pass' as const },
          mercados,
          resumen,
        }
      }),
    }
  },

  async runImprover(strategyId: number): Promise<ImproverResult> {
    const r = await real<ImproverResult>('run_improver', { strategyId })
    if (r) return r
    await delay(1000)
    const s = db.strategies.find(x => x.id === strategyId)
    const base = s?.lastRun?.metrics
    const bip = (m: Metrics | null | undefined, f: number): Metrics | null =>
      m ? { ...m, netProfit: m.netProfit * (1 + f), profitFactor: m.profitFactor * (1 + f * 0.6) } : null
    const mk = (name: string, f: number) => ({
      name, metrics: bip(base, f),
      improvement: base ? (base.netProfit * f) : null,
    })
    const variants = [
      mk('Entrada: ATR(fast) en lugar de fixed', 0.08 + rnd() * 0.2),
      mk('Salida: time-stop de 40 barras', -0.1 + rnd() * 0.3),
      mk('Filtro: sesión europea', 0.05 + rnd() * 0.25),
      mk('Filtro: rango de ATR mínimo', -0.2 + rnd() * 0.4),
    ]
    const original = base ?? null
    const best = variants.reduce((b, v) => ((v.improvement ?? -1) > (b?.improvement ?? -1) ? v : b), variants[0])
    const bestMetrics = best.metrics && original && best.metrics.netProfit > original.netProfit ? best.metrics : null
    this._fakeAnalysis(strategyId, 'improver',
      (best.improvement ?? 0) > 0 ? 'pass' : 'warn',
      best.name,
      bestMetrics ? { netProfit: bestMetrics.netProfit, profitFactor: bestMetrics.profitFactor, maxDD: bestMetrics.maxDD } : {})
    return { strategyId, original, variants, bestVariant: bestMetrics ? best.name : null }
  },

  async runOptimizer(strategyId: number, opts: { paramA: string; paramB: string; steps: number; kind: 'simple' | 'wf' }): Promise<OptimizerResult> {
    const r = await real<OptimizerResult>('run_optimizer', { strategyId, paramA: opts.paramA, paramB: opts.paramB, steps: opts.steps, kind: opts.kind })
    if (r) return r
    await delay(1100)
    const N = Math.max(3, Math.min(9, opts.steps))
    const aValues = Array.from({ length: N }, (_, i) => 5 + i * 5)
    const bValues = Array.from({ length: N }, (_, i) => +(0.5 + i * 0.25).toFixed(2))
    const wfDiscount = opts.kind === 'wf' ? 0.55 : 1.0
    const cells = aValues.flatMap((a, i) =>
      bValues.map((b, j) => {
        const stable = i >= 1 && i <= 4 && j >= 2 && j <= 4
        const pf = (stable ? 1.6 + rnd() * 0.8 : 0.5 + rnd() * 1.1) * wfDiscount * (1 + rnd() * 0.1)
        return { a, b, pf: +pf.toFixed(2), maxDD: +(8 + rnd() * (stable ? 18 : 45)).toFixed(1) }
      }))
    const best = cells.reduce((m, c) => (c.pf > m.pf ? c : m), cells[0])
    this._fakeAnalysis(strategyId, 'optimizer',
      best.pf >= 1.15 ? 'pass' : best.pf >= 0.95 ? 'warn' : 'fail',
      `${opts.paramA}=${best.a} · ${opts.paramB}=${best.b}`,
      { pf: best.pf, maxDD: best.maxDD }, { ...opts })
    return { strategyId, kind: opts.kind, paramA: opts.paramA, paramB: opts.paramB, aValues, bValues, cells, best }
  },

  async runSpp(strategyId: number): Promise<SppResult> {
    const r = await real<SppResult>('run_spp', { strategyId })
    if (r) return r
    await delay(1200)
    const tests = [
      { key: 'entry_reversal', label: 'Entradas revertidas' },
      { key: 'exit_reversal', label: 'Salidas revertidas' },
      { key: 'trade_split', label: 'Operaciones divididas' },
      { key: 'opposite', label: 'Señal opuesta' },
      { key: 'param_permute', label: 'Permutación de parámetros' },
      { key: 'random_entry', label: 'Entradas aleatorias' },
    ]
    const s = db.strategies.find(x => x.id === strategyId)
    const strong = (s?.lastRun?.metrics?.profitFactor ?? 1) > 1.35
    const results = tests.map(t => {
      const impact = +(2 + rnd() * (strong ? 12 : 38)).toFixed(1)  // % de cambio del net
      return { ...t, impactPct: impact, verdict: impact < 10 ? ('pass' as const) : impact < 25 ? ('warn' as const) : ('fail' as const) }
    })
    const overall = results.every(r => r.verdict !== 'fail') ? 'pass' : 'fail'
    const peor = results.reduce((m, r) => Math.max(m, r.impactPct), 0)
    this._fakeAnalysis(strategyId, 'spp', overall, `impacto maximo ${peor.toFixed(1)} %`,
      { impactoMax: Math.round(peor * 10) / 10 })
    return { strategyId, results, overall }
  },

  async getCorrelationMatrix(strategyIds: number[]): Promise<CorrelationMatrix> {
    const r = await real<CorrelationMatrix>('get_correlation_matrix', { strategyIds })
    if (r) return r
    await delay(400)
    const n = strategyIds.length
    const ids = strategyIds
    const values: number[][] = []
    for (let i = 0; i < n; i++) {
      values[i] = []
      for (let j = 0; j < n; j++) {
        values[i][j] = i === j ? 1.0 : +(0.2 + Math.abs((ids[i] - ids[j]) % 5) / 10 + rnd() * 0.25).toFixed(2)
        if (values[i][j] > 1) values[i][j] = 0.97
      }
    }
    // simétrica
    for (let i = 0; i < n; i++)
      for (let j = 0; j < n; j++) values[j][i] = values[i][j]
    return { ids, values }
  },

  async getPortfolioEquity(strategyIds: number[]): Promise<PortfolioEquity> {
    const r = await real<PortfolioEquity>('get_portfolio_equity', { strategyIds })
    if (r) return r
    await delay(500)
    const members = strategyIds.map(id => {
      const r = mulberry32(2000 + id)
      const eq: number[] = [10_000]
      let bal = 10_000
      const m = db.strategies.find(s => s.id === id)?.lastRun?.metrics
      const edge = (m?.netProfit ?? 3000) > 0 ? 0.004 : -0.002   // razonable para el fake
      void r
      for (let i = 0; i < 240; i++) { bal *= (1 + edge + (rnd() - 0.5) * 0.01); eq.push(bal) }
      return { strategyId: id, equity: eq.map(v => +v.toFixed(2)) }
    })
    const combined = members[0]?.equity.map((_, i) =>
      +members.reduce((a, m) => a + m.equity[i] / members.length, 0).toFixed(2)) ?? []
    return { members, combined }
  },

  // custom projects (fake)
  async runProject(steps: string[]): Promise<ProjectRunResult> {
    const r = await real<ProjectRunResult>('run_project', { steps })
    if (r) return r
    await delay(300)
    const progressed: ProjectRunResult['steps'] = []
    for (const s of steps) {
      await delay(600)
      const fail = s === 'optimizer' && rnd() < 0.2   // paso opcional puede fallar
      progressed.push({ step: s, status: fail ? 'error' : 'done' })
      if (fail) break
    }
    return { steps: progressed, finishedAt: Date.now() }
  },

  // export
  async exportStrategy(strategyId: number, platform: Platform): Promise<ExportResult> {
    const r = await real<ExportResult>('export_strategy', { strategyId, platform })
    if (r) return r
    await delay(300)
    const s = db.strategies.find(x => x.id === strategyId)
    return {
      code: genStubCode(platform, s?.name ?? 'strategy'),
      verificacion: {
        regla: 'Regla de ejemplo',
        parametros: { p1: 20, p2: 50 },
        supuestos: [
          'Los datos son de BARRAS, no de ticks: dentro de la barra el EA ve un precio que el backtest no ve.',
          'El backtest aplico 1.0 pips de spread, 0.5 de slippage y 0.00 por lote de comision; el broker puede aplicar otros.',
          'El tamano de posicion fue fijo (0.1 lotes) y el balance inicial 10000 USD: no hay gestion de capital dinamica.',
          'La version actual solo opera en largo (entra y sale), sin vender en corto.',
          'No se aplicaron horarios ni filtros de sesion: la regla actua en todas las barras.',
          'Si el dataset tenia huecos, el backtest los relleno de forma continua.',
        ],
        checklist: [
          'Carga el EA en MT5 con los valores por defecto que trae el archivo.',
          'Usa el MISMO simbolo y la MISMA temporalidad que en el backtest.',
          'Comprueba el spread real del broker ahora y comparalo con el supuesto.',
          'Comprueba si tu broker cobra comision por lote y ajustala en el input del EA.',
          'Cuenta las operaciones: deben ser similares a las del backtest.',
          'Mira el informe del tester de MT5 y compara el equity linea a linea.',
          'Si el dataset tenia huecos, el backtest es mas optimista que el real.',
          'El backtest es en largo y de posicion fija: no anadas gestion de capital al EA.',
        ],
        plataforma: 'MetaTrader 4/5',
        documento: 'replica/export-verificacion.md',
      },
    }
  },
  async saveExportFile(fileName: string, code: string): Promise<boolean> {
    await delay(80) // en fake: solo confirma
    void fileName; void code
    return true
  },

  // ── Fase 4: profile de optimización · resultados de cartera ·
  // quant data manager · ajustes · analyzer (fake) ──

  async runOptProfile(strategyId: number): Promise<OptProfileResult> {
    const r = await real<OptProfileResult>('run_opt_profile', { strategyId })
    if (r) return r
    await delay(1100)
    // Perfil tipo Pardo: curva net vs valor del parámetro. Estrategia robusta = meseta
    // estable (unimodal, plana alrededor del óptimo); frágil = pico afilado de azar.
    const s = db.strategies.find(x => x.id === strategyId)
    const strong = (s?.lastRun?.metrics?.profitFactor ?? 1) > 1.3
    const N = 12
    const values = Array.from({ length: N }, (_, i) => 4 + i * 4)   // periodo 4..48
    const points = values.map((v, i) => {
      const center = 6   // ideal en i≈6 (valor 28)
      const d = Math.abs(i - center)
      const plateau = strong ? Math.exp(-d * d * 0.02) : Math.exp(-d * d * 0.35)
      const net = 4200 * plateau - 400 + rnd() * 350
      return { param: v, netProfit: +net.toFixed(2) }
    })
    const bestIdx = points.reduce((m, p, i) => (p.netProfit > points[m].netProfit ? i : m), 0)
    // meseta: rango de valores con net >= 80% del mejor
    const best = points[bestIdx].netProfit
    const ok = points.filter(p => p.netProfit >= best * 0.8).map(p => p.param)
    const shape = strong ? (Math.max(...ok) - Math.min(...ok) >= values[1] * 4 ? 'meseta' : 'unimodal') : 'puntual'
    this._fakeAnalysis(strategyId, 'opt_profile',
      shape === 'meseta' ? 'pass' : shape === 'unimodal' ? 'warn' : 'fail',
      `mejor ${points[bestIdx].param}`,
      { bestParam: points[bestIdx].param, stableFrom: Math.min(...ok), stableTo: Math.max(...ok) })
    return {
      strategyId, param: 'p1 · periodo', points,
      stableFrom: Math.min(...ok), stableTo: Math.max(...ok),
      shape,
      bestParam: points[bestIdx].param,
    }
  },

  /**
   * Calcula la cartera con un modelo de ponderacion y guarda la corrida.
   * Modelos: equiponderado, inverso a volatilidad, paridad de riesgo o
   * pesos propios.
   */
  async runPortfolioWeighting(
    strategyIds: number[],
    model: WeightingModel = 'equal_vol',
    customWeights: number[] = [],
  ): Promise<{ runId: number; resultado: PortfolioCalc }> {
    const r = await real<{ runId: number; resultado: PortfolioCalc }>('run_portfolio_weighting', {
      strategyIds, model, customWeights,
    })
    if (r) return r
    await delay(700)
    // fake: pesos coherentes con el modelo (la menos volatil pesa mas)
    const n = strategyIds.length
    const vols = strategyIds.map((id, i) => 4 + i)
    const brutos = vols.map((v, i) => {
      if (model === 'equal_weight') return 1
      if (model === 'equal_vol') return 1 / Math.max(v, 0.1)
      if (model === 'risk_parity') return 1 / Math.max(v, 0.1)
      return customWeights[i] ?? 1
    })
    const total = brutos.reduce((a, b) => a + b, 0) || 1
    const pesos = brutos.map(b => b / total)
    const miembros: PortfolioMember[] = strategyIds.map((id, i) => {
      const m = db.strategies.find(s => s.id === id)?.lastRun?.metrics ?? null
      return {
        strategyId: id,
        nombre: db.strategies.find(s => s.id === id)?.name ?? `#${id}`,
        metrics: m,
        correlation: 0.2,
        volatility: vols[i],
        weight: pesos[i],
        weightPct: Math.round(pesos[i] * 1000) / 10,
        riskContribution: Math.round(pesos[i] * 1000) / 10,
      }
    })
    const equity = [10_000, 10_400, 10_250, 10_700, 10_900]
    return {
      runId: 1,
      resultado: {
        model,
        modelLabel: model === 'equal_vol' ? 'Inverso a volatilidad' : model === 'risk_parity'
          ? 'Paridad de riesgo' : model === 'custom' ? 'Personalizado' : 'Equiponderado',
        miembros,
        combinada: {
          equity,
          maxDD: 6.4,
          netProfit: equity[equity.length - 1] - equity[0],
          volatility: 7.2,
        },
        portfolioVol: 7.2,
      },
    }
  },

  /** Historial de corridas de cartera. */
  async listPortfolioRuns(limit = 50): Promise<{ rows: Array<{ id: number; model: string; modelLabel: string; metrics: unknown; createdAt: number }> }> {
    const r = await real<{ rows: Array<{ id: number; model: string; modelLabel: string; metrics: unknown; createdAt: number }> }>(
      'list_portfolio_runs', { limit })
    if (r?.rows) return r
    await delay(100)
    return { rows: [] }
  },

  async getPortfolioResults(strategyIds: number[], model: WeightingModel = 'equal_vol', customWeights: number[] = []): Promise<PortfolioResults> {
    const fetched = await real<PortfolioResults>('get_portfolio_results', { strategyIds, model, customWeights })
    if (fetched) return fetched
    await delay(900)
    const members = strategyIds.map(id => {
      const r = mulberry32(3100 + id)
      const m = db.strategies.find(s => s.id === id)?.lastRun?.metrics
      const net = (m?.netProfit ?? 2500) * (0.7 + r() * 0.6)
      const corr = +(0.15 + r() * 0.65).toFixed(2)
      return {
        strategyId: id,
        netProfit: +net.toFixed(2),
        maxDD: +(5 + r() * 18).toFixed(1),
        correlation: corr,
        weight: +(100 / strategyIds.length).toFixed(1),
        verdict: corr >= 0.75 ? ('warn' as const) : ('pass' as const),
      }
    })
    const combinedNet = +members.reduce((a, m) => a + m.netProfit, 0).toFixed(2)
    const combinedDD = +Math.max(...members.map(m => m.maxDD)) * 0.62   // diversificación suaviza
    const r = mulberry32(9999)
    const combinedEquity: number[] = [10_000]
    let bal = 10_000
    for (let i = 0; i < 240; i++) {
      bal *= 1 + 0.0035 - (r() * 0.008)
      combinedEquity.push(+bal.toFixed(2))
    }
    return {
      members, combinedEquity,
      totals: {
        netProfit: combinedNet,
        maxDD: +combinedDD.toFixed(1),
        sharpe: +((1.4 + r() * 0.9)).toFixed(2),
        diversification: +((1 - combinedDD / Math.max(...members.map(m => m.maxDD))).toFixed(2)),
      },
    }
  },

  // Importacion: en escritorio usa el selector nativo (plugin de dialogo) y el
  // command real import_data_csv; en el navegador sigue el input de ficheros.
  // elija el CSV y pasárselo a import_data_csv (el command ya existe en Rust).
  /** ¿Estamos en la app de escritorio (Tauri) o en el navegador de desarrollo? */
  isDesktop(): boolean {
    return isTauri()
  },

  /**
   * Selector de ficheros nativo (plugin de diálogo de Tauri).
   * Devuelve la ruta elegida o null si el usuario cancela.
   * En el navegador (modo de desarrollo web) devuelve null: ahí se usa el
   * input[type=file] de siempre.
   */
  async pickCsvFile(): Promise<string | null> {
    if (!isTauri()) return null
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Datos de mercado (CSV)', extensions: ['csv', 'txt'] }],
    })
    return typeof res === 'string' ? res : null
  },

  /** Importa un CSV de verdad: el motor lo lee, lo limpia y lo guarda como QFN. */
  async importDatasetFileFromPath(
    path: string, symbolId: number, timeframe: string,
  ): Promise<{ dataset: DataFileRec; calidad: ImportQuality }> {
    const r = await realStrict<{ dataset: DataFileRec; calidad: ImportQuality }>(
      'import_data_csv', { symbolId, timeframe, path })
    if (!r) throw new Error('La importacion real solo funciona en la app de escritorio.')
    return r
  },

  async importDatasetFile(fileName: string, sizeBytes: number): Promise<DataFileRec> {
    await delay(700)
    void sizeBytes
    const df: DataFileRec = {
      id: 1000 + db.dataFiles.length,
      symbolId: 1,
      timeframe: 'm1',
      storageKind: 'imported_csv',
      compressedBytes: Math.round(sizeBytes || 4_200_000),
      nBars: 950_000 + Math.floor(rnd() * 400_000),
      dateFrom: now - 2 * 365 * DAY,
      dateTo: now - 3 * DAY,
      status: 'ready',
      createdAt: Date.now(),
    }
    db.dataFiles.push(df)
    return df
  },

  async exportDataset(dataFileId: number, format: 'csv' | 'qfd'): Promise<{ fileName: string; bytes: number }> {
    const r = await real<{ fileName: string; bytes: number }>('export_dataset', { dataFileId, format })
    if (r) return r
    await delay(500)
    const df = db.dataFiles.find(d => d.id === dataFileId)
    const sym = db.symbols.find(s => s.id === df?.symbolId)?.name ?? 'DATA'
    return {
      fileName: `${sym}_${df?.timeframe ?? 'm1'}.${format}`,
      bytes: df?.compressedBytes ?? 0,
    }
  },

  /**
   * Guarda la regla editada y recalcula el backtest sobre el dataset de la
   * estrategia. El motor solo ejecuta las tres reglas que expone el editor:
   * no se ofrecen bloques que el motor no sepa evaluar.
   */
  async saveStrategyLogic(
    strategyId: number,
    templateId: number,
    params: number[],
    logic: Record<string, unknown>,
  ): Promise<SavedStrategy> {
    const r = await realStrict<SavedStrategy>('save_strategy_logic', { strategyId, templateId, params, logic })
    if (r) return r
    await delay(500)
    const strategy = db.strategies.find(s => s.id === strategyId)
    if (!strategy) throw new Error(`Estrategia ${strategyId} no encontrada`)
    return {
      strategyId,
      nombre: strategy.name,
      plantilla: 'vista previa',
      params: { p: params },
      metrics: strategy.lastRun?.metrics ?? null,
      equity: strategy.lastRun?.equity ?? [],
      oosFrac: 0.4,
      nTrades: strategy.lastRun?.metrics?.nTrades ?? 0,
      preview: true,
    }
  },

  /** Registra una entrada en el historial del fake (el escritorio usa la DB). */
  _fakeAnalysis(
    strategyId: number,
    kind: string,
    verdict: AnalysisRun['verdict'],
    label: string,
    metrics: Record<string, number | string | null>,
    params: Record<string, unknown> = {},
  ): void {
    const nombres: Record<string, string> = {
      retest: 'Retesteo', improver: 'Refinador', optimizer: 'Optimizador',
      wf_matrix: 'Matriz WF', monte_carlo: 'Monte Carlo',
      spp: 'Permutaciones (SPP)', opt_profile: 'Perfil de optimizacion',
    }
    db.analysisRuns.push({
      id: db.analysisRuns.length + 1,
      strategyId,
      nombre: db.strategies.find(s => s.id === strategyId)?.name ?? `#${strategyId}`,
      kind,
      kindLabel: nombres[kind] ?? kind,
      verdict,
      label,
      metrics,
      params,
      detail: null,
      createdAt: Math.floor(Date.now() / 1000),
      comparacion: [],
      hayPrevia: false,
    })
  },

  /**
   * Historial de analisis. En escritorio sale de la base de datos; en el
   * navegador, del almacen en memoria (para poder probar el flujo).
   */
  async listAnalysisRuns(
    strategyId?: number,
    kind?: string,
    limit = 200,
  ): Promise<{ rows: AnalysisRun[]; kinds: AnalysisKind[] }> {
    const r = await real<{ rows: AnalysisRun[]; kinds: AnalysisKind[] }>(
      'list_analysis_runs', { strategyId: strategyId ?? null, kind: kind ?? null, limit })
    if (r?.rows) return r

    await delay(120)
    const kinds: AnalysisKind[] = [
      { key: 'retest', label: 'Retesteo' },
      { key: 'improver', label: 'Refinador' },
      { key: 'optimizer', label: 'Optimizador' },
      { key: 'wf_matrix', label: 'Matriz WF' },
      { key: 'monte_carlo', label: 'Monte Carlo' },
      { key: 'spp', label: 'Permutaciones (SPP)' },
      { key: 'opt_profile', label: 'Perfil de optimizacion' },
    ]
    const todas = db.analysisRuns
      .filter(r => strategyId == null || r.strategyId === strategyId)
      .filter(r => !kind || r.kind === kind)
      .sort((a, b) => b.id - a.id)
      .slice(0, limit)
    const rows = todas.map(r => {
      const previa = db.analysisRuns
        .filter(x => x.strategyId === r.strategyId && x.kind === r.kind && x.id < r.id)
        .sort((a, b) => b.id - a.id)[0]
      const comparacion = previa
        ? Object.keys(r.metrics)
            .filter(k => typeof r.metrics[k] === 'number' && typeof previa.metrics[k] === 'number')
            .map(k => ({
              metrica: k,
              antes: previa.metrics[k] as number,
              despues: r.metrics[k] as number,
              delta: Math.round(((r.metrics[k] as number) - (previa.metrics[k] as number)) * 100) / 100,
            }))
        : []
      return { ...r, comparacion, hayPrevia: !!previa }
    })
    return { rows, kinds }
  },

  /** Resamplea un dataset a una temporalidad mayor y lo registra. */
  async resampleDataset(dataFileId: number, targetTf: string): Promise<DataFileRec> {
    const r = await realStrict<DataFileRec>('resample_dataset', { dataFileId, targetTf })
    if (!r) throw new Error('El resample real solo funciona en la app de escritorio.')
    return r
  },

  /** Temporalidades a las que se puede resamplear un dataset. */
  async listResampleTargets(timeframe: string): Promise<string[]> {
    const r = await real<{ targets: string[] }>('list_resample_targets', { timeframe })
    if (r?.targets) return r.targets
    const todos = ['m1', 'm5', 'm15', 'm30', 'h1', 'h4', 'd1', 'w1', 'mn1']
    const min: Record<string, number> = {
      m1: 1, m5: 5, m15: 15, m30: 30, h1: 60, h4: 240, d1: 1440, w1: 10080, mn1: 43200,
    }
    const f = min[timeframe] ?? 1
    return todos.filter(t => (min[t] ?? 0) > f)
  },

  /**
   * Abre el selector de ficheros nativo y devuelve la ruta elegida.
   * `extensions` filtra por tipo (p. ej. ['py']).
   */
  async pickFile(extensions: string[] = []): Promise<string | null> {
    if (!isTauri()) return null
    const { open } = await import('@tauri-apps/plugin-dialog')
    const res = await open({
      multiple: false,
      directory: false,
      filters: extensions.length ? [{ name: extensions.join('/'), extensions }] : undefined,
    })
    return typeof res === 'string' ? res : null
  },

  /**
   * Ejecuta un script de Python sobre los datos de la estrategia.
   * El script recibe --input <json> y debe escribir --output <json>.
   */
  async runCustomAnalysis(
    strategyId: number, scriptPath: string,
  ): Promise<{ id: number; resultado: { veredicto: string; etiqueta: string; metricas: Record<string, number>; notas: string }; input: unknown; script: string }> {
    const r = await realStrict<{ id: number; resultado: { veredicto: string; etiqueta: string; metricas: Record<string, number>; notas: string }; input: unknown; script: string }>(
      'run_custom_analysis', { strategyId, scriptPath })
    if (!r) throw new Error('El analisis personalizado solo funciona en la app de escritorio.')
    return r
  },

  /** Historial de analisis personalizados. */
  async listCustomAnalyses(strategyId?: number): Promise<Array<{ id: number; script: string; verdict: string; label: string; output: { veredicto: string; etiqueta: string; metricas: Record<string, number>; notas: string }; createdAt: number }>> {
    const r = await real<{ rows: Array<{ id: number; script: string; verdict: string; label: string; output: { veredicto: string; etiqueta: string; metricas: Record<string, number>; notas: string }; createdAt: number }> }>(
      'list_custom_analyses', { strategyId: strategyId ?? null })
    if (r?.rows) return r.rows
    await delay(80)
    return []
  },

  /** Exporta las barras de la estrategia a CSV (util para scripts). */
  async exportStrategyCsv(strategyId: number): Promise<string> {
    const r = await realStrict<string>('export_strategy_csv', { strategyId })
    if (!r) throw new Error('La exportacion real solo funciona en la app de escritorio.')
    return r
  },

  /** Borra una entrada del historial. */
  async deleteAnalysisRun(id: number): Promise<boolean> {
    const r = await real<boolean>('delete_analysis_run', { id })
    if (r != null) return r
    db.analysisRuns = db.analysisRuns.filter(x => x.id !== id)
    return true
  },

  /** Metricas por las que se puede filtrar y pesos del score (los pide el motor). */
  async listBuildMetrics(): Promise<{ metrics: BuildMetric[]; pesos: RobustnessWeight[] }> {
    const r = await real<{ metrics: BuildMetric[]; pesos: RobustnessWeight[] }>('list_build_metrics')
    if (r?.metrics) return r
    return {
      metrics: [
        { key: 'profitFactor', label: 'Profit factor' },
        { key: 'sharpe', label: 'Sharpe' },
        { key: 'sortino', label: 'Sortino' },
        { key: 'maxDD', label: 'Drawdown máximo (%)' },
        { key: 'rdd', label: 'Recovery factor' },
        { key: 'netProfit', label: 'Beneficio neto' },
        { key: 'grossProfit', label: 'Beneficio bruto' },
        { key: 'winRate', label: 'Tasa de acierto' },
        { key: 'zScore', label: 'Z-score' },
        { key: 'rSquared', label: 'R²' },
        { key: 'avgTrade', label: 'Beneficio medio por operación' },
        { key: 'nTrades', label: 'Número de operaciones' },
      ],
      pesos: [
        { key: 'wfe', peso: 0.35, desc: 'Eficiencia walk-forward: cuánto se conserva fuera de muestra' },
        { key: 'mc', peso: 0.3, desc: 'Supervivencia frente al remuestreo de operaciones' },
        { key: 'spp', peso: 0.2, desc: 'Estabilidad frente a las permutaciones de la regla (SPP)' },
        { key: 'perfil', peso: 0.15, desc: 'Forma del perfil: meseta frente a pico aislado' },
      ],
    }
  },

  /**
   * Sugiere que estrategias meter en la cartera, con el motivo de cada descarte.
   */
  async suggestPortfolio(
    candidateIds: number[], maxCorrelation = 0.7, maxMembers = 8,
  ): Promise<{
    elegidos: Array<{ strategyId: number; nombre: string; volatility: number; calidad: number; pesoSugerido: number }>
    descartados: Array<{ strategyId: number; nombre: string; motivo: string; correlacion?: number }>
    notas: string
  }> {
    const r = await real<{
      elegidos: Array<{ strategyId: number; nombre: string; volatility: number; calidad: number; pesoSugerido: number }>
      descartados: Array<{ strategyId: number; nombre: string; motivo: string; correlacion?: number }>
      notas: string
    }>('suggest_portfolio', { candidateIds, maxCorrelation, maxMembers })
    if (r) return r
    await delay(500)
    // fake: las cuatro primeras se eligen y el resto se descarta por solape
    const elegidos = candidateIds.slice(0, Math.min(4, candidateIds.length)).map((id, i) => ({
      strategyId: id,
      nombre: db.strategies.find(s => s.id === id)?.name ?? `#${id}`,
      volatility: 4 + i,
      calidad: 1.2 - i * 0.1,
      pesoSugerido: 100 / Math.min(4, candidateIds.length),
    }))
    const descartados = candidateIds.slice(elegidos.length).map(id => ({
      strategyId: id,
      nombre: db.strategies.find(s => s.id === id)?.name ?? `#${id}`,
      motivo: 'se solapa con las elegidas (correlacion 0.81)',
      correlacion: 0.81,
    }))
    return {
      elegidos, descartados,
      notas: 'seleccion greedily por calidad (beneficio por volatilidad) evitando correlaciones por encima de 0.70',
    }
  },

  /**
   * Perfil de riesgo de una estrategia: MAE/MFE, expectativa y capital diario.
   */
  async analyzeStrategyRisk(strategyId: number): Promise<{
    maePct: number; mfePct: number; peorMaePct: number; mejorMfePct: number
    expectativa: number; expectativaPct: number; nTrades: number
    capitalDiario: Array<{ t: number; v: number }>
    notas: string
  }> {
    const r = await realStrict<{
      maePct: number; mfePct: number; peorMaePct: number; mejorMfePct: number
      expectativa: number; expectativaPct: number; nTrades: number
      capitalDiario: Array<{ t: number; v: number }>
      notas: string
    }>('analyze_strategy_risk', { strategyId })
    if (!r) throw new Error('El analisis de riesgo real solo funciona en la app de escritorio.')
    return r
  },

  /** Metadatos de mercado de los simbolos (los que usa el motor de verdad). */
  async listSymbolMeta(): Promise<Array<{
    id: number; name: string; assetClass: string
    tickSize: number; pipSize: number; commission: number; contractSize: number
    swapLong: number; swapShort: number
    spreadAsia: number | null; spreadLondon: number | null; spreadNy: number | null
  }>> {
    const r = await real<{ symbols: Array<{
      id: number; name: string; assetClass: string
      tickSize: number; pipSize: number; commission: number; contractSize: number
      swapLong: number; swapShort: number
      spreadAsia: number | null; spreadLondon: number | null; spreadNy: number | null
    }> }>('get_symbol_meta')
    if (r?.symbols) return r.symbols
    await delay(80)
    return []
  },

  /** Actualiza los metadatos de mercado de un simbolo. */
  async updateSymbolMeta(symbolId: number, meta: {
    tickSize?: number; pipSize?: number; commission?: number; contractSize?: number
    swapLong?: number; swapShort?: number
    spreadAsia?: number | null; spreadLondon?: number | null; spreadNy?: number | null
  }): Promise<boolean> {
    const r = await realStrict<boolean>('update_symbol_meta', { symbolId, ...meta })
    if (r == null) return false
    return r
  },

  /** Veredictos ya calculados (cache en la DB). */
  async listVerdicts(strategyIds: number[]): Promise<StrategyVerdicts[]> {
    const r = await real<{ rows: StrategyVerdicts[] }>('get_strategy_verdicts', { strategyIds })
    if (r?.rows) return r.rows
    await delay()
    return strategyIds.map(id => ({
      strategyId: id,
      nombre: db.strategies.find(s => s.id === id)?.name ?? `#${id}`,
      checks: {
        walkForward: { estado: 'pending' as const, detalle: 'sin calcular', valor: null },
        monteCarlo: { estado: 'pending' as const, detalle: 'sin calcular', valor: null },
        spp: { estado: 'pending' as const, detalle: 'sin calcular', valor: null },
        perfil: { estado: 'pending' as const, detalle: 'sin calcular', valor: null },
      },
      overall: 'pending' as const,
      calculadoEn: null,
    }))
  },

  /** Calcula los cuatro veredictos con el motor real y los guarda. */
  async computeVerdicts(strategyIds: number[]): Promise<StrategyVerdicts[]> {
    const r = await real<{ rows: StrategyVerdicts[] }>('compute_strategy_verdicts', { strategyIds })
    if (r?.rows) return r.rows
    await delay(600)
    return strategyIds.map(id => ({
      strategyId: id,
      nombre: db.strategies.find(s => s.id === id)?.name ?? `#${id}`,
      checks: {
        walkForward: { estado: 'warn' as const, detalle: 'WFE 48.2 % · umbral 50 %', valor: 48.2 },
        monteCarlo: { estado: 'pass' as const, detalle: '9/9 pruebas superadas', valor: 1 },
        spp: { estado: 'pass' as const, detalle: 'impacto máximo 6.4 %', valor: 6.4 },
        perfil: { estado: 'warn' as const, detalle: 'unimodal · zona estable 12.0–24.0', valor: 12 },
      },
      overall: 'warn' as const,
      calculadoEn: Math.floor(Date.now() / 1000),
    }))
  },

  async getSettings(): Promise<AppSettings> {
    await delay(150)
    return { ...appSettings }
  },

  async updateSettings(patch: Partial<AppSettings>): Promise<AppSettings> {
    await delay(250)
    Object.assign(appSettings, patch)
    return { ...appSettings }
  },

  async runPortfolioMc(strategyIds: number[]): Promise<PortfolioMcResult> {
    const r = await real<PortfolioMcResult>('run_portfolio_mc', { strategyIds })
    if (r) return r
    await delay(1000)
    const trials = 500
    const base = mulberry32(4242)
    const eq: number[] = [10_000]
    let bal = 10_000
    for (let i = 0; i < 240; i++) {
      bal *= 1 + 0.003 - base() * 0.01
      eq.push(+bal.toFixed(2))
    }
    const p5 = eq[Math.floor(eq.length * 0.05)]
    const p50 = eq[Math.floor(eq.length * 0.5)]
    const p95 = eq[eq.length - 1]
    void strategyIds
    return { trials, p5, p50, p95, ruinProb: +(base() * 0.04).toFixed(3) }
  },
}

export interface Verdict2 {
  testType: 'monte_carlo'
  verdict: 'pass' | 'fail'
  details: { trials: number; passed: number }
}

export interface McSuiteResult {
  strategyId: number
  results: Array<{
    test: string
    passed: boolean
    passRate: number
    percentiles: { p5: number; p50: number; p95: number }
  }>
}

export interface WfMatrixResult {
  strategyId: number
  paramA: string
  paramB: string
  aValues: number[]
  bValues: number[]
  cells: Array<{ a: number; b: number; wfe: number; verdict: 'pass' | 'warn' | 'fail' }>
  /** Bandera si la muestra OOS es insuficiente (< 30 trades o < 1 año). */
  insufficientOos?: boolean
  /** Approximate number of OOS trades (proxy a partir del PF de la mejor celda). */
  oosTradesApprox?: number
  /** Approximate number of OOS years (proxy a partir de los tiempos de las trades). */
  oosYearsApprox?: number
  /** Bandera si la equity OOS rompió la regla de parada (DD > 80% del peak IS). */
  stopped?: boolean
}

export interface ChartSeries {
  candles: Array<{ o: number; h: number; l: number; c: number }>
  trades: Array<{ idx: number; dir: 'buy' | 'sell'; entry: number; exit: number }>
  /** Medias dibujadas sobre el precio (EMA 20/50, SMA 200), alineadas con las velas. */
  overlays?: Array<{ key: string; label: string; points: Array<number | null> }>
  /** Fraccion del dataset que es out-of-sample (mismo corte que la prueba walk-forward). */
  oosFrom?: number
  oosLabel?: string
}

// ── Fase 3 ──
export interface RetestRow {
  strategyId: number
  original: Metrics | null
  retest: Metrics | null
  verdict: 'pass' | 'warn' | 'fail'
}
export interface RetestMercado {
  symbolId: number
  simbolo: string
  /** pass | warn | fail | sin datos */
  verdict: 'pass' | 'warn' | 'fail' | 'sin datos'
  nota: string
  retest: Metrics | null
  nBars?: number
}

export interface RetestResumen {
  verdict: 'pass' | 'warn' | 'fail' | 'pending'
  detalle: string
  mercadosConDatos: number
  pass: number
  warn: number
  fail: number
}

export interface RetestPeriodo {
  indice: number
  desde: number
  hasta: number
  barras: number
  verdict: 'pass' | 'warn' | 'fail'
  nota: string
  metrics: Metrics
}

export interface RetestConsistencia {
  verdict: 'pass' | 'warn' | 'fail' | 'pending'
  detalle: string
  pasados: number
  total: number
  peorPF?: number
}

export interface RetestFila {
  strategyId: number
  /** out-of-sample dentro de su propio mercado */
  origen: RetestRow
  /** ventanas OOS evaluadas por separado */
  periodos?: RetestPeriodo[]
  /** consistencia entre esas ventanas */
  consistencia?: RetestConsistencia
  /** la misma regla en cada mercado destino */
  mercados: RetestMercado[]
  resumen: RetestResumen
}

export interface RetestResult {
  timeframe: string
  mercados: Array<{ symbolId: number; simbolo: string; conDatos: boolean }>
  rows: RetestFila[]
}
export interface ImproverResult {
  strategyId: number
  original: Metrics | null
  variants: Array<{ name: string; metrics: Metrics | null; improvement: number | null }>
  bestVariant: string | null
}
export interface OptimizerResult {
  strategyId: number
  kind: 'simple' | 'wf'
  paramA: string
  paramB: string
  aValues: number[]
  bValues: number[]
  cells: Array<{ a: number; b: number; pf: number; maxDD: number }>
  best: { a: number; b: number; pf: number; maxDD: number }
}
export interface SppResult {
  strategyId: number
  results: Array<{ key: string; label: string; impactPct: number; verdict: 'pass' | 'warn' | 'fail' }>
  overall: 'pass' | 'fail'
}
export interface CorrelationMatrix {
  ids: number[]
  values: number[][]
}
// ── Editor sin código: guardar y recalcular ─────────────────────────
export interface SavedStrategy {
  strategyId: number
  nombre: string
  plantilla: string
  params: { p: number[] }
  metrics: Metrics | null
  equity: number[]
  oosFrac: number
  nTrades: number
  preview?: boolean
}

// ── Verificacion del export ─────────────────────────────────────────
export interface ExportVerificacion {
  regla: string
  parametros: Record<string, number>
  supuestos: string[]
  checklist: string[]
  plataforma: string
  documento: string
}

export interface ExportResult {
  code: string
  verificacion: ExportVerificacion
}

// ── Diagnostico de calidad de un CSV importado ─────────────────────
export interface ImportQuality {
  nBars: number
  skipped: number
  dateFrom: number
  dateTo: number
  avisos: string[]
  huecos: number
  huecoMax: number
  sinVolumen: number
  preciosIncoherentes: number
  timestampsRepetidos: number
  limpio: boolean
}

// ── Filtros del generador (cualquier metrica del motor) ──────────────
export interface MetricFilter {
  metric: string
  op: '>=' | '<=' | '>' | '<'
  value: number
}

export interface BuildMetric {
  key: string
  label: string
}

export interface RobustnessWeight {
  key: string
  peso: number
  desc: string
}

// ── Historial de analisis ───────────────────────────────────────────
export interface AnalysisRun {
  id: number
  strategyId: number
  nombre: string
  kind: string
  kindLabel: string
  verdict: 'pass' | 'warn' | 'fail' | 'pending' | 'running'
  label: string
  metrics: Record<string, number | string | null>
  params: Record<string, unknown>
  detail: unknown
  createdAt: number
  /** Delta frente a la corrida anterior del mismo tipo. */
  comparacion: Array<{ metrica: string; antes: number; despues: number; delta: number }>
  hayPrevia: boolean
}

export interface AnalysisKind {
  key: string
  label: string
}

// ── Veredictos consolidados (tarjeta de robustez) ────────────────────
export type VerdictEstado = 'pass' | 'warn' | 'fail' | 'pending'

export interface VerdictCheck {
  estado: VerdictEstado
  detalle: string
  valor: number | null
}

export interface StrategyVerdicts {
  strategyId: number
  nombre: string
  checks: {
    walkForward: VerdictCheck
    monteCarlo: VerdictCheck
    spp: VerdictCheck
    perfil: VerdictCheck
  }
  overall: VerdictEstado
  calculadoEn: number | null
}

export const VERDICT_LABELS: Record<keyof StrategyVerdicts['checks'], string> = {
  walkForward: 'Walk-forward',
  monteCarlo: 'Monte Carlo',
  spp: 'Permutaciones (SPP)',
  perfil: 'Perfil de optimización',
}

export type WeightingModel = 'equal_weight' | 'equal_vol' | 'risk_parity' | 'custom'

export interface PortfolioMember {
  strategyId: number
  nombre: string
  metrics: Metrics | null
  correlation: number
  volatility: number
  weight: number
  weightPct: number
  riskContribution: number
}

export interface PortfolioCalc {
  model: WeightingModel
  modelLabel: string
  miembros: PortfolioMember[]
  combinada: {
    equity: number[]
    maxDD: number
    netProfit: number
    volatility: number
  }
  portfolioVol: number
}

export interface PortfolioEquity {
  members: Array<{ strategyId: number; equity: number[] }>
  combined: number[]
}
export interface ProjectRunResult {
  steps: Array<{ step: string; status: 'done' | 'error' }>
  finishedAt: number
}

// ── Fase 4 ──
export interface OptProfilePoint {
  param: number
  netProfit: number
}
export interface OptProfileResult {
  strategyId: number
  param: string
  points: OptProfilePoint[]
  stableFrom: number
  stableTo: number
  shape: 'meseta' | 'unimodal' | 'puntual'
  bestParam: number
}
export interface PortfolioMemberResult {
  strategyId: number
  netProfit: number
  maxDD: number
  correlation: number
  weight: number
  verdict: 'pass' | 'warn'
}
export interface PortfolioResults {
  members: PortfolioMemberResult[]
  combinedEquity: number[]
  totals: {
    netProfit: number
    maxDD: number
    sharpe: number
    diversification: number   // 0..1, cuánta DD se evitó por diversificar
  }
}
export interface AppSettings {
  dataDir: string
  proxy: string
  theme: 'dark' | 'light'
  language: string
  autosave: boolean
  engineThreads: number
}
export interface PortfolioMcResult {
  trials: number
  p5: number
  p50: number
  p95: number
  ruinProb: number
}

// stub de generación de código (Fase 1: MQL5 real vendrá del Rust core)
function genStubCode(platform: Platform, name: string): string {
  const guard = `// SophieQuant export — ${platform.toUpperCase()} (stub de build V1)\n// Estrategia: ${name}\n`
  if (platform === 'mql5' || platform === 'mql4') {
    return guard + `
#property strict
// inputs generados
input int InpPeriod = 20;
input double InpThreshold = 1.20;

// NOTE: implementación completa generada por el motor Rust; este stub solo valida el pipeline.`
  }
  return guard + `// código fuente para ${platform} — pendiente (Fase 3 exporters)`
}