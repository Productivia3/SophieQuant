# Verificación del export a MetaTrader

Fecha: 21-10-2026 · Estado: activo (la app lo muestra al generar el EA)

## Por qué existe

La queja más repetida sobre el producto de referencia no es «el software es
malo», sino esta:

> «the results of the strategies do not match» — al exportar a MT, los resultados
> del backtest y los del broker no coinciden.

Y el motivo no suele ser un fallo del export: es que **el backtest y el broker
no pueden ser iguales** y nadie te dice dónde se diferencian. Este documento y el
panel de verificación que aparece en la app al generar el código son la
respuesta: en vez de prometer paridad imposible, se declara **qué se ha asumido**
y se da una lista para comprobarlo.

## Lo que el backtest ha asumido (se muestra en la app, con los valores reales)

| Supuesto | Consecuencia en MetaTrader |
| --- | --- |
| Los datos son de **barras**, no de ticks | Dentro de la barra, el EA ve un precio que el backtest nunca vio. Las entradas se Filling en el tick real, no en el cierre de la barra. |
| Spread, slippage y comisión del modelo propio | Tu broker puede aplicar otros. Con spreads altos, el resultado se degrada. |
| Posición **fija** y balance inicial fijo | No hay gestión de capital dinámica ni pirámide. El EA exportado tampoco la tiene. |
| Solo **largo** (entrar y salir) | Sin posiciones cortas. |
| Sin horarios ni filtros de sesión | La regla actúa en todas las barras. |
| El diagnóstico de intervalos no conoce las sesiones del símbolo | Los intervalos estimados pueden incluir cierres normales; comprueba el calendario de negociación antes de concluir que faltan barras. |

## Checklist para comprobar el EA en MetaTrader

1. Carga el EA con los **valores por defecto** del archivo, sin tocarlos.
2. Usa el **mismo símbolo y la misma temporalidad** que en el backtest.
3. Comprueba el **spread real** del broker ahora y compáralo con el supuesto. Si
   difiere mucho, el resultado cambiará y no es un fallo del export.
4. Comprueba si tu broker **cobra comisión por lote**. Si la cobra, ajústala en el
   input del EA antes de comparar.
5. **Cuenta las operaciones**: deben ser similares a las del backtest. Si difieren
   mucho, la lógica no está bien portada (eso sí sería un fallo del export).
6. Mira el **informe del tester de MT5** y compara el equity línea a línea con el
   del backtest.
7. Compara los intervalos estimados con las sesiones del símbolo. Las barras
   realmente ausentes durante horas de negociación pueden afectar las señales;
   los cierres normales no son datos perdidos. El volumen cero también limita
   la información disponible.
8. Recuerda: el backtest es **en largo y de posición fija**. Si añades gestión de
   capital al EA, los números dejan de ser comparables.

## Qué NO puede hacer este export (y no te lo vamos a vender)

- **No es un motor tick.** Los resultados en ticks del broker **no** tienen por qué
  coincidir. Si necesitas ese nivel de fidelidad, el bloque pendiente es un
  backtester tick-level (y el export tendría que consumir los mismos ticks).
- **No hay gestión de capital** (ni pirámide, ni ATR dynamic sizing, ni salidas
  escalonadas tipo ATM). El EA es de una entrada y una salida.
- **No hay short.** La versión actual es solo largo.
- **No reproduce los filtros de la generación**: si la estrategia se generó con
  filtros de robustez, eso no viaja al EA (y no debería: son criterios de
  investigación, no parte de la regla).

## Cómo se calcula en el motor

`engine/export.rs::verification()` genera el bloque a partir de:

- la plantilla y los parámetros de la estrategia (los mismos del código);
- la **configuración real con la que se corrió el backtest**
  (`backtest_runs.engine_cfg`): spread, slippage, comisión, lote y balance.

Por eso el texto no es genérico: si cambias el spread en Ajustes, cambia el
supuesto que se muestra.

## Diagnóstico de calidad del CSV importado

Relacionado con lo anterior: el importador de CSV ahora **diagnostica el
fichero** y lo dice (`engine/datastore.rs::import_csv`):

- filas descartadas por estar mal formadas;
- **intervalos sin barras estimados** (y el mayor de ellos; el cálculo no conoce
  las sesiones del instrumento);
- barras **sin volumen**;
- precios **incoherentes** (close fuera de [low, high]);
- **timestamps repetidos**.

Un intervalo estimado no demuestra que falten datos: puede corresponder al
cierre diario, al fin de semana o a un festivo. Una barra realmente ausente
durante la negociación puede afectar el backtest; el informe no determina cuál
de estos casos aplica sin el calendario del símbolo.
