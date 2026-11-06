# Test plan: SophieQuant

Fecha de última verificación: 2026-10-29 · Entorno: Windows, Vite con datos fake + Rust local.

## Alcance de esta pasada

- **Automatizado**: suite completa de Playwright + axe y tests de motor Rust; resultados actuales resumidos al final.
- **Manual / bloqueado**: solo los recorridos que necesitan ejecutar la ventana Tauri contra SQLite y ficheros locales.

Leyenda: e2e = Playwright sobre la UI web · Rust = unit/integration test del motor · Tauri = requiere la ventana de escritorio.

| case | flow | type | steps | expected | auto | result |
| --- | --- | --- | --- | --- | --- | --- |
| F01-H1 | generar estrategias | happy | S01 → "Nuevo build" → plantilla/TF → "Correr build" → S03 → "Ver Bank" | S03 con progreso + bank visible | e2e | ✓ |
| F01-T1 | generar estrategias | contrato Tauri | iniciar desde S02 con `invoke` | configuración enviada dentro de `config` | e2e | ✓ |
| F01-E1 | generar estrategias | edge: TF sin dataset | elegir d1 (solo m1 en seed) | aviso "No hay datos históricos" + enlace a Datos navega | e2e | ✓ |
| F01-E2 | generar estrategias | edge: doble clic en "Correr build" | doble submit rápido | sin crash, aterriza en S03 | e2e | ✓ |
| F01-E3 | generar estrategias | edge: refresh mid-build | refrescar S03 mientras el build está activo/pausado | estado y número de pasadas recuperados desde SQLite | Tauri | ✓ (build 4: paused, 15/100 tras refresh; cancelado al acabar) |
| F01-E4 | generar estrategias | edge: objetivo fuera de rango | nTarget=9 | input clampa a min 10, corre igual | e2e | ✓ |
| F01-R1 | robustez estricta | unitario | probar límites WFE/OOS y cada check MC, SPP y perfil | solo califica si todos pasan y hay al menos 30 operaciones OOS | Rust | ✓ |
| F01-R2 | robustez estricta | edge: evidencia incompleta o dataset distinto | persistencia/lectura del score y veredictos | pendiente, estrategia no calificada o dataset distinto no presenta score vigente | Rust | ✓ |
| F01-N1 | generar estrategias | negative: build inexistente | /#/build/run/9999 | sin crash (badge pending, 0 pasadas) | e2e | ✓ |
| F02-H1 | revisar resultados | happy | S04 → fila → S05 | 6 statcards + equity, sin NaN/undefined | e2e | ✓ |
| F02-E1 | revisar resultados | edge: estrategia dirty | abrir estrategia 3 | aviso de revisión visible | e2e | ✓ |
| F02-E2 | revisar resultados | edge: estrategia sin runs | /#/strategy/9999 | pantalla de carga, sin crash | e2e | ✓ |
| F03-H1 | improver | happy | S05 → S09 | variantes comparadas con el original | e2e | ✓ |
| F04-H1 | optimizar + 3D | happy | S10 → S11 | grid y superficie renderizados | e2e | ✓ |
| F05-H1 | robustez MC integrada | happy | S02 checkbox MC ON | checkbox accesible (BUG-002 fixed) | e2e | ✓ (persistencia real = block) |
| F06-H1 | custom project | happy | S16 workflow | pasos ejecutados con estado | e2e | ✓ |
| F07-H1 | portfolio | happy | S18 → S19 | correlación y equity combinada | e2e | ✓ |
| F08-H1 | exportar | happy | S22: estrategia → MT5 → "Generar código" | pre con código | e2e | ✓ |
| F08-E1 | exportar | edge: sin estrategia | click Generar sin selección | botón disabled | e2e | ✓ |
| F08-E2 | exportar | edge: plataformas anunciadas | generar en MT4 y MT5 | código coherente con la plataforma seleccionada | e2e | ✓ |
| F08-N1 | exportar | negative: id inexistente | command Tauri con id=999999 | error `estrategia no encontrada`, sin export parcial | Tauri | ✓ (solo quedó el registro de la exportación válida) |
| F08-H3 | exportar | happy: export nativo | generar MT5 desde estrategia del banco | código y verificación; command termina | Tauri | ✓ (código generado, 2046 caracteres) |
| F09-H1 | data manager | happy | S20 → "Importar histórico" | abre el Gestor de datos; no simula una descarga | e2e | ✓ |
| F09-E1 | datos | negative: comando Tauri falla | forzar rechazo de `invoke` al abrir S20 | error visible y sin datos fake | e2e | ✓ |
| F09-H2 | data manager + constructor + banco + exportar | happy: SQLite/QFN real | importar CSV → build → refrescar → banco → export MT5 | dataset listo, build persistido, estrategias listadas y export disponible | Tauri | ✓ (CSV sintético limpio, 800 barras; 2 estrategias; import command real) |
| F09-E3 | data manager | edge: dialectos CSV comunes | importar separado por `;`/tab, fecha/hora separadas y decimal europeo | OHLC parseado; fecha y volumen correctos | Rust | ✓ |
| F09-E2 | data manager | edge: CSV con ruido/gaps | importer CSV | filas malas contabilizadas y saltadas | Rust | ✓ |
| S23-H1 | ajustes | happy: metadatos de mercado | abrir S23 y consultar símbolos | muestra los campos reales editables | e2e | ✓ |
| F10-H1 | retest | happy | S04 multi → S08 | resultados con veredicto | e2e | ✓ |
| F11-H1 | analyzer | happy | S24 | resumen, MC de cartera y gestión de capital | e2e | ✓ |
| F13-H1 | Monte Carlo | happy: command nativo devuelve `{ strategyId, results }` | S13 → "Correr simulaciones" | tabla muestra métrica/unidad, percentiles e iteraciones | e2e | ✓ |
| F13-E2 | Monte Carlo | edge: abrir desde navegador sin motor Rust | /#/montecarlo en navegador | botón bloqueado e indicación de que no se generan resultados simulados | e2e | ✓ |
| F13-E1 | Monte Carlo | edge: fallo del command | S13 → "Correr simulaciones" → `invoke` rechaza | error visible, controles recuperados y puede reintentarse | e2e | ✓ |
| F05-E1 | SPP | edge: abrir en navegador sin motor Rust | /#/robustness | botón bloqueado e indicación de que no se generan resultados simulados | e2e | ✓ |
| V-H1/V-H2 | veredictos | edge: navegador sin motor Rust | veredictos en detalle y banco | no se calculan ni presentan resultados de robustez simulados | e2e | ✓ |
| H-E1 | historial | edge: SPP no ejecutado en navegador | abrir SPP y luego Historial | aviso de requisito Tauri; no aparece una ejecución ficticia | e2e | ✓ |
| X-H1 | transversal | navegación completa | rutas del Shell | sin console errors ni 5xx | e2e | ✓ (22/22) |
| X-H2 | transversal | axe | 6 pantallas clave | 0 violaciones | e2e | ✓ (tras BUG-001/002) |
| X-404 | transversal | negative | /#/no-existe | 404 propio | e2e | ✓ |
| X-H1b | transversal | mobile 420px | /#/bank con viewport 420 | sin crash ni overflow horizontal | e2e | ✓ |
| X-E2 | transversal | keyboard only | tab + flecha en objetivo de S02 | control alcanzable, modificable y con foco visible | e2e | ✓ |

