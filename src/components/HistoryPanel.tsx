import { useEffect, useState } from 'react'
import { Badge } from './Badge'
import { Button } from './Button'
import { EmptyState } from './EmptyState'
import { TableSkeleton } from './TableSkeleton'
import { api, type AnalysisKind, type AnalysisRun } from '../lib/api'
import { fmtNum } from '../lib/utils'

interface Props {
  /** Si se indica, solo se muestran las corridas de esa estrategia. */
  strategyId?: number
  /** Filtro por tipo de analisis. */
  kind?: string
  title?: string
  /** Muestra el selector de tipo (historial global). */
  allowFilter?: boolean
  className?: string
}

const tone = (v: AnalysisRun['verdict']): 'pass' | 'fail' | 'running' | 'pending' =>
  v === 'pass' ? 'pass' : v === 'fail' ? 'fail' : v === 'warn' ? 'running' : 'pending'

const etiqueta = (v: AnalysisRun['verdict']) =>
  v === 'pass' ? 'PASA' : v === 'fail' ? 'FALLA' : v === 'warn' ? 'ALERTA' : 'PENDIENTE'

/** Metricas que tienen sentido comparar entre corridas del mismo tipo. */
const interesting = ['netProfit', 'profitFactor', 'maxDD', 'nTrades', 'wfe', 'tasaMedia', 'impactoMax', 'mejora']

function fmtMetrica(k: string, v: number | string | null | undefined): string {
  if (v == null) return '—'
  if (typeof v === 'string') return v
  if (k === 'netProfit') return v.toLocaleString('es-ES', { maximumFractionDigits: 0 })
  if (k === 'maxDD') return `${fmtNum(v, 1)} %`
  return fmtNum(v, 2)
}

/**
 * Historial de analisis: cada retesteo, refinamiento, optimizacion, matriz WF,
 * Monte Carlo, SPP o perfil queda registrado. La columna de comparacion muestra
 * el delta frente a la corrida anterior del mismo tipo: es la manera de ver si
 * un cambio de regla ha mejorado o ha roto la estrategia.
 */
export function HistoryPanel({ strategyId, kind, title = 'Historial de analisis', allowFilter = false, className }: Props) {
  const [rows, setRows] = useState<AnalysisRun[]>([])
  const [kinds, setKinds] = useState<AnalysisKind[]>([])
  const [filtro, setFiltro] = useState<string>(kind ?? '')
  const [cargando, setCargando] = useState(true)

  useEffect(() => {
    let vivo = true
    setCargando(true)
    api.listAnalysisRuns(strategyId, kind, 200).then(r => {
      if (!vivo) return
      setRows(r.rows)
      setKinds(r.kinds)
      setCargando(false)
    })
    return () => { vivo = false }
  }, [strategyId, kind])

  async function borrar(id: number) {
    await api.deleteAnalysisRun(id)
    setRows(prev => prev.filter(r => r.id !== id))
  }

  const visibles = filtro ? rows.filter(r => r.kind === filtro) : rows

  return (
    <section className={`rounded-md border border-border bg-surface p-3 ${className ?? ''}`}>
      <header className="mb-2 flex flex-wrap items-center gap-2">
        <h2 className="text-sm font-semibold text-muted">{title}</h2>
        {allowFilter && kinds.length > 0 && (
          <select
            className="h-7 rounded-md border border-border-input bg-surface px-2 text-xs"
            aria-label="Filtrar historial por tipo"
            value={filtro}
            onChange={e => setFiltro(e.target.value)}>
            <option value="">Todos los tipos</option>
            {kinds.map(k => <option key={k.key} value={k.key}>{k.label}</option>)}
          </select>
        )}
        <span className="text-xs text-muted">{visibles.length} corridas</span>
      </header>

      {cargando ? <TableSkeleton rows={4} height={160} /> : visibles.length === 0 ? (
        <EmptyState
          title="Todavia no hay analisis registrados"
          hint="Cada retesteo, refinamiento, optimizacion, Monte Carlo, SPP o perfil que corras queda guardado aqui, con su veredicto y la comparacion con la corrida anterior."
        />
      ) : (
        <div className="overflow-x-auto qf-scroll">
          <table className="w-full min-w-[640px] text-sm">
            <thead>
              <tr className="bg-surface-raised text-left text-xs text-muted">
                <th className="px-2 py-1.5 font-medium">Cuando</th>
                {!strategyId && <th className="px-2 py-1.5 font-medium">Estrategia</th>}
                <th className="px-2 py-1.5 font-medium">Tipo</th>
                <th className="px-2 py-1.5 font-medium">Resultado</th>
                <th className="px-2 py-1.5 font-medium">Metricas</th>
                <th className="px-2 py-1.5 font-medium">Frente a la anterior</th>
                <th className="px-2 py-1.5" />
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {visibles.map(r => {
                const claves = Object.keys(r.metrics).filter(k => interesting.includes(k))
                const deltas = r.comparacion.filter(d => interesting.includes(d.metrica))
                return (
                  <tr key={r.id} className="hover:bg-surface-raised">
                    <td className="px-2 py-1.5 font-mono text-xs text-muted">
                      {new Date(r.createdAt * 1000).toLocaleString('es-ES', {
                        day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit',
                      })}
                    </td>
                    {!strategyId && (
                      <td className="px-2 py-1.5 text-xs">{r.nombre}</td>
                    )}
                    <td className="px-2 py-1.5 text-xs">{r.kindLabel}</td>
                    <td className="px-2 py-1.5">
                      <Badge tone={tone(r.verdict)} label={etiqueta(r.verdict)} />
                      {r.label && <span className="ml-2 text-xs text-muted">{r.label}</span>}
                    </td>
                    <td className="px-2 py-1.5 font-mono text-xs">
                      {claves.length === 0
                        ? <span className="text-muted">—</span>
                        : claves.map(k => (
                          <span key={k} className="mr-2">
                            <span className="text-muted">{k}</span> {fmtMetrica(k, r.metrics[k])}
                          </span>
                        ))}
                    </td>
                    <td className="px-2 py-1.5 font-mono text-xs">
                      {!r.hayPrevia
                        ? <span className="text-muted">primera corrida</span>
                        : deltas.length === 0
                          ? <span className="text-muted">sin metricas comparables</span>
                          : deltas.map(d => (
                            <span key={d.metrica} className="mr-2">
                              <span className="text-muted">{d.metrica}</span>{' '}
                              <span className={d.delta > 0 ? 'text-success' : d.delta < 0 ? 'text-danger' : 'text-muted'}>
                                {d.delta > 0 ? '+' : ''}{fmtNum(d.delta, 2)}
                              </span>
                            </span>
                          ))}
                    </td>
                    <td className="px-2 py-1.5 text-right">
                      <Button size="sm" variant="ghost" aria-label={`Borrar analisis ${r.id}`}
                        onClick={() => void borrar(r.id)}>&times;</Button>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
          <p className="mt-2 text-xs text-muted">
            «Frente a la anterior» compara con la corrida previa del mismo tipo y la misma
            estrategia: si cambiaste una regla, aqui se ve si has mejorado o la has roto.
          </p>
        </div>
      )}
    </section>
  )
}
