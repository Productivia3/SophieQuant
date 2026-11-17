// Verificacion del export y diagnostico de calidad del import.
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

test.beforeEach(async ({ page }) => {
  const errors: string[] = []
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()) })
  page.on('pageerror', e => errors.push(String(e)))
  page.on('response', res => { if (res.status() >= 500) errors.push(`${res.status()} en ${res.url()}`) })
  ;(page as unknown as { _qfErrors: string[] })._qfErrors = errors
})

test.afterEach(async ({ page }) => {
  const errors = (page as unknown as { _qfErrors: string[] })._qfErrors
  if (errors.length) throw new Error(`Errores en la página:\n${errors.join('\n')}`)
})

test.describe('Verificación del export', () => {
  test('X-H1 generar el EA muestra los supuestos y el checklist', async ({ page }) => {
    await page.goto('/#/export')
    await page.locator('li button').first().click()
    await page.getByRole('button', { name: /generar código/i }).click()
    await expect(page.locator('pre')).toBeVisible({ timeout: 10_000 })

    // el panel de verificacion aparece con los supuestos y el checklist
    await expect(page.getByText(/verificacion antes de fiarte del ea/i)).toBeVisible()
    await expect(page.getByText(/lo que el backtest ha asumido/i)).toBeVisible()
    await expect(page.getByText(/checklist en metatrader/i)).toBeVisible()
    await expect(page.getByText(/diagnostico de intervalos no conoce las sesiones/i)).toBeVisible()
    // los supuestos clave estan
    const cuerpo = await page.locator('body').textContent()
    expect(cuerpo).toMatch(/BARRAS, no de ticks/)
    expect(cuerpo).toMatch(/largo/)
    expect(cuerpo).toMatch(/documento completo/i)

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('X-H2 el codigo se copia y guarda manualmente, sin prometer guardado nativo', async ({ page }) => {
    await page.goto('/#/export')
    await page.locator('li button').first().click()
    await page.getByRole('button', { name: /generar código/i }).click()
    await expect(page.locator('pre')).toBeVisible({ timeout: 10_000 })
    await expect(page.getByRole('button', { name: /guardar fichero/i })).toHaveCount(0)
    await expect(page.getByText(/copia el código y guárdalo como archivo/i)).toBeVisible()
  })

})
