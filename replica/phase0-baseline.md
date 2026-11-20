# Línea base Fase 0

**Fecha de ejecución:** 2026-11-20  
**Workspace:** Windows, SophieQuant  
**Revisión:** `d1f6a72d1b6ee6c95dd9f4995c0e78f7a61326b0` (`main`)  
**Alcance:** línea base automatizada y primeros recorridos Tauri sobre un perfil
QA aislado con barras sintéticas. No se usó la DB habitual ni ningún histórico
personal.

## Entorno Windows

| Componente | Línea base |
|---|---|
| SO | Windows 11 Home 10.0.26200, 64 bits |
| CPU | Intel Core i7-8550U, 4 núcleos / 8 procesadores lógicos |
| RAM visible | 8,295,652 KB (~7.91 GiB); libre en la captura: 1,066,528 KB (~1.02 GiB) |
| Node.js | v24.20.0 |
| npm | 11.19.0 |
| Rust | rustc 1.99.1; cargo 1.99.1 |
| Tauri CLI | 2.12.3 |
| Rust crate Tauri | 2.12.4 |
| MetaTrader 5 | 5.0.0.6230 |
| MetaEditor 5 | 5.0.0.6230 |

MT5 y MetaEditor están instalados en Windows. No se abrió MetaTrader, MetaEditor
ni el terminal: no se validó login, broker, ticks ni compilación del EA. MT5
no forma parte de los comandos `PATH` descubiertos, pero se localizaron los
ejecutables en `C:\Program Files\MetaTrader 5`.

### Linux de destino

Según el propietario: máquina compartida, 8 GB RAM y 4 vCPU, SophieQuant local
nativa y MT5 bajo Wine; no hay servicio remoto SophieQuant. La distribución y
versión Linux no están decididas ni disponibles en esta sesión. El plan de
instalación y el benchmark Linux quedan pendientes hasta conocerlas.

## Comprobaciones automatizadas

Ejecutadas en el commit indicado:

| Comando | Resultado | Observaciones |
|---|---|---|
| `cargo test --manifest-path src-tauri\Cargo.toml` | **94 passed, 0 failed** | Warning preexistente: variable `i` sin usar en `engine/ops.rs:1714`. |
| `cargo check --manifest-path src-tauri\Cargo.toml` | **Correcto** | Warning: `strictly_qualified` sin uso en `engine/robust.rs:339`. |
| `npm run build` | **Correcto** | TypeScript y Vite completaron; npm avisa de una clave de configuración de usuario `*` desconocida. |
| `npx playwright test` | **115 passed** | 1 worker, 3.5 min. Es UI web con backend fake/mocks, no aceptación de Tauri ni MT5. |
| `git diff --check` | **Correcto al iniciar Fase 0** | No había cambios en el worktree al iniciar; `ROADMAP_FINAL.md` estaba registrado. Los dos documentos nuevos de Fase 0 se mantienen sin staging. |

Estos tests son una línea base de regresión, no certifican las tareas nuevas de
1.0. Los tests del motor describen el código actual, no su paridad con ticks de
MT5. El build de escritorio y el instalador no se compilaron en esta pasada.

## Estado R0.1–R0.6

| ID | Resultado inicial | Evidencia / pendiente |
|---|---|---|
| R0.1 | **Parcial** | Se creó `replica/acceptance-matrix.md`, que mapea cada tarea del roadmap a evidencia automática/manual y estado inicial. Los casos de regresión nuevos aún se deben convertir en tests concretos cuando se implementen sus fases. |
| R0.2 | **Parcial** | Se registró revisión, rama, SO, hardware y versiones arriba. Tauri QA se ejecutó con identidad `com.sophiequant.qa`; se verificó `%APPDATA%\\com.sophiequant.qa\\sophiequant.db`. Se usaron solo fixtures sintéticas en `%TEMP%\\SophieQuantQA-f0-20261120`. La DB habitual no fue objetivo de ninguna operación. Falta distro Linux y política de backup/restore. |
| R0.3 | **Parcial** | Las cuatro suites/comandos indicados pasaron. En Tauri QA pasaron B1/B2 (800 barras sintéticas), B6 (CSV exportado y comprobado en disco), C1, C4 y C6; C2/C7 quedaron parciales, C5 falló (BUG-021) y C7 reveló una discrepancia de carga en la ficha (BUG-022). Resto de recorridos de escritorio y MT5 pendientes. |
| R0.4 | **Parcial** | Se contrastaron los componentes centrales de reglas, indicadores, backtest, importación de datos, generación y exportación con el alcance 1.0 (ver auditoría inicial abajo). Se validó además el comportamiento de C1 en Tauri; resta revisar cada módulo/UI y completar el inventario contra la matriz. |
| R0.5 | **Parcial** | Se generó e importó en Tauri QA un CSV sintético de 43.000 barras H1 (USDJPY), 2.451.037 bytes, del 02-01-2020 al 27-11-2024; calidad: 0 intervalos estimados, 0 filas descartadas, 0 sin volumen y 0 precios incoherentes. Dos búsquedas Windows con la misma configuración produjeron el mismo resumen; la segunda se muestreó cada segundo. Falta repetir en Linux y acordar presupuestos de recursos/tiempo. |
| R0.6 | **Bloqueado** | El propietario facilitó ubicaciones para ticks e informes. En el entorno actual, `Datos\\Ticks`, `Datos\\Informe` y `Datos\\Sinteticos` no muestran ficheros; la ruta indicada `Datos\\Informes` no existe (la carpeta disponible es singular). No se inspeccionaron los CSV situados en la raíz de `Datos`. Faltan rutas exactas a ticks GBPUSD H1 y reporte MT5; símbolo/broker/settings y reproducibilidad aún no comprobados. |

