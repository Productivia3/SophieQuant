// phase3.tsx — pantallas de Fase 3: Retesteo, Refinador, Optimizador+3D, SPP,
// Proyectos y Cartera maestra. Clean-room sobre la capa fake.
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Play, Repeat, Layers, Ruler, ArrowUp, ArrowDown, X, Grid3x3, FolderKanban, PieChart } from 'lucide-react'
import { api } from '../lib/api'
import type {
  CorrelationMatrix, ImproverResult, OptimizerResult, PortfolioEquity,
  RetestResult, SppResult,
} from '../lib/api'
import type { StrategyRec, SymbolRec, Timeframe } from '../lib/types'
import { Page } from '../components/Page'
import { Button } from '../components/Button'
import { Badge } from '../components/Badge'
import { EmptyState } from '../components/EmptyState'
import { TableSkeleton } from '../components/TableSkeleton'
import { EquityChart } from '../components/EquityChart'
import { StatCard } from '../components/StatCard'
import { fmtMoney, fmtNum } from '../lib/utils'

const selectCls = 'h-8 rounded-md border border-border-input bg-surface px-2 text-sm'

function verdictTone(v: 'pass' | 'warn' | 'fail') {
  return v === 'pass' ? 'pass' : v === 'warn' ? 'running' : 'fail'
}
function verdictLabel(v: 'pass' | 'warn' | 'fail') {
  return v === 'pass' ? 'PASA' : v === 'warn' ? 'ALERTA' : 'FALLA'
}

/* ───────────────────────── S08 · Retesteo ───────────────────────── */

