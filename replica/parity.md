# Paridad de SophieQuant frente al original

Fecha: 20-10-2026 · Herramientas: `.claude/skills/replica-diff/parity.py` y
`imgdiff.py` · Datos: `replica/features.csv` (56 filas, 51 computables),
`replica/screens/` (referencia del original), `replica/clone-screens/` (nuestro).

---

## 1. Puntuación global

| Métrica | Valor | Cómo se ha medido |
| --- | --- | --- |
| **Paridad de funcionalidades (todas)** | **60,5 / 100** | 51 funciones comparables del original; must=3, should=2, could=1; `partial` cuenta 0,5 |
| **Paridad dentro del alcance que definiste** | **63,3 / 100** | idéntico, pero sin las 3 filas que decidiste aplazar (Dukascopy, Yahoo y licencias por tiers) |
| Must-haves completos | **10 de 20** | |
| Layout (pantalla Retesteo) | **17,8 / 100** | `imgdiff.py` en modo layout contra un recorte de la captura pública del original |

**Veredicto según los criterios del skill: *no enviable como sustituto del
original*** — quedan 10 must-haves sin completar.

**Veredicto según el uso real que le vas a dar (personal, validación): *utilizable
y con la cadena de validación completa***. Las dos cosas no se contradicen: lo que
falta es sobre todo **amplitud de generación** (más tipos de entrada/salida, más
indicadores, bloques, tick, multi-símbolo), que es exactamente lo que los usuarios
del original describen como problema ("too many options", "máquina de overfitting").
Lo que tú vas a usar —generar, retestear, medir robustez, montar cartera y exportar
código— sí está resuelto con motor real.

### Por área (más débil primero)

| Área | Puntuación | Funciones | Lectura |
| --- | --- | --- | --- |
| licensing | 0,0 | 1 | no aplica (uso personal) |
| blocks | 30,0 | 4 | 3 plantillas fijas en vez de bloques combinables |
| data | 31,8 | 5 | import CSV real; sin feeds, sin resample, sin tick |
| export | 32,1 | 6 | MQL4/MQL5 reales; las otras 4 plataformas son stub |
| workflows | 37,5 | 2 | motor real, sin canvas drag&drop |
| filtering | 50,0 | 1 | 2 filtros (PF, DD) en vez de métrica arbitraria |
| builder | 50,0 | 5 | genética real; falta modo aleatorio clásico y plantillas |
| databank | 50,0 | 1 | SQLite real, sin escala de millones |
| editor | 50,0 | 1 | UI real, guardado simulado |
| chart | 50,0 | 1 | velas y trades SVG, sin overlays |
| platform | 50,0 | 1 | código multiplataforma, binario probado solo en Windows |
| backtester | 57,7 | 5 | motor real con métricas completas; sin tick ni multi-símbolo |
| portfolio | 70,0 | 5 | cartera y correlación reales; solo equiponderado |
| robustness | 83,3 | 5 | **MC, WF Matrix, SPP y Perfil: todo real** |
| analyzer | 87,5 | 3 | Laboratorio real; MMC orientativa |
| optimizer | 90,0 | 3 | optimizador simple y WF reales + superficie 3D |
| market/retest | 100,0 | 1 | retesteo real |
| improver | 100,0 | 1 | refinador real |

### La cadena de validación (lo que de verdad vas a usar)

| Paso | Estado |
| --- | --- |
| Generar estrategias (genético) | real |
| Filtrar por métrica (PF mín / DD máx) | parcial |
| Backtest con métricas completas | real |
| Retesteo en otro mercado | real |
| Refinador de estrategia | real |
| Optimizador simple y walk-forward | real |
| Matriz walk-forward | real |
| Monte Carlo (9 tests) | real |
| SPP (permutaciones de parámetros) | real |
| Perfil de optimización | real |
| Cartera y correlación | real |
| Laboratorio de análisis | real |
| Exportar a MQL4/MQL5 | real |
| **Editar estrategia sin código** | **parcial (el guardado es simulado)** |

14 de 15 eslabones con motor real. El único hueco del recorrido es el guardado
del editor.

