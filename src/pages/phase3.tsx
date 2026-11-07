// phase3.tsx — pantallas de Fase 3: Retesteo, Refinador, Optimizador+3D, SPP,
// Proyectos y Cartera maestra. Clean-room sobre la capa fake.
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Play, Repeat, Layers, Ruler, ArrowUp, ArrowDown, X, Grid3x3, FolderKanban, PieChart } from 'lucide-react'
import { api } from '../lib/api'
import type { PortfolioCalc, WeightingModel } from '../lib/api'
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
  const [symbolIds, setSymbolIds] = useState<number[]>([])
  const [timeframe, setTimeframe] = useState<Timeframe>('h1')
  const [datasets, setDatasets] = useState<Awaited<ReturnType<typeof api.listDataFiles>>>([])
  const [busy, setBusy] = useState(false)
  const [result, setResult] = useState<RetestResult | null>(null)

  useEffect(() => {
    api.listBank({}).then(rs => setRows(rs))
    api.listSymbols().then(ss => { setSymbols(ss); setSymbolIds(ss[0] ? [ss[0].id] : []) })
    api.listDataFiles().then(setDatasets)
  }, [])

  const toggle = (id: number) =>
    setSelIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  const toggleMercado = (id: number) =>
    setSymbolIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  /** Ese simbolo tiene datos cargados para la temporalidad elegida. */
  const tieneDatos = (id: number) =>
    datasets.some(d => d.symbolId === id && d.timeframe === timeframe && d.status === 'ready')

  async function run() {
    if (!selIds.length) return
    setBusy(true); setResult(null)
    const res = await api.runRetest(selIds, symbolIds, timeframe)
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
            <h2 className="mb-2 text-sm font-semibold text-muted">Mercados del retesteo</h2>
            <label className="mb-2 block text-xs text-muted" htmlFor="ret-tf">Temporalidad</label>
            <select id="ret-tf" className={`w-full ${selectCls}`} value={timeframe}
              onChange={e => setTimeframe(e.target.value as Timeframe)}>
              {tfs.map(tf => <option key={tf} value={tf}>{tf}</option>)}
            </select>
            <p className="mb-2 mt-3 text-xs text-muted">
              Marca uno o varios mercados: la misma regla se prueba en cada uno.
            </p>
            <ul className="max-h-56 space-y-1 overflow-auto qf-scroll text-sm" aria-label="Mercados a retestear">
              {symbols.map(s => (
                <li key={s.id}>
                  <label className="flex cursor-pointer items-center gap-2 rounded px-1 py-0.5 hover:bg-surface-raised">
                    <input type="checkbox" aria-label={`Retestear en ${s.name}`}
                      checked={symbolIds.includes(s.id)} onChange={() => toggleMercado(s.id)} />
                    <span className="flex-1 truncate">{s.name}</span>
                    {tieneDatos(s.id)
                      ? <span className="text-xs text-success">con datos</span>
                      : <span className="text-xs text-muted">sin datos</span>}
                  </label>
                </li>
              ))}
            </ul>
            <p className="mt-2 text-xs text-muted">
              Un mercado sin datos no cuenta como fallo: sale como «sin datos».
            </p>
          </div>
        </div>

        <div className="min-w-0">
          {!result ? (
            <EmptyState title="Sin retesteo todavia"
              hint="Marca estrategias del banco, elige uno o varios mercados y corre el retesteo: veras si la regla aguanta fuera de donde nacio." />
          ) : (
              <div className="overflow-x-auto qf-scroll">
              <table className="w-full min-w-[760px] text-sm">
                <thead>
                  <tr className="bg-surface text-left text-xs text-muted">
                    <th className="px-3 py-2 font-medium">Estrategia</th>
                    <th className="px-3 py-2 font-medium">Origen (OOS 30 %)</th>
                    {result.mercados.map(m => (
                      <th key={m.symbolId} className="px-3 py-2 font-medium">
                        {m.simbolo}
                        {m.conDatos ? '' : <span className="ml-1 text-muted">(sin datos)</span>}
                      </th>
                    ))}
                    <th className="px-3 py-2 font-medium">Sobrevive</th>
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
                      <td className="px-3 py-2">
                        <Badge tone={verdictTone(r.origen.verdict)} label={verdictLabel(r.origen.verdict)} />
                        <span className="ml-2 font-mono text-xs text-muted">
                          PF {fmtNum(r.origen.retest?.profitFactor)} · DD {fmtNum(r.origen.retest?.maxDD, 1)}%
                        </span>
                      </td>
                      {r.mercados.map(m => (
                        <td key={m.symbolId} className="px-3 py-2">
                          {m.verdict === 'sin datos'
                            ? <span className="text-xs text-muted">sin datos</span>
                            : (
                              <>
                                <Badge tone={verdictTone(m.verdict)} label={verdictLabel(m.verdict)} />
                                <span className="ml-2 font-mono text-xs text-muted">
                                  PF {fmtNum(m.retest?.profitFactor)}
                                </span>
                              </>
                            )}
                        </td>
                      ))}
                      <td className="px-3 py-2">
                        {r.consistencia ? (
                          <>
                            <Badge tone={verdictTone(r.consistencia.verdict === 'pending' ? 'fail' : r.consistencia.verdict)}
                              label={r.consistencia.detalle} />
                            {r.periodos && r.periodos.length > 1 && (
                              <span className="ml-2 font-mono text-xs text-muted" title="Veredicto de cada periodo OOS">
                                {r.periodos.map(p => (p.verdict === 'pass' ? 'P' : p.verdict === 'warn' ? 'A' : 'F')).join(' ')}
                              </span>
                            )}
                          </>
                        ) : <span className="text-muted">—</span>}
                      </td>
                      <td className="px-3 py-2">
                        <Badge tone={verdictTone(r.resumen.verdict === 'pending' ? 'fail' : r.resumen.verdict)}
                          label={r.resumen.detalle} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <p className="mt-2 text-xs text-muted">
                Cada celda es la MISMA regla corrida en ese mercado. «Sobrevive» resume en cuantos
                mercados aguanta la estrategia. Temporalidad: <b className="text-text">{result.timeframe}</b>.
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

function OptimizerHeatmap({ res }: { res: OptimizerResult }) {
  const scoreTone = (cell: OptimizerResult['cells'][number]) => {
    if ('verdict' in cell) {
      return cell.verdict === 'pass' ? 'bg-success/15 text-success'
        : cell.verdict === 'warn' ? 'bg-warning/15 text-warning'
          : 'bg-danger/15 text-danger'
    }
    return cell.pf >= 1.15 ? 'bg-success/15 text-success'
      : cell.pf >= 1 ? 'bg-warning/15 text-warning' : 'bg-danger/15 text-danger'
  }
  const getCell = (a: number, b: number) => res.cells.find(cell => cell.a === a && cell.b === b)
  const metric = res.kind === 'wf' ? 'WFE' : 'Factor de beneficio (PF)'
  const selectionRule = res.kind === 'wf'
    ? 'Walk-Forward resalta la mayor puntuación compuesta (WFE − 0,3 × drawdown).'
    : 'Simple resalta la combinación con mayor factor de beneficio (PF).'

  return (
    <section className="space-y-3" aria-label="Resultados de la optimización">
      {res.kind === 'wf' ? (
        <div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
          <StatCard label="WFE de la combinación seleccionada" value={res.best.wfe} unit="%" hint="Porcentaje del profit factor de entrenamiento que se conserva fuera de muestra." />
          <StatCard label="Mejor combinación" value={`${res.best.a} · ${res.best.b}`} mono={false} />
          <StatCard label="Drawdown de esa combinación" value={res.best.maxDD} unit="%" />
          <StatCard label="Veredicto" value={res.best.verdict === 'pass' ? 'Pasa' : res.best.verdict === 'warn' ? 'Alerta' : 'Falla'}
            tone={res.best.verdict === 'pass' ? 'pass' : res.best.verdict === 'fail' ? 'fail' : 'neutral'} />
        </div>
      ) : (
        <div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
          <StatCard label="Mejor PF" value={res.best.pf} hint="Beneficio bruto dividido por pérdida bruta en el histórico completo." />
          <StatCard label="Mejor combinación" value={`${res.best.a} · ${res.best.b}`} mono={false} />
          <StatCard label="Drawdown de esa combinación" value={res.best.maxDD} unit="%" />
        </div>
      )}
      <div className="overflow-x-auto qf-scroll rounded-md border border-border bg-surface p-3">
        <p className="mb-2 text-sm font-semibold">Mapa de calor · {metric}</p>
        <p className="mb-3 text-xs text-muted">
          Columnas: {res.paramA}. Filas: {res.paramB}. Cada celda es una combinación; el borde resalta la combinación seleccionada.
        </p>
        <table className="min-w-[620px] border-separate border-spacing-1 text-center text-xs"
          aria-label={`Mapa de calor de optimización ${res.kind === 'wf' ? 'Walk-Forward por WFE' : 'simple por factor de beneficio'}`}>
          <thead>
            <tr>
              <th scope="col" className="px-2 py-1 text-left text-muted">{res.paramB} ↓ / {res.paramA} →</th>
              {res.aValues.map(a => <th scope="col" key={a} className="px-2 py-1 font-mono">{fmtNum(a, 1)}</th>)}
            </tr>
          </thead>
          <tbody>
            {res.bValues.map(b => (
              <tr key={b}>
                <th scope="row" className="px-2 py-1 text-right font-mono text-muted">{fmtNum(b, 1)}</th>
                {res.aValues.map(a => {
                  const cell = getCell(a, b)
                  if (!cell) return <td key={a} className="min-w-16 rounded border border-border px-2 py-2 text-muted">—</td>
                  const isBest = cell.a === res.best.a && cell.b === res.best.b
                  const value = 'wfe' in cell ? `${fmtNum(cell.wfe, 1)}%` : fmtNum(cell.pf, 2)
                  const detail = 'wfe' in cell
                    ? `${cell.verdict === 'pass' ? 'Pasa' : cell.verdict === 'warn' ? 'Alerta' : 'Falla'} · DD ${fmtNum(cell.maxDD, 1)}%`
                    : `DD ${fmtNum(cell.maxDD, 1)}%`
                  return (
                    <td key={a} className={`min-w-16 rounded border px-2 py-1.5 ${scoreTone(cell)} ${isBest ? 'border-text ring-1 ring-text font-bold' : 'border-transparent'}`}
                      title={`${res.paramA}: ${a} · ${res.paramB}: ${b} · ${metric}: ${value} · ${detail}${isBest ? ' · mejor combinación' : ''}`}>
                      <span className="block font-mono">{value}</span>
                      <span className="block text-[10px] opacity-80">{detail}</span>
                    </td>
                  )
                })}
              </tr>
            ))}
          </tbody>
        </table>
        <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted" aria-label="Leyenda del mapa">
          {res.kind === 'wf' ? (
            <>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-danger/60" />WFE &lt;35% · Falla</span>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-warning/60" />35–49,9% · Alerta</span>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-success/60" />≥50% · Pasa</span>
            </>
          ) : (
            <>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-danger/60" />PF &lt;1 · pierde</span>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-warning/60" />1–1,14 · cerca del equilibrio</span>
              <span><i className="mr-1 inline-block h-2.5 w-2.5 rounded-sm bg-success/60" />≥1,15 · favorable en muestra</span>
            </>
          )}
        </div>
      </div>
      <p className="text-xs text-muted">
        Los valores de cada eje recorren aproximadamente del 40% al 220% del parámetro actual. {selectionRule} La mejor celda no garantiza resultados futuros; confirma cualquier elección con datos fuera de muestra.
      </p>
    </section>
  )
}

export function S10Optimizer() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [kind, setKind] = useState<'simple' | 'wf'>('simple')
  const [steps, setSteps] = useState(6)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<OptimizerResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null); setError(null)
    try {
      setRes(await api.runOptimizer(sel.id, { paramA: 'Parámetro 1', paramB: 'Parámetro 2', steps, kind }))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
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
            value={sel?.id ?? ''} onChange={e => {
              setSel(rows.find(r => r.id === +e.target.value) ?? null)
              setRes(null); setError(null)
            }}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          <fieldset className="mt-3">
            <legend className="mb-2 text-xs text-muted">Modo</legend>
            <div className="flex gap-2" role="radiogroup" aria-label="Modo de optimización">
              {(['simple', 'wf'] as const).map(k => (
                <button key={k} type="button" role="radio" aria-checked={kind === k}
                  onClick={() => { setKind(k); setRes(null); setError(null) }}
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
            onChange={e => { setSteps(+e.target.value); setRes(null) }} className="w-full" />
        </div>

        <div className="min-w-0 space-y-3">
          {error && <p className="rounded-md border border-danger/40 bg-danger/10 p-3 text-sm text-danger" role="alert">
            No se pudo ejecutar el optimizador: {error}
          </p>}
          {!res ? (
            <EmptyState title="Sin optimización todavía"
              hint={kind === 'wf'
                ? 'Corre el análisis para ver una tabla coloreada de WFE fuera de muestra por cada combinación de parámetros.'
                : 'Corre el análisis para comparar el factor de beneficio y el drawdown de cada combinación de parámetros.'} />
          ) : <OptimizerHeatmap res={res} />}
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
  const [error, setError] = useState<string | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null); setError(null)
    try {
      setRes(await api.runOptimizer(sel.id, { paramA: 'Parámetro 1', paramB: 'Parámetro 2', steps: 7, kind: 'wf' }))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => { if (sel && !res && !busy && !error) void run() /* lint-safe: correr una vez */ }, [sel, res, busy, error])

  return (
    <Page title="Mapa de optimización" icon={<Ruler size={16} />}
      actions={
        <select aria-label="Estrategia para el mapa" className={selectCls} value={sel?.id ?? ''}
          onChange={e => {
            setSel(rows.find(r => r.id === +e.target.value) ?? null)
            setRes(null); setError(null)
          }}>
          {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
        </select>
      }>
      <div className="space-y-3 p-4">
        {error && <p className="rounded-md border border-danger/40 bg-danger/10 p-3 text-sm text-danger" role="alert">
          No se pudo generar el mapa: {error}
        </p>}
        {!rows.length && !busy ? <EmptyState title="No hay estrategias disponibles" hint="Genera una estrategia primero." />
          : res ? <OptimizerHeatmap res={res} /> : <TableSkeleton />}
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
  const [error, setError] = useState<string | null>(null)
  const desktopAvailable = api.isDesktop()

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel || busy) return
    setBusy(true)
    setRes(null)
    setError(null)
    try {
      setRes(await api.runSpp(sel.id))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const bar = (impact: number) => Math.min(100, impact * 2.2)

  return (
    <Page title="SPP · pruebas de robustez" icon={<PieChart size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!desktopAvailable || !sel} onClick={run}>
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
            SPP = pruebas de estrés de señales y parámetros. Cada perturbación compara el beneficio
            con el original; se prueban 20 variantes de entradas aleatorias.
          </p>
          {!desktopAvailable && (
            <p className="mt-2 text-xs text-warning" role="status">
              SPP requiere la aplicación de escritorio; el navegador no genera resultados simulados.
            </p>
          )}
          <Link to="/profile" className="mt-3 inline-block text-sm text-accent hover:underline">
            Ver Perfil de optimización →
          </Link>
        </div>

        <div className="min-w-0 space-y-3">
          {error && <p className="text-sm text-danger" role="alert">No se pudo ejecutar SPP: {error}</p>}
          {!res ? (
            <EmptyState title="Sin SPP todavía"
              hint="Corre las pruebas para saber si la estrategia es robusta: con señales rotas casi no debería cambiar el resultado." />
          ) : (
            <>
              <div className="flex items-center gap-2">
                <Badge tone={verdictTone(res.overall)} label={verdictLabel(res.overall)} />
                <span className="text-sm text-muted">
                  {res.overall === 'pass'
                    ? 'Todas las perturbaciones evaluadas quedaron bajo el umbral.'
                    : res.overall === 'warn'
                      ? 'Hay perturbaciones en zona de alerta; no se considera un PASS estricto.'
                      : 'Hay perturbaciones que superan el umbral: señales frágiles.'}
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
  const [model, setModel] = useState<WeightingModel>('equal_vol')
  const [custom, setCustom] = useState<Record<number, number>>({})
  const [pesos, setPesos] = useState<PortfolioCalc | null>(null)
  const [sug, setSug] = useState<Awaited<ReturnType<typeof api.suggestPortfolio>> | null>(null)

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
    const customWeights = selIds.map(id => custom[id] ?? 1)
    const [c, e, w] = await Promise.all([
      api.getCorrelationMatrix(selIds),
      api.getPortfolioEquity(selIds),
      api.runPortfolioWeighting(selIds, model, customWeights),
    ])
    setCorr(c); setEq(e); setPesos(w.resultado); setBusy(false)
  }
  const toggle = (id: number) =>
    setSelIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  async function sugerir() {
    if (!rows?.length) return
    setBusy(true); setSug(null)
    try {
      const s = await api.suggestPortfolio(rows.map(r => r.id))
      setSug(s)
    } finally {
      setBusy(false)
    }

  }

  function usarSugerencia() {
    if (!sug?.elegidos?.length) return
    setSelIds(sug.elegidos.map(e => e.strategyId))
    setSug(null); setCorr(null); setEq(null); setPesos(null)
  }


  return (
    <Page title="Cartera maestra" icon={<PieChart size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={selIds.length < 2} onClick={run}>
          <Play size={14} aria-hidden /> Analizar cartera ({selIds.length})
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <div className="mb-2 flex items-center gap-2">
            <h2 className="text-sm font-semibold text-muted">Estrategias (2–8)</h2>
            <Button size="sm" variant="secondary" loading={busy}
              disabled={!rows?.length}
              onClick={() => void sugerir()} className="ml-auto">
              Sugerir cartera
            </Button>
          </div>
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
          {sug && (
            <section className="mb-4 rounded-md border border-border bg-surface p-3">
              <h2 className="mb-2 text-sm font-semibold text-muted">Sugerencia de cartera</h2>
              <p className="mb-2 text-xs text-muted">{sug.notas}</p>
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left text-xs text-muted">
                    <th className="px-2 py-1 font-medium">Se queda</th>
                    <th className="px-2 py-1 font-medium">Volatilidad</th>
                    <th className="px-2 py-1 font-medium">Peso sugerido</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {sug.elegidos.map(e => (
                    <tr key={e.strategyId}>
                      <td className="px-2 py-1 truncate">{e.nombre}</td>
                      <td className="px-2 py-1 font-mono text-xs">{fmtNum(e.volatility, 2)} %</td>
                      <td className="px-2 py-1 font-mono text-xs">{fmtNum(e.pesoSugerido, 1)} %</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {sug.descartados.length > 0 && (
                <>
                  <h3 className="mb-1 mt-3 text-xs font-semibold text-muted">Se descartan y por que</h3>
                  <ul className="space-y-0.5">
                    {sug.descartados.map(d => (
                      <li key={d.strategyId} className="text-xs text-muted">
                        <span className="text-text">{d.nombre}</span>: {d.motivo}
                      </li>
                    ))}
                  </ul>
                </>
              )}
              <Button variant="primary" size="sm" className="mt-3" onClick={usarSugerencia}>
                Usar esta cartera
              </Button>
            </section>
          )}

          <section className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Ponderacion de la cartera</h2>
            <div className="flex flex-wrap items-center gap-2">
              <label className="text-xs text-muted" htmlFor="pf-modelo">Modelo</label>
              <select id="pf-modelo" className={selectCls} value={model}
                aria-label="Modelo de ponderacion"
                onChange={e => setModel(e.target.value as WeightingModel)}>
                <option value="equal_vol">Inverso a volatilidad</option>
                <option value="risk_parity">Paridad de riesgo</option>
                <option value="equal_weight">Equiponderado</option>
                <option value="custom">Personalizado</option>
              </select>
              {model === 'custom' && (
                <span className="text-xs text-muted">Pesos propios: editalos en la tabla.</span>
              )}
            </div>

            {pesos && (
              <div className="mt-3">
                <table className="w-full text-sm" aria-label="Ponderaciones por miembro">
                  <thead>
                    <tr className="text-left text-xs text-muted">
                      <th className="px-2 py-1 font-medium">Miembro</th>
                      <th className="px-2 py-1 font-medium">Volatilidad</th>
                      <th className="px-2 py-1 font-medium">Peso</th>
                      <th className="px-2 py-1 font-medium">Riesgo</th>
                      {model === 'custom' && <th className="px-2 py-1 font-medium">Peso propio</th>}
                      <th className="px-2 py-1"><span className="sr-only">Acciones</span></th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border">
                    {pesos.miembros.map(m => (
                      <tr key={m.strategyId}>
                        <td className="px-2 py-1 truncate">{m.nombre}</td>
                        <td className="px-2 py-1 font-mono text-xs">{fmtNum(m.volatility, 2)} %</td>
                        <td className="px-2 py-1 font-mono text-xs">{fmtNum(m.weightPct, 1)} %</td>
                        <td className="px-2 py-1 font-mono text-xs">{fmtNum(m.riskContribution, 1)} %</td>
                        {model === 'custom' && (
                          <td className="px-2 py-1">
                            <input type="number" step={0.1} min={0}
                              className="h-7 w-20 rounded-md border border-border-input bg-surface px-2 font-mono text-xs"
                              aria-label={`Peso propio de ${m.nombre}`}
                              value={custom[m.strategyId] ?? 1}
                              onChange={e => setCustom({ ...custom, [m.strategyId]: +e.target.value })} />
                          </td>
                        )}
                      </tr>
                    ))}
                  </tbody>
                </table>
                <p className="mt-2 text-xs text-muted">
                  Volatilidad de cartera {fmtNum(pesos.portfolioVol, 2)} % · Max DD {fmtNum(pesos.combinada.maxDD, 1)} %
                  {' '}· Beneficio {fmtMoney(pesos.combinada.netProfit)} · modelo: {pesos.modelLabel}.
                  En paridad de riesgo todos aportan lo mismo; en inverso a volatilidad pesa mas la que
                  se mueve menos.
                </p>
              </div>
            )}
          </section>

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
                        <th className="px-2 py-1"><span className="sr-only">Estrategia</span></th>
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
                                    // paleta actual: violeta (baja correlacion), ambar
                                    // (media) y rojo (alta). Texto siempre claro para
                                    // mantener el contraste AA sobre el tinte.
                                    background: v <= 0.35
                                      ? `rgba(157,123,255,${alpha * 0.5})`
                                      : high ? 'rgba(239,74,82,0.35)' : `rgba(240,192,74,${alpha * 0.5})`,
                                    color: 'var(--color-text)',
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
                <h2 className="mb-2 text-base font-semibold">Equity combinada (con las ponderaciones elegidas)</h2>
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
