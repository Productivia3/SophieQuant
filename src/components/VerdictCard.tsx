import { useState } from 'react'
import { Badge } from './Badge'
import { Button } from './Button'
import { api, VERDICT_LABELS, type StrategyVerdicts } from '../lib/api'
import { estadoTone, estadoTexto } from '../lib/verdicts'

interface Props {
  strategyIds: number[]
  /** Titulo de la tarjeta; por defecto, "Veredictos de robustez". */
  title?: string
  /** Si es false, la tarjeta no muestra el boton de calcular (solo lectura). */
  canCompute?: boolean
  className?: string
}

const CHECKS = ['walkForward', 'monteCarlo', 'spp', 'perfil'] as const

/**
 * Tarjeta de veredictos: el resultado de las cuatro pruebas de robustez de una
 * estrategia, en un solo sitio. Responde a la complaint mas repetida del
 * original ("la validacion nativa parece una caja negra"): aqui se ve que
 * prueba se paso, cual no, y con que numero.
 */
export function VerdictCard({ strategyIds, title = 'Veredictos de robustez', canCompute = true, className }: Props) {
  const desktopAvailable = api.isDesktop()
  const [rows, setRows] = useState<StrategyVerdicts[]>([])
  const [busy, setBusy] = useState(false)
  const [loaded, setLoaded] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)

  async function load(compute: boolean) {
    if (!strategyIds.length) return
    setBusy(true)
    setMsg(null)
    try {
      const res = compute
        ? await api.computeVerdicts(strategyIds)
        : await api.listVerdicts(strategyIds)
      setRows(res)
      setLoaded(true)
      if (compute) setMsg(`Veredictos calculados con el motor real (${new Date().toLocaleTimeString('es-ES')}).`)
    } catch (e) {
      setMsg(`No se pudieron calcular: ${e instanceof Error ? e.message : String(e)}`)
    } finally {
      setBusy(false)
    }
  }

  return (
    <section className={`rounded-md border border-border bg-surface p-3 ${className ?? ''}`}>
      <header className="mb-2 flex items-center gap-2">
        <h2 className="text-sm font-semibold text-muted">{title}</h2>
        <span className="text-xs text-muted">
          OOS ≥ 30 trades · Walk-forward ≥ 50 % · Monte Carlo 6/6 · SPP todo PASS · meseta en parámetros activos
        </span>
        {canCompute && desktopAvailable && (
          <Button
            variant="primary"
            size="sm"
            loading={busy}
            className="ml-auto"
            onClick={() => void load(true)}
          >
            {loaded ? 'Recalcular' : 'Calcular veredictos'}
          </Button>
        )}
      </header>

      {msg && <p className="mb-2 text-xs text-muted">{msg}</p>}
      {!desktopAvailable && (
        <p className="mb-2 text-xs text-warning" role="status">
          El veredicto requiere la aplicación de escritorio; el navegador no calcula ni inventa resultados de robustez.
        </p>
      )}

      {!loaded && !busy && (
        <p className="py-3 text-sm text-muted">
          Todavía no hay veredictos calculados para estas estrategias. {desktopAvailable
            ? 'Pulsa «Calcular veredictos» para correr las cuatro pruebas con el motor real.'
            : 'Abre SophieQuant de escritorio para ejecutar las comprobaciones reales.'}
        </p>
      )}

      {rows.map(r => (
        <div key={r.strategyId} className="mb-3 rounded-md border border-border bg-surface-raised p-2 last:mb-0">
          <div className="mb-1 flex items-center gap-2">
            <span className="text-sm font-medium text-text">{r.nombre}</span>
            <Badge tone={estadoTone(r.overall)} label={estadoTexto(r.overall)} />
            {r.calculadoEn && (
              <span className="ml-auto text-xs text-muted">
                {new Date(r.calculadoEn * 1000).toLocaleString('es-ES')}
              </span>
            )}
          </div>
          <ul className="grid gap-1 md:grid-cols-2">
            {CHECKS.map(k => {
              const c = r.checks[k]
              return (
                <li key={k} className="flex items-center gap-2 text-xs">
                  <Badge tone={estadoTone(c.estado)} label={estadoTexto(c.estado)} />
                  <span className="w-44 shrink-0 text-muted">{VERDICT_LABELS[k]}</span>
                  <span className="truncate text-text">{c.detalle}</span>
                </li>
              )
            })}
          </ul>
        </div>
      ))}
    </section>
  )
}
