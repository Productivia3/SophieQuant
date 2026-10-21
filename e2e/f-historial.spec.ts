// Historial de analisis: cada corrida queda registrada y se puede comparar
// con la anterior del mismo tipo.
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

test.describe('Historial de análisis', () => {
  test('H-H1 correr un análisis deja registro en el historial', async ({ page }) => {
    // el detalle de la estrategia
    await page.goto('/#/strategy/3')
    await expect(page.getByRole('heading', { name: /historial de analisis/i })).toBeVisible()

    // correr un SPP: es el análisis más rápido de la app
    await page.goto('/#/robustness')
    await page.getByRole('button', { name: /correr spp/i }).click()
    await expect(page.getByText(/entradas revertidas/i)).toBeVisible({ timeout: 15_000 })

    // el historial del Laboratorio lo recoge
    await page.goto('/#/analyzer')
    await page.getByRole('tab', { name: /historial/i }).click()
    await expect(page.getByRole('heading', { name: /historial de analisis/i })).toBeVisible()
    // la corrida de SPP queda registrada con su tipo y su veredicto
    const tabla = page.getByRole('table')
    await expect(tabla).toBeVisible()
    await expect(tabla.getByText(/permutaciones \(spp\)/i)).toBeVisible()
  })

  test('H-H2 el historial se puede filtrar por tipo', async ({ page }) => {
    await page.goto('/#/analyzer')
    await page.getByRole('tab', { name: /historial/i }).click()
    const filtro = page.getByLabel(/filtrar historial por tipo/i)
    await expect(filtro).toBeVisible()
    await filtro.selectOption('spp')
    await expect(filtro).toHaveValue('spp')
  })

  test('H-H3 el detalle de la estrategia muestra su historial', async ({ page }) => {
    await page.goto('/#/strategy/3')
    const panel = page.getByRole('heading', { name: /historial de analisis de esta estrategia/i })
    await expect(panel).toBeVisible()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })
})
