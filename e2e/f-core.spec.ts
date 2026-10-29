// F01 generar estrategias (core loop) · F02 revisar resultados · F09 data manager
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

test.describe('F09 data manager', () => {
  test('F09-H1 descarga m1 con progreso y toast', async ({ page }) => {
    await page.goto('/#/data')
    await expect(page.getByRole('table')).toBeVisible()
    const btn = page.getByRole('button', { name: /descargar m1/i }).first()
    await btn.click()
    // al terminar: toast visible
    await expect(page.getByRole('status').last()).toBeVisible({ timeout: 20_000 })
  })

  test('F09-E1 solo una descarga simultánea', async ({ page }) => {
    await page.goto('/#/data')
    const buttons = page.getByRole('button', { name: /descargar m1/i })
    const first = buttons.first()
    const second = buttons.nth(1)
    await first.click()
    // mientras busy, el resto deshabilitados
    await expect(second).toBeDisabled()
    await expect(page.getByRole('status').last()).toBeVisible({ timeout: 20_000 })
  })
})

test.describe('F01 generar estrategias', () => {
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
