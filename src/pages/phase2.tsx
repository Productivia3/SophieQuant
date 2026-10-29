import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { Play, Repeat, Columns3, AlertTriangle, RefreshCw, CheckCircle } from 'lucide-react'
import { api } from '../lib/api'
import type { McSuiteResult, WfMatrixResult, ChartSeries, SavedStrategy } from '../lib/api'
import type { StrategyRec } from '../lib/types'
import { Page } from '../components/Page'
import { Button } from '../components/Button'
import { Badge } from '../components/Badge'
import { EmptyState } from '../components/EmptyState'
import { TableSkeleton } from '../components/TableSkeleton'
import { fmtMoney, fmtNum } from '../lib/utils'

/* ───────────────────────── S13 · Monte Carlo ────────────────────── */

const mcTests = [
  { key: 'shuffle', label: 'Reordenar trades' },
  { key: 'skip', label: 'Saltar trades al azar' },
  { key: 'randomize_params', label: 'Aleatorizar parámetros' },
  { key: 'use_variants', label: 'Variantes de dirección' },
  { key: 'ohlc_interpolation', label: 'Interpolación OHLC' },
  { key: 'simulation_returns', label: 'Reconstruir retornos' },
  { key: 'drawdown', label: 'Simulación de drawdown' },
  { key: 'duplicate', label: 'Duplicar operaciones' },
  { key: 'skip_certain', label: 'Omitir períodos' },
]

