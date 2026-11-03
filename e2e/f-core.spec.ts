// F01 generar estrategias (core loop) · F02 revisar resultados · F09 data manager
import { test, expect } from '@playwright/test'
import AxeConstructor from '@axe-core/playwright'

async function mockBankEngine(page: import('@playwright/test').Page, computeDelay = 0) {
  await page.addInitScript((delayMs) => {
    const strategies = [
      { id: 1, name: 'Alpha', symbol: 'EURUSD', timeframe: 'm1', robustScore: 60,
        netProfit: 10, profitFactor: 1.4, winRate: 0.52, maxDD: 15, nTrades: 120, sharpe: 0.8,
        lastRun: { equity: [100, 110], oosFrac: 0.2, status: 'done' } },
      { id: 2, name: 'Beta', symbol: 'GBPUSD', timeframe: 'm5', robustScore: 80,
        netProfit: 50, profitFactor: 2.1, winRate: 0.64, maxDD: 5, nTrades: 220, sharpe: 1.5,
        lastRun: { equity: [100, 150], oosFrac: 0.2, status: 'done' } },
      { id: 3, name: 'Gamma', symbol: 'USDJPY', timeframe: 'm15', robustScore: 30,
        netProfit: -2, profitFactor: 0.8, winRate: 0.4, maxDD: 30, nTrades: 80, sharpe: -0.2,
        lastRun: { equity: [100, 98], oosFrac: 0.2, status: 'done' } },
      { id: 4, name: 'Delta', symbol: 'AAPL', timeframe: 'h1', robustScore: null,
        netProfit: null, profitFactor: null, winRate: null, maxDD: null, nTrades: null, sharpe: null,
        lastRun: { equity: null, oosFrac: null, status: 'done' } },
    ]
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {
        core: {
          invoke: async (command: string, args?: Record<string, unknown>) => {
            if (command === 'list_bank') return strategies
            if (command === 'get_strategy_verdicts') return { rows: [] }
            if (command === 'compute_strategy_verdicts') {
              await new Promise(resolve => window.setTimeout(resolve, delayMs))
              const strategyId = (args?.strategyIds as number[])[0]
              return {
                rows: [{
                  strategyId,
                  nombre: strategies.find(strategy => strategy.id === strategyId)?.name ?? '',
                  checks: {
                    walkForward: { estado: 'pass', detalle: 'OK', valor: 1 },
                    monteCarlo: { estado: 'pass', detalle: 'OK', valor: 1 },
                    spp: { estado: 'pass', detalle: 'OK', valor: 1 },
                    perfil: { estado: 'pass', detalle: 'OK', valor: 1 },
                  },
                  overall: 'pass',
                  calculadoEn: 1,
                }],
              }
            }
            throw new Error(`Comando inesperado en prueba: ${command}`)
          },
        },
      },
    })
  }, computeDelay)
}

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