## Resultado final (2026-10-29)

- Playwright + axe: **106/106** correctos.
- Rust: **74/74** tests correctos.
- `npm run build`: correcto.
- `cargo build --manifest-path src-tauri/Cargo.toml`: correcto; build Tauri QA arrancó con datos aislados.
- `git diff --check`: correcto.
- Smoke tests Tauri: importación CSV, generación/persistencia tras refresco, banco, exportación válida y negativa correctos.
- La ventana usó el identificador `com.sophiequant.qa`; la base normal `%APPDATA%\\com.sophiequant.app\\sophiequant.db` no se modificó.
- El import se invocó con un CSV sintético de 800 barras mediante el command real; el selector nativo de archivos no se automatizó.

## Revalidación Monte Carlo (2026-11-06)

- Playwright: suite completa, **112/112** correctos; incluye F13-H1, F13-E1 y F13-E2.
- Rust: **78/78** tests correctos.
- `npm run build`: correcto.
- F13-H1 comprueba el contrato `{ strategyId, results }`, las métricas/unidades e iteraciones; F13-E1 verifica la recuperación tras error y F13-E2 que el navegador no ofrece resultados sintéticos.
- El cálculo nativo se movió a `spawn_blocking` y ya no conserva el mutex de SQLite durante las iteraciones. Falta un pase manual en la ventana Tauri con un dataset real grande para medir su duración; no se usó ningún histórico personal.
- F13 ofrece seis pruebas implementadas; shuffle y bootstrap de drawdown calculan drawdown, se rechazan tipos sin implementación y los percentiles incluyen su métrica/unidad.

## Revalidación de robustez estricta

- Rust: **81/81** tests correctos. La calificación requiere todos los criterios; la puntuación solo ordena candidatos ya calificados.
- `npm run build`: correcto.
- Playwright: suite completa, **112/112** correctos tras actualizar una prueba obsoleta. El primer intento tuvo un timeout en H-H1 porque esperaba un resultado SPP del navegador; ahora el navegador bloquea correctamente esos cálculos. Se cambió a H-E1, que verifica el bloqueo y que no se crea una entrada falsa; el spec enfocado pasó **3/3** y la suite completa posterior pasó **112/112**.
- Los veredictos antiguos se invalidan por versión; las puntuaciones solo se exponen si la estrategia sigue calificada para el mismo dataset.
- El requisito estricto puede dejar el banco sin estrategias calificadas. La validación en ventana Tauri y con datos reales aún está pendiente; los tests no garantizan rentabilidad futura.

## Bugs encontrados y su estado

- BUG-001 (S2) progressbar sin aria-label → **fixed**
- BUG-002 (S2) checkbox sin label → **fixed**
- BUG-003 (test) selector ambiguo → **fixed in test**
- BUG-004 (S3) tablas sin contenedor de scroll en móvil → **fixed**
- BUG-005 (S3) sidebar no colapsa en móvil → **fixed** (drawer móvil, tests X-BUG005-1/2)
- Detalles en `replica/bugs.md`. No hay S1/S2 abiertos → nada bloquea continuar por esta vía.

## Límite de verificación nativa

No se abrió el diálogo nativo de archivos ni se usó un CSV personal. El command real de importación se verificó con un fichero sintético aislado. Para uso personal, todavía se puede hacer un pase visual del selector nativo y de los avisos de calidad con un CSV propio; no es un bloqueo para el flujo backend validado.