export function S13MonteCarlo() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [tests, setTests] = useState<string[]>(['shuffle', 'skip', 'randomize_params'])
  const [busy, setBusy] = useState(false)
  const [suite, setSuite] = useState<McSuiteResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  const toggle = (k: string) => setTests(prev => prev.includes(k) ? prev.filter(t => t !== k) : [...prev, k])

  async function run() {
    if (!sel) return
    setBusy(true); setSuite(null)
    const res = await api.runMonteCarloSuite(sel.id, tests)
    setSuite(res); setBusy(false)
  }

  return (
    <Page title="Monte Carlo" icon={<Repeat size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel || tests.length === 0} onClick={run}>
          <Play size={14} aria-hidden /> Correr simulaciones
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="space-y-4">
          <div className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Estrategia</h2>
            <select aria-label="Estrategia" className="h-8 w-full rounded-md border border-border-input bg-surface px-2 text-sm"
              value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
              {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
            </select>
          </div>
          <div className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Simulaciones ({tests.length}/9)</h2>
            <ul className="space-y-1.5">
              {mcTests.map(t => (
                <li key={t.key}>
                  <label className="flex items-center gap-2 text-sm">
                    <input type="checkbox" checked={tests.includes(t.key)}
                      onChange={() => toggle(t.key)} />{t.label}
                  </label>
                </li>
              ))}
            </ul>
            {tests.length === 0 && (
              <p className="mt-2 flex items-center gap-1 text-xs text-warning">
                <AlertTriangle size={11} aria-hidden /> Selecciona al menos una simulación
              </p>
            )}
          </div>
        </div>

        <div>
          <h2 className="mb-2 text-base font-semibold">Resultados</h2>
          {!suite ? (
            busy
              ? <TableSkeleton rows={5} height={44} />
              : <EmptyState title="Sin corrida" hint="Elige simulaciones y corre. Los tests que no superen el 50% marcan FAIL y pueden filtrarse automáticamente en la generación." />
          ) : (
            <table className="w-full min-w-[640px] text-sm">
              <thead>
                <tr className="bg-surface text-left text-xs text-muted">
                  <th className="px-3 py-2 font-medium">Simulación</th>
                  <th className="px-3 py-2 font-medium">Veredicto</th>
                  <th className="px-3 py-2 font-medium">% pasó</th>
                  <th className="px-3 py-2 font-medium">PF p5</th>
                  <th className="px-3 py-2 font-medium">PF p50</th>
                  <th className="px-3 py-2 font-medium">PF p95</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {suite.results.map(r => (
                  <tr key={r.test} className="hover:bg-surface-raised">
                    <td className="px-3 py-2">{mcTests.find(t => t.key === r.test)?.label ?? r.test}</td>
                    <td className="px-3 py-2"><Badge tone={r.passed ? 'pass' : 'fail'} /></td>
                    <td className="px-3 py-2 text-right font-mono">{(r.passRate * 100).toFixed(0)}%</td>
                    <td className="px-3 py-2 text-right font-mono">{fmtNum(r.percentiles.p5)}</td>
                    <td className="px-3 py-2 text-right font-mono">{fmtNum(r.percentiles.p50)}</td>
                    <td className="px-3 py-2 text-right font-mono">{fmtNum(r.percentiles.p95)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S12 · WFMatrix heatmap ──────────────── */

export function S12WfMatrix() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [busy, setBusy] = useState(false)
  const [matrix, setMatrix] = useState<WfMatrixResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setMatrix(null)
    const res = await api.runWfMatrix(sel.id, { a: 'PMA periodo', b: 'Umbral OOS', steps: 5 })
    setMatrix(res); setBusy(false)
  }

  return (
    <Page title="Matriz walk-forward" icon={<Columns3 size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel} onClick={run}>
          <Play size={14} aria-hidden /> Correr matriz
        </Button>
      }>
      <div className="p-4">
        <div className="mb-4 flex items-center gap-3">
          <select aria-label="Estrategia"
            className="h-8 max-w-xs rounded-md border border-border-input bg-surface px-2 text-sm"
            value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          <span className="text-sm text-muted">
            La matriz reoptimiza cada combinación de parámetros: busca <b className="text-text">zonas estables</b>, no picos aislados.
          </span>
        </div>

        {!matrix
          ? (busy
              ? <TableSkeleton rows={6} height={40} />
              : <EmptyState title="Sin matriz" hint="Corre la matriz para ver el heatmap de WFE por combinación de parámetros." />)
          : (
            <Heatmap matrix={matrix} />
          )}
      </div>
    </Page>
  )
}

function Heatmap({ matrix }: { matrix: WfMatrixResult }) {
  const cellColor = (wfe: number) => {
    if (wfe >= 50) return `color-mix(in srgb, var(--color-success) ${Math.round(15 + wfe * 0.7)}%, var(--color-surface))`
    if (wfe >= 35) return `color-mix(in srgb, var(--color-warning) ${Math.round(20 + wfe * 0.5)}%, var(--color-surface))`
    return `color-mix(in srgb, var(--color-danger) ${Math.round(12 + wfe * 0.6)}%, var(--color-surface))`
  }
  return (
    <div className="overflow-auto qf-scroll">
      <table className="border-separate border-spacing-0.5 text-xs">
        <thead>
          <tr>
            <th className="px-2 py-1 font-medium text-muted" aria-label="filas / columnas">
              {matrix.paramA} ↓ · {matrix.paramB} →
            </th>
            {matrix.bValues.map(b => (
              <th key={b} className="px-2 py-1 font-mono font-medium text-muted">{b}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.aValues.map((a, i) => (
            <tr key={a}>
              <th className="px-2 py-1 text-right font-mono font-medium text-muted">{a}</th>
              {matrix.bValues.map((b, j) => {
                const cell = matrix.cells[i * matrix.bValues.length + j]
                return (
                  <td key={`${a}-${b}`}
                    title={`${matrix.paramA}=${a} · ${matrix.paramB}=${b} · WFE ${cell.wfe}`}
                    className="h-14 w-16 min-w-16 rounded-sm text-center font-mono text-xs text-text"
                    style={{ background: cellColor(cell.wfe) }}>
                    {cell.wfe.toFixed(0)}
                  </td>
                )
              })}
            </tr>
          ))}
        </tbody>
      </table>
      {/* ---- Nuevos mensajes de validación Block 1 item 1.4 ---- */}
      <div className="mt-3 flex flex-col gap-2 text-sm">
        {matrix.insufficientOos && (
          <div className="flex items-center gap-2 text-danger">
            <AlertTriangle size={14} aria-hidden />
            <span>poca data OOS: &lt;30 trades o &lt;1 a&ntilde;o: resultados no confiables</span>
          </div>
        )}
        {matrix.stopped && (
          <div className="flex items-center gap-2 text-warning">
            <AlertTriangle size={14} aria-hidden />
            <span>estrategia detenida por regla de parada: DD OOS &gt;80% del peak IS</span>
          </div>
        )}
        {!matrix.insufficientOos && !matrix.stopped && (
          <div className="flex items-center gap-2 text-success">
            <CheckCircle size={14} aria-hidden />
            <span>validación Block 1 pasada: muestra OOS suficiente y sin detener</span>
          </div>
        )}
      </div>
      <p className="mt-2 flex items-center gap-2 text-xs text-muted">
        <span aria-hidden className="h-3 w-8 rounded-sm" style={{ background: cellColor(70) }} /> estable
        <span aria-hidden className="h-3 w-8 rounded-sm" style={{ background: cellColor(40) }} /> débil
        <span aria-hidden className="h-3 w-8 rounded-sm" style={{ background: cellColor(10) }} /> falla
        — las celdas verdes contiguas indican una zona de parámetros estable.
      </p>
      <div role="table" aria-label="Descripción de la matriz" className="sr-only">
        Matriz de WFE: filas {matrix.paramA}, columnas {matrix.paramB}.
      </div>
    </div>
  )
}

/* ───────────────────────── S06 · Chart de operaciones ───────────── */

interface Candle { o: number; h: number; l: number; c: number }

export function S06Chart() {
  const params = useParams()
  const strategyId = +(params.id ?? 1)
  const [series, setSeries] = useState<ChartSeries | null>(null)
  const [name, setName] = useState('')
  const [overlayOn, setOverlayOn] = useState<Record<string, boolean>>({ ema20: true, ema50: true, sma200: false })

  useEffect(() => {
    api.getStrategy(strategyId).then(s => setName(s?.name ?? `#${strategyId}`))
    api.getStrategyChart(strategyId).then(setSeries)
  }, [strategyId])

  return (
    <Page title={name ? `${name} — operaciones` : '...'} icon={<Columns3 size={16} />}>
      {!series ? <TableSkeleton rows={5} height={300} /> : (
        <div className="p-4">
          <div className="rounded-md border border-border bg-surface p-3">
            <CandleChart candles={series.candles} trades={series.trades}
              oosFrom={series.oosFrom} oosLabel={series.oosLabel}
              overlays={series.overlays} overlayOn={overlayOn} />
          </div>
          <div className="mt-2 flex flex-wrap items-center gap-3">
            <span className="text-xs text-muted">
              {series.trades.length} operaciones · marcadores teal = compra, rosa = venta.
              {series.oosFrom != null && ' La zona sombreada es fuera de muestra: los parámetros no se eligieron con esos datos.'}
            </span>
            {(series.overlays ?? []).map(o => (
              <label key={o.key} className="flex cursor-pointer items-center gap-1 text-xs text-muted">
                <input type="checkbox" checked={overlayOn[o.key] !== false}
                  aria-label={`Mostrar ${o.label}`}
                  onChange={e => setOverlayOn({ ...overlayOn, [o.key]: e.target.checked })} />
                <span aria-hidden className="inline-block h-0.5 w-4 rounded"
                  style={{ background: OVERLAY_COLORS[o.key] ?? 'var(--color-accent)' }} />{o.label}
              </label>
            ))}
          </div>
        </div>
      )}
    </Page>
  )
}

const OVERLAY_COLORS: Record<string, string> = {
  ema20: 'var(--chart-buy)',
  ema50: 'var(--color-accent)',
  sma200: 'var(--color-warning)',
}

function CandleChart({ candles, trades, oosFrom, oosLabel, overlays, overlayOn }: {
  candles: Candle[]
  trades: ChartSeries['trades']
  oosFrom?: number
  oosLabel?: string
  overlays?: ChartSeries['overlays']
  overlayOn?: Record<string, boolean>
}) {
  const { w, h } = { w: 1000, h: 320 }
  const pad = 6
  const lo = Math.min(...candles.map(c => c.l))
  const hi = Math.max(...candles.map(c => c.h))
  const span = hi - lo || 1
  const y = (v: number) => h - pad - ((v - lo) / span) * (h - pad * 2)
  const x = (i: number) => (i / (candles.length - 1)) * w
  const bw = Math.max(1.6, w / candles.length - 2)

  const markers = trades.map((t, i) => ({
    ...t, x: x(t.idx), yE: y(t.entry), yX: y(t.exit), key: i,
  }))

  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="h-[320px] w-full" role="img" aria-label="Gráfico de velas con operaciones">
      {[0, 1, 2, 3, 4].map(g => (
        <line key={g} x1={0} x2={w} y1={pad + (g * (h - pad * 2)) / 4} y2={pad + (g * (h - pad * 2)) / 4}
          stroke="var(--chart-grid)" strokeWidth={1} />
      ))}
      {oosFrom != null && oosFrom > 0 && oosFrom < 1 && (
        <>
          <rect x={x(Math.round((1 - oosFrom) * (candles.length - 1)))} y={0}
            width={w - x(Math.round((1 - oosFrom) * (candles.length - 1)))} height={h}
            fill="var(--color-surface-raised)" opacity={0.9} />
          <line x1={x(Math.round((1 - oosFrom) * (candles.length - 1)))} y1={0}
            x2={x(Math.round((1 - oosFrom) * (candles.length - 1)))} y2={h}
            stroke="var(--color-border-input)" strokeWidth={1} strokeDasharray="5 4" />
        </>)}
      {candles.map((c, i) => {
        const up = c.c >= c.o
        const color = up ? 'var(--chart-up)' : 'var(--chart-down)'
        const xC = x(i)
        return (
          <g key={i}>
            <line x1={xC} x2={xC} y1={y(c.h)} y2={y(c.l)} stroke={color} strokeWidth={1} />
            <rect x={xC - bw / 2} width={bw}
              y={Math.min(y(c.o), y(c.c))} height={Math.max(1.5, Math.abs(y(c.o) - y(c.c)))}
              fill={color} />
          </g>
        )
      })}
      {(overlays ?? []).map(o => {
        if (overlayOn && overlayOn[o.key] === false) return null
        const pts: string[] = []
        o.points.forEach((v, i) => {
          if (v == null) return
          pts.push(`${x(i).toFixed(1)},${y(v).toFixed(1)}`)
        })
        if (pts.length < 2) return null
        return (
          <polyline key={o.key} points={pts.join(' ')} fill="none"
            stroke={OVERLAY_COLORS[o.key] ?? 'var(--color-accent)'}
            strokeWidth={1.2} opacity={0.9} vectorEffect="non-scaling-stroke" />
        )
      })}
      {markers.map(m => (
        <g key={m.key}>
          <circle cx={m.x} cy={m.yE} r={3.2}
            fill={m.dir === 'buy' ? 'var(--chart-buy)' : 'var(--chart-sell)'} />
          <line x1={m.x} x2={m.x} y1={m.yE} y2={m.yX} strokeDasharray="3 3"
            stroke={m.dir === 'buy' ? 'var(--chart-buy)' : 'var(--chart-sell)'} strokeWidth={1} opacity={0.55} />
          <circle cx={m.x} cy={m.yX} r={2.2}
            fill="none" stroke={m.dir === 'buy' ? 'var(--chart-buy)' : 'var(--chart-sell)'} strokeWidth={1.2} />
        </g>
      ))}
    </svg>
  )
}

/* ───────────────────────── S07 · Editor de estrategia ───────────── */

interface RuleRow { left: string; op: string; right: string }

const indicatorOptions = ['RSI(14)', 'MA(20) close', 'EMA(50) close', 'MACD(12,26,9) signal', 'Stoch(14,3) %K', 'CCI(14)', 'ATR(14)', 'Bollinger(20,2) upper', 'Bollinger(20,2) lower']
const marketOptions = ['High', 'Low', 'Open', 'Close', 'Previous bar close']
const opOptions = ['>', '<', '≥', '≤', 'crosses above', 'crosses below']
const targetOptions = ['Open trade', 'Close trade', 'Nothing (continuar)']
const valueOptions = ['70', '30', '1.5 × ATR(14)', 'Close de ayer']

/** Reglas que el motor ejecuta de verdad (engine/rules.rs). El editor solo ofrece estas: no hay bloques decorativos que luego el motor no sepa evaluar. `paramNames` debe coincidir con `Template::param_names()` del motor. */
interface Regla {
  id: number
  nombre: string
  resumen: string
  paramNames: [string, string, string]
  labels: [string, string, string]
  defaults: [number, number, number]
}

const reglas: Regla[] = [
  { id: 1, nombre: 'Cruce de medias', resumen: 'Entra cuando la media rapida cruza por encima de la lenta; sale cuando cruza por debajo.', paramNames: ['p_fast', 'p_slow', '-'], labels: ['Media rapida', 'Media lenta', '—'], defaults: [20, 50, 0] },
  { id: 2, nombre: 'Reversión RSI', resumen: 'Entra cuando el RSI sale de sobreventa; sale cuando sale de sobrecompra.', paramNames: ['p_rsi', 'th_low', 'th_high'], labels: ['Periodo RSI', 'Umbral de sobreventa', 'Umbral de sobrecompra'], defaults: [14, 30, 70] },
  { id: 3, nombre: 'Ruptura con filtro de tendencia', resumen: 'Entra al romper el maximo de la ventana; sale al perder el minimo.', paramNames: ['p_lookback', 'sl_atr', 'tp_atr'], labels: ['Ventana de ruptura', 'Stop (x ATR)', 'Objetivo (x ATR)'], defaults: [20, 2, 3] },
]

export function S07Editor() {
  const params = useParams()
  const strategyId = +(params.id ?? 1)
  const navigate = useNavigate()
  const [strategy, setStrategy] = useState<StrategyRec | null>(null)
  const [reglaId, setReglaId] = useState<number>(1)
  const [vals, setVals] = useState<[number, number, number]>([20, 50, 0])
  const [busy, setBusy] = useState(false)
  const [guardado, setGuardado] = useState<SavedStrategy | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api.getStrategy(strategyId).then(s => {
      if (!s) return
      setStrategy(s)
      const tpl = s.templateId ?? 1
      const regla = reglas.find(r => r.id === tpl) ?? reglas[0]
      const p = (s.params as { p?: number[] } | null)?.p
      setReglaId(regla.id)
      setVals([typeof p?.[0] === 'number' ? p[0] : regla.defaults[0], typeof p?.[1] === 'number' ? p[1] : regla.defaults[1], typeof p?.[2] === 'number' ? p[2] : regla.defaults[2]])
    })
  }, [strategyId])

  const regla = reglas.find(r => r.id === reglaId) ?? reglas[0]

  function cambiarRegla(id: number) {
    const r = reglas.find(x => x.id === id) ?? reglas[0]
    setReglaId(r.id)
    setVals([...r.defaults])
    setGuardado(null)
    setError(null)
  }

  async function guardar() {
    setBusy(true); setError(null); setGuardado(null)
    try {
      const res = await api.saveStrategyLogic(strategyId, regla.id, [...vals], {
        regla: regla.nombre,
        params: { [regla.paramNames[0]]: vals[0], [regla.paramNames[1]]: vals[1], [regla.paramNames[2]]: vals[2] },
        accion: 'long_only',
      })
      setGuardado(res)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  const antes = strategy?.lastRun?.metrics

  return (
    <Page
      title={strategy ? `Editar ${strategy.name}` : 'Editor de estrategia'}
      icon={<RefreshCw size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!strategy || busy} onClick={() => void guardar()}>
          Guardar y recalcular
        </Button>
      }
    >
      <div className="mx-auto max-w-3xl space-y-4 p-4">
        {!strategy ? <TableSkeleton rows={4} height={180} /> : (
          <>
            <section className="rounded-md border border-border bg-surface p-3">
              <h2 className="mb-2 text-sm font-semibold text-muted">Regla de la estrategia</h2>
              <div className="space-y-2" role="radiogroup" aria-label="Regla de la estrategia">
                {reglas.map(r => (
                  <label key={r.id}
                    className={`flex cursor-pointer items-start gap-2 rounded-md border p-2 text-sm ${ reglaId === r.id ? 'border-accent text-text' : 'border-border text-muted hover:border-border-input'}`}>
                    <input type="radio" name="regla" value={r.id} checked={reglaId === r.id} onChange={() => cambiarRegla(r.id)} className="mt-0.5" />
                    <span>
                      <span className="font-medium text-text">{r.nombre}</span>
                      <span className="block text-xs text-muted">{r.resumen}</span>
                    </span>
                  </label>
                ))}
              </div>
              <p className="mt-2 text-xs text-muted">
                El motor ejecuta estas tres reglas y nada mas: por qué el editor no ofrece bloques que luego no sabría evaluar (MACD, Stochastic, CCI, patrones de vela…).
              </p>
            </section>

            <section className="rounded-md border border-border bg-surface p-3">
              <h2 className="mb-2 text-sm font-semibold text-muted">Parametros de la regla</h2>
              <div className="grid gap-2 md:grid-cols-3">
                {regla.labels.map((label, i) => (
                  <label key={i} className="block text-xs text-muted">
                    <span className="mb-1 block">
                      {label}
                      <span className="ml-1 font-mono text-[10px] text-muted/70">{regla.paramNames[i]}</span>
                    </span>
                    <input
                      type="number"
                      step="0.1"
                      aria-label={label}
                      disabled={regla.paramNames[i] === '-'}
                      value={Number.isFinite(vals[i]) ? vals[i] : ''}
                      onChange={e => {
                        const value = Number(e.target.value)
                        setVals(previous => {
                          const next: [number, number, number] = [...previous]
                          next[i] = Number.isFinite(value) ? value : previous[i]
                          return next
                        })
                        setGuardado(null)
                      }}
                      className="h-8 w-full rounded-md border border-border-input bg-surface px-2 font-mono text-sm text-text disabled:opacity-50"
                    />
                  </label>
                ))}
              </div>
              <p className="mt-2 text-xs text-muted">Al cumplirse: <b>comprar</b> y cerrar al salir. La version actual es solo largo.</p>
            </section>

            {error && (<div role="alert" className="rounded-md border border-danger/40 bg-danger/10 p-3 text-sm text-danger">{error}</div>)}
            {guardado && (
              <div role="status" className="rounded-md border border-success/40 bg-success/10 p-3 text-sm text-success">
                <p>
                  {guardado.preview
                    ? <><b>Vista previa</b>: en el navegador no se guarda la regla ni se ejecuta el motor Rust.</>
                    : <><b>Regla guardada y corrida recalculada</b> sobre tu dataset.</>}
                </p>
                <p className="mt-1 font-mono text-xs">
                  Beneficio neto {fmtMoney(guardado.metrics?.netProfit)} · PF {fmtNum(guardado.metrics?.profitFactor)} ·
                  Max DD {fmtNum(guardado.metrics?.maxDD, 1)} % · {guardado.nTrades} operaciones
                </p>
                {!guardado.preview && antes && (
                  <p className="mt-1 text-xs">
                    Antes: {fmtMoney(antes.netProfit)} · PF {fmtNum(antes.profitFactor)} ·
                    Max DD {fmtNum(antes.maxDD, 1)} % · {antes.nTrades} operaciones
                  </p>
                )}
              </div>
            )}

            <div className="flex gap-2">
              <Button variant="secondary" onClick={() => navigate(`/strategy/${strategyId}`)}>Volver a resultados</Button>
              {guardado && (<Button variant="ghost" onClick={() => navigate(`/strategy/${strategyId}`)}>Ver la estrategia actualizada</Button>)}
            </div>
          </>
        )}
      </div>
    </Page>
  )
}

function Section2({ title, children }: { title: string; children: React.ReactNode }) {
  return (<fieldset className="rounded-md border border-border bg-surface p-3"><legend className="px-1 text-sm font-semibold text-muted">{title}</legend><div className="space-y-2">{children}</div></fieldset>)
}

function Select({ v, opts, on }: { v: string; opts: string[]; on: (v: string) => void }) {
  return (<select aria-label="valor" className="h-8 rounded-md border border-border-input bg-surface px-2 text-sm" value={v} onChange={e => on(e.target.value)}>{opts.map(o => <option key={o} value={o}>{o}</option>)}</select>)
}