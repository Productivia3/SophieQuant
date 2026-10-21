// Retesteo multi-mercado: la misma regla probada en varios mercados.
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

test.describe('Retesteo multi-mercado', () => {
  test('M-H1 se pueden elegir varios mercados y el retesteo corre', async ({ page }) => {
    await page.goto('/#/retester')

    // panel de mercados con una casilla por símbolo (acotado a su lista)
    const mercados = page.getByRole('list', { name: /mercados/i }).getByRole('checkbox')
    await expect(mercados.first()).toBeVisible()
    const cuantos = await mercados.count()
    expect(cuantos).toBeGreaterThan(1)

    // marcar el primero y el segundo mercado
    await mercados.nth(0).check()
    await mercados.nth(1).check()

    // marcar una estrategia del banco (acotada a su lista)
    const estrategias = page.getByRole('list', { name: /estrategias/i }).getByRole('checkbox')
    await estrategias.first().check()

    await page.getByRole('button', { name: /retesteo de 1/i }).click()
    await expect(page.getByRole('table')).toBeVisible({ timeout: 15_000 })

    // la tabla tiene columna por mercado y la columna de resumen
    await expect(page.getByRole('columnheader', { name: /sobrevive/i })).toBeVisible()
    await expect(page.getByRole('columnheader', { name: /origen/i })).toBeVisible()
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/sobrevive en \d+ de \d+|falla en \d+ de \d+|ningun mercado con datos/)
    expect(body).not.toContain('NaN')
  })

  test('M-H2 el panel distingue los mercados con datos de los que no', async ({ page }) => {
    await page.goto('/#/retester')
    // la semilla tiene simbolos con datos y simbolos sin ellos: el panel lo dice
    await expect(page.getByText(/^con datos$/).first()).toBeVisible()
    await expect(page.getByText(/^sin datos$/).first()).toBeVisible()
    await expect(page.getByText(/un mercado sin datos no cuenta como fallo/i)).toBeVisible()

    // y al retestear, un mercado sin datos no se cuenta como mercado con datos
    const mercados = page.getByLabel(/Retestear en /)
    await mercados.nth(0).check()
    await page.getByLabel(/Retestear /).first().check()
    await page.getByRole('button', { name: /retesteo de 1/i }).click()
    await expect(page.getByRole('table')).toBeVisible({ timeout: 15_000 })
    const body = await page.locator('body').textContent()
    expect(body).toMatch(/sobrevive en \d+ de \d+|falla en \d+ de \d+|ningun mercado con datos/)

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('M-E1 sin estrategias marcadas el botón está deshabilitado', async ({ page }) => {
    await page.goto('/#/retester')
    await expect(page.getByRole('button', { name: /retesteo de/i })).toBeDisabled()
  })
})
