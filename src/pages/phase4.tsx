// phase4.tsx — pantallas de Fase 4: Perfil de optimización (S15), Resultados de
// Cartera (S19), Gestor de datos (S21), Ajustes (S23) y Laboratorio (S24).
import { useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  Play, TrendingUp, Wallet, FileUp, FileDown, Save, Activity, Boxes,
} from 'lucide-react'
import { api } from '../lib/api'
import type {
  AppSettings, OptProfilePoint, OptProfileResult, PortfolioMcResult, PortfolioResults,
} from '../lib/api'
import type { DataFileRec, StrategyRec } from '../lib/types'
import { Page } from '../components/Page'
import { Button } from '../components/Button'
import { Badge } from '../components/Badge'
import { EmptyState } from '../components/EmptyState'
import { VerdictCard } from '../components/VerdictCard'
import { TableSkeleton } from '../components/TableSkeleton'
import { EquityChart } from '../components/EquityChart'
import { StatCard, StatRow } from '../components/StatCard'
import { fmtMoney, fmtNum } from '../lib/utils'

const selectCls = 'h-8 rounded-md border border-border-input bg-surface px-2 text-sm'

/* ───────────────────── S15 · Perfil de optimización ────────────────── */

/** Curva net vs valor del parámetro con banda de meseta estable (SVG propio). */
function ProfileChart({ res }: { res: OptProfileResult }) {
  const W = 900, H = 260, padL = 56, padB = 34, padT = 14
  const nets = res.points.map(p => p.netProfit)
  const min = Math.min(...nets), max = Math.max(...nets)
  const span = max - min || 1
  const x = (p: OptProfilePoint) => padL + (res.points.indexOf(p) / (res.points.length - 1)) * (W - padL - 16)
  const y = (v: number) => padT + (1 - (v - min) / span) * (H - padT - padB)

  const stablePts = res.points.filter(p => p.param >= res.stableFrom && p.param <= res.stableTo)
  const bandX0 = stablePts.length ? x(stablePts[0]) : padL
  const bandX1 = stablePts.length ? x(stablePts[stablePts.length - 1]) : padL

  const line = res.points.map(p => `${x(p).toFixed(1)},${y(p.netProfit).toFixed(1)}`).join('L')
  const area = `M${padL},${H - padB}L${line}L${W - 16},${H - padB}Z`

  return (
    <div className="overflow-x-auto qf-scroll rounded-md border border-border bg-surface p-3">
      <svg viewBox={`0 0 ${W} ${H}`} className="min-w-[560px]" role="img"
        aria-label={`Perfil de optimización: forma ${res.shape}, rango estable de ${res.stableFrom} a ${res.stableTo}, mejor valor ${res.bestParam}`}>
        {/* banda de meseta */}
        <rect x={bandX0} y={padT} width={Math.max(0, bandX1 - bandX0)}
          height={H - padT - padB} fill="var(--color-success)" opacity={0.08}
          aria-hidden />
        <line x1={bandX0} y1={padT} x2={bandX0} y2={H - padB}
          stroke="var(--color-success)" strokeDasharray="4 3" opacity={0.5} aria-hidden />
        <line x1={bandX1} y1={padT} x2={bandX1} y2={H - padB}
          stroke="var(--color-success)" strokeDasharray="4 3" opacity={0.5} aria-hidden />
        <text x={bandX0 + 6} y={padT + 16} fill="var(--color-success)" fontSize={12}
          aria-hidden>zona estable</text>
        {/* eje y */}
        <text x={6} y={padT + 12} fill="var(--color-text-muted)" fontSize={11} aria-hidden>
          {fmtMoney(max)}</text>
        <text x={6} y={H - padB} fill="var(--color-text-muted)" fontSize={11} aria-hidden>
          {fmtMoney(min)}</text>
        {/* playground */}
        <path d={area} fill="var(--color-accent)" opacity={0.1} aria-hidden />
        <path d={`M${line}`} fill="none" stroke="var(--color-accent)" strokeWidth={1.6}
          vectorEffect="non-scaling-stroke" aria-hidden />
        {res.points.map(p => (
          <circle key={p.param} cx={x(p)} cy={y(p.netProfit)} r={2.5}
            fill={p.param === res.bestParam ? 'var(--color-text)' : 'var(--color-accent)'}
            aria-hidden />
        ))}
        <text x={padL} y={H - 10} fill="var(--color-text-muted)" fontSize={11} aria-hidden>
          {res.points[0].param}</text>
        <text x={W - 40} y={H - 10} fill="var(--color-text-muted)" fontSize={11} aria-hidden>
          {res.points[res.points.length - 1].param}</text>
        <text x={W / 2 - 40} y={H - 10} fill="var(--color-text-muted)" fontSize={12} aria-hidden>
          {res.param} →</text>
      </svg>
    </div>
  )
}

