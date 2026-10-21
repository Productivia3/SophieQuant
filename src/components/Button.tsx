import type { ButtonHTMLAttributes, ReactNode } from 'react'
import { cn } from '../lib/utils'

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger'
type Size = 'sm' | 'md' | 'lg'

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
  size?: Size
  loading?: boolean
  children: ReactNode
}

const variantCls: Record<Variant, string> = {
  primary: 'bg-accent text-on-accent hover:brightness-110 active:brightness-95',
  secondary: 'bg-surface-raised border border-border hover:border-border-input text-text',
  ghost: 'hover:bg-surface-raised text-text',
  danger: 'bg-danger/15 text-danger hover:bg-danger/25',
}
const sizeCls: Record<Size, string> = {
  sm: 'h-7 px-2.5 text-sm gap-1.5',
  md: 'h-8 px-3 text-sm gap-2',
  lg: 'h-10 px-4 text-base gap-2',
}

export function Button({ variant = 'secondary', size = 'md', loading, disabled, className, children, ...rest }: Props) {
  return (
    <button
      className={cn(
        'inline-flex items-center justify-center rounded-md font-medium transition-fast',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg',
        'disabled:opacity-40 disabled:pointer-events-none',
        variantCls[variant], sizeCls[size], className,
      )}
      disabled={disabled || loading}
      {...rest}
    >
      {loading && (
        <span aria-hidden className="mr-1 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
      )}
      {children}
    </button>
  )
}