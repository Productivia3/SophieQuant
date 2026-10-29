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
  test('V-H1 la tarjeta del detalle muestra los cuatro checks', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByRole('heading', { name: /veredictos de robustez/i })).toBeVisible()
    await page.getByRole('button', { name: /calcular veredictos/i }).click()
    await expect(page.getByText(/walk-forward/i).first()).toBeVisible({ timeout: 15_000 })
    await expect(page.getByText(/monte carlo/i).first()).toBeVisible()
    await expect(page.getByText(/permutaciones \(spp\)/i).first()).toBeVisible()
    await expect(page.getByText(/perfil de optimizaci/i).first()).toBeVisible()
    // los umbrales se muestran: el veredicto nunca es una caja negra
    await expect(page.getByText(/umbral 50 %/i).first()).toBeVisible()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('V-H2 el banco muestra la columna de robustez y calcula veredictos', async ({ page }) => {
    await page.goto('/#/bank')
    await expect(page.getByRole('columnheader', { name: /robustez/i })).toBeVisible()
    await page.getByRole('button', { name: /calcular veredictos/i }).click()
    await expect(page.getByText(/veredictos calculados para/i)).toBeVisible({ timeout: 20_000 })
    // cada fila muestra un estado (PASSED / FAILED / AVISO / PENDING)
    await expect(page.getByText(/passed|failed|aviso|pending/i).first()).toBeVisible()
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
