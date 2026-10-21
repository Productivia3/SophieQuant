# Análisis crítico del proyecto: qué le falta para ser un programa experto

Fecha: 21-10-2026 · Alcance: motor, datos, validación, producto · Base: revisión del
código real (`src-tauri/src/engine/`, `src-tauri/src/commands.rs`), no del documentación.

---

## 0. La premisa hay que corregirla antes de construir nada

El objetivo que hasplantado es **«encontrar estrategias de trading rentables para
cualquier configuración»**. Voy a ser directo, porque es la decisión de producto más
importante que vas a tomar:

**Ese objetivo es imposible, y es exactamente el que hace que los productos de
trading seaniserie mala fama.**

- Es imposible: no existe un generador que encuentre estrategias rentables *para
  cualquier configuración*. Si existiera, cualquier strategy would be arbitraria
  rentable y el mercado dejaría de funcionar. Lo que se puede aspirar no es «encontrar
  rentables», sino **«no encontrar nada por accidente, y demostrar que lo que
  encuentra sobrevive fuera de muestra»**.
- Es el objetivo que la investigación de tus propios usuarios califica de
  peaje: *«The overfitting risk is structural, not incidental»*, *«only 45 % of
  strategies survive a 1-year forward test even after 5 years of training»*,
  *«the native WFA felt like a black box»*.
- Y es el que explica la queja más repetida del producto de referencia: *«the
  results of the strategies do not match»* al exportar a MT.

**El objetivo correcto para un programa experto es la tasa de falsos positivos
cercana a cero, no la tasa de hallazgos.** Un programa que genera 500 estrategias
y dice «ninguna sobrevive» es **mucho más útil** que uno que te da 12 strategies
«rentables» de las que 11 son ruido. Y, paradójicamente, es lo que la gente
paga: no quiere señales, quiere saber si su signal es real.

Todo lo de abajo va en esa dirección. some mejoras que sí propongo (más
indicadores, más plantillas) están deliberadamente al final y marcadas como
**no prioritarias**, porque multiplican el sobreajuste en vez de reducirlo.

---

## 1. Hallazgos verificados en el código

Ordenados por impacto. Cada uno indica el fichero y el efecto.

### 1.1 El motor asume Forex y descarta los metadatos que ya tiene — **crítico**

`EngineCfg::default()` se usa en **19 sitios** (`commands.rs`, `ops.rs`). Su
`pip_size` es 0,0001 y el P&L asume **1 lote = 100 000 unidades**.

Pero la tabla `symbols` **ya tiene** `tick_size`, `pip_size` y `commission`, y la
semilla los rellena bien (`USDJPY` con pip 0,01). El motor **nunca los lee**.

Consecuencias concretas:

| Símbolo | Qué calcula el motor | Qué debería calcular |
| --- | --- | --- |
| EURUSD | correcto | correcto |
| USDJPY | spread y slippage **100× menores** de lo real (pip 0,0001 en vez de 0,01) → resultado **optimista** | costes reales |
| ES (futuro) | 1 punto = 100 000 USD | 1 punto = 50 USD (multiplicador) → P&L **errático** |
| AAPL / BTC | igual: 100 000 por punto | tamaño de contrato / notional |

**Hoy el backtest solo es válido para pares USD-quote de Forex.** Eso reduce
muchísimo el universo con el que puedes trabajar, y es un bug silencioso: no
falla, da números.

### 1.2 El Max DD se calcula sobre una curva muestreada — **crítico**

`backtest.rs`:
```rust
let mut equity = metrics::equity_series(&out.trades, cfg.initial_balance, 500);
out.metrics = metrics::compute(&out.trades, &equity, cfg.initial_balance);
```
`equity_series` **muestrea a 500 puntos** y `compute` calcula el drawdown sobre
esa muestra. Con más de 500 operaciones —lo normal en m1— el **drawdown real
queda subestimado**, y como el DD es también filtro, el filtro deja pasar
estrategias que en realidad toleraban más caída de la que crees. El RDD y las
tarjetas de riesgo heredan el mismo error.

### 1.3 No hay nada de significación estadística — **crítico**

