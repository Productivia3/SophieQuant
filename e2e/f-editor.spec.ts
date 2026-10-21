// Editor sin código: guardar de verdad y recalcular la corrida.
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

test.describe('Editor sin código', () => {
  test('E-H1 el editor ofrece solo las tres reglas que el motor ejecuta', async ({ page }) => {
    await page.goto('/#/strategy/3/edit')
    await expect(page.getByRole('heading', { name: /regla de la estrategia/i })).toBeVisible()
    await expect(page.getByRole('radio', { name: /cruce de medias/i })).toBeVisible()
    await expect(page.getByRole('radio', { name: /reversión rsi/i })).toBeVisible()
    await expect(page.getByRole('radio', { name: /ruptura con filtro de tendencia/i })).toBeVisible()
    // no se ofrecen bloques que el motor no ejecuta
    await expect(page.getByRole('radio', { name: /macd/i })).toHaveCount(0)
    await expect(page.getByRole('radio', { name: /stoch/i })).toHaveCount(0)
    await expect(page.getByRole('radio', { name: /cci/i })).toHaveCount(0)
  })

  test('E-H2 cambiar la regla y guardar recalcula la corrida', async ({ page }) => {
    await page.goto('/#/strategy/3/edit')
    await page.getByRole('radio', { name: /reversión rsi/i }).click()
    // el parametro propio de la regla (periodo RSI)
    await expect(page.getByLabel('Periodo RSI')).toBeVisible()
    await page.getByLabel('Periodo RSI').fill('9')
    await page.getByRole('button', { name: /guardar y recalcular/i }).click()
    await expect(page.getByText(/regla guardada y corrida recalculada/i)).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/beneficio neto/i).first()).toBeVisible()
    // la vista anterior y la nueva se comparan
    await expect(page.getByText(/antes:/i)).toBeVisible()
  })

  test('E-H3 el detalle ofrece el acceso al editor', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await page.getByRole('button', { name: /editar regla/i }).click()
    await expect(page).toHaveURL(/\/strategy\/3\/edit/)
    await expect(page.getByRole('heading', { name: /regla de la estrategia/i })).toBeVisible()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })
})
