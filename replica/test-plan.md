# Test plan: SophieQuant

Fecha de última verificación: 2026-10-29 · Entorno: Windows, Vite con datos fake + Rust local.

## Alcance de esta pasada

- **Automatizado**: suite completa de Playwright + axe y tests de motor Rust; resultados actuales resumidos al final.
- **Manual / bloqueado**: solo los recorridos que necesitan ejecutar la ventana Tauri contra SQLite y ficheros locales.

Leyenda: e2e = Playwright sobre la UI web · Rust = unit/integration test del motor · Tauri = requiere la ventana de escritorio.

| case | flow | type | steps | expected | auto | result |
| --- | --- | --- | --- | --- | --- | --- |
| F01-H1 | generar estrategias | happy | S01 → "Nuevo build" → plantilla/TF → "Correr build" → S03 → "Ver Bank" | S03 con progreso + bank visible | e2e | ✓ |
| F01-H2 | generar estrategias | dataset EURUSD H1 | seleccionar H1 con datasets M1 y H1; comprobar las fechas mostradas | usa las fechas del dataset H1 y muestra fechas correctas desde timestamps Rust en segundos | e2e | ✓ |
| F01-H3 | generar estrategias | objetivo no alcanzado | mock de generación completada con 0/10 y diagnóstico guardado | estado COMPLETADO, resumen visible y razones de descarte persistidas tras cargar la ejecución | e2e/Rust | ✓ |
| F01-R3 | generar estrategias | rechazos y robustez | filtros deliberadamente imposibles y checks estrictos fallidos | desglose independiente por filtro, mínimo de operaciones y motivo de cada validación estricta; conteos solapados identificados | Rust | ✓ |
| F01-R4 | generar estrategias | búsqueda sin supervivientes/plantilla fija | filtros base imposibles; inspeccionar siguiente generación del generador | muta las mejores candidatas y conserva la exploración; una plantilla fija nunca recibe parámetros por defecto de otra plantilla; no guarda candidatos no calificados | Rust | ✓ |
| F01-T1 | generar estrategias | contrato Tauri | iniciar desde S02 con `invoke` | configuración enviada dentro de `config` | e2e | ✓ |
| F01-E1 | generar estrategias | edge: TF sin dataset | alternar entre d1 sin datos y m1 con datos | sin datos: aviso y enlace visibles, controles de plantilla/filtros/robustez ocultos y generación deshabilitada; con datos vuelven a mostrarse | e2e | ✓ |
| F01-E2 | generar estrategias | edge: doble clic en "Correr build" | doble submit rápido | sin crash, aterriza en S03 | e2e | ✓ |
| F01-E3 | generar estrategias | edge: refresh mid-build | refrescar S03 mientras el build está activo/pausado | estado y número de pasadas recuperados desde SQLite | Tauri | ✓ (build 4: paused, 15/100 tras refresh; cancelado al acabar) |
| F01-E4 | generar estrategias | edge: objetivo fuera de rango | nTarget=9 | input clampa a min 10, corre igual | e2e | ✓ |
| F01-R1 | robustez estricta | unitario | probar límites WFE/OOS, MC, perturbación de parámetros SPP y perfiles | califica con WFE ≥50%, 30 operaciones OOS, todas las pruebas MC, perturbación de parámetros aprobada y meseta en cada parámetro activo; los controles negativos SPP son diagnósticos, no requisitos de preservar beneficio | Rust | ✓ |
| F01-R2 | robustez estricta | edge: evidencia incompleta o dataset distinto | persistencia/lectura del score y veredictos | pendiente, estrategia no calificada o dataset distinto no presenta score vigente | Rust | ✓ |
| F01-N1 | generar estrategias | negative: build inexistente | /#/build/run/9999 | sin crash (badge pending, 0 pasadas) | e2e | ✓ |
| F04-H1 | optimizador | Simple | S10 → Simple → Optimizar | tabla por combinación con ejes, PF, drawdown, leyenda y mejor punto | e2e | ✓ |
| F04-H2 | optimizador | Walk-Forward | S10 → Walk-Forward → Optimizar | tabla WFE/DD/veredicto; sin campos undefined | e2e | ✓ |
| F04-H3 | optimizador | detalle | S11 → elegir estrategia | mapa Walk-Forward se carga para la estrategia elegida | e2e | ✓ |
| F04-R1 | optimizador | contrato del motor | evaluar grid simple y WF | cada modo devuelve sus métricas propias y una mejor combinación válida | Rust | ✓ |
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
| B6 | data manager | exportación de dataset | pulsar Exportar y confirmar una ruta con diálogo nativo | el archivo existe en la ubicación elegida; la UI confirma la ruta y el tamaño | Tauri | pendiente de repetir manualmente |
| B7 | importador CSV | diagnóstico de rechazos | reimportar fichero y consultar desglose de causas | contadores por motivo suman el total descartado | Rust/e2e | ✓ (caso sintético); MT5 reportó 0 filas descartadas |
| B8 | importador CSV | intervalos estimados vs sesiones | importar el CSV MT5 y revisar avisos | avisa que no conoce el calendario de sesiones y no los declara datos perdidos | Rust/manual | pendiente: confirmar símbolo y timeframe; cotejar intervalos con horario del mercado |
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

## Revalidación de búsqueda genética (2026-11-18)

