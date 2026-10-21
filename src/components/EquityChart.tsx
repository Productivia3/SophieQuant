import { useMemo } from 'react'

interface Props {
  series: number[] | null | undefined
  height?: number
  label?: string
  className?: string
  /**
   * Fracción out-of-sample: 0.4 = el 40 % final de la serie es fuera de muestra.
   * Es el mismo corte que usa la comprobación walk-forward del motor.
   */
  oosFrom?: number | null
  /** Texto de la marca (por defecto, "Fuera de muestra"). */
  oosLabel?: string
}

/** Equity chart v1: SVG propio, ligero (el engine real vendrá con lightweight-charts si hace falta) */
export function EquityChart({ series, height = 220, label, className, oosFrom, oosLabel }: Props) {
  const { path, up, oosX } = useMemo(() => {
    if (!series || series.length < 2) return { path: null, up: true, oosX: null as number | null }
    const w = 1000, h = 100
    const min = Math.min(...series), max = Math.max(...series)
    const span = max - min || 1
    const pts = series.map((v, i) =>
      `${((i / (series.length - 1)) * w).toFixed(1)},${(h - ((v - min) / span) * h).toFixed(1)}`)
    const oos = oosFrom != null && oosFrom > 0 && oosFrom < 1
      ? w * (1 - oosFrom)
      : null
    return { path: `M${pts.join('L')}`, up: (series[series.length - 1] ?? 0) >= 0, oosX: oos }
  }, [series, oosFrom])

  if (!series || !path) {
    return (
      <div className={className} style={{ height }}
        aria-label="Gráfico de equity sin datos">
        <div className="flex h-full items-center justify-center rounded-md border border-border bg-surface text-sm text-muted">
          Sin resultados todavía
        </div>
      </div>
    )
  }

  return (
    <div className={className} style={{ height }}>
      <svg viewBox="0 0 1000 100" preserveAspectRatio="none" className="h-full w-full"
        role="img" aria-label={label ?? 'Gráfico de equity'} data-up={up}>
        {oosX != null && (
          <>
            {/* zona out-of-sample: se sombrea con el fondo de panel, no con color de marca */}
            <rect x={oosX} y={0} width={1000 - oosX} height={100}
              fill="var(--color-surface-raised)" opacity={0.85} />
            <line x1={oosX} y1={0} x2={oosX} y2={100}
              stroke="var(--color-border-input)" strokeWidth={1} strokeDasharray="4 3"
              vectorEffect="non-scaling-stroke" />
          </>
        )}
        <path d={path} fill="none"
          stroke={up ? 'var(--chart-up)' : 'var(--chart-down)'}
          strokeWidth={up ? 1.4 : 1.4} vectorEffect="non-scaling-stroke" />
      </svg>
      {oosX != null && (
        <div className="mt-1 flex items-center gap-2 text-xs text-muted">
          <span aria-hidden className="inline-block h-2 w-4 rounded-sm bg-surface-raised ring-1 ring-border" />
          {oosLabel ?? 'Fuera de muestra'} — a partir de aquí los parámetros ya no se eligieron con estos datos.
        </div>
      )}
    </div>
  )
}