Búsqueda literal en el motor: **cero** apariciones de *deflated Sharpe*,
*PBO*, *significancia*, *corrección por múltiples intentos*.

Esto es el corazón del asunto. Una build prueba **4 000 iteraciones** por
configuración (`n_iterations: 4000`). El mejor de 4 000 intentos sesga
sistemáticamente hacia arriba: aunque la estrategia fuese ruido puro, el
ganador delazar **parecerá** rentable. Sin corrección, el motor te está
mintiendo con precisión estadística.

Falta:
- **Deflated Sharpe Ratio** (Bailey & López de Prado): ajustar el Sharpe por el
  número de intentos y por la duración de la muestra.
- **PBO** (probability of backtest overfitting): reparticiones CSCV para
  estimar cuantos de tus «mejores» son ruido.
- **Regla de mínimo de muestra** (operaciones y años) y, sobre todo, que el
  programa pueda **terminar sin resultados**.

### 1.4 El «walk-forward» no re-optimiza por ventana — **alto**

`wfe_of` hace **un único corte 60/40** sobre el dataset: no hay ventanas
encadenadas. La «Matriz WF» es una rejilla de WFE calculada **sobre los mismos
datos**, y el optimizador en modo `wf` se limita a informar del WFE de cada
celda (el `wfDiscount` del fake del navegador es un parche visual, no una
validación real).

El walk-forward de verdad es **rolling**: en cada ventana *i* se optimiza, y el
resultado se puntúa sobre la ventana *i+1* (nunca vista). La métrica de la
estrategia es la **concatenación de los tramos OOS**, no el in-sample. Sin eso,
toda la sección de robustez es decorativa.

### 1.5 El espacio de búsqueda es diminuto — **alto (para el objetivo tal como lo planteaste)**

`genetic.rs` muta **3 números** sobre **3 plantillas fijas**. No hay:
representación de reglas por bloques (árboles), ni crossover, ni condiciones
compuestas, ni multitimeframe, ni «cualquier configuración» en ningún sentido
utilizable. Con 3 plantillas × 3 parámetros continuos, el programa **no puede
encontrar nada que no sea un cruce de medias o un RSI**. Ese es el techo real
del producto hoy, y conviene decirlo sin rodeos.

### 1.6 No hay swap ni coste overnight — **alto**

Cero apariciones de *swap*, *overnight*, *interest* en el motor. Una estrategia
que aguanta tres sesiones tiene un resultado dominado por los swaps, y el motor
no los conoce. En Forex, en estrategias de medio plazo, el swap es del mismo
orden que el spread.

### 1.7 El spread es constante — **medio**

`spread` es un número fijo en pips, igual las 24 horas. El spread real se
multiplica por 3-5 en la apertura de Londres/NY y se estrecha en Asia. Para
intradía, esto solo (sin swap) puede explicar la mitad de la diferencia entre
tu backtest y el broker.

### 1.8 El Sharpe de la app no es el Sharpe de MT5 — **medio**

`metrics::compute` calcula Sharpe y Sortino sobre **retornos por operación**, sin
annualizar. Son números sin significado comparables con los de una serie de
equity por tiempo. Cuando compares el resultado de la app con el del tester de
MT5, **no serán el mismo número**, y el usuario lo leerá como un bug.

### 1.9 El Monte Carlo no usa bootstrap por bloques — **medio**

Las 9 pruebas (`shuffle`, `skip`, `randomize_params`, `use_variants`,
`ohlc_interpolation`, `simulation_returns`, `drawdown`, `duplicate`,
`skip_certain`) son buenas, pero `shuffle` reordena operaciones **iid**: los
retornos de una estrategia tienen autocorrelación (rachas), y el iid subestima
el riesgo. Falta el **block bootstrap** (resampling por bloques) y la prueba
clásica de «**quita las N peores operaciones**» y «**mueve las entradas en el
tiempo**» (esta última está en el SPP, pero como variante de regla, no como
prueba de robustez del resultado).

### 1.10 No hay datos: no hay programa — **alto**