- El registro local del build 3 identifica USDJPY H1, plantilla Reversión osciladora, objetivo 10, PF ≥ 1.2, DD ≤ 35 %, Recovery Factor ≥ 1, mínimo 300 operaciones y 4.000 evaluaciones; 0 candidatos superaron los filtros base.
- Se corrigió la evolución para conservar las mejores candidatas no calificadas cuando no hay supervivientes, manteniendo parte de la población como exploración desde los parámetros por defecto. Los filtros y la calificación estricta no se relajan.
- Se corrigió además el uso de parámetros por defecto de otra plantilla en generaciones posteriores cuando el usuario fija una plantilla.
- El resumen incluye ahora los mejores PF/Recovery/operaciones observados durante el entrenamiento, además del desglose de rechazos ya añadido.
- Rust: **91/91** tests correctos. Playwright de resumen persistido: **1/1**. Retests posteriores muestran resultados distintos con distintas configuraciones; no deben compararse como una misma búsqueda.

## Seguimiento de robustez estricta (2026-11-18)

- El build 4 (antes de esta corrección SPP) fue USDJPY H1 + Cruce de medias, objetivo 10, PF ≥ 1.2, DD ≤ 35 %, Recovery Factor ≥ 1 y mínimo 200 operaciones: 24/24 pasaron filtros base y 0/10 robustez. El build 5, ya con el binario compilado después de los cambios, fue EURUSD H1 + Reversión osciladora, sin filtro Recovery: 0/4008 filtros base y PF máximo 0.47. Como cambian símbolo, plantilla y filtros, el build 5 no vuelve a probar la corrección SPP.
- Se encontró un error en la calificación SPP: exigía que entradas/salidas invertidas, señales opuestas y entradas aleatorias conservaran casi el mismo beneficio. Son controles negativos y no deben ser requisitos de robustez. La calificación y el componente SPP del score ahora usan la prueba de perturbación de parámetros; Monte Carlo y las mesetas siguen siendo requisitos estrictos.
- El resumen ahora detalla qué pruebas Monte Carlo y qué parámetros/perfiles no pasan, y diferencia errores de evaluación.
- Rust focalizado: **10/10** tests de robustez correctos (incluye que los controles negativos no bloqueen la calificación). Sigue pendiente repetir en Tauri la configuración del build 4 para validar SPP con finalistas y obtener el nuevo desglose de Monte Carlo/perfiles.

## Revalidación del optimizador (2026-11-07)

- `npm run build`: correcto.
- Playwright: suite completa, **113/113** correctos.
- Rust: **82/82** tests correctos, incluido el contrato separado de métricas para Simple y Walk-Forward.
- F04-H1/H2 verifican el mapa con los valores adecuados por modo; F04-H3 comprueba la pantalla de detalle y la estrategia seleccionada.
- La interfaz dejó de usar la superficie SVG isométrica y ahora explica el mapa tabular con valores, ejes, drawdown y leyendas. No se hizo una validación visual manual dentro de la ventana Tauri.

## Seguimiento manual de datos y exportación (2026-11-09)

- El usuario observó aproximadamente 17.000 filas descartadas de un CSV de unas 42.000 en tres símbolos; todavía no está verificado si son filas inválidas o un formato parcialmente incompatible.
- `ImportReport` ahora expone desglose por columnas, fecha/hora, cada campo numérico y OHLC incoherente; una prueba Rust verifica los contadores y su serialización.
- La exportación abre el diálogo nativo de guardado y escribe en la ruta elegida; el command Rust verifica CSV/QFD y tamaño. Falta repetir B6 en la aplicación Tauri; la prueba en navegador confirma que ya no informa falsamente que exportó.
- Validación de código: `npm run build` correcto; `cargo check` correcto; Rust **84/84**; suite completa de Playwright **113/113**. `cargo build` no pudo reemplazar `sophiequant.exe` por acceso denegado (probablemente está abierto); no se cerró ningún proceso.

## Revalidación Tauri de Fase 0 (2026-11-20)

- C2: dos búsquedas de USDJPY H1 sintético terminaron COMPLETADO con 24/24 en filtros base y 10/10 aprobadas por robustez estricta. Falta probar el filtro deliberadamente exigente y persistencia tras recarga.
- C4/C6: el Banco mostró 20 estrategias con métricas; la ficha muestra parámetros, métricas, curva y marcador OOS.
- C5: **falla**; el orden numérico inicial es descendente y el siguiente clic lo cambia a ascendente. BUG-021.
- C7: progreso completo 20/20; las 20 filas quedaron FAILED. En la ficha revisada se vio WFE 100 %/94 trades OOS, MC 6/6 PASS, SPP FAILED por impacto máximo 171,8 % y perfil 2/2 en meseta. La ficha no cargó los veredictos existentes hasta recalcular; BUG-022.
- La generación estricta evalúa perturbación de parámetros en SPP, mientras el veredicto independiente evalúa permutaciones de componentes de regla; no son el mismo criterio. Aclarar la diferencia en la interfaz antes de cerrar R0.4/C2.

## Bugs encontrados y su estado

- BUG-001 (S2) progressbar sin aria-label → **fixed**
- BUG-002 (S2) checkbox sin label → **fixed**
- BUG-003 (test) selector ambiguo → **fixed in test**
- BUG-004 (S3) tablas sin contenedor de scroll en móvil → **fixed**
- BUG-005 (S3) sidebar no colapsa en móvil → **fixed** (drawer móvil, tests X-BUG005-1/2)
- BUG-021 y BUG-022 son S3 abiertos; los detalles están en `replica/bugs.md`. No hay S1/S2 abiertos.

## Límite de verificación nativa

No se abrió el diálogo nativo de archivos ni se usó un CSV personal. El command real de importación se verificó con un fichero sintético aislado. Para uso personal, todavía se puede hacer un pase visual del selector nativo y de los avisos de calidad con un CSV propio; no es un bloqueo para el flujo backend validado. Para R0.6 se facilitaron carpetas para ticks e informes, pero no contienen ficheros visibles en este entorno; la carpeta disponible se llama `Datos/Informe` (singular), no `Datos/Informes`. No se inspeccionaron los CSV de la raíz de `Datos`.
