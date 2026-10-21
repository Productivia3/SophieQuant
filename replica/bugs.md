# Bugs — QuantForge

Build: sin commit · Date: 2026-10-20 · Env: local capa fake, Playwright sobre http://localhost:1420, Chromium 156 (Chrome for Testing)

Suíte: 40 casos automatizados (e2e/f-core, e2e/f08-export, e2e/x-transversal) + axe. Resultado final de la pasada: **39 ✓ / 1 ✘ (BUG-005, S3 documentado en test)**.

### BUG-001: Progressbar sin nombre accesible

- Severity: S2 (a11y WCGA 1.1.1 — herramienta de WA, core flow)
- Flow / case: F01 / F01-H1
- Screen: S03
- Build: sin commit · Browser: Chromium 156, 1440px

Steps
1. Ir a /#/build/new y pulsar "Correr build"
2. Escanear axe en S03

Expected: el role="progressbar" tiene accessible name
Actual: `aria-progressbar-name` — impact serious
Evidence: `e2e/f-core.spec.ts` F01-H1 (fallo de la pasada 1) + html `<div role="progressbar" …>`
Suspected cause: componente Progressbar sin aria-label
Status: **fixed** — `aria-label="Progreso del build"` en el componente (src/pages/index.tsx). Test: F01-H1 pasa.

### BUG-002: Checkbox "Monte Carlo integrado" sin label accesible

- Severity: S2 (a11y WCAG 4.1.2, impact critical en axe)
- Flow / case: F05 / F01
- Screen: S02
- Build: sin commit · Browser: Chromium 156

Steps
1. Ir a /#/build/new
2. Axe scan

Expected: todo input tiene label
Actual: `label` — `<input type="checkbox">` sin label ni aria-label
Evidence: e2e x-transversal axe /build/new (fallo pasada 1)
Suspected cause: input inline sin aria-label
Status: **fixed** — `aria-label="Monte Carlo integrado"` añadido. Test: axe /build/new pasa.

### BUG-003: selector de test ambiguo con dos enlaces "Datos" (falso positivo)

- Severity: no es bug de la app — bug del propio test (strict mode)
- Flow / case: F01 / F01-E1
- Screen: S02

Steps
1. Test: elegir TF d1 → aviso "no hay datos" → click en enlace "Datos"

Expected: navegar a /#/data
Actual: dos enlaces coinciden (sidebar + el del aviso)
Evidence: strict mode violation
Status: **fixed in test** — scoped a `p` del aviso. Test pasa.

### BUG-004: documento desbordaba el viewport en móvil por tablas sin contenedor de scroll

- Severity: S3
- Flow / case: transversal / X-H1b
- Screen: S04, S20
- Steps: viewport 420px → /#/bank → scrollWidth 435 > 420
- Status: **fixed** — tablas envueltas en `div.overflow-x-auto` (S04 y S20). La parte restante es BUG-005.

### BUG-005: sidebar fija de 224px no colapsa en pantallas ≤700px

- Severity: S3 (usability móvil; no bloquea desktop — producto desktop)
- Flow / case: transversal / X-H1b
- Screen: todas (layout shell)
- Build: sin commit · Browser: Chromium 156, 420px x 800

Steps
1. `page.setViewportSize({width:420})` → /#/bank
2. medir documentElement.scrollWidth

Expected: sidebar colapsa a drawer/hamburguesa; scrollWidth ≤ viewport
Actual: `nav.flex w-[224px] shrink-0` siempre visible; main queda en 196px; documentEl.scrollWidth 435 (recortado por body overflow-x hidden, sin scroll lateral real)
Evidence: e2e-probe + test X-H1b (documentado como expectativa en vez de falla)
Suspected cause: shell sin breakpoints responsive
Status: **fixed (Fase 4)** — drawer fijo desplazado fuera de pantalla con hamburguesa
(`data-testid="nav-burger"`), overlay clicable con `aria-label="Cerrar menú"`, `aria-expanded`
en el burger, cierre al navegar. Tests: `f-phase4.spec.ts` X-BUG005-1/2 + axe móvil 0 violaciones
+ X-H1b actualizado (scrollWidth ≤ 420). Suite completa: **77/77 ✓**.

### To check (no reproducido todavía — requiere núcleo Rust o Fase 3)

- F01: que un build con Monte Carlo integrado filtre de verdad al guardar (requiere engine).
- F09: importer CSV con timestamps ISO, filas basura y gaps (commands reales no compilados aún).
- S08 retest, S09-S10 optimizaciones, S16 canvas drag&drop, S18 correlación: pantallas stub — se testean al construirse.
- Determinismo del generador genético y golden-file MQL4/5 (unit tests de Rust, tras `cargo check`).

### BUG-006 – Parseo de timestamps ISO roto en el importador Rust (hallado por los tests QA)

- Found: tanda QA Rust (14 unit tests escritos tras `cargo check` OK); el trazo con `--nocapture`
  mostró `2024-01-02 09:00:00` → epoch `638653781001600` (año leído como `20240102`).
- Cause: `parse_time` borraba los guiones ANTES de dividir, fusionando `2024-01-02` en un solo entero.
- Fix: reescrito `parse_time` (datastore.rs): división explícita fecha/hora, soporta `T`, puntos (MT4)
  y barras (Yahoo), exige y-m-d completos; además filas con timestamp roto ahora se **saltan** (antes
  abortaban todo el import) y las cabeceras `time,open,...` se ignoran sin contar como ruido; y las
  barras se ordenan por tiempo tras importar (prerrequisito del motor, nadie lo hacía).
- Bonus QA: `Platform::from_str` ahora acepta `MT4/MT5` en cualquier capitalización (era case-sensitive).
- Status: **fixed** – 14/14 unit tests Rust en verde (importer ruido/gaps, QFN roundtrip, no-lookahead,
  determinismo del backtest y del genético, golden mínimo MQL4/5).
