// Ponderacion de cartera: modelo, pesos y contribucion al riesgo.
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

test.describe('Ponderación de cartera', () => {
  test('C-H1 se elige modelo y se calculan los pesos', async ({ page }) => {
    await page.goto('/#/portfolio')
    await expect(page.getByRole('heading', { name: /ponderacion de la cartera/i })).toBeVisible()

    const modelo = page.getByLabel(/modelo de ponderacion/i)
    await expect(modelo).toBeVisible()
    await modelo.selectOption('risk_parity')

    await page.getByRole('button', { name: /analizar cartera/i }).click()
    const tabla = page.getByRole('table', { name: /ponderaciones por miembro/i })
    await expect(tabla).toBeVisible({ timeout: 15_000 })

    // la tabla muestra volatilidad, peso y contribucion al riesgo de cada miembro
    const cuerpo = await tabla.textContent()
    expect(cuerpo).toMatch(/Volatilidad/)
    expect(cuerpo).toMatch(/Peso/)
    expect(cuerpo).toMatch(/Riesgo/)
    // y el resumen de cartera va debajo
    const pagina = await page.locator('body').textContent()
    expect(pagina).toMatch(/Volatilidad de cartera/)
    expect(pagina).toMatch(/paridad de riesgo/i)
  })

  test('C-H2 el modelo personalizado permite editar los pesos', async ({ page }) => {
    await page.goto('/#/portfolio')
    await page.getByLabel(/modelo de ponderacion/i).selectOption('custom')
    await page.getByRole('button', { name: /analizar cartera/i }).click()
    await expect(page.getByRole('table', { name: /ponderaciones por miembro/i })).toBeVisible({ timeout: 15_000 })

    const inputs = page.getByLabel(/peso propio de /i)
    await expect(inputs.first()).toBeVisible()
    await inputs.first().fill('3')
    await expect(inputs.first()).toHaveValue('3')

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })
})