const shapeTone = { meseta: 'pass', unimodal: 'running', puntual: 'fail' } as const
const shapeLabel = { meseta: 'MESETA', unimodal: 'UNIMODAL', puntual: 'PUNTUAL' } as const
const shapeHint = {
  meseta: 'Zona amplia de parámetros con buen resultado: robusta a la elección exacta.',
  unimodal: 'Un solo valle claro: correcta, pero el óptimo es más sensible.',
  puntual: 'Pico afilado: el resultado depende de acertar un valor de azar — frágil.',
}

export function S15OptProfile() {
  const [rows, setRows] = useState<StrategyRec[]>([])
  const [sel, setSel] = useState<StrategyRec | null>(null)
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<OptProfileResult | null>(null)

  useEffect(() => { api.listBank({}).then(rs => { setRows(rs); setSel(rs[0] ?? null) }) }, [])

  async function run() {
    if (!sel) return
    setBusy(true); setRes(null)
    setRes(await api.runOptProfile(sel.id))
    setBusy(false)
  }

  return (
    <Page title="Perfil de optimización" icon={<TrendingUp size={16} />}
      actions={
        <Button variant="primary" loading={busy} disabled={!sel} onClick={run}>
          <Play size={14} aria-hidden /> Analizar perfil
        </Button>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Estrategia</h2>
          <select aria-label="Estrategia para el perfil" className={`w-full ${selectCls}`}
            value={sel?.id ?? ''} onChange={e => setSel(rows.find(r => r.id === +e.target.value) ?? null)}>
            {rows.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
          </select>
          <p className="mt-3 text-xs text-muted">
            Perfil tipo Pardo: recorre un parámetro de punta a punta y mira el resultado.
            Lo que buscas no es el pico más alto — es una <b className="text-text">meseta</b>.
          </p>
          <Link to="/robustness" className="mt-3 inline-block text-sm text-accent hover:underline">
            ← volver a Robustez (SPP)
          </Link>
        </div>

        <div className="min-w-0 space-y-3">
          {!res ? (
            <EmptyState title="Sin perfil todavía"
              hint="Corre el análisis para ver la curva de resultado por valor del parámetro y su zona estable." />
          ) : (
            <>
              <div className="flex flex-wrap items-center gap-3">
                <Badge tone={shapeTone[res.shape]} label={shapeLabel[res.shape]} />
                <span className="text-sm text-muted">{shapeHint[res.shape]}</span>
              </div>
              <StatRow>
                <StatCard label="Mejor valor del parámetro" value={res.bestParam} />
                <StatCard label="Zona estable" value={`${res.stableFrom}–${res.stableTo}`} mono={false} />
                <StatCard label="Anchura de la meseta" value={res.stableTo - res.stableFrom} unit="pasos" />
              </StatRow>
              <ProfileChart res={res} />
            </>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────── S19 · Resultados de cartera ───────────────── */

export function S19PortfolioResults() {
  const [rows, setRows] = useState<StrategyRec[] | null>(null)
  const [selIds, setSelIds] = useState<number[]>([])
  const [busy, setBusy] = useState(false)
  const [res, setRes] = useState<PortfolioResults | null>(null)
  const [names, setNames] = useState<Record<number, string>>({})

  useEffect(() => {
    api.listBank({}).then(rs => {
      setRows(rs)
      const map: Record<number, string> = {}
      rs.forEach(r => { map[r.id] = r.name })
      setNames(map)
      setSelIds(rs.slice(0, 4).map(r => r.id))
    })
  }, [])

  async function run() {
    if (selIds.length < 2) return
    setBusy(true); setRes(null)
    setRes(await api.getPortfolioResults(selIds))
    setBusy(false)
  }

  const toggle = (id: number) =>
    setSelIds(p => p.includes(id) ? p.filter(x => x !== id) : [...p, id])

  return (
    <Page title="Cartera · resultados" icon={<Wallet size={16} />}
      actions={
        <> <Button variant="primary" loading={busy} disabled={selIds.length < 2} onClick={run}>
          <Play size={14} aria-hidden /> Re-analizar ({selIds.length})
        </Button>
          <Link to="/portfolio" style={{ display: 'contents' }} aria-label="Volver al composer">
            <Button variant="ghost">← Composer</Button>
          </Link>
        </>
      }>
      <div className="grid gap-4 p-4 xl:grid-cols-[320px_1fr]">
        <div className="rounded-md border border-border bg-surface p-3">
          <h2 className="mb-2 text-sm font-semibold text-muted">Miembros (2+)</h2>
          {!rows ? <TableSkeleton /> : (
            <ul className="max-h-96 divide-y divide-border overflow-auto qf-scroll text-sm"
              aria-label="Miembros de la cartera">
              {rows.map(r => (
                <li key={r.id}>
                  <label className="flex cursor-pointer items-center gap-2 px-2 py-1.5 hover:bg-surface-raised">
                    <input type="checkbox" aria-label={`Incluir ${r.name} en el análisis`}
                      checked={selIds.includes(r.id)} onChange={() => toggle(r.id)} />
                    <span className="flex-1 truncate">{r.name}</span>
                    <span className="font-mono text-xs text-muted">{fmtMoney(r.lastRun?.metrics?.netProfit)}</span>
                  </label>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="min-w-0 space-y-4">
          {!res ? (
            <EmptyState title="Sin resultados de cartera"
              hint="Marca los miembros y re-analiza: versas la equity combinada, el aporte de cada uno y la correlación." />
          ) : (
            <>
              <StatRow>
                <StatCard label="Net combinado" value={res.totals.netProfit} unit="USD"
                  tone={res.totals.netProfit >= 0 ? 'pass' : 'fail'} />
                <StatCard label="Max DD de la cartera" value={res.totals.maxDD} unit="%" tone="fail" />
                <StatCard label="Sharpe combinado" value={res.totals.sharpe} />
                <StatCard label="Diversificación" value={`${Math.round(res.totals.diversification * 100)}`}
                  unit="% DD evitada" tone={res.totals.diversification >= 0.3 ? 'pass' : 'fail'} />
              </StatRow>

              <div>
                <h2 className="mb-2 text-base font-semibold">Equity combinada</h2>
                <div className="rounded-md border border-border bg-surface p-3">
                  <EquityChart series={res.combinedEquity} label="Equity combinada de la cartera" height={200} />
                </div>
              </div>

              <div>
                <h2 className="mb-2 text-base font-semibold">Aporte por miembro</h2>
                <div className="overflow-x-auto qf-scroll">
                  <table className="w-full min-w-[640px] text-sm">
                    <thead>
                      <tr className="bg-surface text-left text-xs text-muted">
                        <th className="px-3 py-2 font-medium">Estrategia</th>
                        <th className="px-3 py-2 font-medium">Peso</th>
                        <th className="px-3 py-2 font-medium">Net</th>
                        <th className="px-3 py-2 font-medium">Max DD</th>
                        <th className="px-3 py-2 font-medium">Correlación media</th>
                        <th className="px-3 py-2 font-medium">Veredicto</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border">
                      {res.members.map(m => (
                        <tr key={m.strategyId} className="hover:bg-surface-raised">
                          <td className="px-3 py-2">
                            <Link className="text-accent hover:underline" to={`/strategy/${m.strategyId}`}>
                              {names[m.strategyId] ?? `#${m.strategyId}`}
                            </Link>
                          </td>
                          <td className="px-3 py-2 font-mono">{m.weight}%</td>
                          <td className={`px-3 py-2 font-mono ${m.netProfit >= 0 ? 'text-success' : 'text-danger'}`}>
                            {fmtMoney(m.netProfit)}
                          </td>
                          <td className="px-3 py-2 font-mono">{fmtNum(m.maxDD, 1)}%</td>
                          <td className={`px-3 py-2 font-mono ${m.correlation >= 0.75 ? 'text-danger' : ''}`}>
                            {fmtNum(m.correlation)}
                          </td>
                          <td className="px-3 py-2">
                            <Badge tone={m.verdict === 'pass' ? 'pass' : 'fail'}
                              label={m.verdict === 'pass' ? 'OK' : 'CORRELADA'} />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </Page>
  )
}

/* ───────────────────── S21 · Gestor de datos ────────────────────── */

export function S21DataManager() {
  const [files, setFiles] = useState<DataFileRec[] | null>(null)
  const [symbols, setSymbols] = useState<Record<number, string>>({})
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)
  const [fmtSel, setFmtSel] = useState<'csv' | 'qfd'>('csv')
  const [importSymbol, setImportSymbol] = useState<number>(1)
  const [importTf, setImportTf] = useState<string>('m1')
  const fileRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    api.listDataFiles().then(setFiles)
    api.listSymbols().then(ss => {
      const map: Record<number, string> = {}
      ss.forEach(s => { map[s.id] = s.name })
      setSymbols(map)
    })
  }, [])

  /** Importa un CSV: en escritorio usa el selector nativo; en el navegador, el input. */
  async function importar() {
    setBusy(true); setNotice(null)
    try {
      const ruta = await api.pickCsvFile()
      if (!ruta) {
        // sin ruta: en el navegador usamos el input de ficheros; en escritorio
        // significa que el usuario cancelo el dialogo nativo.
        if (!api.isDesktop()) fileRef.current?.click()
        setBusy(false)
        return
      }
      const df = await api.importDatasetFileFromPath(ruta, importSymbol, importTf)
      setFiles(prev => [...(prev ?? []), df])
      setNotice(`Importado ${ruta.split(/[\\/]/).pop()}: ${fmtNum(df.nBars, 0)} barras listas.`)
    } catch (e) {
      setNotice(`No se pudo importar: ${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setBusy(false)
    }
  }

  async function onImportFile(e: React.ChangeEvent<HTMLInputElement>) {
    const f = e.target.files?.[0]
    if (!f) return
    setBusy(true); setNotice(null)
    const df = await api.importDatasetFile(f.name, f.size)
    setFiles(prev => [...(prev ?? []), df])
    setNotice(`Importado ${f.name}: ${fmtNum(df.nBars, 0)} barras listas.`)
    setBusy(false)
    e.target.value = ''   // permite reimportar el mismo fichero
  }

  async function onExport(id: number) {
    setBusy(true); setNotice(null)
    const r = await api.exportDataset(id, fmtSel)
    setNotice(`Exportado a ${r.fileName} (${fmtNum(r.bytes, 0)} bytes).`)
    setBusy(false)
  }

  const symName = (id: number) => symbols[id] ?? `#${id}`
  const kindLabel: Record<DataFileRec['storageKind'], string> = {
    local_qfd: 'QFD local', imported_csv: 'CSV importado', dukascopy: 'Dukascopy', yahoo: 'Yahoo',
  }

  return (
    <Page title="Gestor de datos" icon={<FileUp size={16} />}
      actions={
        <>
          <Button variant="ghost" loading={busy} onClick={() => void importar()}>
            <FileUp size={14} aria-hidden /> Importar CSV
          </Button>
          <div className="flex items-center gap-2">
            <label className="text-xs text-muted" htmlFor="qdm-dest-sym">Importar a</label>
            <select id="qdm-dest-sym" className={selectCls} value={importSymbol}
              aria-label="Símbolo de destino de la importación"
              onChange={e => setImportSymbol(+e.target.value)}>
              {Object.entries(symbols).map(([id, name]) => (
                <option key={id} value={id}>{name}</option>
              ))}
            </select>
            <select className={selectCls} value={importTf}
              aria-label="Temporalidad de destino de la importación"
              onChange={e => setImportTf(e.target.value)}>
              {['m1', 'm5', 'm15', 'm30', 'h1', 'h4', 'd1'].map(tf => (
                <option key={tf} value={tf}>{tf}</option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-2">
            <label className="text-xs text-muted" htmlFor="qdm-fmt">Formato</label>
            <select id="qdm-fmt" className={selectCls} value={fmtSel}
              onChange={e => setFmtSel(e.target.value as 'csv' | 'qfd')}>
              <option value="csv">csv</option>
              <option value="qfd">qfd</option>
            </select>
          </div>
        </>
      }>
      <input ref={fileRef} type="file" accept=".csv,text/csv" className="hidden"
        aria-label="Fichero CSV a importar" onChange={onImportFile} data-testid="qdm-file" />

      {notice && (
        <p className="mx-4 mb-2 rounded-md border border-success/40 bg-success/10 p-3 text-sm text-success"
          role="status">{notice}</p>
      )}

      <div className="p-4">
        {!files ? <TableSkeleton /> : files.length === 0 ? (
          <EmptyState title="Sin datasets" hint="Importa un CSV o descarga histórico desde el módulo Datos." />
        ) : (
          <div className="overflow-x-auto qf-scroll">
            <table className="w-full min-w-[720px] text-sm">
              <thead>
                <tr className="bg-surface text-left text-xs text-muted">
                  <th className="px-3 py-2 font-medium">Símbolo</th>
                  <th className="px-3 py-2 font-medium">Temporalidad</th>
                  <th className="px-3 py-2 font-medium">Origen</th>
                  <th className="px-3 py-2 font-medium">Barras</th>
                  <th className="px-3 py-2 font-medium">Tamaño</th>
                  <th className="px-3 py-2 font-medium">Estado</th>
                  <th className="px-3 py-2 font-medium">Acciones</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {files.map(df => (
                  <tr key={df.id} className="hover:bg-surface-raised">
                    <td className="px-3 py-2">{symName(df.symbolId)}</td>
                    <td className="px-3 py-2 font-mono">{df.timeframe}</td>
                    <td className="px-3 py-2">{kindLabel[df.storageKind]}</td>
                    <td className="px-3 py-2 font-mono">{fmtNum(df.nBars, 0)}</td>
                    <td className="px-3 py-2 font-mono">{fmtNum(df.compressedBytes / 1e6, 1)} MB</td>
                    <td className="px-3 py-2">
                      <Badge tone={df.status === 'ready' ? 'pass' : 'running'}
                        label={df.status === 'ready' ? 'LISTO' : '…'} />
                    </td>
                    <td className="px-3 py-2">
                      <Button size="sm" variant="ghost" loading={busy}
                        onClick={() => onExport(df.id)} aria-label={`Exportar ${symName(df.symbolId)} ${df.timeframe}`}>
                        <FileDown size={13} aria-hidden /> Exportar
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </Page>
  )
}

/* ───────────────────── S23 · Ajustes globales ────────────────────── */

export function S23Settings() {
  const [s, setS] = useState<AppSettings | null>(null)
  const [busy, setBusy] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => { api.getSettings().then(setS) }, [])

  async function save() {
    if (!s) return
    setBusy(true); setSaved(false)
    await api.updateSettings(s)
    setSaved(true); setBusy(false)
  }

  if (!s) return <Page title="Ajustes" icon={<Activity size={16} />}><TableSkeleton /></Page>

  const field = 'rounded-md border border-border-input bg-surface px-2 py-1.5 text-sm'

  return (
    <Page title="Ajustes" icon={<Activity size={16} />}
      actions={
        <Button variant="primary" loading={busy} onClick={save}>
          <Save size={14} aria-hidden /> Guardar ajustes
        </Button>
      }>
      <div className="mx-auto max-w-2xl space-y-4 p-4">
        {saved && (
          <p className="rounded-md border border-success/40 bg-success/10 p-3 text-sm text-success" role="status">
            Ajustes guardados.
          </p>
        )}

        <div className="rounded-md border border-border bg-surface p-4">
          <h2 className="mb-3 text-sm font-semibold text-muted">Datos y motor</h2>
          <label className="mb-1 block text-xs text-muted" htmlFor="st-datadir">
            Carpeta de datos locales
          </label>
          <input id="st-datadir" className={`mb-3 w-full ${field}`} value={s.dataDir}
            onChange={e => setS({ ...s, dataDir: e.target.value })} />
          <label className="mb-1 block text-xs text-muted" htmlFor="st-threads">
            Hilos del motor: <span className="font-mono text-text">{s.engineThreads}</span>
          </label>
          <input id="st-threads" type="range" min={1} max={16} value={s.engineThreads}
            onChange={e => setS({ ...s, engineThreads: +e.target.value })} className="w-full" />
        </div>

        <div className="rounded-md border border-border bg-surface p-4">
          <h2 className="mb-3 text-sm font-semibold text-muted">Red</h2>
          <label className="mb-1 block text-xs text-muted" htmlFor="st-proxy">
            Proxy (vacío = directo). Para descargas de datos detrás de cortafuegos.
          </label>
          <input id="st-proxy" className={`w-full ${field}`} value={s.proxy} placeholder="http://host:puerto"
            onChange={e => setS({ ...s, proxy: e.target.value })} />
        </div>

        <div className="rounded-md border border-border bg-surface p-4">
          <h2 className="mb-3 text-sm font-semibold text-muted">Apariencia y idioma</h2>
          <div className="mb-3 flex items-center justify-between gap-4">
            <label htmlFor="st-theme" className="text-sm">Tema</label>
            <select id="st-theme" className={selectCls} value={s.theme}
              onChange={e => setS({ ...s, theme: e.target.value as AppSettings['theme'] })}>
              <option value="dark">Oscuro</option>
              <option value="light">Claro (llega con el motor de temas)</option>
            </select>
          </div>
          <div className="mb-3 flex items-center justify-between gap-4">
            <label htmlFor="st-lang" className="text-sm">Idioma</label>
            <select id="st-lang" className={selectCls} value={s.language}
              onChange={e => setS({ ...s, language: e.target.value })}>
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </div>
          <label className="flex cursor-pointer items-center gap-2 text-sm">
            <input type="checkbox" checked={s.autosave} aria-label="Autoguardado de generaciones"
              onChange={e => setS({ ...s, autosave: e.target.checked })} />
            Autoguardar generaciones en curso
          </label>
        </div>

        <p className="text-xs text-muted">
          {api.isDesktop()
            ? 'Los ajustes se guardan de verdad en la base de datos local (fichero app_settings), junto a todo lo demas.'
            : 'En el navegador los ajustes se guardan solo en la sesion: la app de escritorio los escribe en la base de datos local.'}
        </p>
      </div>
    </Page>
  )
}

/* ───────────────────── S24 · Laboratorio ───────────────────────── */

type AnalyzerTab = 'resumen' | 'verdicts' | 'mc' | 'mm'

export function S24Analyzer() {
  const [rows, setRows] = useState<StrategyRec[] | null>(null)
  const [files, setFiles] = useState<string[]>([])
  const [tab, setTab] = useState<AnalyzerTab>('resumen')
  const [risk, setRisk] = useState(1.0)
  const [busy, setBusy] = useState(false)
  const [mc, setMc] = useState<PortfolioMcResult | null>(null)
  const fileRef = useRef<HTMLInputElement>(null)

  useEffect(() => { api.listBank({}).then(setRows) }, [])

  async function onImport(e: React.ChangeEvent<HTMLInputElement>) {
    const got = Array.from(e.target.files ?? []).map(f => f.name)
    if (got.length) {
      setFiles(prev => [...prev, ...got])
      setTab('resumen')
    }
    e.target.value = ''
  }

  async function runMc() {
    const ids = (rows ?? []).slice(0, 8).map(r => r.id)
    if (!ids.length) return
    setBusy(true); setMc(null)
    setMc(await api.runPortfolioMc(ids))
    setBusy(false)
  }

  const loaded = rows && rows.length > 0
  const tabs: Array<{ key: AnalyzerTab; label: string }> = [
    { key: 'resumen', label: 'Resumen' },
    { key: 'verdicts', label: 'Veredictos' },
    { key: 'mc', label: 'Monte Carlo de cartera' },
    { key: 'mm', label: 'Gestión de capital' },
  ]
  const winRate = rows?.length ? rows.reduce((a, r) => a + (r.lastRun?.metrics?.winRate ?? 0), 0) / rows.length : 0

  return (
    <Page title="Laboratorio" icon={<Boxes size={16} />}
      actions={
        <Button variant="ghost" onClick={() => fileRef.current?.click()}>
          <FileUp size={14} aria-hidden /> Importar resultados o CSV
        </Button>
      }>
      <input ref={fileRef} type="file" multiple className="hidden"
        aria-label="Ficheros de análisis a importar" onChange={onImport} data-testid="qa-file" />

      <div className="p-4">
        {!loaded || !rows?.length ? (
          <EmptyState title="Sin archivos de análisis"
            hint="Importa un archivo de resultados o usa el banco local: el panel analiza métricas, Monte Carlo de cartera y gestión de capital." />
        ) : (
          <div className="space-y-4">
            {/* tabs */}
            <div role="tablist" aria-label="Análisis" className="flex flex-wrap gap-2">
              {tabs.map(t => (
                <button key={t.key} role="tab" aria-selected={tab === t.key} type="button"
                  onClick={() => setTab(t.key)}
                  className={`rounded-md border px-3 py-1.5 text-sm ${
                    tab === t.key ? 'border-accent text-accent' : 'border-border text-muted hover:border-border-input'}`}>
                  {t.label}
                </button>
              ))}
            </div>

            {files.length > 0 && (
              <p className="text-xs text-muted" aria-label="Archivos importados">
                Archivos importados en esta sesión: {files.join(', ')}
              </p>
            )}

            {tab === 'resumen' && (
              <div className="space-y-3">
                <StatRow>
                  <StatCard label="Estrategias cargadas" value={rows.length} />
                  <StatCard label="Tasa de acierto media" value={winRate} unit="%" />
                  <StatCard label="Beneficio neto acumulado" value={rows.reduce((a, r) => a + (r.lastRun?.metrics?.netProfit ?? 0), 0)} unit="USD" />
                  <StatCard label="Con problemas detectados" value={rows.filter(r => r.problems.length).length} tone="fail" />
                </StatRow>
                <div className="overflow-x-auto qf-scroll">
                  <table className="w-full min-w-[640px] text-sm">
                    <thead>
                      <tr className="bg-surface text-left text-xs text-muted">
                        <th className="px-3 py-2 font-medium">Estrategia</th>
                        <th className="px-3 py-2 font-medium">Net</th>
                        <th className="px-3 py-2 font-medium">PF</th>
                        <th className="px-3 py-2 font-medium">Max DD</th>
                        <th className="px-3 py-2 font-medium">Problemas</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border">
                      {rows.slice(0, 20).map(r => (
                        <tr key={r.id} className="hover:bg-surface-raised">
                          <td className="px-3 py-2">
                            <Link className="text-accent hover:underline" to={`/strategy/${r.id}`}>{r.name}</Link>
                          </td>
                          <td className={`px-3 py-2 font-mono ${(r.lastRun?.metrics?.netProfit ?? 0) >= 0 ? 'text-success' : 'text-danger'}`}>
                            {fmtMoney(r.lastRun?.metrics?.netProfit)}
                          </td>
                          <td className="px-3 py-2 font-mono">{fmtNum(r.lastRun?.metrics?.profitFactor)}</td>
                          <td className="px-3 py-2 font-mono">{fmtNum(r.lastRun?.metrics?.maxDD, 1)}%</td>
                          <td className="px-3 py-2 text-xs">{r.problems.length ? r.problems.join('; ') : '—'}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {tab === 'verdicts' && (
              <VerdictCard
                strategyIds={(rows ?? []).map(r => r.id)}
                title="Veredictos de robustez por estrategia"
              />
            )}

            {tab === 'mc' && (
              <div className="max-w-2xl space-y-3">
                <Button variant="primary" loading={busy} onClick={runMc}>
                  <Play size={14} aria-hidden /> Correr Monte Carlo de cartera
                </Button>
                {!mc ? (
                  <EmptyState title="Sin simulación"
                    hint="500 simulaciones de la cartera: percentiles de equity final y probabilidad de rotura." />
                ) : (
                  <>
                    <StatRow>
                      <StatCard label="Simulaciones" value={mc.trials} />
                      <StatCard label="Equity p95 (ilusionista)" value={mc.p95} unit="USD" />
                      <StatCard label="Equity mediana" value={mc.p50} unit="USD" />
                      <StatCard label="Equity p5 (conservador)" value={mc.p5} unit="USD" tone="fail" />
                      <StatCard label="Prob. de rotura" value={mc.ruinProb} unit="%" mono
                        tone={mc.ruinProb < 0.02 ? 'pass' : 'fail'} />
                    </StatRow>
                    <EquityChart series={[10_000, mc.p5, mc.p50, mc.p95]} height={140}
                      label="Curva indicativa de percentiles" />
                  </>
                )}
              </div>
            )}

            {tab === 'mm' && (
              <div className="max-w-2xl space-y-3">
                <label className="block text-xs text-muted" htmlFor="qa-risk">
                  Riesgo por operación: <span className="font-mono text-text">{fmtNum(risk, 1)}%</span>
                </label>
                <input id="qa-risk" type="range" min={0.2} max={5} step={0.1} value={risk}
                  onChange={e => setRisk(+e.target.value)} className="w-full" />
                <StatRow>
                  <StatCard label="Tasa de acierto media usada" value={winRate} unit="%" />
                  <StatCard label="Ganancia esperada / op" value={fmtNum(risk * (winRate / 100) * 1.6, 2)} unit="%" />
                  <StatCard label="DD esperado con este riesgo" value={fmtNum(risk * 14, 1)} unit="%"
                    tone={risk * 14 > 30 ? 'fail' : 'neutral'} />
                </StatRow>
                <p className="text-xs text-muted">
                  Simulación equiponderada orientativa (banco local). Con el puente Rust, el
                  mm usa la distribución real de trades de cada estrategia de la cartera.
                </p>
              </div>
            )}
          </div>
        )}
      </div>
    </Page>
  )
}