No hay fuente de datos integrada (el downloader se pospuso por decisión tuya) y
solo hay import de CSV. **Sin datos no hay programa.** Y para tu caso hay una
vía que no hemos tocado: **tú ya tienes MetaTrader 5**, cuyo terminal exporta
histórico (incluido tick) y además corre el Strategy Tester. Eso abre dos
posibilidades que valen más que cualquier feed de pago:

1. **Importador del histórico de MT5** (misma resolución y formato que el broker).
2. **Comparador backtest ↔ tester de MT5**: subes el informe del tester (HTML) y
   la app te dice **dónde divergen** (nº de operaciones, timings, spread real).
   Eso convierte la queja «los resultados no coinciden» en un proceso medido.

### 1.11 No hay forward test ni verificación con el broker — **alto**

El ciclo de la verdad (backtest → forward en demo → broker) no existe. Todo lo
que el programa dice está validado **contra su propio modelo**, que es
precisamente el problema que la gente denuncia: *«me fío porque no sé qué sale
de mi máquina»*.

### 1.12 El programa no sabe decir «no hay nada» — **medio**

La build puede terminar con 0 estrategias, pero **no lo explica**. Un programa
experto cierra con: «de 4 000 intentos, 37 pasaron el filtro de PF; el que más
bloqueó fue el de drawdown; ninguna superó el mínimo de 200 operaciones en
out-of-sample; **recomendación: baja el listón o cambia de símbolo**». Eso es
información accionable; un «0 resultados» sin más es unafstrella.

### 1.13 No hay registro de experimentos — **medio**

El historial guarda análisis **por estrategia**, pero no responde a «¿ya he
probado esto con estos datos?». Falta un registro de experimentos: dataset +
configuración + número de intentos + resultado + veredicto, consultable. Sin él,
repetirás trabajo sin darte cuenta.

---

## 2. Lo que está bien hecho (y no hay que tocar)

- Determinismo total (mismo seed → mismo resultado) y tests de determinismo.
- Sin lookahead: la señal de la barra *i* se ejecuta en la apertura de *i+1*.
- Coste de spread, slippage y comisión en el cálculo del beneficio.
- El motor calcula MAE/MFE y la curva de capital diario (ya hecho).
- Los 9 tests de Monte Carlo con perturbaciones realmente distintas.
- La trazabilidad: cada estrategia guarda su configuración, su dataset y su corrida.
- La honestidad del producto: umbrales visibles, Supuestos del export declarados,
  diagnóstico de calidad del CSV, «sin datos» ≠ «fallo».
- Todo el tema de la privacidad: nada sale del equipo, sin telemetría.

---

## 3. Plan por bloques

Orden **por lo que cambia la confianza en los resultados**, no por lo que
impressiona. Los bloques 0 y 1 son los que convierten esto en un programa cuyo
veredicto se puede creer.

### Bloque 0 — Arreglar el motor (1-2 días) · sin esto, los números mienten

1. **Cfg por símbolo**: leer `pip_size`, `tick_size` y `commission` de la tabla
   `symbols` (ya existen) y construir el `EngineCfg` a partir de ahí. Añadir
   `contract_size` (futuros) y `swap_long`/`swap_short` ( Forex).
2. **Max DD sobre la curva real**: calcular el drawdown y el RDD con la serie
   completa de operaciones; usar la muestreada **solo para pintar**.
3. **Sharpe por periodo y anualizado**: dos métricas separadas y con nombre
   explícito («Sharpe por operación» / «Sharpe anualizado»), para que se
   pueda comparar con MT5 sin confusiones.
4. **Swap overnight**: aplicar el coste de cada nocheheld a las operaciones
   que la cruzan.
5. **Spread por sesión**: como mínimo, tres perfiles (Asia / Londres / NY) en
   vez de un número fijo.

### Bloque 1 — Validación que se puede creer (3-5 días) · el bloque que más valor da

6. **Walk-forward rolling de verdad**: optimizar en la ventana *i*, puntuar en la
   *i+1*; la estrategia se mide por la concatenación OOS. Rediseñar la «Matriz
   WF» y el optimizador con este criterio, y mostrar **la equity OOS** (que es la
   única que cuenta).