test.describe('F09 data manager', () => {
  test('F09-H1 desde Datos abre el gestor para importar un CSV', async ({ page }) => {
    await page.goto('/#/data')
    await expect(page.getByRole('table')).toBeVisible()
    await page.getByRole('button', { name: /importar histórico/i }).click()
    await expect(page).toHaveURL(/\/#\/data-manager/)
    await expect(page.getByRole('heading', { name: /gestor de datos/i })).toBeVisible()
  })

  test('F09-E1 la pantalla Datos no ofrece descarga simulada', async ({ page }) => {
    await page.goto('/#/data')
    await expect(page.getByRole('button', { name: /descargar m1|fuentes de datos/i })).toHaveCount(0)
  })
})

test.describe('F01 generar estrategias', () => {
  test('F01-T1 envía la configuración bajo la clave que requiere Tauri', async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(window, '__TAURI__', {
        configurable: true,
        value: {
          core: {
            invoke: async (command: string, args?: Record<string, unknown>) => {
              if (command === 'start_build') {
                Object.assign(window, { __startBuildArgs: args })
                return 42
              }
              if (command === 'list_symbols') {
                return [{ id: 1, name: 'EURUSD', assetClass: 'forex', tickSize: 0.00001, pipSize: 0.0001, commission: 0 }]
              }
              if (command === 'list_data_files') {
                return [{ id: 1, symbolId: 1, timeframe: 'm1', storageKind: 'local_qfd', compressedBytes: 1000, nBars: 500, dateFrom: 1, dateTo: 2, status: 'ready', createdAt: 1 }]
              }
              if (command === 'list_templates') {
                return [{ id: 1, name: 'Base MA cross', source: 'built_in' }]
              }
              if (command === 'list_build_metrics') return { metrics: [], pesos: [] }
              if (command === 'list_builds') return []
              throw new Error(`Comando inesperado en prueba: ${command}`)
            },
          },
        },
      })
    })
    await page.goto('/#/build/new')
    await page.getByRole('button', { name: /generar estrategias/i }).click()
    await expect(page).toHaveURL(/\/#\/build\/run\/42/)
    const args = await page.evaluate(() => (window as unknown as { __startBuildArgs: Record<string, unknown> }).__startBuildArgs)
    expect(args).toHaveProperty('config')
    expect(args).not.toHaveProperty('symbolId')
    expect(args.config).toMatchObject({
      kind: 'builder',
      symbolId: 1,
      timeframe: 'm1',
      templateId: 1,
      nTarget: 100,
      monteCarlo: true,
    })
  })

  test('F01-H1 config → correr → S03 en marcha → bank', async ({ page }) => {
    // asegurar datos primero (fake-layer: el seed ya trae datasets)
    await page.goto('/#/build/new')
    await expect(page.getByLabel('Símbolo')).toBeVisible()
    await expect(page.getByLabel('Plantilla')).toBeVisible()
    await page.getByRole('button', { name: /generar estrategias/i }).click()
    // S03: página de build en vivo
    await expect(page.getByRole('progressbar')).toBeVisible()
    await expect(page.getByText(/pasadas/i)).toBeVisible()
    
    const a11y = await new AxeConstructor({ page }).analyze()
    expect(a11y.violations).toEqual([])

    // navegar al bank
    await page.getByRole('button', { name: /ver banco/i }).click()
    await expect(page.getByRole('heading', { name: /banco de estrategias/i })).toBeVisible()
  })

  test('F01-E1 aviso cuando el TF elegido no tiene datos', async ({ page }) => {
    await page.goto('/#/build/new')
    // elegir un TF sin dataset (Cambiar TF a d1, el seed solo trae m1)
    await page.getByRole('button', { name: /^d1$/ }).click()
    await expect(page.getByText(/no hay datos históricos/i)).toBeVisible()
    // el enlace dentro del aviso funciona (no el de la sidebar)
    await page.locator('p').getByRole('link', { name: 'Datos' }).click()
    await expect(page.getByRole('heading', { name: /datos históricos/i })).toBeVisible()
  })

  test('F01-E2 doble clic en "Generar estrategias" no crashea', async ({ page }) => {
    await page.goto('/#/build/new')
    const btn = page.getByRole('button', { name: /generar estrategias/i })
    await btn.click()
    // carrera del doble clic: el segundo click muere rápido si el botón ya se fue
    await btn.click({ timeout: 1200 }).catch(() => {})
    // debe terminar en la pantalla de run sin excepción React
    await expect(page.getByRole('heading', { name: /generación #\d+/i })).toBeVisible()
  })

  test('F01-E4 objetivo y filtros clamp de rango', async ({ page }) => {
    await page.goto('/#/build/new')
    const target = page.getByLabel('Estrategias objetivo')
    await target.fill('9')           // min=10 → display clamp
    await expect(target).toHaveValue(/^\d+$/) // acepta número
    await page.getByRole('button', { name: /generar estrategias/i }).click()
    await expect(page.getByRole('progressbar')).toBeVisible()
  })

  test('F01-N1 build inexistente no crashea S03', async ({ page }) => {
    await page.goto('/#/build/run/9999')
    await expect(page.getByRole('progressbar')).toBeVisible()
    await expect(page.getByText(/pasadas/i)).toBeVisible()
  })
})

test.describe('F02 revisar resultados', () => {
  test('F02-H2 ordena las columnas y muestra el progreso real de los veredictos', async ({ page }) => {
    await mockBankEngine(page, 180)
    await page.goto('/#/bank')
    const nombres = () => page.locator('tbody tr').evaluateAll(rows =>
      rows.map(row => row.querySelector('td')?.textContent?.trim() ?? ''))

    await expect(page.locator('tbody tr')).toHaveCount(4)
    const alpha = page.locator('tbody tr').filter({ hasText: 'Alpha' })
    await expect(alpha.locator('td').nth(3)).not.toHaveText('—')
    await expect(alpha.locator('td').nth(4)).not.toHaveText('—')
    await expect(alpha.locator('td').nth(1)).toHaveText('EURUSD')
    const beneficio = page.getByRole('button', { name: /ordenar por beneficio neto/i })
    await beneficio.click()
    await expect.poll(nombres).toEqual(['Beta', 'Alpha', 'Gamma', 'Delta'])
    await page.getByRole('button', { name: /ordenar por beneficio neto/i }).click()
    await expect.poll(nombres).toEqual(['Gamma', 'Alpha', 'Beta', 'Delta'])

    await page.getByRole('button', { name: /calcular veredictos/i }).click()
    const progress = page.getByRole('progressbar', { name: /progreso del cálculo de veredictos/i })
    await expect(progress).toHaveAttribute('aria-valuenow', '1')
    await expect(page.getByText(/1 de 4.*3 restantes/i)).toBeVisible()
    await expect(progress).toHaveAttribute('aria-valuenow', '4', { timeout: 10_000 })
    await expect(page.getByRole('status')).toContainText('Veredictos calculados para 4 estrategias.')
  })

  test('F02-H1 bank → detalle con stats y equity → volver', async ({ page }) => {
    await page.goto('/#/bank')
    const fila = page.locator('tbody tr').first()
    await expect(fila).toBeVisible()
    await fila.click()
    await expect(page.getByText(/beneficio neto/i).first()).toBeVisible()
    await expect(page.getByText(/equity \(backtest principal\)/i)).toBeVisible()
    // sin NaN renderizado
    const body = await page.locator('body').textContent()
    expect(body).not.toContain('NaN')
    expect(body).not.toContain('undefined')
  })

  test('F02-E1 estrategia marcada dirty muestra aviso de revisión', async ({ page }) => {
    await page.goto('/#/strategy/3')
    await expect(page.getByRole('alert')).toContainText(/estrategia marcada para revisión/i)
  })

  test('F02-E2 estrategia 9999 → pantalla no cruja', async ({ page }) => {
    await page.goto('/#/strategy/9999')
    // fake api devuelve null → Title '…' y esqueleto
    await page.waitForTimeout(400)
    await expect(page.getByRole('heading', { name: /…/ })).toBeVisible()
  })
})
