// Constructor: filtros por cualquier metrica y seleccion por robustez.
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

test.describe('Filtros y selección por robustez', () => {
  test('F-H1 los filtros se pueden poner sobre cualquier métrica', async ({ page }) => {
    await page.goto('/#/build/new')
    await expect(page.getByText('Filtros por cualquier m\u00e9trica')).toBeVisible()

    // el filtro por defecto es PF mínimo
    const metrica = page.getByLabel('Métrica del filtro 1')
    await expect(metrica).toBeVisible()
    await metrica.selectOption('sharpe')
    await expect(metrica).toHaveValue('sharpe')

    // se pueden añadir y quitar filtros
    const antes = await page.getByLabel(/Métrica del filtro/).count()
    await page.getByRole('button', { name: '+ filtro' }).click()
    const despues = await page.getByLabel(/Métrica del filtro/).count()
    expect(despues).toBe(antes + 1)
    await page.getByRole('button', { name: /quitar filtro 2/i }).click()
    expect(await page.getByLabel(/Métrica del filtro/).count()).toBe(antes)
  })

  test('F-H2 el selector de robustez explica los pesos del score', async ({ page }) => {
    await page.goto('/#/build/new')
    await expect(page.getByLabel(/ordenar por robustez/i)).toBeChecked()
    await expect(page.getByText(/gana la estrategia que aguanta/i)).toBeVisible()
    await expect(page.getByText(/eficiencia walk-forward/i).first()).toBeVisible()
    // al desactivarlo desaparece la explicacion
    await page.getByLabel(/ordenar por robustez/i).uncheck()
    await expect(page.getByText(/gana la estrategia que aguanta/i)).toHaveCount(0)
  })

  test('F-H3 el banco muestra la columna de score', async ({ page }) => {
    await page.goto('/#/bank')
    await expect(page.getByRole('columnheader', { name: /^score$/i })).toBeVisible()
    await expect(page.getByRole('columnheader', { name: /robustez/i })).toBeVisible()

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })
})
