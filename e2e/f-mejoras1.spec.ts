// Mejoras del bloque 1: overlays en el chart, sugerencia de cartera y
// perfil de riesgo (MAE/MFE + capital diario).
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

test.beforeEach(async ({ page }) => {
  const errors: string[] = []
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()) })
  page.on('pageerror', e => errors.push(String(e)))
  ;(page as unknown as { _qfErrors: string[] })._qfErrors = errors
})

test.afterEach(async ({ page }) => {
  const errors = (page as unknown as { _qfErrors: string[] })._qfErrors
  if (errors.length) throw new Error(`Errores en la página:\n${errors.join('\n')}`)
})

test.describe('Bloque de mejoras 1', () => {
  test('M1-H1 el chart dibuja las medias y se pueden activar y desactivar', async ({ page }) => {
    await page.goto('/#/strategy/3/chart')
    await expect(page.locator('svg[role="img"]').first()).toBeVisible({ timeout: 10_000 })
    // las medias aparecen con su interruptor
    const ema20 = page.getByLabel(/mostrar ema 20/i)
    await expect(ema20).toBeVisible()
    await expect(page.getByLabel(/mostrar ema 50/i)).toBeVisible()
    // y se pueden apagar
    await ema20.uncheck()
    await expect(ema20).not.toBeChecked()
  })

  test('M1-H2 la cartera sugiere miembros y explica los descartes', async ({ page }) => {
    await page.goto('/#/portfolio')
    // esperar a que la lista de estrategias este cargada
    await expect(page.getByLabel(/añadir .* a la cartera/i).first()).toBeVisible({ timeout: 15_000 })
    await page.getByRole('button', { name: /sugerir cartera/i }).click()
    await expect(page.getByRole('heading', { name: /sugerencia de cartera/i })).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/se descartan y por que/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /usar esta cartera/i })).toBeVisible()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('M1-H3 el detalle ofrece el perfil de riesgo con MAE, MFE y capital diario', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByRole('heading', { name: /riesgo real/i })).toBeVisible()
    await expect(page.getByText(/MAE es lo peor que se puso la posicion/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /calcular perfil de riesgo/i })).toBeVisible()
  })
})
