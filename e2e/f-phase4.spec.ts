// Fase 4: S15 opt profile · S19 portfolio results · S21 Gestor de datos ·
// S23 ajustes · S24 Laboratorio · sidebar móvil (BUG-005)
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

test.beforeEach(async ({ page }) => {
  const errors: string[] = []
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()) })
  page.on('response', res => { if (res.status() >= 500) errors.push(`${res.status()} en ${res.url()}`) })
  ;(page as unknown as { _qfErrors: string[] })._qfErrors = errors
})

test.afterEach(async ({ page }) => {
  const errors = (page as unknown as { _qfErrors: string[] })._qfErrors
  if (errors.length) throw new Error(`Errores en la página:\n${errors.join('\n')}`)
})

test.describe('S15 optimization profile', () => {
  test('S15-H1 corría el perfil, curva con zona estable y forma', async ({ page }) => {
    await page.goto('/#/profile')
    await page.getByRole('button', { name: /analizar perfil/i }).click()
    const svg = page.getByRole('img', { name: /perfil de optimización/i })
    await expect(svg).toBeVisible({ timeout: 15_000 })
    await expect(svg).toContainText('zona estable')
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/meseta|unimodal|puntual/)
    expect(body).toMatch(/zona estable|mejor valor/i)
  })

  test('S15-N1 enlace desde S14 llega al perfil', async ({ page }) => {
    await page.goto('/#/robustness')
    await page.getByRole('link', { name: /perfil de optimización/i }).click()
    await expect(page.getByRole('heading', { name: /perfil de optimización/i })).toBeVisible()
  })
})

test.describe('S19 resultados de cartera', () => {
  test('S19-H1 totals + equity + tabla por miembro', async ({ page }) => {
    await page.goto('/#/portfolio/results')
    await page.getByRole('button', { name: /re-analizar/i }).click()
    await expect(page.getByText(/net combinado/i)).toBeVisible({ timeout: 15_000 })
    await expect(page.getByRole('img', { name: /equity combinada/i })).toBeVisible()
    await expect(page.getByText(/aporte por miembro/i)).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).not.toContain('NaN')
  })

  test('S19-N1 enlace desde S18 → resultados', async ({ page }) => {
    await page.goto('/#/portfolio')
    await page.getByRole('link', { name: /resultados de la cartera/i }).click()
    await expect(page.getByRole('heading', { name: /cartera · resultados/i })).toBeVisible()
  })
})

test.describe('S21 Gestor de datos', () => {
  test('S21-H1 lista datasets y exporta seleccionando formato', async ({ page }) => {
    await page.goto('/#/data-manager')
    await expect(page.getByRole('table')).toBeVisible()
    await page.getByLabel('Formato').selectOption('qfd')
    await page.getByRole('button', { name: /exportar /i }).first().click()
    await expect(page.getByText(/exportado a /i)).toBeVisible({ timeout: 10_000 })
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/\.qfd/)
  })

  test('S21-H2 importa un CSV real vía file picker', async ({ page }) => {
    await page.goto('/#/data-manager')
    await page.setInputFiles('[data-testid="qdm-file"]', {
      name: 'GBPUSD_m5_import.csv',
      mimeType: 'text/csv',
      buffer: Buffer.from('time,open,high,low,close,volume\n1697040000,1.09,1.10,1.08,1.095,100'),
    })
    await expect(page.getByText(/importado gbpusd_m5_import/i)).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/barras listas/i)).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/CSV importado/)
  })
})

test.describe('S20 Datos', () => {
  test('S20-H1 conduce al gestor sin ofrecer descarga simulada', async ({ page }) => {
    await page.goto('/#/data')
    await expect(page.getByRole('button', { name: /descargar m1|fuentes de datos/i })).toHaveCount(0)
    await page.getByRole('button', { name: /importar histórico/i }).click()
    await expect(page).toHaveURL(/\/#\/data-manager/)
    await expect(page.getByRole('heading', { name: /gestor de datos/i })).toBeVisible()
  })

  test('S20-E1 no muestra datos ficticios si falla un comando Tauri', async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(window, '__TAURI__', {
        configurable: true,
        value: { core: { invoke: () => Promise.reject(new Error('Fallo de prueba del motor')) } },
      })
    })
    await page.goto('/#/data')
    await expect(page.getByRole('alert')).toContainText('Fallo de prueba del motor')
    await expect(page.getByText('EURUSD')).toHaveCount(0)
  })
})

test.describe('S23 ajustes', () => {
  test('S23-H1 muestra los metadatos de mercado que usa el motor', async ({ page }) => {
    await page.goto('/#/settings')
    await expect(page.getByRole('heading', { name: /metadatos de mercado por símbolo/i })).toBeVisible()
    await expect(page.getByLabel(/carpeta de datos|hilos del motor|autoguardado/i)).toHaveCount(0)
  })
})