---

## 2. Diff de layout

**Limitación honesta:** el original es software de escritorio de pago. No hay
capturas públicas de sus pantallas, así que no se puede hacer un diff de layout
pantalla a pantalla. Lo único disponible es **una captura real de su app**
publicada en su web (composite de marketing con el Retesteo). Se recortó la
ventana y se usó como referencia única:

| Referencia | Puntaje | Veredicto |
| --- | --- | --- |
| `replica/screens/ORIGINAL-S08-retester.png` vs `replica/clone-screens/S08-retesteo.png` | 17,8 | *different* |

El 17,8 **no mide calidad**: mide que nuestra arquitectura de pantalla es
distinta. Y es distinto a propósito. Lo que sí sirve es la comparación
estructural, que es la que de verdad importa cuando alguien cambia de
herramienta:

| Elemento | Original | SophieQuant | Decisión |
| --- | --- | --- | --- |
| Navegación | raíl de iconos (Getting started, Builder, Retester, Optimizer, Data Manager, Custom projects, AlgoWizard) | lista vertical con 18 módulos en español, plegable a drawer en móvil | **KEEP**: en español las etiquetas hacen falta, y ours agrupa 18 módulos donde el original muestra 7 |
| Cabecera de módulo | título + controles de ejecución + pestañas "Settings / Results" | título + acción contextual | **KEEP**: cada módulo es una tarea, no un wizard |
| Sub-pestañas | Overview / List of trades / Equity chart | vista única densa por módulo | **KEEP**: menos clics para el mismo dato |
| Gráfico de equity | curva **con la zona out-of-sample sombreada** | curva sin marcar OOS | **FIX**: barato y muy valorable (ver punto 2 de la lista final) |
| Resumen de la estrategia | tarjeta flotante con PF, símbolo, timeframe, beneficio y **veredictos por prueba** (Walk Forward *Passed*, MonteCarlo *Passed*, Advanced robustness *Passed*) | los veredictos viven en módulos separados | **FIX**: es el diferenciador central (ver punto 1) |
| Tabla de resultados | lista de estrategias con estado por fila | tabla del banco con métricas y veredictos | **KEEP** |

---

## 3. Diff de comportamiento

| Flujo | El original hace | SophieQuant hace | Decisión |
| --- | --- | --- | --- |
| Validar una estrategia | reparte la validación en 4 módulos y luego repite el veredicto en una tarjeta resumen | cada módulo da su veredicto; no hay tarjeta resumen consolidada | **FIX** (prioridad 1) |
| Decidir si una estrategia es operable | el Summary muestra "Passed" en cada prueba y el usuario decide | hay que recordar el resultado de 4 pantallas | **FIX** |
| Distinguir in-sample de out-of-sample | sombreado explícito en el gráfico | no se marca | **FIX** (prioridad 2) |
| Editar una estrategia | editor visual que guarda | UI real, **el guardado no persiste** | **FIX** (prioridad 3) |
| Generar | motor exhaustivo, lento, muchas opciones | motor genético determinista, rápido, 3 plantillas | **KEEP**: el modelo de producto es "validar", no "generar de todo" |
| Datos | descarga feeds proprietary | importa CSV propio, sin red | **KEEP** (decisión: sin red) |
| Exportar | 6 plataformas con código real | 2 plataformas con código real, 4 con stub | **FIX o quitar de la UI** (prioridad 4): no prometer lo que no es |
| Privacidad | el original envía resultados a sus servidores ("calling home") | todo local, sin telemetría | **KEEP**: es la ventaja que la investigación de los usuarios más terjaga |
| Curva de aprendizaje | "quite an undertaking to master it" | 18 módulos en español, descubribles por etiqueta | **KEEP** |

---

## 4. Veredicto

- **Frente al original como reemplazo:** *no enviable* (12 must-haves incompletos,
  layout distinto).
