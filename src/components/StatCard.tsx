import type { ReactNode } from 'react'
import { cn, fmtNum } from '../lib/utils'

interface Props {
  label: string
  value: number | string
  unit?: string
  delta?: number             // vs baseline; positivo = mejor (se pinta success)
  tone?: 'neutral' | 'pass' | 'fail'
  mono?: boolean
  className?: string
  /**
   * Explicacion de como se calcula la metrica. Se muestra al pasar el raton y
   * como texto para lectores de pantalla: la transparencia de las formulas es
   * una de las cosas que mas se echa de menos en las herramientas de la categoria.
   */
  hint?: string
}

export function StatCard({ label, value, unit, delta, tone = 'neutral', mono = true, className, hint }: Props) {
  const isNum = typeof value === 'number'
  return (
    <div className={cn(
      'rounded-md bg-surface-raised border border-border px-3 py-2 min-w-[104px]',
      className,
    )} title={hint}>
      <div className="text-xs text-muted font-medium">
        {label}
        {hint && <span aria-hidden className="ml-1 cursor-help text-muted/70">?</span>}
      </div>
      <div className={cn(
        'text-lg mt-0.5',
        mono && 'font-mono',
        tone === 'pass' && 'text-success',
        tone === 'fail' && 'text-danger',
      )}>
        {isNum ? fmtNum(value as number) : String(value)}
        {unit && <span className="text-xs text-muted ml-1">{unit}</span>}
        {delta != null && delta !== 0 && (
          <span className={cn('text-xs ml-2', delta > 0 ? 'text-success' : 'text-danger')}>
            {delta > 0 ? '▲' : '▼'} {fmtNum(Math.abs(delta), 0)}%
          </span>
        )}
      </div>
      {hint && <span className="sr-only">{hint}</span>}
    </div>
  )
}

export function StatRow({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn('flex flex-wrap gap-2', className)}>{children}</div>
}
