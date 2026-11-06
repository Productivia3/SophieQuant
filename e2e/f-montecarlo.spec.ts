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

test('F13-H1 muestra los resultados devueltos por el motor sin dejar la pantalla en blanco', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {
        core: {
          invoke: async (command: string) => {
            if (command === 'list_bank') {
              return [{
                id: 1,
                name: 'Estrategia de prueba',
                symbol: 'EURUSD',
                timeframe: 'm1',
                netProfit: 100,
                lastRun: { equity: [100, 200], status: 'done' },
              }]
            }
            if (command === 'run_mc_suite') {
              return {
                strategyId: 1,
                results: [{
                  test: 'shuffle',
                  passed: true,
                  passRate: 0.87,
                  metric: 'maxDrawdown',
                  trials: 100,
                  percentiles: { p5: 10, p50: 100, p95: 150 },
                }],
              }
            }
            throw new Error(`Comando inesperado en prueba: ${command}`)
          },
        },
      },
    })
  })

  await page.goto('/#/montecarlo')
  await page.getByRole('button', { name: /correr simulaciones/i }).click()
  await expect(page.getByRole('table')).toBeVisible()
  const result = page.getByRole('row').filter({ hasText: 'Reordenar trades' })
  await expect(result).toContainText('87%')
  await expect(result).toContainText(/100\.00%/)
  await expect(page.getByText(/100 iteraciones/)).toBeVisible()
  await expect(page.getByRole('columnheader', { name: /métrica p5/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Monte Carlo' })).toBeVisible()

  const a11y = await new AxeConstructor({ page }).analyze()
  expect(a11y.violations).toEqual([])
})

test('F13-E2 el navegador no ofrece resultados Monte Carlo simulados', async ({ page }) => {
  await page.goto('/#/montecarlo')

  await expect(page.getByRole('button', { name: /correr simulaciones/i })).toBeDisabled()
  await expect(page.getByRole('status')).toContainText(/requiere la aplicación de escritorio/i)
  await expect(page.getByText(/no se generan resultados simulados/i)).toBeVisible()
})

test('F13-E1 un error de Monte Carlo se informa y permite volver a intentarlo', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {
        core: {
          invoke: async (command: string) => {
            if (command === 'list_bank') {
              return [{
                id: 1,
                name: 'Estrategia de prueba',
                symbol: 'EURUSD',
                timeframe: 'm1',
                netProfit: 100,
                lastRun: { equity: [100, 200], status: 'done' },
              }]
            }
            if (command === 'run_mc_suite') {
              await new Promise(resolve => window.setTimeout(resolve, 300))
              throw new Error('Error controlado de prueba')
            }
            throw new Error(`Comando inesperado en prueba: ${command}`)
          },
        },
      },
    })
  })

  await page.goto('/#/montecarlo')
  const run = page.getByRole('button', { name: /correr simulaciones/i })
  await run.click()
  await expect(page.getByRole('status')).toContainText(/la aplicaci[oó]n sigue disponible/i)
  await expect(page.getByLabel('Saltar trades al azar')).toBeEnabled()
  await expect(page.getByRole('alert')).toContainText('Error controlado de prueba')
  await expect(run).toBeEnabled()

  const a11y = await new AxeConstructor({ page }).analyze()
  expect(a11y.violations).toEqual([])
})