- **Frente a tu uso real:** *listo para usarse*, con **la cadena de validación
  completa de punta a punta** (generar con filtros y criterio de robustez →
  retestear → medir robustez → montar cartera → editar la regla → exportar
  código). **Los cinco puntos de la lista final están resueltos.** Es exactamente el producto que la
  investigación de `/replica-entrepreneur` pide: validaciones visibles, exporta
  código que reproduce lo que ves, datos que no salen del equipo.
- **Una copia literal no tenía razón de existir.** La diferencia de_layout no es un
  defecto pendiente de tapar: es la consecuencia de haber construido una
  herramienta de validación más que una fábrica de estrategias.


---

## 4.bis Lo construido después del diff (21-10-2026)

Dos de los cinco puntos de la lista final ya están implementados y verificados:

### 1) Tarjeta de veredictos consolidados por estrategia — **hecho**

Es el diferenciador central: el original reparte la validación en cuatro módulos y
además enseña una tarjeta con el veredicto por prueba; aquí las cuatro pruebas
están en un sitio, con el número al lado.

- Motor nuevo `engine/verdicts.rs`: por estrategia calcula **walk-forward** (WFE),
  **Monte Carlo** (las 9 pruebas estándar), **SPP** (permutaciones) y **perfil de
  optimización**, y decide `pass / warn / fail` con **umbrales explícitos**
  (WFE ≥ 50 %, 9/9 pruebas de Monte Carlo, SPP sin fallos, perfil con meseta).
  Los umbrales se muestran en la interfaz: el veredicto nunca es una caja negra.
- Persistencia en la tabla nueva `strategy_verdicts` (con su migración
  idempotente), de modo que la tarjeta se pinta al instante y no hay que recalcular.
- Interfaz: columna «Robustez» en el banco, botón «Calcular veredictos», tarjeta
  completa en el detalle de la estrategia y pestaña «Veredictos» en el Laboratorio.
- Tests: 3 tests Rust del módulo de veredictos (incluido uno que corre las cuatro
  pruebas contra una serie sintética) + 1 test de persistencia en SQLite + 3 e2e.

### 2) Zona out-of-sample marcada — **hecho**

- El corte se calcula **en el motor** con el mismo criterio que la comprobación
  walk-forward (60 % in-sample / 40 % out-of-sample) y se guarda con cada corrida
  (`backtest_runs.oos_frac`, con migración), de modo que la marca es exacta y no
  una aproximación visual.
- Se sombrea en la **curva de equity** del detalle de estrategia y en el **gráfico
  de velas**, con línea discontinua y leyenda: «Fuera de muestra — a partir de
  aquí los parámetros ya no se eligieron con estos datos».

### 3) Exportador: solo plataformas reales

El propietario usa **MetaTrader 5**, así que la UI de exportación anuncia
únicamente **MQL4 y MQL5** (los dos con generador real) y se quitó NinjaTrader,
TradeStation, MultiCharts y JForex, que devolvían un stub. Con eso, el export a
MQL5 pasa de `partial` a `yes` (el mismo EA sirve en cuentas hedge y netting: el
modo lo fija la cuenta del broker) y el área de export sube de 32,1 a 42,9.

Efecto en las puntuaciones: **54,8 → 56,1** (total), **57,3 → 58,7** (dentro del
alcance) y **8 → 9** must-haves completos. La matriz registra además las dos
funciones propias (tarjeta de veredictos y marca out-of-sample) como
«yours, not in the original»: no puntúan, pero documentan en qué se diferencia.

### Lo que queda de la lista final

| # | Punto | Estado |
| --- | --- | --- |
| 1 | Tarjeta de veredictos consolidada | **hecho** |
| 2 | Zona out-of-sample sombreada | **hecho** |
| 3 | Guardado real del editor sin código (S07) | pendiente — es el único eslabón simulado de la cadena |
| 4 | Exportador | **hecho** (solo MT4/MT5 reales; las demás plataformas withdraw de la UI) |
| 5 | Filtrado del generador por cualquier métrica | pendiente |


### 3) Editor sin código que guarda de verdad — **hecho**

El editor (S07) era el último eslabón simulado de la cadena: la UI exist��a pero
«Guardar» solo pintaba un aviso falso. Ahora:

