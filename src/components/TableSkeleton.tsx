export function TableSkeleton({ rows = 8, height = 34 }: { rows?: number; height?: number }) {
  return (
    <div className="space-y-2 p-4" aria-label="Cargando">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="animate-pulse rounded-md bg-surface" style={{ height, opacity: 1 - i * 0.1 }} />
      ))}
    </div>
  )
}