### R0.5 — primer benchmark Windows

**Configuración:** perfil `com.sophiequant.qa`; CSV sintético USDJPY H1, 43.000
barras; plantilla «Cruce de medias»; objetivo 10; filtros Profit factor ≥ 1,15
y drawdown máximo ≤ 35%; mínimo 20 operaciones; robustez estricta activada
(incluye Monte Carlo).

**Resultado funcional:** 24 candidatos evaluados y 24 superaron filtros y mínimo
de operaciones; 10/10 candidatos aprobaron robustez estricta y se guardaron
10/10 estrategias. Tiempo observado desde el clic hasta el estado final: ~8 s
(el registro pasó de evaluación a resumen entre 18:24:46 y 18:24:50). Las mejores
métricas del dataset sintético fueron PF 999, recovery 0 y 141 operaciones como
máximo; no son evidencia de calidad financiera ni de realismo.

**Recursos observados (no son picos aislados):** antes, a las 18:24:30.981, el
proceso llevaba 37,422 s de CPU, 65,8 MiB de working set y 18,6 MiB privados,
con 2,07 GiB libres en el sistema. Después, a las 18:25:14.733, llevaba 53,281 s
de CPU, 71,3 MiB de working set y 20,3 MiB privados, con 1,91 GiB libres. La
ventana entre capturas incluye unos 11 s antes de iniciar la búsqueda y unos 25 s
después de terminar; por tanto, el delta de CPU de 15,859 s y las lecturas de RAM
no se atribuyen exclusivamente a la búsqueda. No se muestreó el pico de RAM.
La UI no muestra una semilla de búsqueda en esta configuración; la coincidencia
del resumen demuestra repetibilidad funcional en dos corridas, no reproducibilidad
por semilla ni identidad de las estrategias.

### Tauri QA — recorridos C2 y C4–C7

- **C2 parcial:** las dos corridas sintéticas USDJPY H1 terminaron en estado
  COMPLETADO, con 24/24 candidatos superando filtros base y 10/10 aprobando
  robustez estricta. No se probó en esta sesión un filtro deliberadamente
  exigente ni la recuperación del resumen tras recargar.
- **C4/C6:** el Banco mostró 20 estrategias (10 de cada corrida), todas USDJPY H1
  y con métricas. La ficha seleccionada mostró regla/parámetros, métricas, curva
  de equity y marcador OOS, sin `undefined` ni `NaN`.
- **C5 falló:** al ordenar Beneficio neto, el primer clic ordenó descendente y el
  segundo ascendente (el tercero volvió a descendente), contrario al criterio
  de la guía. Reproducido en el build `d1f6a72`; ver [BUG-021](bugs.md).
- **C7 parcial:** el cálculo global llegó a 20/20 y las 20 filas mostraron
  FAILED. En la ficha inspeccionada, al recalcular, Walk-Forward pasó (WFE
  100 %, 94 trades OOS), Monte Carlo pasó 6/6, SPP falló (impacto máximo
  171,8 %) y el perfil pasó con 2/2 parámetros en meseta. Antes de recalcular,
  la ficha afirmó que no había veredictos, aunque el Banco ya mostraba los
  resultados; ver [BUG-022](bugs.md).
- **Criterios SPP:** la generación estricta califica la perturbación de
  parámetros; el veredicto independiente evalúa permutaciones de componentes
  de regla. Por eso el SPP del veredicto puede fallar sin contradecir el PASS
  estricto. La interfaz no deja suficientemente clara esta diferencia; revisar
  su rotulado antes de cerrar R0.4/C2.

### R0.5 — repetición y muestreo Windows