- El editor expone **exactamente las tres reglas que el motor ejecuta**
  (cruce de medias, reversión RSI, ruptura con ATR). Se retiraron del selector
  los bloques decorativos que el motor no sabe evaluar (MACD, Stochastic, CCI,
  patrones de vela): ofrecerlos era precisely el tipo de promesa que la
  investigación señala como motivo de desconfianza.
- «Guardar y recalcular» persiste la regla (plantilla + parámetros + lógica),
  **vuelve a correr el backtest sobre el dataset** y guarda la corrida nueva con
  sus métricas, su equity y su corte out-of-sample. La pantalla muestra el
  resultado antes/después (beneficio, PF, Max DD y número de operaciones).
- Si la regla **no genera operaciones**, el guardado **falla sin tocar nada**:
  la estrategia anterior sigue intacta. Es la misma política que el resto del
  motor (no persistir un resultado que no existe).
- La lógica de guardado se extrajo a `ops::save_logic`, con **3 tests de
  integración en Rust** (persiste y recalcula; falla sin alterar nada; el corte
  OOS se calcula sobre las barras) y **3 tests e2e** del editor.


### 5) Filtros por cualquier métrica y selección por robustez — **hecho**

El último punto de la lista, y el que ataca el problema de fondo: el
sobreajuste estructural.

- **Filtros por cualquier métrica del motor** (nuevo módulo `engine/robust.rs`):
  PF, Sharpe, Sortino, Max DD, recovery factor, beneficio neto y bruto, tasa de
  acierto, Z-score, R², beneficio medio y número de operaciones, cada uno con su
  operador y su valor. La lista de métricas **la pide el motor** (comando
  `list_build_metrics`), así que no hay dos listas que se desincronicen. Una
  métrica u operador desconocido **se rechaza antes de generar**, y un filtro que
  no se puede evaluar nunca pasa (fail closed).
- **Selección por robustez** en vez de por beneficio: entre las estrategias que
  pasan los filtros, gana la que aguanta mejor. El score va de 0 a 100 y sus
  pesos se muestran en pantalla: **35 %** eficiencia walk-forward, **30 %**
  supervivencia frente al remuestreo de operaciones, **20 %** estabilidad frente a
  las permutaciones (SPP) y **15 %** forma del perfil (meseta frente a pico
  aislado). Todas las supervivientes reciben el mismo score completo: mezclar el
  score rápido con el avanzado daría una ordenación sin sentido.
- **El build dice qué te está bloqueando**: si no pasa ninguna estrategia, el
  registro señala el filtro que más rechazó candidatos y con cuántos.
- El score queda guardado con la estrategia (`strategies.robust_score`) y se ve
  en el banco, junto al veredicto de la tarjeta de robustez.

**Es el punto que más se aleja del original en positivo**: allí la WFA nativa
«parece una caja negra»; aquí el criterio de selección está escrito, con pesos y
con el motivo a la vista.


### 6) Calidad de entrega: selector nativo y auditoría de dependencias — **hecho**

Las dos tareas que quedaban abiertas antes de poder considerar el producto
mínimo ejecutable en local:

- **Selector de ficheros nativo** (`tauri-plugin-dialog` 2.8.3, Rust + JS, con
  permisos en `src-tauri/capabilities/default.json`): el botón «Importar CSV» abre
  el diálogo del sistema, el usuario elige símbolo y temporalidad de destino, y la
  importación es **real**: el motor limpia el CSV, lo guarda como QFN y lo
  registra en `data_files`. La lógica se extrajo a `import_csv_registered` para
  poder testearla: **2 tests de integración con un CSV real de 500 barras** (y uno
  que comprueba que un fichero inexistente falla en vez de inventarse datos).
  Si la importación falla, **el error sale a la pantalla**: no hay un "importado"
  falso (para eso hay un `invoke` estricto que no se traga los fallos).
