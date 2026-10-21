import { useEffect, useMemo, useState, type ReactNode } from 'react'
import { useNavigate, useParams, Link } from 'react-router-dom'
import {
  Play, Pause, Square, Database, Download, Wand2, ChevronRight, FileCode2,
  AlertTriangle, Info, LibraryBig, GitCompareArrows,
} from 'lucide-react'
import { api, onBuildEvent } from '../lib/api'
import type { BuildRec, BuildStatus, Platform, StrategyRec, Timeframe } from '../lib/types'
import { Page } from '../components/Page'
import { Button } from '../components/Button'
import { Badge } from '../components/Badge'
import { StatCard, StatRow } from '../components/StatCard'
import { EmptyState } from '../components/EmptyState'
import { EquityChart } from '../components/EquityChart'
import { VerdictCard } from '../components/VerdictCard'
import { HistoryPanel } from '../components/HistoryPanel'
import { estadoTone, estadoTexto } from '../lib/verdicts'
import type { StrategyVerdicts, MetricFilter, BuildMetric, RobustnessWeight, ExportVerificacion } from '../lib/api'
import { BuildLog } from '../components/BuildLog'
import { fmtBytes, fmtDate, fmtMoney, fmtNum } from '../lib/utils'

const selectCls = 'h-8 rounded-md border border-border-input bg-surface px-2 text-sm text-text'
const inputCls = selectCls

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <fieldset className="rounded-md border border-border bg-surface p-3">
      <legend className="px-1 text-sm font-semibold text-muted">{title}</legend>
      <div className="space-y-2">{children}</div>
    </fieldset>
  )
}
function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <span className="w-48 shrink-0 text-sm text-muted">{label}</span>
      {children}
    </div>
  )
}
function symName(id: number) {
  const map: Record<number, string> = { 1: 'EURUSD', 2: 'GBPUSD', 3: 'USDJPY', 4: 'AAPL', 5: 'ES' }
  return map[id] ?? `#${id}`
}
function kindLabel(k: BuildRec['kind']) {
  return ({
    builder: 'Constructor', retester: 'Retesteo', improver: 'Refinador',
    optimizer: 'Optimizador', monte_carlo: 'Monte Carlo', custom_project: 'Proyecto',
  })[k]
}
function StatusBadge({ status }: { status: BuildStatus }) {
  const tone = status === 'running' ? 'running' : status === 'done' ? 'pass' : 'pending'
  const label = ({ running: 'Corriendo', done: 'Completado', paused: 'Pausado', queued: 'En cola', error: 'Error', cancelled: 'Cancelado' })[status]
  return <Badge tone={tone} label={label} />
}
function TableSkeleton({ rows = 8, height = 34 }: { rows?: number; height?: number }) {
  return (
    <div className="space-y-2 p-4" aria-label="Cargando">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="animate-pulse rounded-md bg-surface" style={{ height, opacity: 1 - i * 0.1 }} />
      ))}
    </div>
  )
}
// nota: la versión compartida vive en components/TableSkeleton (usada también por phase2)
function Progressbar({ value }: { value: number }) {
  return (
    <div role="progressbar" aria-label="Progreso de la generación" aria-valuenow={Math.round(value * 100)} aria-valuemin={0} aria-valuemax={100}
      className="h-1 flex-1 overflow-hidden rounded-pill bg-border">
      <div className="h-full bg-accent transition-base" style={{ width: `${value * 100}%` }} />
    </div>
  )
}

function Placeholder({ title, icon, hint }: { title: string; icon?: ReactNode; hint: string }) {
  return (
    <Page title={title} icon={icon}>
      <EmptyState title="Pendiente en esta fase" hint={hint} />
    </Page>
  )
}

/* ───────────────────────── S01 · Dashboard ─────────────────────── */

