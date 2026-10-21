import type { ReactNode } from 'react'
import { cn } from '../lib/utils'

interface Props {
  title: string
  icon?: ReactNode
  actions?: ReactNode
  children: ReactNode
  className?: string
  bodyClassName?: string
}

/** Toolbar de módulo (44px) + cuerpo scrolleable con el tema de los dockables */
export function Page({ title, icon, actions, children, className, bodyClassName }: Props) {
  return (
    <section className={cn('flex h-full flex-col min-w-0', className)}>
      <header className="flex h-[44px] shrink-0 items-center gap-3 border-b border-border bg-bg px-4">
        <div className="flex items-center gap-2 text-text">
          {icon}
          <h1 className="text-lg font-semibold">{title}</h1>
        </div>
        <div className="flex-1" />
        {actions}
      </header>
      <div className={cn('flex-1 min-h-0 overflow-auto qf-scroll', bodyClassName)}>
        {children}
      </div>
    </section>
  )
}
