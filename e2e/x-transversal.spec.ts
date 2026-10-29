// X â€” transversal: navegaciÃ³n completa sin errores de consola ni 5xx + axe
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

const routes = [
  { path: '/', name: 'Dashboard S01' },
  { path: '/build/new', name: 'Constructor S02' },
  { path: '/bank', name: 'Banco de estrategias S04' },
  { path: '/strategy/1', name: 'Detalle S05' },
  { path: '/strategy/1/chart', name: 'Chart S06' },
  { path: '/retester', name: 'Retesteo S08' },
  { path: '/improver', name: 'Refinador S09' },
  { path: '/optimizer', name: 'Optimizador S10' },
  { path: '/strategy/1/opt3d', name: '3D S11' },
  { path: '/wfmatrix', name: 'Matriz WF S12' },
  { path: '/montecarlo', name: 'Monte Carlo S13' },
  { path: '/robustness', name: 'SPP S14' },
  { path: '/profile', name: 'OptProfile S15' },
  { path: '/portfolio/results', name: 'CarteraRes S19' },
  { path: '/projects', name: 'Proyectos S16' },
  { path: '/portfolio', name: 'Cartera S18' },
  { path: '/data', name: 'Datos S20' },
  { path: '/data-manager', name: 'Data Manager S21' },
  { path: '/export', name: 'Export S22' },
  { path: '/settings', name: 'Settings S23' },
  { path: '/analyzer', name: 'Analyzer S24' },
  { path: '/design', name: 'Design' },
]

test.beforeEach(async ({ page }) => {
  const errors: string[] = []
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()) })
  page.on('response', res => { if (res.status() >= 500) errors.push(`${res.status()} en ${res.url()}`) })
  ;(page as unknown as { _qfErrors: string[] })._qfErrors = errors
})

test.afterEach(async ({ page }) => {
  const errors = (page as unknown as { _qfErrors: string[] })._qfErrors
  if (errors.length) throw new Error(`Errores en la pÃ¡gina:\n${errors.join('\n')}`)
})

test.describe('X-H1 navegaciÃ³n completa', () => {
  for (const r of routes) {
    test(`ruta ${r.path} renderiza sin errores`, async ({ page }) => {
      const resp = await page.goto('/#' + r.path)
      expect(resp?.status()).toBeLessThan(400)
      await expect(page.locator('body')).toBeVisible()
      // ningÃºn texto de crash de React
      await expect(page.getByText('Something went wrong')).toHaveCount(0)
    })
  }
})

test.describe('X-H2 accesibilidad (axe) en pantallas reales', () => {
  const axeRoutes = ['/', '/build/new', '/bank', '/strategy/1', '/data', '/export']
  for (const r of axeRoutes) {
    test(`axe 0 violaciones en ${r}`, async ({ page }) => {
      await page.goto('/#' + r)
      await page.waitForTimeout(600)
      const a11y = await new AxeConstructor({ page }).analyze()
      expect(a11y.violations).toEqual([])
    })
  }
})

test.describe('X-otros', () => {
  test('X-E2 teclado: el formulario de generación es alcanzable y editable sin ratón', async ({ page }) => {
    await page.goto('/#/build/new')
    const target = page.getByRole('spinbutton', { name: 'Estrategias objetivo' })
    let focused = false
    for (let i = 0; i < 80 && !focused; i++) {
      await page.keyboard.press('Tab')
      focused = await target.evaluate(element => document.activeElement === element)
    }

    expect(focused).toBe(true)
    const visibleFocus = await target.evaluate(element => {
      const style = getComputedStyle(element)
      return style.outlineStyle !== 'none' && style.outlineColor !== 'rgba(0, 0, 0, 0)'
    })
    expect(visibleFocus).toBe(true)
    await page.keyboard.press('ArrowUp')
    await expect(target).toHaveValue('101')
  })

  test('ruta desconocida muestra 404 propio y no crashea', async ({ page }) => {
    await page.goto('/#/no-existe')
    await expect(page.getByRole('heading', { name: /404|no encontrad/i })).toBeVisible()
  })

  test('X-H1b mobile 420px: sin crash ni overflow (BUG-005 cerrado con drawer)', async ({ page }) => {
    await page.setViewportSize({ width: 420, height: 800 })
    await page.goto('/#/bank')
    // sin crash y el body recorta el overflow (overflow-x hidden)
    await expect(page.locator('body')).toBeVisible()
    const overflow = await page.evaluate(() => getComputedStyle(document.body).overflowX)
    expect(overflow).toBe('hidden')
    // BUG-005 FIX: el nav es drawer fijo desplazado fuera de pantalla -
    // la pagina ya no se desborda horizontalmente en movil.
    const scrollW = await page.evaluate(() => document.documentElement.scrollWidth)
    expect(scrollW).toBeLessThanOrEqual(420)
  })
})