7. **Deflated Sharpe Ratio y PBO**: que cada estrategia salga con
   «Sharpe deflactado 0,4 (esperado por azar con 4 000 intentos: 0,9)» y
   «probabilidad de sobreajuste 68 %». Es la diferencia entre «esto parece
   rentable» y «esto probablemente es ruido».
8. **Reglas de mínimo y de parada**: mínimo de operaciones (p. ej. 200) y de
   años (p. ej. 3) **en la parte out-of-sample**; si no se cumple, la estrategia
   se marca como «sin muestra suficiente» y no compite.
9. **Block bootstrap + «quita las peores operaciones»** en el Monte Carlo.
10. **Veredicto con explicación**: por qué se descartó cada candidata y cuál fue
    el filtro que más bloqueó.

### Bloque 2 — Datos (2-4 días) · sin datos no hay programa

11. **Importador del histórico de MT5** (el que ya tienes) y del informe del
    Strategy Tester.
12. **Comparador backtest ↔ tester**: el informe HTML del tester se importa y se
    comparan operaciones, timings y equity. Cierra la queja más repetida.
13. **Metadatos completos por símbolo**: contract size, swaps, sesiones, zona
    horaria, trading days.

### Bloque 3 — Generador creíble (5-10 días) · solo si el bloque 1 ya está

14. **Reglas como árboles de bloques** (indicadores, comparaciones, cruces,
    entradas/salidas) con crossover y mutación sobre el árbol, y **una sola
    definición** que alimenta el motor **y** el export a MQL (que es lo que
    garantiza que lo que ves es lo que se ejecuta).
15. **Multitimeframe de verdad**: una regla puede leer m15 y h1 a la vez.
16. **Más indicadores solo si el export los porta** (MACD, CCI, Stoch): la regla
    es «si el export no lo sabe hacer, el bloque no existe».

### Bloque 4 — Producto (2-3 días)

17. **«No he encontrado nada» como resultado de primera clase**, con informe de
    por qué y qué haría falta para encontrar algo.
18. **Registro de experimentos** consultable (dataset, configuración, intentos,
    resultado).
19. **Informe de estrategia exportable** (Markdown/PDF) con la regla, los
    veredictos, el historial y la checklist de MT5.

---

## 4. Lo que NO hay que hacer

- **No** añadir 40 plantillas ni un editor visual de bloques: multiplican el
  espacio de búsqueda y, con el bloque 1 sin hacer, **empeoran** el producto.
  La complaint real («too many options») es exactamente eso.
- **No** un motor tick ahora. Es caro (otro motor, otro volumen de datos) y su
  retorno depende de haber arreglado antes los costes y la validación. Cuando
  llegue, tiene que ser para que el EA y el backtest usen **los mismos ticks**.
- **No** un «Q» de IA que proponga reglas: una IA que propone estrategias sobre
  datos sin validación out-of-sample solo acelera el sobreajuste.
- **No** publicar nada ni añadir licencias: es una herramienta personal.

---

## 5. La respuesta corta a tu pregunta

**¿Qué falta para que este programa encuentre estrategias rentables para
cualquier configuración?** Falta casi todo lo que hace que un resultado sea
real, y ninguna de las dos piezas de «más configuración»:

- **Falta que los números sean ciertos** (bloque 0): hoy el motor no usa los
  metadatos del símbolo, subestima el Max DD y no conoce los swaps. Cualquier
  estrategia que mida sobre estos números está mal medida.
- **Falta que la validación sea real** (bloque 1): el walk-forward no re-optimiza
  por ventana y no hay corrección por los 4 000 intentos. Sin eso, el programa
  te da exactamente el «overfitting estructural» del que se queja la gente.
- **Falta que haya datos** (bloque 2) y la vuelta al broker (comparador con el
  tester de MT5).
- **Falta poder decir «no»** (bloque 4).

**¿Y la amplitude de reglas?** Es lo más fácil de añadir y lo que menos
rentable: sin el bloque 1, más reglas significan más falsos positivos. Por eso
va el último y marcada como tal.

Con los bloques 0, 1 y 2, este programa deja de ser «un generador que produce
resultados» y pasa a ser **«un instrumento que dice si un resultado es real»**.
Eso es exactamente el hueco que la investigación de tus usuarios décritió, y es
el único que no tiene la competencia.