export function S01Dashboard() {
  const [builds, setBuilds] = useState<BuildRec[] | null>(null)
  const [bankSize, setBankSize] = useState<number | null>(null)
  const navigate = useNavigate()

  useEffect(() => {
    let alive = true
    const refresh = async () => {
      const [b, s] = await Promise.all([api.listBuilds(), api.listBank({})])
      if (alive) { setBuilds(b); setBankSize(s.length) }
    }
    refresh()
    const iv = setInterval(refresh, 1000)
    return () => { alive = false; clearInterval(iv) }
  }, [])

  return (
    <Page
      title="Proyecto"
      icon={<Database size={16} />}
      actions={
        <Button variant="primary" onClick={() => navigate('/build/new')}>
          <Wand2 aria-hidden size={14} /> Nueva generación
        </Button>
      }
    >
      <div className="grid gap-4 p-4 xl:grid-cols-3">
        <div className="rounded-md border border-border bg-surface p-4 xl:col-span-2 min-h-[280px]">
          <h2 className="mb-3 text-base font-semibold">Generaciones</h2>
          {!builds ? <TableSkeleton /> : builds.length === 0 ? (
            <EmptyState
              title="Aún no hay generaciones"
              hint="Configura tu primera generación: símbolo, datos, plantilla y filtros. El motor genera y filtra automáticamente."
              action={<Button variant="primary" onClick={() => navigate('/build/new')}>Empezar una generación</Button>}
            />
          ) : (
            <ul className="divide-y divide-border">
              {builds.map(b => (
                <li key={b.id} className="flex items-center gap-3 py-2">
                  <span className="font-mono text-sm text-muted">#{b.id}</span>
                  <span className="flex-1 truncate text-sm">{kindLabel(b.kind)}</span>
                  <span className="font-mono text-xs text-muted">{b.nPassed}/{b.nTotal}</span>
                  <StatusBadge status={b.status} />
                  <Button size="sm" variant="ghost" onClick={() => navigate(`/build/run/${b.id}`)}>
                    Abrir <ChevronRight size={12} aria-hidden />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="rounded-md border border-border bg-surface p-4">
          <h2 className="mb-3 text-base font-semibold">Resumen</h2>
          <StatRow className="flex-col items-stretch">
            <StatCard label="Estrategias en el banco" value={String(bankSize ?? 0)} />
            <StatCard label="Generaciones totales" value={String(builds?.length ?? 0)} />
            <StatCard
              label="Núcleo"
              value={api.isDesktop() ? 'Rust (motor real)' : 'simulado (navegador)'}
              mono={false} />
          </StatRow>
          <Link to="/data" className="mt-4 inline-flex items-center gap-1 text-sm text-accent hover:underline">
            <Database size={13} aria-hidden /> Gestionar datos históricos
          </Link>
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S02 · Constructor config ────────────────── */

export function S02BuildConfig() {
  const navigate = useNavigate()
  const [symbols, setSymbols] = useState<Awaited<ReturnType<typeof api.listSymbols>>>([])
  const [dataFiles, setDataFiles] = useState<Awaited<ReturnType<typeof api.listDataFiles>>>([])
  const [templates, setTemplates] = useState<Awaited<ReturnType<typeof api.listTemplates>>>([])
  const [busy, setBusy] = useState(false)
  const [form, setForm] = useState({
    symbolId: 1,
    timeframe: 'm1' as Timeframe,
    templateId: 1,
    nTarget: 100,
    minTrades: 20,
    porRobustez: true,
    monteCarlo: true,
  })
  const [filters, setFilters] = useState<MetricFilter[]>([
    { metric: 'profitFactor', op: '>=', value: 1.15 },
    { metric: 'maxDD', op: '<=', value: 35 },
  ])
  const [metricas, setMetricas] = useState<BuildMetric[]>([])
  const [pesos, setPesos] = useState<RobustnessWeight[]>([])

  useEffect(() => {
    api.listSymbols().then(setSymbols)
    api.listDataFiles().then(setDataFiles)
    api.listTemplates().then(setTemplates)
    api.listBuildMetrics().then(r => { setMetricas(r.metrics); setPesos(r.pesos) })
  }, [])

  const tfList: Timeframe[] = ['m1', 'm5', 'm15', 'm30', 'h1', 'h4', 'd1']
  const dataForSymbol = dataFiles.filter(d => d.symbolId === form.symbolId)
  const hasData = dataForSymbol.some(d => d.timeframe === form.timeframe)

  async function start() {
    setBusy(true)
    const buildId = await api.startBuild({
      kind: 'builder', symbolId: form.symbolId, timeframe: form.timeframe,
      nTarget: form.nTarget, templateId: form.templateId,
      filters, minTrades: form.minTrades, porRobustez: form.porRobustez,
    })
    navigate(`/build/run/${buildId}`)
  }

  return (
    <Page title="Constructor — nueva generación" icon={<Wand2 size={16} />}
      actions={
        <Button variant="primary" loading={busy} onClick={start}>
          <Play size={14} aria-hidden /> Generar estrategias
        </Button>
      }>
      <form className="mx-auto max-w-3xl space-y-4 p-4" onSubmit={e => e.preventDefault()}>
        <Section title="Datos">
          <Row label="Símbolo">
            <select className={selectCls} value={form.symbolId}
              onChange={e => setForm({ ...form, symbolId: +e.target.value })} aria-label="Símbolo">
              {symbols.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          </Row>
          <Row label="Timeframe">
            <div className="flex gap-1">
              {tfList.map(tf => (
                <button key={tf} type="button" onClick={() => setForm({ ...form, timeframe: tf })}
                  className={`rounded-md border px-2 py-1 font-mono text-sm ${
                    form.timeframe === tf ? 'border-accent text-accent' : 'border-border text-muted hover:border-border-input'}`}>
                  {tf}
                </button>
              ))}
            </div>
          </Row>
          {hasData ? (
            <p className="flex items-center gap-1 text-xs text-success">
              <Info size={12} aria-hidden /> Datos listos: {fmtBytes(dataForSymbol[0].compressedBytes)} · {fmtDate(dataForSymbol[0].dateFrom)} → {fmtDate(dataForSymbol[0].dateTo)}
            </p>
          ) : (
            <p className="flex items-center gap-1 text-xs text-warning">
              <AlertTriangle size={12} aria-hidden /> No hay datos históricos para este símbolo/TF. Descárgalos en <Link to="/data" className="underline">Datos</Link>.
            </p>
          )}
        </Section>

        <Section title="Plantilla y volumen">
          <Row label="Plantilla base">
            <select className={selectCls} value={form.templateId}
              onChange={e => setForm({ ...form, templateId: +e.target.value })} aria-label="Plantilla">
              {templates.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
            </select>
          </Row>
          <Row label="Estrategias objetivo">
            <input type="number" min={10} max={2000} className={inputCls} aria-label="Estrategias objetivo"
              value={form.nTarget} onChange={e => setForm({ ...form, nTarget: Math.max(10, +e.target.value) })} />
          </Row>
        </Section>

        <Section title="Filtros por cualquier métrica">
          <div className="space-y-2">
            {filters.map((f, i) => (
              <div key={i} className="flex flex-wrap items-center gap-2">
                <select className={selectCls} aria-label={`Métrica del filtro ${i + 1}`}
                  value={f.metric}
                  onChange={e => setFilters(filters.map((x, j) => j === i ? { ...x, metric: e.target.value } : x))}>
                  {metricas.map(m => <option key={m.key} value={m.key}>{m.label}</option>)}
                </select>
                <select className={selectCls} aria-label={`Operador del filtro ${i + 1}`}
                  value={f.op}
                  onChange={e => setFilters(filters.map((x, j) => j === i ? { ...x, op: e.target.value as MetricFilter['op'] } : x))}>
                  <option value=">=">&ge;</option>
                  <option value="<=">&le;</option>
                  <option value=">">&gt;</option>
                  <option value="<">&lt;</option>
                </select>
                <input type="number" step={0.05} className={`${inputCls} w-28`}
                  aria-label={`Valor del filtro ${i + 1}`} value={f.value}
                  onChange={e => setFilters(filters.map((x, j) => j === i ? { ...x, value: +e.target.value } : x))} />
                {filters.length > 1 && (
                  <Button size="sm" variant="ghost" aria-label={`Quitar filtro ${i + 1}`}
                    onClick={() => setFilters(filters.filter((_, j) => j !== i))}>&times;</Button>
                )}
              </div>
            ))}
            <Button size="sm" variant="secondary"
              onClick={() => setFilters([...filters, { metric: 'sharpe', op: '>=', value: 0.5 }])}>
              + filtro
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted">
            Todas las métricas del motor se pueden filtrar. Si no pasa ninguna estrategia, el
            registro del build dice qué filtro está bloqueando.
          </p>
        </Section>

        <Section title="Selección y robustez">
          <Row label="Ordenar por">
            <label className="flex items-center gap-2 text-sm">
              <input type="checkbox" checked={form.porRobustez} aria-label="Ordenar por robustez"
                onChange={e => setForm({ ...form, porRobustez: e.target.checked })} />
              <span>Robustez en vez de beneficio</span>
            </label>
          </Row>
          <Row label="Operaciones mínimas">
            <input type="number" min={1} className={`${inputCls} w-28`} aria-label="Operaciones mínimas"
              value={form.minTrades} onChange={e => setForm({ ...form, minTrades: +e.target.value })} />
          </Row>
          <Row label="Monte Carlo integrado">
            <input type="checkbox" checked={form.monteCarlo} aria-label="Monte Carlo integrado"
              onChange={e => setForm({ ...form, monteCarlo: e.target.checked })} />
          </Row>
          {form.porRobustez && pesos.length > 0 && (
            <div className="mt-2 rounded-md border border-border bg-surface-raised p-2">
              <p className="mb-1 text-xs font-semibold text-muted">
                Score de robustez: gana la estrategia que aguanta, no la que más gana
              </p>
              <ul className="space-y-0.5">
                {pesos.map(w => (
                  <li key={w.key} className="text-xs text-muted">
                    <span className="font-mono text-text">{Math.round(w.peso * 100)} %</span> · {w.desc}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </Section>
      </form>
    </Page>
  )
}

/* ───────────────────────── S03 · Build en vivo ─────────────────── */

export function S03BuildRun() {
  const params = useParams()
  const buildId = params.buildId ? +params.buildId : 1
  const navigate = useNavigate()
  const [b, setB] = useState<BuildRec | null>(null)
  const [lines, setLines] = useState<Array<{ level: 'info' | 'warn' | 'error'; message: string; ts: number }>>([])
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    const iv = setInterval(async () => {
      const all = await api.listBuilds()
      setB(all.find(x => x.id === buildId) ?? null)
    }, 700)
    return () => clearInterval(iv)
  }, [buildId])

  useEffect(() => {
    const off = onBuildEvent(e => {
      if (e.buildId !== buildId) return
      setLines(prev => [{ level: e.level, message: e.message, ts: e.ts }, ...prev].slice(0, 300))
      if (e.progress != null) setProgress(p => Math.max(p, e.progress!))
    })
    return off
  }, [buildId])

  const running = b?.status === 'running'

  return (
    <Page title={`Generación #${buildId}`} icon={<Wand2 size={16} />}
      actions={
        <>
          <Button size="sm" variant="ghost" onClick={() => api.pauseBuild(buildId)} disabled={!running}>
            <Pause size={14} aria-hidden /> Pausar
          </Button>
          <Button size="sm" variant="ghost" onClick={() => api.resumeBuild(buildId)} disabled={running}>
            <Square size={14} aria-hidden /> Reanudar
          </Button>
          <Button size="sm" variant="danger" onClick={() => api.cancelBuild(buildId)} disabled={!running}>
            Detener
          </Button>
        </>
      }>
      <div className="flex h-full flex-col">
        <div className="shrink-0 border-b border-border bg-surface p-4">
          <div className="mb-2 flex items-center gap-3">
            <Progressbar value={progress} />
            <Badge tone={running ? 'running' : b?.status === 'done' ? 'pass' : 'pending'} />
          </div>
          <div className="flex gap-3 text-sm text-muted">
            <span>Pasadas: <b className="text-text">{b?.nPassed ?? 0}</b></span>
            <span>Objetivo: <b className="text-text">{b?.nTotal ?? '—'}</b></span>
          </div>
        </div>
        <div className="min-h-0 flex-1">
          <BuildLog lines={lines} running={running} className="h-full" />
        </div>
        <div className="shrink-0 border-t border-border p-2">
          <Button size="sm" variant="primary" onClick={() => navigate('/bank')}>
            <LibraryBig aria-hidden size={14} /> Ver Banco de estrategias
          </Button>
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── S04 · Bank ──────────────────────────── */

export function S04Bank() {
  const [rows, setRows] = useState<StrategyRec[] | null>(null)
  const navigate = useNavigate()

  useEffect(() => { api.listBank({}).then(setRows) }, [])

  const cols = useMemo(() => ([
    { key: 'name', label: 'Estrategia' },
    { key: 'symbol', label: 'Símbolo' },
    { key: 'tf', label: 'TF' },
    { key: 'net', label: 'Beneficio neto' },
    { key: 'pf', label: 'PF' },
    { key: 'wr', label: 'Tasa de acierto' },
    { key: 'dd', label: 'Max DD' },
    { key: 'trades', label: 'Trades' },
    { key: 'sharpe', label: 'Sharpe' },
    { key: 'rob', label: 'Robustez' },
    { key: 'score', label: 'Score' },
  ] as const), [])

  const [verdicts, setVerdicts] = useState<Record<number, StrategyVerdicts>>({})
  const [calcBusy, setCalcBusy] = useState(false)
  const [calcMsg, setCalcMsg] = useState<string | null>(null)

  // veredictos ya calculados (cache) para las estrategias del banco
  useEffect(() => {
    if (!rows?.length) return
    let alive = true
    api.listVerdicts(rows.map(r => r.id)).then(res => {
      if (!alive) return
      const map: Record<number, StrategyVerdicts> = {}
      for (const v of res) map[v.strategyId] = v
      setVerdicts(map)
    })
    return () => { alive = false }
  }, [rows])

  async function calcularVerdicts() {
    if (!rows?.length || calcBusy) return
    setCalcBusy(true)
    setCalcMsg(null)
    try {
      const res = await api.computeVerdicts(rows.map(r => r.id))
      const map: Record<number, StrategyVerdicts> = {}
      for (const v of res) map[v.strategyId] = v
      setVerdicts(map)
      setCalcMsg(`Veredictos calculados para ${res.length} estrategias.`)
    } catch (e) {
      setCalcMsg(`No se pudieron calcular: ${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setCalcBusy(false)
    }
  }

  return (
    <Page
      title="Banco de estrategias"
      icon={<LibraryBig aria-hidden size={16} />}
      actions={
        <div className="flex items-center gap-3">
          {calcMsg && <span className="text-xs text-muted">{calcMsg}</span>}
          <span className="text-sm text-muted">{rows?.length ?? '…'} estrategias</span>
          <Button variant="primary" size="sm" loading={calcBusy} onClick={() => void calcularVerdicts()}>
            Calcular veredictos
          </Button>
        </div>
      }
    >
      {!rows ? <TableSkeleton /> : rows.length === 0 ? (
        <EmptyState title="El banco está vacío"
          hint="Genera tu primera tanda: el motor dejará aquí las estrategias que pasen los filtros."
          action={<Button variant="primary" onClick={() => navigate('/build/new')}>Nueva generación</Button>} />
      ) : (
        <div className="overflow-x-auto qf-scroll">
          <table className="w-full min-w-[900px] text-sm">
          <thead>
            <tr className="bg-surface text-left text-xs text-muted">
              {cols.map(c => <th key={c.key} className="px-3 py-2 font-medium">{c.label}</th>)}
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {rows.map(s => {
              const m = s.lastRun?.metrics
              return (
                <tr key={s.id} className="cursor-pointer hover:bg-surface-raised"
                  onClick={() => navigate(`/strategy/${s.id}`)}
                  tabIndex={0}
                  onKeyDown={e => { if (e.key === 'Enter') navigate(`/strategy/${s.id}`) }}>
                  <td className="px-3 py-2">
                    <span className="flex items-center gap-2">
                      {s.status === 'dirty' && <AlertTriangle aria-hidden size={13} className="text-danger" />}
                      {s.name}
                    </span>
                  </td>
                  <td className="px-3 py-2 font-mono">{symName(s.symbolId)}</td>
                  <td className="px-3 py-2 font-mono text-muted">{s.timeframe}</td>
                  <td className={`px-3 py-2 text-right font-mono ${(m?.netProfit ?? 0) >= 0 ? 'text-success' : 'text-danger'}`}>
                    {fmtMoney(m?.netProfit)}
                  </td>
                  <td className="px-3 py-2 text-right font-mono">{fmtNum(m?.profitFactor)}</td>
                  <td className="px-3 py-2 text-right font-mono">{m ? `${(m.winRate * 100).toFixed(0)}%` : '—'}</td>
                  <td className="px-3 py-2 text-right font-mono">{fmtNum(m?.maxDD, 1)}%</td>
                  <td className="px-3 py-2 text-right font-mono">{m?.nTrades ?? '—'}</td>
                  <td className="px-3 py-2 text-right font-mono">{fmtNum(m?.sharpe)}</td>
                  <td className="px-3 py-2">
                    {verdicts[s.id]
                      ? <Badge tone={estadoTone(verdicts[s.id].overall)} label={estadoTexto(verdicts[s.id].overall)} />
                      : <span className="text-xs text-muted">sin calcular</span>}
                  </td>
                  <td className="px-3 py-2 text-right font-mono text-xs text-muted">
                    {s.robustScore != null ? fmtNum(s.robustScore, 0) : '—'}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
        </div>
      )}
    </Page>
  )
}

/* ───────────────────────── S05 · Detalle ───────────────────────── */

export function S05Strategy() {
  const { id } = useParams()
  const navigate = useNavigate()
  const [s, setS] = useState<StrategyRec | null>(null)
  const [script, setScript] = useState<string | null>(null)
  const [analisis, setAnalisis] = useState<{ veredicto: string; etiqueta: string; metricas: Record<string, number>; notas: string } | null>(null)
  const [historial, setHistorial] = useState<Array<{ id: number; script: string; verdict: string; label: string; createdAt: number }>>([])
  const [riesgo, setRiesgo] = useState<Awaited<ReturnType<typeof api.analyzeStrategyRisk>> | null>(null)
  const [riesgoMsg, setRiesgoMsg] = useState<string | null>(null)
  const [riesgoBusy, setRiesgoBusy] = useState(false)
  const [caMsg, setCaMsg] = useState<string | null>(null)
  const [caBusy, setCaBusy] = useState(false)
  useEffect(() => { api.getStrategy(+id!).then(setS) }, [id])
  useEffect(() => {
    api.listCustomAnalyses(+id!).then(setHistorial).catch(() => setHistorial([]))
  }, [id])
  const m = s?.lastRun?.metrics

  async function analizarRiesgo() {
    if (!s) return
    setRiesgoBusy(true); setRiesgoMsg(null)
    try {
      const r = await api.analyzeStrategyRisk(s.id)
      setRiesgo(r)
    } catch (e) {
      setRiesgoMsg(e instanceof Error ? e.message : String(e))
    } finally {
      setRiesgoBusy(false)
    }
  }

  async function elegirScript() {
    const ruta = await api.pickFile(['py'])
    if (ruta) { setScript(ruta); setCaMsg(null) }
  }

  async function ejecutarScript() {
    if (!script || !s) return
    setCaBusy(true); setCaMsg(null)
    try {
      const r = await api.runCustomAnalysis(s.id, script)
      setAnalisis(r.resultado)
      setHistorial(await api.listCustomAnalyses(s.id))
    } catch (e) {
      setCaMsg(e instanceof Error ? e.message : String(e))
    } finally {
      setCaBusy(false)
    }
  }

  async function exportarCsv() {
    if (!s) return
    setCaMsg(null)
    try {
      const ruta = await api.exportStrategyCsv(s.id)
      setCaMsg(`Barras exportadas a ${ruta}`)
    } catch (e) {
      setCaMsg(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Page title={s ? s.name : '…'} icon={<LibraryBig size={16} />}
      actions={
        s ? (
          <Button variant="secondary" onClick={() => navigate(`/strategy/${s.id}/edit`)}>
            Editar regla
          </Button>
        ) : undefined
      }>
      {!s ? <TableSkeleton /> : (
        <div className="space-y-4 p-4">
          <StatRow>
            <StatCard label="Beneficio neto" value={m?.netProfit ?? 0} unit="USD" tone={(m?.netProfit ?? 0) >= 0 ? 'pass' : 'fail'} />
            <StatCard label="Profit factor" value={m?.profitFactor ?? 0} tone={(m?.profitFactor ?? 0) >= 1.3 ? 'pass' : 'fail'} />
            <StatCard label="Tasa de acierto" value={(m?.winRate ?? 0) * 100} unit="%" />
            <StatCard label="Max DD" value={m?.maxDD ?? 0} unit="%" tone="fail" />
            <StatCard label="Sharpe" value={m?.sharpe ?? 0} />
            <StatCard label="Trades" value={m?.nTrades ?? 0} />
          </StatRow>

          {(s.problems?.length ?? 0) > 0 && (
            <div className="rounded-md border border-danger/40 bg-danger/10 p-3 text-sm text-danger" role="alert">
              <b>Detección automática:</b> {s.problems.join(', ')}
            </div>
          )}

          <div className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-base font-semibold">Curva de equity (backtest principal)</h2>
            <EquityChart series={s.lastRun?.equity ?? null} label="Equity del backtest" height={240}
              oosFrom={s.lastRun?.oosFrac ?? 0.4} oosLabel="Fuera de muestra" />
          </div>

          <VerdictCard strategyIds={[s.id]} title="Veredictos de robustez de esta estrategia" />

          <section className="rounded-md border border-border bg-surface p-3">
            <div className="mb-2 flex items-center gap-2">
              <h2 className="text-sm font-semibold text-muted">Riesgo real (MAE / MFE y capital diario)</h2>
              <Button variant="secondary" size="sm" loading={riesgoBusy} className="ml-auto"
                onClick={() => void analizarRiesgo()}>
                Calcular perfil de riesgo
              </Button>
            </div>
            <p className="mb-2 text-xs text-muted">
              MAE es lo peor que se puso la posicion; MFE, lo mejor que llego a tomar. Si el MFE es
              alto y el resultado final mediocre, la salida se esta comiendo el recorrido.
            </p>
            {riesgoMsg && <p className="text-xs text-muted" role="status">{riesgoMsg}</p>}
            {riesgo && (
              <>
                <div className="mb-2 flex flex-wrap gap-3 text-xs text-muted">
                  <span>MAE medio <b className="text-text">{fmtNum(riesgo.maePct, 2)} %</b></span>
                  <span>MFE medio <b className="text-text">{fmtNum(riesgo.mfePct, 2)} %</b></span>
                  <span>peor MAE <b className="text-text">{fmtNum(riesgo.peorMaePct, 2)} %</b></span>
                  <span>mejor MFE <b className="text-text">{fmtNum(riesgo.mejorMfePct, 2)} %</b></span>
                  <span>expectativa <b className="text-text">{fmtMoney(riesgo.expectativa)}</b>
                    {' '}({fmtNum(riesgo.expectativaPct, 2)} %)</span>
                </div>
                <div className="rounded-md border border-border bg-surface p-2">
                  <p className="mb-1 text-xs font-semibold text-muted">Capital diario</p>
                  <EquityChart
                    series={riesgo.capitalDiario.map(p => p.v)}
                    label="Curva de capital diario"
                    height={180} />
                </div>
              </>
            )}
          </section>

          <HistoryPanel strategyId={s.id} title="Historial de analisis de esta estrategia" />

          <section className="rounded-md border border-border bg-surface p-3">
            <h2 className="mb-2 text-sm font-semibold text-muted">Analisis personalizado (Python)</h2>
            <p className="mb-2 text-xs text-muted">
              Elige un script y el motor le pasa la estrategia y sus barras. El script recibe
              <span className="font-mono"> --input &lt;json&gt; </span> y escribe
              <span className="font-mono"> --output &lt;json&gt; </span>. Contrato en
              <span className="font-mono"> replica/custom-analysis.md</span>.
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <Button variant="secondary" size="sm" onClick={() => void elegirScript()}>
                {script ? 'Cambiar script' : 'Elegir script'}
              </Button>
              {script && <span className="font-mono text-xs text-muted">{script.split(/[\\/]/).pop()}</span>}
              <Button variant="primary" size="sm" loading={caBusy} disabled={!script}
                onClick={() => void ejecutarScript()}>
                Ejecutar
              </Button>
              <Button variant="ghost" size="sm" onClick={() => void exportarCsv()}>
                Exportar barras a CSV
              </Button>
            </div>

            {caMsg && <p className="mt-2 text-xs text-muted" role="status">{caMsg}</p>}

            {analisis && (
              <div className="mt-3 rounded-md border border-border bg-surface-raised p-2">
                <p className="text-sm">
                  <Badge tone={analisis.veredicto === 'pass' ? 'pass' : analisis.veredicto === 'fail' ? 'fail' : 'running'}
                    label={analisis.etiqueta || analisis.veredicto.toUpperCase()} />
                </p>
                {Object.keys(analisis.metricas).length > 0 && (
                  <p className="mt-1 font-mono text-xs text-muted">
                    {Object.entries(analisis.metricas).map(([k, v]) => (
                      <span key={k} className="mr-2">{k} {fmtNum(v, 3)}</span>
                    ))}
                  </p>
                )}
                {analisis.notas && <p className="mt-1 text-xs text-muted">{analisis.notas}</p>}
              </div>
            )}

            {historial.length > 0 && (
              <ul className="mt-3 space-y-1">
                {historial.map(h => (
                  <li key={h.id} className="text-xs text-muted">
                    {new Date(h.createdAt * 1000).toLocaleString('es-ES')} · {h.script.split(/[\\/]/).pop()} · {h.label || h.verdict}
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      )}
    </Page>
  )
}

/* ───────────────────────── S20 · Datos ─────────────────────────── */

export function S20Data() {
  const [symbols, setSymbols] = useState<Awaited<ReturnType<typeof api.listSymbols>>>([])
  const [files, setFiles] = useState<Awaited<ReturnType<typeof api.listDataFiles>>>([])
  const [busy, setBusy] = useState<number | null>(null)
  const [progress, setProgress] = useState(0)
  const [toast, setToast] = useState<string | null>(null)

  useEffect(() => { api.listSymbols().then(setSymbols); api.listDataFiles().then(setFiles) }, [])

  async function download(symbolId: number) {
    setBusy(symbolId); setProgress(0)
    try {
      await api.downloadData(symbolId, 'm1', Date.now() - 2 * 365 * 86_400_000, Date.now(), setProgress)
      setFiles(await api.listDataFiles())
      setToast(`Histórico descargado para ${symName(symbolId)}`)
      setTimeout(() => setToast(null), 3500)
    } finally {
      setBusy(null)
    }
  }

  return (
    <Page title="Datos históricos" icon={<Database size={16} />}
      actions={<Button size="sm" variant="secondary"><Download size={14} aria-hidden /> Fuentes de datos</Button>}>
      {toast && (
        <div role="status"
          className="absolute right-4 top-14 rounded-md border border-success/40 bg-success/10 px-3 py-2 text-sm text-success shadow-pop">
          {toast}
        </div>
      )}
      <div className="overflow-x-auto qf-scroll">
        <table className="w-full min-w-[720px] text-sm">
          <thead>
            <tr className="bg-surface text-left text-xs text-muted">
              <th className="px-3 py-2 font-medium">Símbolo</th>
              <th className="px-3 py-2 font-medium">Clase</th>
              <th className="px-3 py-2 font-medium">Datasets</th>
              <th className="px-3 py-2 font-medium"><span className="sr-only">Acciones</span></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {symbols.map(s => {
              const ds = files.filter(f => f.symbolId === s.id)
              return (
                <tr key={s.id} className="hover:bg-surface-raised">
                  <td className="px-3 py-2 font-mono">{s.name}</td>
                  <td className="px-3 py-2 text-muted">{s.assetClass}</td>
                  <td className="px-3 py-2 text-muted">
                    {ds.length === 0
                      ? <span className="text-warning">sin datos</span>
                      : ds.map(d => (
                        <span key={d.id} className="mr-1 rounded-pill bg-surface-raised px-1.5 py-0.5 font-mono text-xs">
                          {d.timeframe} · {fmtBytes(d.compressedBytes)}
                        </span>
                      ))}
                  </td>
                  <td className="px-3 py-2 text-right">
                    <Button size="sm" loading={busy === s.id} disabled={busy != null && busy !== s.id}
                      onClick={() => download(s.id)}>
                      {busy === s.id ? `${Math.round(progress * 100)}%` : (<><Download size={13} aria-hidden /> Descargar m1</>)}
                    </Button>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </Page>
  )
}

/* ───────────────────────── S22 · Export ────────────────────────── */

// Solo se anuncian plataformas con export real. El motor emite MQL4 y MQL5;
// anadir mas plataformas sin generador seria prometer algo que no existe.
const platforms: Array<{ key: Platform; label: string; note: string }> = [
  { key: 'mql5', label: 'MetaTrader 5', note: 'El mismo EA sirve en cuentas hedge y netting: el modo lo fija la cuenta del broker.' },
  { key: 'mql4', label: 'MetaTrader 4', note: 'EA clásico con OnTick y gestión de posición.' },
]

export function S22Export() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [platform, setPlatform] = useState<Platform>('mql5')
  const [code, setCode] = useState<string | null>(null)
  const [verif, setVerif] = useState<ExportVerificacion | null>(null)
  const [busy, setBusy] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => { api.listBank({}).then(setRows) }, [])

  async function generate() {
    if (!sel) return
    setBusy(true); setCode(null); setSaved(false)
    const out = await api.exportStrategy(sel.id, platform)
    setCode(out.code); setVerif(out.verificacion); setBusy(false)
  }

  return (
    <Page title="Export código fuente" icon={<FileCode2 size={16} />}>
      <div className="grid gap-4 p-4 xl:grid-cols-[1fr_1.2fr]">
        <div>
          <h2 className="mb-2 text-base font-semibold">1. Elige la estrategia</h2>
          <div className="max-h-64 overflow-auto qf-scroll rounded-md border border-border">
            {rows.length === 0 ? (
              <EmptyState title="Sin estrategias" hint="Genera una estrategia primero." />
            ) : (
              <ul className="divide-y divide-border">
                {rows.map(r => (
                  <li key={r.id}>
                    <button
                      className={`flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-surface-raised ${sel?.id === r.id ? 'bg-surface-raised' : ''}`}
                      onClick={() => { setSel(r); setCode(null); setSaved(false) }}
                      aria-pressed={sel?.id === r.id}>
                      <span className="flex-1 truncate">{r.name}</span>
                      <span className="font-mono text-xs text-muted">{fmtMoney(r.lastRun?.metrics?.netProfit)}</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <h2 className="mb-2 mt-4 text-base font-semibold">2. Plataforma</h2>
          <div className="grid grid-cols-2 gap-2" role="radiogroup" aria-label="Plataforma de destino">
            {platforms.map(p => (
              <button key={p.key} role="radio" aria-checked={platform === p.key}
                onClick={() => { setPlatform(p.key); setCode(null) }}
                className={`rounded-md border px-3 py-2 text-left text-sm ${
                  platform === p.key ? 'border-accent text-accent' : 'border-border text-muted hover:border-border-input'}`}>
                {p.label}
              </button>
            ))}
          </div>
          <p className="mt-2 text-xs text-muted">{platforms.find(p => p.key === platform)?.note}</p>
          <p className="mt-1 text-xs text-muted">
            Solo hay export real para MetaTrader. No se anuncian otras plataformas hasta que
            haya un generador de verdad: es preferible a devolver un hueco.
          </p>

          <Button variant="primary" className="mt-4" loading={busy} disabled={!sel} onClick={generate}>
            <Play size={14} aria-hidden /> Generar código
          </Button>
        </div>

        <div className="flex min-h-0 flex-col">
          <h2 className="mb-2 text-base font-semibold">3. Código generado</h2>
          {code ? (
            <>
              <pre className="flex-1 overflow-auto qf-scroll rounded-md border border-border bg-surface p-3 font-mono text-xs text-muted">{code}</pre>
              {verif && (
                <div className="mt-3 rounded-md border border-border bg-surface p-3">
                  <h3 className="mb-2 text-sm font-semibold text-muted">Verificacion antes de fiarte del EA</h3>
                  <p className="mb-2 text-xs text-muted">
                    Regla exportada: <b className="text-text">{verif.regla}</b>
                    {Object.entries(verif.parametros).map(([k, v]) => (
                      <span key={k} className="ml-2 font-mono text-muted">{k}={v}</span>
                    ))}
                  </p>

                  <h4 className="mb-1 text-xs font-semibold text-muted">Lo que el backtest ha asumido</h4>
                  <ul className="mb-3 list-inside list-disc text-xs text-muted">
                    {verif.supuestos.map((s, i) => <li key={i}>{s}</li>)}
                  </ul>

                  <h4 className="mb-1 text-xs font-semibold text-muted">Checklist en MetaTrader</h4>
                  <ol className="mb-2 list-inside list-decimal text-xs text-muted">
                    {verif.checklist.map((c, i) => <li key={i}>{c}</li>)}
                  </ol>
                  <p className="text-xs text-muted">
                    Documento completo: <span className="font-mono">{verif.documento}</span>
                  </p>
                </div>
              )}
              <Button className="mt-2" variant="secondary"
                onClick={async () => { await api.saveExportFile(`${sel!.name}.${platform}`, code); setSaved(true) }}>
                <Download size={14} aria-hidden /> Guardar fichero…
              </Button>
              {saved && <p role="status" className="mt-1 text-xs text-success">Fichero guardado (fake).</p>}
            </>
          ) : (
            <div className="flex flex-1 items-center justify-center rounded-md border border-border bg-surface text-sm text-muted">
              {busy ? 'Generando…' : 'Aún no hay código. Genera uno.'}
            </div>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────────── /design ─────────────────────────────── */

export function DesignPage() {
  const roles = ['bg', 'surface', 'surface-raised', 'border', 'border-input', 'text', 'text-muted', 'accent', 'on-accent', 'danger', 'success', 'warning']
  return (
    <Page title="Sistema de diseño" icon={<Download aria-hidden size={16} />}>
      <div className="space-y-6 p-4">
        <section>
          <h2 className="mb-2 text-base font-semibold">Roles de color</h2>
          <div className="grid grid-cols-4 gap-2">
            {roles.map(r => (
              <div key={r}>
                <div className="h-14 rounded-md border border-border" style={{ background: `var(--color-${r})` }} />
                <p className="mt-1 text-xs text-muted">{r}</p>
              </div>
            ))}
          </div>
        </section>
        <section>
          <h2 className="mb-2 text-base font-semibold">Tipografía</h2>
          <p className="text-xs">xs 12 · <span className="text-sm">sm 13</span> · base 14 · <b className="text-lg">lg 16 semibold</b> · <b className="text-xl">xl 20</b> · <b className="text-display">display 28</b></p>
          <p className="mt-1 font-mono text-sm">mono: JetBrains Mono — 42.230,55</p>
        </section>
        <section>
          <h2 className="mb-2 text-base font-semibold">Botones</h2>
          <div className="flex flex-wrap gap-2">
            <Button variant="primary">Primary</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="danger">Danger</Button>
            <Button loading>Loading</Button>
            <Button disabled>Disabled</Button>
          </div>
        </section>
        <section>
          <h2 className="mb-2 text-base font-semibold">Badges y stats</h2>
          <div className="flex items-center gap-2">
            <Badge tone="pass" /><Badge tone="fail" /><Badge tone="running" /><Badge tone="pending" />
          </div>
          <StatRow className="mt-3">
            <StatCard label="Beneficio neto" value={42230} unit="USD" tone="pass" delta={12} />
            <StatCard label="Max DD" value={18.2} unit="%" tone="fail" delta={-4} />
            <StatCard label="Tasa de acierto" value={63.5} unit="%" />
          </StatRow>
        </section>
      </div>
    </Page>
  )
}

/* ───────────────────── stubs por fase (S06..) ──────────────────── */
// S06, S07, S12, S13 viven en ./phase2.tsx (Fase 2)

export { S08Retester, S09Improver, S10Optimizer, S11Opt3D, S14Robustness, S16Projects, S18Portfolio } from './phase3'
export { S15OptProfile, S19PortfolioResults, S21DataManager, S23Settings, S24Analyzer } from './phase4'

/* ───────────────────────── S25 · Paridad ────────────────────────────
   Datos generados por replica-diff (parity.py + imgdiff.py). Se regeneran
   con: python .claude/skills/replica-diff/parity.py replica/features.csv  */
import parity from '../data/parity.json'

const tone = (score: number): 'pass' | 'fail' | 'neutral' =>
  score >= 80 ? 'pass' : score >= 60 ? 'neutral' : 'fail'

export const S25Diff = () => (
  <Page title="Paridad con el original" icon={<GitCompareArrows size={16} />}>
    <div className="space-y-4 p-4">
      <section className="rounded-md border border-border bg-surface p-3">
        <h2 className="mb-2 text-sm font-semibold text-muted">Puntuación global</h2>
        <div className="grid gap-3 md:grid-cols-4">
          <StatCard label="Paridad total" value={parity.paridad_global} unit="/100" tone={tone(parity.paridad_global)} />
          <StatCard label="Dentro del alcance" value={parity.paridad_alcance} unit="/100" tone={tone(parity.paridad_alcance)} />
          <StatCard label="Must-haves" value={`${parity.must_haves.hechos}/${parity.must_haves.total}`} />
          <StatCard label="Layout (Retesteo)" value={parity.layout.puntuacion} unit="/100" tone="fail" />
        </div>
        <p className="mt-3 text-xs text-muted">
          Medido el {parity.fecha} por {parity.generado_por} sobre {parity.funciones_contadas} funciones
          comparables. «Dentro del alcance» excluye lo que decidiste aplazar (descargas de feeds
          externos y licencias por tiers). El layout usa como referencia un recorte de la captura
          pública de la web del original: es orientativo, no una medida de calidad.
        </p>
      </section>

      <section className="rounded-md border border-border bg-surface p-3">
        <h2 className="mb-2 text-sm font-semibold text-muted">Cadena de validación (lo que usas de verdad)</h2>
        <div className="grid gap-1 md:grid-cols-2">
          {parity.cadena_validacion.map(c => (
            <div key={c.paso} className="flex items-center justify-between gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-raised">
              <span className="text-text">{c.paso}</span>
              <Badge tone={c.estado === 'real' ? 'pass' : 'pending'} label={c.estado === 'real' ? 'real' : 'parcial'} />
            </div>
          ))}
        </div>
      </section>

      <section className="rounded-md border border-border bg-surface p-3">
        <h2 className="mb-2 text-sm font-semibold text-muted">Áreas por puntuación (más débil primero)</h2>
        <ul className="space-y-1">
          {parity.areas.map(a => (
            <li key={a.area} className="flex items-center gap-3 text-sm">
              <span className="w-40 shrink-0 text-muted">{a.area}</span>
              <span className="h-1.5 flex-1 overflow-hidden rounded-pill bg-surface-raised">
                <span className="block h-full rounded-pill bg-accent" style={{ width: `${a.score}%` }} />
              </span>
              <span className="w-14 shrink-0 text-right font-mono text-xs text-text">{a.score}</span>
            </li>
          ))}
        </ul>
      </section>

      <section className="rounded-md border border-border bg-surface p-3">
        <h2 className="mb-2 text-sm font-semibold text-muted">Pendientes de prioridad must</h2>
        <ul className="space-y-2">
          {parity.pendientes_must.map(m => (
            <li key={m.feature} className="text-sm">
              <div className="flex items-center gap-2">
                <Badge tone={m.clone === 'no' ? 'fail' : 'pending'} label={m.clone === 'no' ? 'sin hacer' : 'parcial'} />
                <span className="text-text">{m.feature}</span>
              </div>
              {m.notes && <p className="mt-0.5 pl-1 text-xs text-muted">{m.notes}</p>}
            </li>
          ))}
        </ul>
      </section>

      <section className="rounded-md border border-border bg-surface p-3">
        <h2 className="mb-2 text-sm font-semibold text-muted">Fuera de alcance por decisión (no computan)</h2>
        <ul className="list-inside list-disc text-sm text-muted">
          {parity.fuera_de_alcance.map(f => <li key={f}>{f}</li>)}
        </ul>
      </section>
    </div>
  </Page>
)


export { S06Chart, S07Editor, S12WfMatrix, S13MonteCarlo } from './phase2'