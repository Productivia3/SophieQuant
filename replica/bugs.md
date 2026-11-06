# Bugs — QuantForge

Verificación local: 2026-10-29 · Entorno: Windows, Playwright con Chromium y capa fake; motor Rust local.

Suite completa: Playwright + axe y tests Rust; el recuento final figura en `test-plan.md`.

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
- Status: **fixed** — tablas envueltas en `div.overflow-x-auto` (S04 y S20).

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
+ X-H1b actualizado (scrollWidth ≤ 420). Verificado de nuevo en la suite completa.

### BUG-006 – Parseo de timestamps ISO roto en el importador Rust (hallado por los tests QA)

- Found: tanda QA Rust (14 unit tests escritos tras `cargo check` OK); el trazo con `--nocapture`
  mostró `2024-01-02 09:00:00` → epoch `638653781001600` (año leído como `20240102`).
- Cause: `parse_time` borraba los guiones ANTES de dividir, fusionando `2024-01-02` en un solo entero.
- Fix: reescrito `parse_time` (datastore.rs): división explícita fecha/hora, soporta `T`, puntos (MT4)
  y barras (Yahoo), exige y-m-d completos; además filas con timestamp roto ahora se **saltan** (antes
  abortaban todo el import) y las cabeceras `time,open,...` se ignoran sin contar como ruido; y las
  barras se ordenan por tiempo tras importar (prerrequisito del motor, nadie lo hacía).
- Bonus QA: `Platform::from_str` ahora acepta `MT4/MT5` en cualquier capitalización (era case-sensitive).
- Status: **fixed** – tests Rust en verde (importer ruido/gaps, QFN roundtrip, no-lookahead,
  determinismo del backtest y del genético, golden mínimo MQL4/5).

### BUG-007: veredictos de muestra WF se calculaban con proxies incorrectos

- Severity: S2 (podía presentar como suficiente una muestra OOS insuficiente).
- Flow / case: matriz walk-forward, item de mínimo de muestra y parada.
- Expected: contar los trades OOS, medir la duración real del tramo y comparar el drawdown OOS con el peak IS.
- Actual: se infería el número de trades a partir del profit factor, se asignaba un año a cualquier muestra no vacía y se comparaba `maxDD` contra `0.8`.
- Status: **fixed** — la matriz reejecuta la mejor celda con la configuración del símbolo; los flags salen de trades, fechas y equity. Tests de umbrales en Rust y aviso explícito en la capa fake.

### BUG-008: estrategias marcadas dirty sin detalle no avisaban en resultados

- Severity: S3 (estado visible en el banco, pero no explicado al abrir la estrategia).
- Flow / case: F02-E1, estrategia 3 de la semilla fake.
- Expected: un aviso de revisión también cuando `status=dirty` y no hay mensajes en `problems`.
- Actual: el detalle solo mostraba aviso si `problems` contenía elementos.
- Status: **fixed** — el detalle muestra un aviso y una acción recomendada; e2e F02-E1 pasa.

### BUG-009: controles de formulario sin indicador visible de foco por teclado

- Severity: S2 (problema de accesibilidad por teclado).
- Flow / case: X-E2, formulario de generación S02.
- Expected: foco visible al recorrer y operar el formulario con Tab y flechas.
- Actual: el control recibía foco pero no tenía un estilo de foco global definido.
- Status: **fixed** — `:focus-visible` aplica un contorno acentuado; e2e comprueba alcance, edición y foco visible.

### BUG-010: JSX inválido en cambios locales de Phase 2 bloqueaba el build

- Severity: S2 (la aplicación web no compilaba).
- Flow / case: build de producción.
- Expected: TypeScript y Vite compilan.
- Actual: cierres de expresiones JSX y sintaxis del editor estaban malformados.
- Status: **fixed** — estructura reparada y `npm run build` en verde.

### BUG-011: el Banco fallaba al listar estrategias en Tauri

- Severity: S2 (pantalla principal de resultados bloqueada en escritorio).
- Flow / case: F02 / F09, `list_bank`.
- Steps: importar datos y generar una estrategia; abrir/refrescar el Banco.
- Expected: `list_bank` devuelve las estrategias y sus métricas.
- Actual: error SQLite `Invalid column index: 17`; la consulta devuelve índices 0–13.
- Evidence: reproducido en WebView Tauri contra la base aislada `com.sophiequant.qa`.
- Status: **fixed** — se eliminó la lectura sobrante del índice 17 y se añadió una prueba Rust para listar una estrategia con una corrida vacía.

### BUG-012: la exportación válida quedaba bloqueada en Tauri

- Severity: S2 (la exportación de escritorio no terminaba).
- Flow / case: F08-H3.
- Steps: seleccionar una estrategia existente y ejecutar `export_strategy` en MT5.
- Expected: recibir el código y registrar la exportación.
- Actual: el command esperaba indefinidamente al volver a adquirir el mutex de SQLite mientras aún conservaba el guard anterior.
- Evidence: la invocación IPC válida excedió el timeout; el export inválido sí devolvía su error antes de llegar al bloqueo.
- Status: **fixed** — se libera el guard antes de la segunda adquisición. Verificado en Tauri: código de 2046 caracteres y registro completado.

### BUG-013: el número de pasadas no se persistía durante una generación