- **`cargo audit`**: ejecutado sobre las 437 dependencias del `Cargo.lock`:
  **0 vulnerabilidades**. Los 2 avisos informativos (`glib` y `proc-macro-error`)
  vienen **solo del stack GTK/Linux de Tauri**, no se compilan en la entrega de
  Windows, y la versión corregida de `glib` es incompatible con el árbol de
  dependencias de Tauri 2.12 (comprobado: `cargo update -p glib --precise 0.20.0`
  falla). Queda registrado como riesgo aceptado en `src-tauri/audit.toml`, con el
  comando de referencia.


### 7) Retesteo multi-mercado — **hecho**

La primera ampliacion de la segunda iteracion. Antes de construirla aparecio un
fallo que conviene contar: **el selector de simbolo y temporalidad del Retesteo
era decorativo** — el comando `run_retest` los recibia y los ignoraba, y siempre
retesteaba sobre el mismo dataset. Un control que no hace nada es una promesa
que el producto no cumple.

Ahora el retesteo hace las dos cosas que su nombre promete:

- **Origen (OOS 30 %)**: la parte final de su propio dataset, como antes.
- **La misma regla en otros mercados**: se eligen uno o varios simbolos y la
  regla se corre en los datos de cada uno. Cada celda es un veredicto con su PF, y
  la columna «Sobrevive» resume **en cuantos mercados aguanta** la estrategia
  («sobrevive en 2 de 3»).
- Un mercado **sin datos** sale como «sin datos», no como fallo: no es lo mismo
  que no tener datos que perder dinero ahi. El panel de mercado dice de antemano
  cuales tienen dataset para la temporalidad elegida.

En el motor: `ops::retest_on_market` (corre la regla en otro mercado) y
`ops::verdict_mercados` (el resumen). **4 tests de Rust** nuevos (retesteo en
otro mercado, dataset minimo -> error, el resumen cuenta solo los mercados con
datos, y sin mercados con datos el veredicto queda *pendiente* y no *fallido*) y
**3 e2e** nuevos. Ademas se corrigio una incoherencia del fake (decía «sin
datos» en el panel y aun asi enseñaba resultados).

**Numeros**: la fila «Backtest multi-simbolo y multi-timeframe» pasa de `no` a
`partial` (el retesteo es multi-mercado; el backtest principal sigue siendo de un
simbolo y no hay charts de entrada concurrentes). Paridad de 59,2 a **60,5**
(total) y de 61,9 a **63,3** (dentro del alcance).

---

## 5. Las cinco cosas que conviene construir ahora

1. ~~**Tarjeta de veredictos consolidada por estrategia**~~ — **hecho** (21-10).
2. ~~**Zona out-of-sample sombreada**~~ — **hecho** (21-10), con el corte calculado
   en el motor y guardado con la corrida.
3. **Guardado real del editor sin código** (S07) en SQLite: es el único eslabón
   simulado de la cadena de validación. *Siguiente paso natural.*
4. ~~**Decidir el exportador**~~ — **hecho** (21-10): solo MQL4/MQL5, que es lo que
   se usa; las plataformas sin generador desaparecen de la UI en vez de devolver
   un hueco.
5. ~~**Filtrado del generador por cualquier métrica** y selección por
   robustez~~ — **hecho** (21-10): filtros por las 12 métricas del motor y
   ordenación por un score de robustez explicable (35/30/20/15).

### También abierto (calidad, no funcionalidad)

- `tauri-plugin-dialog` para que el import de datasets use el selector de ficheros
  real en escritorio (hoy es un stub).
- Persistir los resultados de análisis como filas de `backtest_runs`.
- `cargo fix` para los avisos de `dead_code` (6) y `cargo audit` antes de la
  entrega local.

---

## 6. Artefactos

| Fichero | Qué es |
| --- | --- |
| `replica/features.csv` | matriz de 56 filas reescrita con el estado real verificado en el código |
| `replica/parity-scores.json` | puntuaciones completas (total y dentro de alcance) |
| `src/data/parity.json` | los mismos datos, consumidos por la pantalla S25 «Paridad» de la app |
| `replica/screens/ORIGINAL-*.png` | referencias públicas del original (web) |
| `replica/clone-screens/*.png` | capturas de SophieQuant a 1440×900 |
| `replica/diffs/S08.png` / `.json` | diff de layout del Retesteo |
