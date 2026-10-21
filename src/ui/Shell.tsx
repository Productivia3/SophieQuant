import { NavLink, Route, Routes, useNavigate } from 'react-router-dom'
import {
  LayoutDashboard, Database, Wand2, Layers, LibraryBig, RefreshCw,
  ArrowUpNarrowWide as ImproveIcon, SlidersHorizontal, Columns3, Workflow,
  Boxes, BarChart3, Import, Settings, FlaskConical, Activity,
  Download, Repeat, FileDown, GitCompareArrows, Menu, X,
} from 'lucide-react'
import { useState } from 'react'
import { api } from '../lib/api'

const items = [
  { to: '/', label: 'Proyecto', icon: LayoutDashboard },
  { to: '/build/new', label: 'Constructor', icon: Wand2 },
  { to: '/bank', label: 'Banco de estrategias', icon: LibraryBig },
  { to: '/retester', label: 'Retesteo', icon: RefreshCw },
  { to: '/improver', label: 'Refinador', icon: ImproveIcon },
  { to: '/optimizer', label: 'Optimizador', icon: SlidersHorizontal },
  { to: '/wfmatrix', label: 'Matriz WF', icon: Columns3 },
  { to: '/montecarlo', label: 'Monte Carlo', icon: Repeat },
  { to: '/robustness', label: 'Robustez (SPP/Perf)', icon: Activity },
  { to: '/projects', label: 'Proyectos', icon: Workflow },
  { to: '/portfolio', label: 'Cartera', icon: Boxes },
  { to: '/analyzer', label: 'Laboratorio', icon: BarChart3 },
  { to: '/data', label: 'Datos', icon: Database },
  { to: '/data-manager', label: 'Gestor de datos', icon: Import },
  { to: '/export', label: 'Exportar código', icon: FileDown },
  { to: '/diff', label: 'Paridad', icon: GitCompareArrows },
  { to: '/design', label: 'Sistema de diseño', icon: Download },
  { to: '/settings', label: 'Ajustes', icon: Settings },
]

export function Shell() {
  const navigate = useNavigate()
  const [navOpen, setNavOpen] = useState(false)
  const closeNav = () => setNavOpen(false)
  return (
    <div className="flex h-full">
      {/* drawer móvil: overlay + nav deslizante <760px; fijo desde 760px (BUG-005) */}
      {navOpen && (
        <button type="button" aria-label="Cerrar menú" onClick={closeNav}
          className="fixed inset-0 z-30 bg-black/50 md:hidden" data-testid="nav-overlay" />
      )}
      <button type="button" onClick={() => setNavOpen(true)}
        aria-expanded={navOpen} aria-controls="side-nav" aria-label="Abrir menú de módulos"
        className="fixed left-2 top-1.5 z-50 rounded-md border border-border bg-surface p-1.5 md:hidden"
        data-testid="nav-burger">
        {navOpen ? <X size={16} aria-hidden /> : <Menu size={16} aria-hidden />}
      </button>

      <nav id="side-nav" aria-label="Módulos"
        className={`flex w-[224px] shrink-0 flex-col border-r border-border bg-surface qf-scroll
          max-md:fixed max-md:inset-y-0 max-md:left-0 max-md:z-40 max-md:transition-transform
          ${navOpen ? 'max-md:translate-x-0' : 'max-md:-translate-x-full'}`}
      >
        <div className="flex h-[44px] items-center gap-2 border-b border-border px-3">
          <FlaskConical aria-hidden className="text-accent" size={18} />
          <span className="text-base font-semibold tracking-tight">SophieQuant</span>
          <span className="ml-auto rounded-pill bg-surface-raised px-1.5 text-xs text-muted">v0.1</span>
        </div>
        <ul className="flex-1 p-2">
          {items.map(({ to, label, icon: Icon }) => (
            <li key={to}>
              <NavLink
                to={to}
                onClick={closeNav}
                className={({ isActive }) =>
                  `flex items-center gap-2.5 rounded-md px-2 py-1.5 text-sm transition-fast ${
                    isActive
                      ? 'bg-surface-raised text-text shadow-[inset_2px_0_0_var(--color-accent)]'
                      : 'text-muted hover:bg-surface-raised hover:text-text'
                  }`
                }
              >
                <Icon aria-hidden size={16} />
                {label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>

      <div className="flex min-w-0 flex-1 flex-col">
        <main className="min-h-0 flex-1">
          <Routes>
            <Route path="/" element={<S01Dashboard />} />
            <Route path="/build/new" element={<S02BuildConfig />} />
            <Route path="/build/run" element={<S03BuildRun />} />
            <Route path="/build/run/:buildId" element={<S03BuildRun />} />
            <Route path="/bank" element={<S04Bank />} />
            <Route path="/strategy/:id" element={<S05Strategy />} />
            <Route path="/strategy/:id/chart" element={<S06Chart />} />
            <Route path="/strategy/:id/edit" element={<S07Editor />} />
            <Route path="/retester" element={<S08Retester />} />
            <Route path="/improver" element={<S09Improver />} />
            <Route path="/optimizer" element={<S10Optimizer />} />
            <Route path="/strategy/:id/opt3d" element={<S11Opt3D />} />
            <Route path="/wfmatrix" element={<S12WfMatrix />} />
            <Route path="/montecarlo" element={<S13MonteCarlo />} />
            <Route path="/robustness" element={<S14Robustness />} />
            <Route path="/profile" element={<S15OptProfile />} />
            <Route path="/projects" element={<S16Projects />} />
            <Route path="/portfolio" element={<S18Portfolio />} />
            <Route path="/portfolio/results" element={<S19PortfolioResults />} />
            <Route path="/data" element={<S20Data />} />
            <Route path="/data-manager" element={<S21DataManager />} />
            <Route path="/export" element={<S22Export />} />
            <Route path="/design" element={<DesignPage />} />
            <Route path="/diff" element={<S25Diff />} />
            <Route path="/settings" element={<S23Settings />} />
            <Route path="/analyzer" element={<S24Analyzer />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </main>

        <footer role="contentinfo"
          className="flex h-6 shrink-0 items-center gap-4 border-t border-border bg-surface px-3 text-xs text-muted">
          <span>SophieQuant 0.1 · datos locales</span>
          <span className="ml-auto flex items-center gap-1">
            <span aria-hidden className="h-1.5 w-1.5 rounded-full bg-success" />
            <span>{api.isDesktop() ? 'núcleo Rust conectado' : 'núcleo simulado (navegador)'}</span>
          </span>
          <button onClick={() => navigate('/design')} className="hover:text-text">design system</button>
        </footer>
      </div>
    </div>
  )
}

/* ── páginas stub (se rellenan una a una en los siguientes pasos) ── */
import { S01Dashboard, S02BuildConfig, S03BuildRun, S04Bank, S05Strategy, S06Chart,
  S07Editor, S08Retester, S09Improver, S10Optimizer, S11Opt3D, S12WfMatrix,
  S13MonteCarlo, S14Robustness, S15OptProfile, S16Projects, S18Portfolio,
  S19PortfolioResults, S20Data,
  S21DataManager, S22Export, S23Settings, S24Analyzer, S25Diff, DesignPage } from '../pages'
import { EmptyState } from '../components/EmptyState'
export function NotFound() {
  return (
    <EmptyState
      title="Página no encontrada"
      hint="La ruta que buscas no existe en esta versión."
    />
  )
}