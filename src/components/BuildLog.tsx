import { useEffect, useRef, useState } from 'react'

interface Props {
  lines: Array<{ level: 'info' | 'warn' | 'error'; message: string; ts: number }>
  running?: boolean
  className?: string
}

/** Log en vivo del dock inferior */
export function BuildLog({ lines, running, className }: Props) {
  const ref = useRef<HTMLDivElement>(null)
  const [follow, setFollow] = useState(true)

  useEffect(() => {
    if (follow && ref.current) ref.current.scrollTop = ref.current.scrollHeight
  }, [lines.length, follow])

  return (
    <div className={className}>
      <div className="flex items-center justify-between border-b border-border px-2 py-1">
        <span className="text-xs font-semibold text-muted">
          Registro{running ? ' · en ejecución' : ''}
        </span>
        <label className="flex items-center gap-1 text-xs text-muted">
          <input type="checkbox" checked={follow} onChange={e => setFollow(e.target.checked)} />
          seguir
        </label>
      </div>
      <div ref={ref} className="h-full overflow-auto qf-scroll font-mono text-xs leading-5" aria-live="polite">
        {lines.length === 0
          ? <p className="p-2 text-muted"> Sin eventos todavía. </p>
          : lines.map((l, i) => (
            <div key={i} className="px-2">
              <span className={l.level === 'error' ? 'text-danger' : l.level === 'warn' ? 'text-warning' : 'text-muted'}>
                [{new Date(l.ts).toLocaleTimeString('es-ES')}] {l.message}
              </span>
            </div>
          ))}
      </div>
    </div>
  )
}