export function S08Retester() {
  const [rows, setRows] = useState<StrategyRec[] | null>(null)
  const [symbols, setSymbols] = useState<SymbolRec[]>([])
  const [selIds, setSelIds] = useState<number[]>([])
  const [symbolId, setSymbolId] = useState<number>(1)
  const [timeframe, setTimeframe] = useState<Timeframe>('h1')
  const [busy, setBusy] = useState(false)
  const [result, setResult] = useState<RetestResult | null>(null)

  useEffect(() => {
    api.listBank({}).then(rs => setRows(rs))
    api.listSymbols().then(ss => { setSymbols(ss); setSymbolId(ss[0]?.id ?? 1) })
  }, [])

  const toggle = (id: number) =>
    setSelIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  async function run() {
    if (!selIds.length) return
    setBusy(true); setResult(null)
    const res = await api.runRetest(selIds, symbolId, timeframe)
    setResult(res); setBusy(false)
  }

  const tfs: Timeframe[] = ['m1', 'm5', 'm15', 'm30', 'h1', 'h4', 'd1']
  const symName = (id: number) => symbols.find(s => s.id === id)?.name ?? `#${id}`

  return (
    <Page title="Retesteo" icon={<Repeat size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!selIds.length} onClick={run}>
          <Play size={14} aria-hidden /> Retesteo de {selIds.length || '—'}
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="space-y-4">
          <div className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Estrategias del banco</h2>
            {!rows ? <TableSkeleton /> : rows.length === 0 ? (
              <EmptyState title="El banco está vacío" hint="Genera una estrategia primero." />
            ) : (
              <ul className="max-h-80 divide-y divide-border overflow-auto qf-scroll text-sm" aria-label="Selección de estrategias">
                {rows.map(r => (
                  <li key={r.id}>
                    <label className="flex cursor-pointer items-center gap-2 px-2 py-1.5 hover:bg-surface-raised">
                      <input type="checkbox" aria-label={`Retestear ${r.name}`}
                        checked={selIds.includes(r.id)} onChange={() => toggle(r.id)} />
                      <span className="flex-1 truncate">{r.name}</span>
                      <span className="font-mono text-xs text-muted">{fmtMoney(r.lastRun?.metrics?.netProfit)}</span>
                    </label>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Mercado del retesteo</h2>
            <label className="mb-2 block text-xs text-muted" htmlFor="ret-symbol">Símbolo</label>
            <select id="ret-symbol" className={`w-full ${selectCls}`} value={symbolId}
              onChange={e => setSymbolId(+e.target.value)}>
              {symbols.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
            <label className="mb-2 mt-3 block text-xs text-muted" htmlFor="ret-tf">Temporalidad</label>
            <select id="ret-tf" className={`w-full ${selectCls}`} value={timeframe}
              onChange={e => setTimeframe(e.target.value as Timeframe)}>
              {tfs.map(tf => <option key={tf} value={tf}>{tf}</option>)}
            </select>
          </div>
        </div>

        <div className="min-w-0">
          {!result ? (
            <EmptyState title="Sin retesteo todavía"
              hint="Marca estrategias del banco, elige otro símbolo/timeframe y corre el retesteo para ver cómo se comportan fuera de su mercado de origen." />
          ) : (
            <div className="overflow-x-auto qf-scroll">
              <table className="w-full min-w-[760px] text-sm">
                <thead>
                  <tr className="bg-surface text-left text-xs text-muted">
                    <th className="px-3 py-2 font-medium">Estrategia</th>
                    <th className="px-3 py-2 font-medium">Original</th>
                    <th className="px-3 py-2 font-medium">Retest</th>
                    <th className="px-3 py-2 font-medium">PF</th>
                    <th className="px-3 py-2 font-medium">DD</th>
                    <th className="px-3 py-2 font-medium">Veredicto</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {result.rows.map(r => (
                    <tr key={r.strategyId} className="hover:bg-surface-raised">
                      <td className="px-3 py-2">
                        <Link className="text-accent hover:underline"
                          to={`/strategy/${r.strategyId}`}>
                          #{r.strategyId}
                        </Link>
                      </td>
                      <td className="px-3 py-2 font-mono">{fmtMoney(r.original?.netProfit)}</td>
                      <td className={`px-3 py-2 font-mono ${(r.retest?.netProfit ?? 0) >= 0 ? 'text-success' : 'text-danger'}`}>
                        {fmtMoney(r.retest?.netProfit)}
                      </td>
                      <td className="px-3 py-2 font-mono">{fmtNum(r.retest?.profitFactor)}</td>
                      <td className="px-3 py-2 font-mono">{fmtNum(r.retest?.maxDD, 1)}%</td>
                      <td className="px-3 py-2">
                        <Badge tone={verdictTone(r.verdict)} label={verdictLabel(r.verdict)} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <p className="mt-2 text-xs text-muted">
                Mercado del retesteo: <b className="text-text">{symName(result.symbolId)} · {result.timeframe}</b>
              </p>
            </div>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S09 · Improver ───────────────────────── */

export function S09Improver() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<ImproverResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null)
    setRes(await api.runImprover(sel.id))
    setBusy(false)
  }

  return (
    <Page title="Refinador" icon={<Layers size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel} onClick={run}>
          <Play size={14} aria-hidden /> Mejorar
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Estrategia base</h2>
          <select aria-label="Estrategia" className={`w-full ${selectCls}`}
            value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          {sel?.lastRun?.metrics && (
            <div className="mt-3 grid grid-cols-2 gap-2 text-xs">
              <StatCard label="Net (original)" value={sel.lastRun.metrics.netProfit} unit="USD" />
              <StatCard label="PF (original)" value={sel.lastRun.metrics.profitFactor} />
            </div>
          )}
        </div>

        <div className="min-w-0 space-y-3">
          {!res ? (
            <EmptyState title="Sin variantes todavía"
              hint="El refinador genera variantes de entrada/salida con filtros extra y compara con el original: conserva la mejor." />
          ) : (
            <>
              {res.bestVariant ? (
                <p className="rounded-md border border-success/40 bg-success/10 p-3 text-sm text-success" role="status">
                  Mejor variante: <b>{res.bestVariant}</b> — supera al original.
                </p>
              ) : (
                <p className="rounded-md border border-border bg-surface p-3 text-sm text-muted" role="status">
                  Ninguna variante supera al original: <b>se conserva el original</b>.
                </p>
              )}
              <div className="overflow-x-auto qf-scroll">
                <table className="w-full min-w-[640px] text-sm">
                  <thead>
                    <tr className="bg-surface text-left text-xs text-muted">
                      <th className="px-3 py-2 font-medium">Variante</th>
                      <th className="px-3 py-2 font-medium">Net</th>
                      <th className="px-3 py-2 font-medium">PF</th>
                      <th className="px-3 py-2 font-medium">Δ vs original</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border">
                    {res.variants.map(v => (
                      <tr key={v.name} className={v.name === res.bestVariant ? 'bg-success/5' : 'hover:bg-surface-raised'}>
                        <td className="px-3 py-2">{v.name}{v.name === res.bestVariant && (
                          <span className="ml-2 rounded-pill bg-success/15 px-1.5 py-0.5 text-xs text-success">mejor</span>
                        )}</td>
                        <td className="px-3 py-2 font-mono">{fmtMoney(v.metrics?.netProfit)}</td>
                        <td className="px-3 py-2 font-mono">{fmtNum(v.metrics?.profitFactor)}</td>
                        <td className={`px-3 py-2 font-mono ${(v.improvement ?? 0) >= 0 ? 'text-success' : 'text-danger'}`}>
                          {(v.improvement ?? 0) >= 0 ? '+' : ''}{fmtMoney(v.improvement)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S10 · Optimizer + S11 · 3D ─────────────── */

function cellColor(pf: number): string {
  // escala de veredicto a tokens propios (pas/warn/fail)
  if (pf >= 1.5) return 'var(--color-success)'
  if (pf >= 1.15) return 'var(--color-warning)'
  return 'var(--color-danger)'
}

/** Mini-superficie isométrica en SVG (bind propia, sin librerías). */
function IsoSurface({ res }: { res: OptimizerResult }) {
  const size = res.aValues.length
  const cell = 46, dx = cell * 0.9, dy = cell * 0.5
  const W = size * dx * 2 + 120, H = size * dy * 2 + 200
  const proj = (i: number, j: number, z: number): [number, number] =>
    [W / 2 + (j - i) * dx, 60 + (j + i) * dy - z]

  const bars = []
  for (let i = size - 1; i >= 0; i--) {
    for (let j = size - 1; j >= 0; j--) {
      const c = res.cells.find(x => x.a === res.aValues[i] && x.b === res.bValues[j])
      if (!c) continue
      const z = (c.pf - 0.4) * 46
      const [x0, y0] = proj(i, j, z)
      const [x1, y1] = proj(i + 1, j, z)
      const [x2, y2] = proj(i + 1, j + 1, z)
      const [x3, y3] = proj(i, j + 1, z)
      const [, yBase] = proj(i, j, 0)
      bars.push(
        <g key={`${i}-${j}`}>
          {/* cara frontal (hacia vista) */}
          <polygon points={`${x1},${y1} ${x2},${y2} ${x2},${yBase + 4} ${x1},${yBase + 4}`}
            fill={cellColor(c.pf)} opacity={0.85} />
          <polygon points={`${x0},${y0} ${x1},${y1} ${x1},${yBase} ${x0},${yBase}`}
            fill={cellColor(c.pf)} opacity={0.65} />
          <polygon points={`${x0},${y0} ${x1},${y1} ${x2},${y2} ${x3},${y3}`}
            fill={cellColor(c.pf)} opacity={0.95}
            stroke={c.a === res.best.a && c.b === res.best.b ? 'var(--color-text)' : 'var(--color-border)'}
            strokeWidth={c.a === res.best.a && c.b === res.best.b ? 2 : 0.5} />
        </g>,
      )
    }
  }
  return (
    <div className="overflow-auto qf-scroll rounded-md border border-border bg-surface p-2">
      <svg width={W} height={H} role="img" aria-label={`Superficie 3D de profit factor — mejor combo ${res.best.a}·${res.best.b}`}>
        {bars}
        <text x={16} y={H - 8} fill="var(--color-text-muted)" fontSize={12}>
          {res.paramA} → {res.aValues[0]}…{res.aValues[size - 1]} · {res.paramB} → {res.bValues[0]}…{res.bValues[size - 1]} · modo {res.kind === 'wf' ? 'Walk-Forward' : 'Simple'}
        </text>
      </svg>
    </div>
  )
}

export function S10Optimizer() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [kind, setKind] = useState<'simple' | 'wf'>('simple')
  const [steps, setSteps] = useState(6)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<OptimizerResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null)
    setRes(await api.runOptimizer(sel.id, { paramA: 'p1 · periodo', paramB: 'p2 · umbral', steps, kind }))
    setBusy(false)
  }

  return (
    <Page title="Optimizador" icon={<Grid3x3 size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel} onClick={run}>
          <Play size={14} aria-hidden /> Optimizar {kind === 'wf' ? '(Walk-Forward)' : '(Simple)'}
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Estrategia</h2>
          <select aria-label="Estrategia a optimizar" className={`w-full ${selectCls}`}
            value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          <fieldset className="mt-3">
            <legend className="mb-2 text-xs text-muted">Modo</legend>
            <div className="flex gap-2" role="radiogroup" aria-label="Modo de optimización">
              {(['simple', 'wf'] as const).map(k => (
                <button key={k} type="button" role="radio" aria-checked={kind === k}
                  onClick={() => setKind(k)}
                  className={`rounded-md border px-3 py-1.5 text-sm ${kind === k ? 'border-accent text-accent' : 'border-border text-muted hover:border-border-input'}`}>
                  {k === 'simple' ? 'Simple' : 'Walk-Forward'}
                </button>
              ))}
            </div>
          </fieldset>
          <label className="mt-3 block text-xs text-muted" htmlFor="opt-steps">
            Pasos por parámetro: <span className="font-mono text-text">{steps}</span>
          </label>
          <input id="opt-steps" type="range" min={3} max={9} value={steps}
            aria-label="Pasos por parámetro"
            onChange={e => setSteps(+e.target.value)} className="w-full" />
        </div>

        <div className="min-w-0 space-y-3">
          {!res ? (
            <EmptyState title="Sin optimización todavía"
              hint="Corre una optimización para ver la cuadrícula de combinaciones y la superficie 3D del profit factor." />
          ) : (
            <>
              <div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
                <StatCard label="Mejor PF" value={res.best.pf} tone={res.best.pf >= 1.5 ? 'pass' : 'fail'} />
                <StatCard label="Mejor combo" value={`${res.best.a}·${res.best.b}`} mono={false} />
                <StatCard label="Max DD del mejor" value={res.best.maxDD} unit="%" tone="fail" />
              </div>
              <IsoSurface res={res} />
              <p className="text-xs text-muted">
                El marcado es el mejor grupo estable. En modo Walk-Forward el PF se calcula
                también en los segmentos fuera de muestra (EFW separado en la WFMatrix, S12).
              </p>
            </>
          )}
        </div>
      </div>
    </Page>
  )
}

// S11 reusa la herramienta al detalle de una estrategia concreta
export function S11Opt3D() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<OptimizerResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null)
    setRes(await api.runOptimizer(sel.id, { paramA: 'p1 · periodo', paramB: 'p2 · umbral', steps: 7, kind: 'wf' }))
    setBusy(false)
  }

  useEffect(() => { if (sel && !res && !busy) void run() /* lint-safe: correr una vez */ }, [sel, res, busy])
  void run

  return (
    <Page title="Superficie 3D" icon={<Ruler size={16} />}
      actions={
        <select aria-label="Estrategia para superficie" className={selectCls} value={sel?.id ?? ''}
          onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
          {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
        </select>
      }>
      <div className="p-4">
        {res ? <IsoSurface res={res} /> : <TableSkeleton />}
      </div>
    </Page>
  )
}

/* ───────────────────────── S14 · SPP / Perfil de optimización ─────── */

export function S14Robustness() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<SppResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null)
    setRes(await api.runSpp(sel.id))
    setBusy(false)
  }

  const bar = (impact: number) => Math.min(100, impact * 2.2)

  return (
    <Page title="SPP · pruebas de robustez" icon={<PieChart size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel} onClick={run}>
          <Play size={14} aria-hidden /> Correr SPP
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Estrategia</h2>
          <select aria-label="Estrategia para SPP" className={`w-full ${selectCls}`}
            value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          <p className="mt-3 text-xs text-muted">
            SPP = permutaciones de señal: revierte entradas/salidas, parte operaciones o
            mueve parámetros. Si la estrategia depende del azar, el net lo paga.
          </p>
          <Link to="/profile" className="mt-3 inline-block text-sm text-accent hover:underline">
            Ver Perfil de optimización →
          </Link>
        </div>

        <div className="min-w-0 space-y-3">
          {!res ? (
            <EmptyState title="Sin SPP todavía"
              hint="Corre las pruebas para saber si la estrategia es robusta: con señales rotas casi no debería cambiar el resultado." />
          ) : (
            <>
              <div className="flex items-center gap-2">
                <Badge tone={res.overall === 'pass' ? 'pass' : 'fail'} />
                <span className="text-sm text-muted">
                  {res.overall === 'pass'
                    ? 'Sin dependencias frágiles detectadas.'
                    : 'Hay permutaciones que rompen la estrategia: señales frágiles.'}
                </span>
              </div>
              <ul className="space-y-2">
                {res.results.map(r => (
                  <li key={r.key} className="rounded-md border border-border bg-surface p-3">
                    <div className="mb-1 flex items-center justify-between gap-2">
                      <span className="text-sm">{r.label}</span>
                      <Badge tone={verdictTone(r.verdict)} label={verdictLabel(r.verdict)} />
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted">
                      <div className="h-1.5 flex-1 overflow-hidden rounded-pill bg-border">
                        <div className="h-full rounded-pill"
                          style={{ width: `${bar(r.impactPct)}%`, background: 'var(--color-warning)' }} />
                      </div>
                      <span className="font-mono">{r.impactPct}%</span>
                    </div>
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S16+S17 · Proyectos ────────────── */

const projectSteps = [
  { key: 'build', label: 'Generación (plugin constructor)' },
  { key: 'monte_carlo', label: 'Monte Carlo' },
  { key: 'spp', label: 'SPP (robustez)' },
  { key: 'optimizer', label: 'Optimizador WF' },
  { key: 'save', label: 'Guardar en el banco' },
]

export function S16Projects() {
  const [pipeline, setPipeline] = useState<string[]>(['build', 'monte_carlo', 'save'])
  const [busy, setBusy] = useState(false)
  const [statuses, setStatuses] = useState<Array<{ step: string; status: 'done' | 'error' }> | null>(null)

  const add = (k: string) => setPipeline(p => [...p, k])
  const del = (i: number) => setPipeline(p => p.filter((_, x) => x !== i))
  const move = (i: number, d: -1 | 1) => setPipeline(p => {
    const j = i + d
    if (j < 0 || j >= p.length) return p
    const cp = [...p]
    ;[cp[i], cp[j]] = [cp[j], cp[i]]
    return cp
  })

  async function run() {
    if (!pipeline.length) return
    setBusy(true); setStatuses(null)
    const r = await api.runProject(pipeline)
    setStatuses(r.steps); setBusy(false)
  }

  const statusOf = (key: string) => statuses?.find(s => s.step === key)?.status
  const ok = statuses && statuses.length === pipeline.length && statuses.every(s => s.status === 'done')

  return (
    <Page title="Proyectos" icon={<FolderKanban size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!pipeline.length} onClick={run}>
          <Play size={14} aria-hidden /> Ejecutar workflow
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Añadir paso</h2>
          <div className="flex flex-wrap gap-2">
            {projectSteps.map(s => (
              <button key={s.key} type="button" onClick={() => add(s.key)}
                className="rounded-md border border-border px-2 py-1 text-xs hover:border-border-input">
                + {s.label}
              </button>
            ))}
          </div>
        </div>

        <div className="min-w-0 space-y-3">
          {pipeline.length === 0 ? (
            <EmptyState title="Workflow vacío" hint="Añade pasos: el proyecto los ejecuta en cadena y guarda el resultado en el banco." />
          ) : (
            <ol className="space-y-2" aria-label="Pasos del workflow">
              {pipeline.map((k, i) => {
                const st = statusOf(k)
                return (
                  <li key={`${k}-${i}`} className="flex items-center gap-2 rounded-md border border-border bg-surface p-3">
                    <span className="flex h-6 w-6 items-center justify-center rounded-full bg-surface-raised font-mono text-xs">
                      {i + 1}
                    </span>
                    <span className="flex-1 text-sm">{projectSteps.find(s => s.key === k)?.label ?? k}</span>
                    {st === 'done' && <Badge tone="pass" />}
                    {st === 'error' && <Badge tone="fail" />}
                    {busy && !st && <Badge tone="running" />}
                    <Button size="sm" variant="ghost" aria-label={`Subir paso ${i + 1}`} disabled={busy || i === 0} onClick={() => move(i, -1)}>
                      <ArrowUp size={13} aria-hidden />
                    </Button>
                    <Button size="sm" variant="ghost" aria-label={`Bajar paso ${i + 1}`} disabled={busy || i === pipeline.length - 1} onClick={() => move(i, 1)}>
                      <ArrowDown size={13} aria-hidden />
                    </Button>
                    <Button size="sm" variant="ghost" aria-label={`Quitar paso ${i + 1}`} disabled={busy} onClick={() => del(i)}>
                      <X size={13} aria-hidden />
                    </Button>
                  </li>
                )
              })}
            </ol>
          )}
          {statuses && (
            <p className={`rounded-md border p-3 text-sm ${ok ? 'border-success/40 bg-success/10 text-success' : 'border-danger/40 bg-danger/10 text-danger'}`} role="status">
              {ok ? 'Workflow completado: los resultados están en el banco.' : 'Workflow detenido en un paso fallido — revisa la configuración.'}
            </p>
          )}
          <p className="text-xs text-muted">
            Nota honesta de Fase 3: el editor es una lista ordenada de pasos (sin drag&drop
            de nodos todavía). El canvas gráfico llega en la parrilla completa.
          </p>
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S18 · Portfolio maestra ───────────────── */

export function S18Portfolio() {
  const [rows, setRows] = useState<StrategyRec[] | null>(null)
  const [selIds, setSelIds] = useState<number[]>([])
  const [busy, setBusy] = useState(false)
  const [corr, setCorr] = useState<CorrelationMatrix | null>(null)
  const [eq, setEq] = useState<PortfolioEquity | null>(null)
  const [names, setNames] = useState<Record<number, string>>({})

  useEffect(() => {
    api.listBank({}).then(rs => {
      setRows(rs)
      const map: Record<number, string> = {}
      rs.forEach(r => { map[r.id] = r.name })
      setNames(map)
      setSelIds(rs.slice(0, 4).map(r => r.id))   // primera selección práctica
    })
  }, [])

  async function run() {
    if (selIds.length < 2) return
    setBusy(true)
    const [c, e] = await Promise.all([api.getCorrelationMatrix(selIds), api.getPortfolioEquity(selIds)])
    setCorr(c); setEq(e); setBusy(false)
  }

  const toggle = (id: number) =>
    setSelIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  return (
    <Page title="Cartera maestra" icon={<PieChart size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={selIds.length < 2} onClick={run}>
          <Play size={14} aria-hidden /> Analizar cartera ({selIds.length})
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Estrategias (2–8)</h2>
          {!rows ? <TableSkeleton /> : (
            <ul className="max-h-96 divide-y divide-border overflow-auto qf-scroll text-sm" aria-label="Cartera">
              {rows.map(r => (
                <li key={r.id}>
                  <label className="flex cursor-pointer items-center gap-2 px-2 py-1.5 hover:bg-surface-raised">
                    <input type="checkbox" aria-label={`Añadir ${r.name} a la cartera`}
                      checked={selIds.includes(r.id)} onChange={() => toggle(r.id)} />
                    <span className="flex-1 truncate">{r.name}</span>
                    <span className="font-mono text-xs text-muted">{fmtMoney(r.lastRun?.metrics?.netProfit)}</span>
                  </label>
                </li>
              ))}
            </ul>
          )}
          <Link to="/portfolio/results" className="mt-3 inline-block text-sm text-accent hover:underline">
            Ver resultados de la cartera →
          </Link>
        </div>

        <div className="min-w-0 space-y-4">
          {!corr || !eq ? (
            <EmptyState title="Sin análisis de cartera"
              hint="Selecciona 2+ estrategias y corre: verás la correlación entre ellas y la equity combinada." />
          ) : (
            <>
              <div>
                <h2 className="mb-2 text-base font-semibold">Correlación entre miembros</h2>
                <div className="overflow-auto qf-scroll">
                  <table className="min-w-[560px] text-sm">
                    <thead>
                      <tr className="text-xs text-muted">
                        <th className="px-2 py-1"></th>
                        {corr.ids.map(id => <th key={id} className="px-2 py-1 font-mono">#{id}</th>)}
                      </tr>
                    </thead>
                    <tbody>
                      {corr.values.map((row, i) => (
                        <tr key={i}>
                          <td className="px-2 py-1 font-mono text-xs text-muted">#{corr.ids[i]}</td>
                          {row.map((v, j) => {
                            const high = v >= 0.75 && i !== j
                            const alpha = v
                            return (
                              <td key={j} className="px-1 py-1 text-center">
                                <span className="inline-flex h-8 w-12 items-center justify-center rounded font-mono text-xs"
                                  aria-label={`Correlación entre ${names[corr.ids[i]]} y ${names[corr.ids[j]]}: ${v}`}
                                  style={{
                                    background: v <= 0.35
                                      ? `rgba(122,162,255,${alpha * 0.5})`
                                      : high ? 'rgba(239,90,90,0.35)' : `rgba(250,190,60,${alpha * 0.5})`,
                                    color: alpha > 0.6 ? 'var(--color-text)' : 'var(--color-text-muted)',
                                  }}>
                                  {v.toFixed(2)}
                                </span>
                              </td>
                        )
                          })}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <p className="mt-2 text-xs text-muted">Rojo = correlación ≥ 0.75: diversificación débil.
                En el motor real (Rust) la matriz se calcula sobre los returns alineados por fecha.</p>
              </div>

              <div>
                <h2 className="mb-2 text-base font-semibold">Equity combinada (media de familias)</h2>
                <div className="rounded-md border border-border bg-surface p-3">
                  <EquityChart series={eq.combined} label="Equity combinada de la cartera" height={220} />
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </Page>
  )
}