test.describe('S24 Laboratorio', () => {
  test('S24-H1 resumen con métricas del bank tras "importar"', async ({ page }) => {
    await page.goto('/#/analyzer')
    // el bank local carga por defecto: el resumen es visible directamente
    await expect(page.getByRole('tab', { name: /resumen/i })).toBeVisible()
    await expect(page.getByText(/estrategias cargadas/i)).toBeVisible()
    await expect(page.getByText(/beneficio neto acumulado/i)).toBeVisible()
  })

  test('S24-H2 monte carlo de cartera da percentiles y ruina', async ({ page }) => {
    await page.goto('/#/analyzer')
    await page.getByRole('tab', { name: /monte carlo/i }).click()
    await page.getByRole('button', { name: /correr monte carlo/i }).click()
    await expect(page.getByText(/prob\. de rotura/i)).toBeVisible({ timeout: 15_000 })
    const body = await page.locator('body').textContent()
    expect(body).not.toContain('NaN')
  })

  test('S24-H3 tab de gestión de capital actualiza con el slider de riesgo', async ({ page }) => {
    await page.goto('/#/analyzer')
    await page.getByRole('tab', { name: /gestión de capital/i }).click()
    await expect(page.getByText(/dd esperado/i)).toBeVisible()
    const body1 = (await page.locator('body').textContent()) ?? ''
    await page.getByLabel(/riesgo por operación/i).fill('5')
    const body2 = (await page.locator('body').textContent()) ?? ''
    expect(body2).not.toBe(body1)
  })
})

test.describe('sidebar móvil (BUG-005)', () => {
  test.use({ viewport: { width: 480, height: 800 } })

  /** caja x del nav: espera implícita a que la transición del drawer termine */
  async function navX(page: import('@playwright/test').Page): Promise<number> {
    for (let i = 0; i < 8; i++) {
      const box = await page.locator('nav[aria-label="Módulos"]').boundingBox()
      // en drawer abierto x>=0; en cerrado x<0 — asentar: 2 lecturas consecutivas iguales
      const box2 = await page.locator('nav[aria-label="Módulos"]').boundingBox()
      if (box && box2 && box.x === box2.x) return box.x
      await page.waitForTimeout(120)
    }
    return (await page.locator('nav[aria-label="Módulos"]').boundingBox())?.x ?? 0
  }

  test('X-BUG005-1 hamburger abre drawer, enlace navega y cierra', async ({ page }) => {
    await page.goto('/#/')
    const burger = page.getByRole('button', { name: /abrir menú de módulos/i })
    await expect(burger).toBeVisible()
    await expect(burger).toHaveAttribute('aria-expanded', 'false')
    expect(await navX(page)).toBeLessThan(0)
    await burger.click()
    await expect(burger).toHaveAttribute('aria-expanded', 'true')
    expect(await navX(page)).toBeGreaterThanOrEqual(0)
    await expect(page.getByTestId('nav-overlay')).toBeVisible()
    await page.getByRole('link', { name: /banco/i }).click()
    await expect(page).toHaveURL(/\/\#\/bank/)
    // tras navegar el drawer se cierra
    await expect(burger).toHaveAttribute('aria-expanded', 'false')
    expect(await navX(page)).toBeLessThan(0)
    await expect(page.getByTestId('nav-overlay')).toBeHidden()
  })

  test('X-BUG005-2 overlay cierra el drawer sin navegar', async ({ page }) => {
    await page.goto('/#/')
    await page.getByRole('button', { name: /abrir menú de módulos/i }).click()
    expect(await navX(page)).toBeGreaterThanOrEqual(0)
    await page.getByRole('button', { name: /cerrar menú/i }).click()
    expect(await navX(page)).toBeLessThan(0)
    expect(page.url()).toBe('http://localhost:1420/#/')
  })
})

test.describe('a11y fase 4', () => {
  for (const r of ['/profile', '/portfolio/results', '/data-manager', '/settings', '/analyzer']) {
    test(`axe 0 violaciones en ${r}`, async ({ page }) => {
      await page.goto('/#' + r)
      await page.waitForTimeout(600)
      const a11y = await new AxeConstructor({ page }).analyze()
      expect(a11y.violations).toEqual([])
    })
  }

  test.describe('a11y drawer móvil', () => {
    test.use({ viewport: { width: 480, height: 800 } })
    test('axe 0 violaciones en móvil (drawer abierto)', async ({ page }) => {
      await page.goto('/#/')
      await page.getByRole('button', { name: /abrir menú de módulos/i }).click()
      await page.waitForTimeout(400)
      const a11y = await new AxeConstructor({ page }).analyze()
      expect(a11y.violations).toEqual([])
    })
  })
})