Se repitió la misma configuración y el mismo CSV sintético, en el mismo perfil
QA. El resumen volvió a ser idéntico: 24 candidatos evaluados, 24 superaron los
filtros y el mínimo de operaciones, 10/10 aprobaron robustez estricta y 10/10
se guardaron. Las métricas observadas volvieron a ser PF 999, recovery 0 y 141
operaciones como máximo. Son datos sintéticos y no evidencian calidad financiera.

La fase de robustez quedó registrada a las 18:38:11 y los resultados aprobados
a las 18:38:15 (~4 s entre esos registros); no se capturó un inicio fiable, por
lo que ese intervalo no representa el tiempo total de búsqueda.

**Muestreo del proceso `sophiequant` (aprox. 1 s, 18:37:57–18:38:26):** antes
del inicio, 54,52 s de CPU acumulada, 71,3 MiB de working set y 20,2 MiB
privados; RAM libre del sistema 0,81 GiB. El máximo muestreado fue 111,5 MiB
de working set y 56,5 MiB privados; el mínimo muestreado de RAM libre fue
0,72 GiB. Al terminar el muestreo: 70,58 s de CPU acumulada, 76,7 MiB de
working set, 21,6 MiB privados y 1,01 GiB libres. El incremento de CPU (16,06 s)
cubre toda la ventana de muestreo, incluidos segundos anteriores y posteriores
a la búsqueda. Los máximos/mínimos son de muestreo, no picos instantáneos.

## Auditoría inicial R0.4

| Área | Evidencia inspeccionada | Estado observado frente a 1.0 |
|---|---|---|
| Reglas/plantillas | `src-tauri/src/engine/rules.rs` | Tres plantillas cerradas; señales solo `EnterLong`/`ExitLong`. Sin AND/OR configurable, MACD, CCI ni catálogo de patrones definido para 1.0. |
| Indicadores | `src-tauri/src/engine/indicators.rs` | SMA, EMA, RSI, ATR y utilidades highest/lowest. MACD, CCI y patrones del catálogo pendientes. |
| Backtest | `src-tauri/src/engine/backtest.rs`, `types.rs` | Ejecuta barras y solo largo; aunque `Direction` contiene `Short`, el motor registra `Long`. No ejecuta tick bid/ask. |
| Datos | `src-tauri/src/engine/datastore.rs` | CSV/QFN para barras y resample; importación de ticks bid/ask no identificada. Calidad/resample de barras no sustituye al tick feed. |
| Generación | `src-tauri/src/engine/genetic.rs`, `src/pages/index.tsx` | Búsqueda individual con símbolo/TF/plantilla seleccionados. No hace batch de todos los símbolos/métodos, coherente con la decisión de alcance. |
| Exportación | `src-tauri/src/export.rs` | El enum y generadores aceptan MT4 y MT5; las plantillas exportadas documentan lógica long-only y costes/sizing simplificados. MT4 se debe retirar para 1.0, y el EA actual no satisface la paridad/tipos de regla de 1.0. |
| Robustez | `src-tauri/src/engine/robust.rs`, `ops.rs`, `verdicts.rs` | Hay WFE/MC/SPP/perfiles y tests; no se validó fórmula contra el Tester de MT5. Falsos PASS y overfitting necesitan gates definidos por R4. |
| Plataforma | `src-tauri/tauri.conf.json`, `Cargo.toml` | Se encontró configuración Tauri y Windows local. No se halló build/test Linux ni distribución de destino especificada. |

## Pendiente de ejecución

- Resto de casos manuales de UI en la ventana Tauri.
- Copia/restore de `%APPDATA%\com.sophiequant.app` o validación del flujo de backup/restore.
- Importación de un archivo tick o histórico personal.
- MetaEditor, Strategy Tester, Demo MT5 o compilación de `.mq5`.
- Repetir el benchmark en Linux y acordar los presupuestos de recursos/tiempo
  después de disponer de mediciones comparables en ambos sistemas.

## Siguiente paso seguro

1. Continuar los recorridos manuales aplicables en Tauri QA, sin usar la DB
   habitual.
2. Decidir la distribución/versión de Linux y repetir allí el benchmark de
   43.000 barras con la misma configuración.
3. Proporcionar en ubicación de QA la ruta local del tick bid/ask GBPUSD H1 y
   del reporte de Strategy Tester, o mantener R0.6 bloqueado y elegir otra
   referencia reproducible.

## Seguridad de QA

No usar una cuenta real ni ejecutar un EA con dinero real. No registrar número de
cuenta, servidor, credenciales, tokens ni históricos completos en Git. Los
datasets personales deben quedarse fuera del repositorio y sus hashes pueden
registrarse solo si el propietario confirma que el identificador no expone
información sensible.
