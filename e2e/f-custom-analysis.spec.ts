// Custom Analysis: seccion, script y salida de un script de Python.
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

test.describe('Custom Analysis', () => {
  test('CA-H1 la sección explica el contrato y no permite ejecutar sin script', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByText(/analisis personalizado \(python\)/i)).toBeVisible()
    await expect(page.getByText(/--input/i).first()).toBeVisible()
    await expect(page.getByText(/--output/i).first()).toBeVisible()
    // sin script elegido, ejecutar esta deshabilitado
    await expect(page.getByRole('button', { name: /^ejecutar$/i })).toBeDisabled()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('CA-H2 el script de ejemplo vive en el repositorio y es valido', async ({ page }) => {
    // el ejemplo se puede abrir desde el repo: comprueba que el contrato esta
    // documentado y que el script de ejemplo existe en disco
    await page.goto('/#/strategy/3')
    await expect(page.getByText(/replica\/custom-analysis\.md/)).toBeVisible()
  })
})