- Severity: S3 (tras refrescar, el progreso mostraba cero aunque existieran estrategias generadas).
- Flow / case: F01-E3.
- Steps: iniciar un build, dejar que avance y refrescar S03.
- Expected: recuperar estado y pasadas desde SQLite.
- Actual: el estado se persistía, pero `n_passed` se quedaba en cero.
- Evidence: reproducido en Tauri; la base aislada tenía estrategias mientras `list_builds` devolvía cero pasadas.
- Status: **fixed** — el reporter persiste `n_passed`/`n_total` en cada actualización; prueba Rust y refresh Tauri verificados.

### BUG-014: el importador descartaba todas las filas de algunos CSV de mercado

- Severity: S2 (bloqueaba importar datasets exportados con otro separador/columnas de fecha y hora).
- Flow / case: F09 / F09-E3.
- Steps: importar un CSV con `;` o tabuladores, encabezados de plataforma como `<DATE>/<TIME>` y OHLC, o decimales con coma.
- Expected: detectar el dialecto y convertir las barras en OHLC.
- Actual: el importador solo dividía por coma y esperaba timestamp en una única columna; devolvía `0 barras útiles` y descartaba todas.
- Evidence: error reportado por el usuario: `CSV con solo 0 barras útiles (49533 descartadas)`; los dialectos representativos quedaron cubiertos con pruebas de regresión.
- Status: **fixed** — detecta coma, punto y coma o tabulador; reconoce cabeceras OHLC y columnas de fecha/hora separadas; acepta coma decimal con separador `;` y fechas DMY.

### BUG-015: Monte Carlo mostraba resultados engañosos y una tasa de shuffle tautológica

- Severity: S2 (los resultados podían presentarse como fiables aunque varias opciones no simulaban lo que decían).
- Flow / case: F13 / F13-H1.
- Steps: ejecutar Monte Carlo en una estrategia rentable y comparar shuffle, drawdown y opciones de reconstrucción.
- Expected: cada perturbación cambia la magnitud analizada, el resultado explica métrica/unidad e iteraciones, y los tipos no implementados no se muestran como cálculos reales.
- Actual: shuffle medía solo beneficio neto, invariante al orden de trades; algunas opciones no implementadas caían en un jitter genérico ±8%; las cifras se rotulaban como PF aunque eran beneficio neto o drawdown; el navegador generaba resultados sintéticos.
- Evidence: pruebas Rust reproducen el drawdown al agrupar pérdidas y rechazan `use_variants`; `e2e/f-montecarlo.spec.ts` verifica el contrato real, unidad y bloqueo de resultados simulados en navegador.
- Status: **fixed** — shuffle y bootstrap drawdown evalúan DD completo, skip-certain elimina un bloque contiguo, quedan visibles solo seis pruebas implementadas, se reportan métrica/iteraciones/unidades y se desactiva Monte Carlo en navegador. Se conserva `spawn_blocking` sin retener SQLite.
- Pendiente de comprobación manual: comparar resultados de una estrategia real en la ventana Tauri; el porcentaje depende de trades y perturbación y puede ser 100 % en una muestra de 100 iteraciones, pero ya no por conservar el beneficio al barajar.

### BUG-016: la selección y los veredictos no exigían evidencia completa de robustez

- Severity: S2 (podía presentar como robusta una estrategia sin superar todos los controles).
- Flow / case: F01 / robustez estricta; F05 / SPP; veredictos.
- Steps: generar estrategias con el filtro de robustez activo; revisar las pruebas SPP, el perfil de parámetros y el score del banco.
- Expected: solo calificar si la búsqueda usa IS, hay al menos 30 operaciones OOS, WFE ≥50%, todas las pruebas MC y SPP pasan y cada parámetro activo presenta una meseta; advertencias, pendientes y evidencia de otro dataset no califican.
- Actual: el score podía actuar como ordenación en vez de requisito; algunas pruebas SPP no medían degradación real y ciertos estados no aprobados podían contarse como PASS. El navegador también podía mostrar resultados calculados de forma simulada y conservar veredictos antiguos.
- Evidence: cobertura Rust en `src-tauri/src/engine/robust.rs`, `ops.rs`, `verdicts.rs` y `genetic.rs`; pruebas e2e en `e2e/f-filters.spec.ts`, `e2e/f-phase3.spec.ts`, `e2e/f-verdicts.spec.ts` y `e2e/f-historial.spec.ts`.
- Status: **fixed with strict, fail-closed qualification** — se excluyen candidatos que no pasen cada requisito; las advertencias/pendientes no califican; los resultados web simulados están bloqueados; los cachés de versión anterior y los scores de otro dataset no se sirven. La calificación reduce falsos positivos, pero no demuestra rentabilidad futura. Falta validar en la ventana Tauri con un dataset real.

### Límite manual restante

- No se automatizó el diálogo nativo de selección de fichero ni se usaron datos personales. La importación se verificó llamando al command Tauri real con un CSV sintético de 800 barras.
- La importación del fichero concreto del usuario aún debe repetirse con esta versión; si sigue fallando, basta con compartir la fila de cabecera y una línea de ejemplo con valores sustituidos, no el histórico completo.
- No hay S1/S2 abiertos identificados en el código verificado; BUG-014 espera confirmación del CSV concreto.
