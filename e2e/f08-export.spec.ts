// F08 exportar a plataforma
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

test.beforeEach(async ({ page }) => {
  const errors: string[] = []
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()) })
  page.on('response', res => { if (res.status() >= 500) errors.push(`${res.status()} en ${res.url()}`) })
  ;(page as unknown as { _qfErrors: string[] })._qfErrors = errors
})

test.afterEach(async ({ page }) => {
  const errors = (page as unknown as { _qfErrors: string[] })._qfErrors
  if (errors.length) throw new Error(`Errores en la página:\n${errors.join('\n')}`)
})

test.describe('F08 export', () => {
  test('F08-E1 sin estrategia: botón deshabilitado', async ({ page }) => {
    await page.goto('/#/export')
    await expect(page.getByRole('button', { name: /generar código/i })).toBeDisabled()
  })

  test('F08-H1 generar código MQL5 para la primera estrategia', async ({ page }) => {
    await page.goto('/#/export')
    await page.locator('li button').first().click()
    await page.getByRole('button', { name: /generar código/i }).click()
    await expect(page.locator('pre')).toBeVisible({ timeout: 10_000 })
    const code = await page.locator('pre').textContent()
    expect(code).toBeTruthy()
    // el stub/fake mantiene el nombre de plataforma dentro del código
    expect(code!.length).toBeGreaterThan(50)

    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])
  })

  test('F08-E2 las plataformas con export real generan código distinto', async ({ page }) => {
    await page.goto('/#/export')
    await page.locator('li button').first().click()
    // solo se anuncian plataformas con generador real (MQL4 y MQL5)
    const platforms = ['MetaTrader 5', 'MetaTrader 4']
    const seen = new Set<string>()
    for (const p of platforms) {
      await page.getByRole('radio', { name: p }).click()
      await page.getByRole('button', { name: /generar código/i }).click()
      await expect(page.locator('pre')).toBeVisible({ timeout: 10_000 })
      const code = await page.locator('pre').textContent()
      expect(code, `código para ${p}`).toBeTruthy()
      seen.add(code!)
    }
    expect(seen.size).toBeGreaterThanOrEqual(2)
    // no se anuncian plataformas sin generador (no prometer lo que no existe)
    for (const fantasma of ['NinjaTrader', 'TradeStation', 'MultiCharts', 'JForex']) {
      await expect(page.getByRole('radio', { name: fantasma })).toHaveCount(0)
    }
  })
})
