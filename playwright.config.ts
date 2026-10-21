import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  // e2e vive fuera del tsconfig de la app (tsc -b no lo toca)
  timeout: 30_000,
  retries: 0,
  workers: 1,
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'retain-on-failure',
    locale: 'es-ES',
  },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'replica/e2e-report' }]],
  // reutiliza el dev server si vive; si no, lo levanta y apaga solo al terminar
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    timeout: 60_000,
    reuseExistingServer: true,
  },
})
