import { cn } from '../lib/utils'

type Tone = 'pass' | 'fail' | 'running' | 'pending'

const toneCls: Record<Tone, string> = {
  pass: 'bg-success/15 text-success',
  fail: 'bg-danger/15 text-danger',
  running: 'bg-warning/15 text-warning',
  pending: 'bg-surface-raised text-muted',
}

const toneText: Record<Tone, string> = {
  pass: 'PASSED', fail: 'FAILED', running: 'RUNNING', pending: 'PENDING',
}

export function Badge({ tone, label, className }: { tone: Tone; label?: string; className?: string }) {
  return (
    <span className={cn(
      'inline-flex items-center gap-1 rounded-pill px-2 py-0.5 text-xs font-semibold',
      toneCls[tone], className,
    )}>
      {tone === 'running' && <span aria-hidden className="h-2 w-2 animate-pulse rounded-full bg-warning" />}
      {label ?? toneText[tone]}
    </span>
  )
}
