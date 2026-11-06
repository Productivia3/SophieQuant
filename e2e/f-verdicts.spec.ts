// Veredictos de robustez consolidados + marca out-of-sample en el grafico.
// Bloque 1 y 2 de la lista final de replica-diff.
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

test.describe('Veredictos de robustez', () => {
  test('V-H1 el navegador no presenta ni calcula veredictos simulados', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByRole('heading', { name: /veredictos de robustez/i })).toBeVisible()
    await expect(page.getByRole('status')).toContainText(/requiere la aplicación de escritorio/i)
    await expect(page.getByRole('button', { name: /calcular veredictos/i })).toHaveCount(0)
    await expect(page.getByText(/sin dependencias frágiles detectadas/i)).toHaveCount(0)

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('V-H2 el banco no permite calcular robustez simulada en el navegador', async ({ page }) => {
    await page.goto('/#/bank')
    await expect(page.getByRole('columnheader', { name: /robustez/i })).toBeVisible()
    await expect(page.getByRole('button', { name: /calcular veredictos/i })).toBeDisabled()
    await expect(page.getByRole('status')).toContainText(/no calcula ni inventa resultados de robustez/i)
  })

  test('V-H3 el Laboratorio tiene la pestaña de veredictos', async ({ page }) => {
    await page.goto('/#/analyzer')
    await page.getByRole('tab', { name: /veredictos/i }).click()
    await expect(page.getByRole('heading', { name: /veredictos de robustez por estrategia/i })).toBeVisible()
  })
})

test.describe('Zona out-of-sample', () => {
  test('Z-H1 la curva de equity marca la zona fuera de muestra', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByText(/fuera de muestra/i).first()).toBeVisible()
    await expect(page.getByText(/los parámetros ya no se eligieron/i).first()).toBeVisible()
  })

  test('Z-H2 el gráfico de velas también la marca', async ({ page }) => {
    await page.goto('/#/strategy/3/chart')
    await expect(page.locator('svg[role="img"]').first()).toBeVisible({ timeout: 10_000 })
    await expect(page.getByText(/zona sombreada es fuera de muestra/i)).toBeVisible()
  })

  test('Z-H3 la matriz no afirma una muestra suficiente con datos fake', async ({ page }) => {
    await page.goto('/#/wfmatrix')
    await page.getByRole('button', { name: /correr matriz/i }).click()
    await expect(page.getByText(/poca data OOS: <30 trades o <1 año/i)).toBeVisible({ timeout: 10_000 })
  })
})
