export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ')
}

export function fmtMoney(n: number | null | undefined): string {
  if (n == null) return '—'
  const sign = n < 0 ? '-' : ''
  const abs = Math.abs(n)
  if (abs >= 1e9) return `${sign}$${(abs / 1e9).toFixed(2)}B`
  if (abs >= 1e6) return `${sign}$${(abs / 1e6).toFixed(2)}M`
  return `${sign}$${abs.toLocaleString('en-US', { maximumFractionDigits: 0 })}`
}

export function fmtNum(n: number | null | undefined, d = 2): string {
  if (n == null) return '—'
  return n.toLocaleString('en-US', { minimumFractionDigits: d, maximumFractionDigits: d })
}

export function fmtDate(ts: number | null | undefined): string {
  if (!ts) return '—'
  const timestampMs = Math.abs(ts) < 100_000_000_000 ? ts * 1000 : ts
  return new Date(timestampMs).toLocaleDateString('es-ES', { day: '2-digit', month: 'short', year: 'numeric' })
}

export function fmtBytes(b: number): string {
  const units = ['B', 'KB', 'MB', 'GB']
  let u = 0
  let v = b
  while (v >= 1024 && u < units.length - 1) { v /= 1024; u++ }
  return `${v.toFixed(u > 1 ? 0 : 1)} ${units[u]}`
}