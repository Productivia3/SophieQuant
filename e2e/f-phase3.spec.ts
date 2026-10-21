// Fase 3: S08 retest · S09 improver · S10/S11 optimizer · S14 SPP · S16 projects · S18 portfolio
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

test.describe('F10 retest', () => {
  test('F10-H1 select strategies, run retest, table with verdicts', async ({ page }) => {
    await page.goto('/#/retester')
    const boxes = page.getByRole('checkbox')
    await expect(boxes.first()).toBeVisible()
    await boxes.nth(0).check()
    await boxes.nth(1).check()
    await page.getByRole('button', { name: /retesteo de 2/i }).click()
    await expect(page.getByRole('table')).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/mercado del retesteo:/i)).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).not.toContain('NaN')
  })

  test('F10-E1 sin selección: botón deshabilitado', async ({ page }) => {
    await page.goto('/#/retester')
    await page.waitForTimeout(300)
    // ninguna marcada inicialmente → botón aún sin resultado
    await expect(page.getByRole('button', { name: /retesteo de/i })).toBeDisabled()
  })
})

test.describe('F03 improver', () => {
  test('F03-H1 genera variantes y compara con el original', async ({ page }) => {
    await page.goto('/#/improver')
    await page.getByRole('button', { name: /mejorar/i }).click()
    await expect(page.locator('tbody tr').first()).toBeVisible({ timeout: 15_000 })
    // siempre hay veredicto: "mejor variante" o "se conserva el original"
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/supera al original|se conserva el original/)
  })
})

test.describe('F04 optimizer + 3D', () => {
  test('F04-H1 corre optimizer y muestra superficie', async ({ page }) => {
    await page.goto('/#/optimizer')
    await page.getByRole('radio', { name: 'Walk-Forward' }).click()
    await page.getByRole('button', { name: /optimizar/i }).click()
    await expect(page.getByRole('img', { name: /superficie 3d/i })).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/mejor combo/i)).toBeVisible()
  })

  test('F04-H2 la ruta /strategy/1/opt3d renderiza superficie', async ({ page }) => {
    await page.goto('/#/strategy/1/opt3d')
    await expect(page.getByRole('img', { name: /superficie 3d/i })).toBeVisible({ timeout: 15_000 })
  })
})

test.describe('F05 SPP', () => {
  test('F05-H1 corre SPP y lista pruebas con impacto', async ({ page }) => {
    await page.goto('/#/robustness')
    await page.getByRole('button', { name: /correr spp/i }).click()
    await expect(page.getByText(/permutaciones que rompen|dependencias frágiles/i)).toBeVisible({ timeout: 15_000 })
    const items = page.locator('ul li')
    await expect(items.first()).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/entradas revertidas/i)
  })
})

test.describe('F06 custom projects', () => {
  test('F06-H1 workflow por pasos con estado por paso', async ({ page }) => {
    await page.goto('/#/projects')
    // añade un paso extra
    await page.getByRole('button', { name: /spp/i }).click()
    await page.getByRole('button', { name: /ejecutar workflow/i }).click()
    await expect(page.getByText(/workflow completado|workflow detenido/i)).toBeVisible({ timeout: 20_000 })
  })

    test('F06-E1 quitar y reordenar pasos funciona', async ({ page }) => {
    await page.goto('/#/projects')
    await page.getByRole('button', { name: /spp/i }).click()
    // quita el paso recién añadido (último)
    await page.getByRole('button', { name: /quitar paso 4/i }).click()
    await expect(page.getByRole('list', { name: 'Pasos del workflow' }).locator('li')).toHaveCount(3)
    // subir el paso 2
    await page.getByRole('button', { name: /subir paso 2/i }).click()
    const firstItem = page.getByRole('list', { name: 'Pasos del workflow' }).locator('li').first()
    await expect(firstItem).toContainText('Monte Carlo')
  })
})

test.describe('F07 portfolio', () => {
  test('F07-H1 matrizen de correlación + equity combinada', async ({ page }) => {
    await page.goto('/#/portfolio')
    await page.getByRole('button', { name: /analizar cartera \(4\)/i }).click()
    await expect(page.getByText(/correlación entre miembros/i)).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/equity combinada/i)).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).not.toContain('NaN')
  })

  test('F07-E1 con una sola estrategia el análisis bloquea', async ({ page }) => {
    await page.goto('/#/portfolio')
    await page.waitForTimeout(400)
    // deselecciona 3 dejando 1
    const boxes = page.getByRole('checkbox', { name: /añadir/i })
    await boxes.nth(0).uncheck()
    await boxes.nth(1).uncheck()
    await boxes.nth(2).uncheck()
    await expect(page.getByRole('button', { name: /analizar cartera \(1\)/i })).toBeDisabled()
  })
})

test.describe('a11y fase 3', () => {
  for (const r of ['/retester', '/improver', '/optimizer', '/robustness', '/projects', '/portfolio']) {
    test(`axe 0 violaciones en ${r}`, async ({ page }) => {
      await page.goto('/#' + r)
      await page.waitForTimeout(600)
      const a11y = await new AxeConstructor({ page }).analyze()
      expect(a11y.violations).toEqual([])
    })
  }
})
