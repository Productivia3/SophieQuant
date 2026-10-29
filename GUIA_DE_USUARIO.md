# Guía de usuario de SophieQuant

Manual práctico para utilizar SophieQuant sin experiencia técnica. Está pensado
para la **aplicación de escritorio de Windows** y describe las funciones que hay
en esta versión.

> **Importante:** SophieQuant sirve para investigar y probar ideas, no para
> recomendar inversiones. Un resultado positivo en un histórico no garantiza
> que una estrategia vaya a ganar dinero en el futuro. No conectes una estrategia
> nueva a una cuenta real sin revisarla y probarla primero en una cuenta demo.

## Contenido

1. [Antes de empezar](#1-antes-de-empezar)
2. [El recorrido recomendado](#2-el-recorrido-recomendado)
3. [Preparar e importar los datos](#3-preparar-e-importar-los-datos)
4. [Generar estrategias](#4-generar-estrategias)
5. [Entender el Banco y los resultados](#5-entender-el-banco-y-los-resultados)
6. [Probar si una estrategia es resistente](#6-probar-si-una-estrategia-es-resistente)
7. [Editar y mejorar una estrategia](#7-editar-y-mejorar-una-estrategia)
8. [Carteras y Laboratorio](#8-carteras-y-laboratorio)
9. [Exportar a MetaTrader](#9-exportar-a-metatrader)
10. [Qué hace cada módulo](#10-qué-hace-cada-módulo)
11. [Problemas frecuentes](#11-problemas-frecuentes)
12. [Glosario sencillo](#12-glosario-sencillo)
13. [Privacidad y copia de seguridad](#13-privacidad-y-copia-de-seguridad)

## 1. Antes de empezar

### Comprueba que has abierto la aplicación correcta

En la parte inferior de la ventana debe aparecer **«núcleo Rust conectado»**.
Eso indica que estás usando la aplicación de escritorio con el motor de cálculo.
Si aparece **«núcleo simulado (navegador)»**, estás en una vista de demostración:
sus datos y resultados pueden ser ficticios y no debes usarlos para sacar
conclusiones ni para exportar una estrategia.

### Qué necesitas

- SophieQuant abierto en el ordenador.
- Un archivo histórico en formato CSV con precios de mercado.
- Saber a qué símbolo pertenece el archivo (por ejemplo, EURUSD) y qué
  temporalidad contiene (por ejemplo, m1 para barras de un minuto).
- Algo de tiempo para revisar los resultados: el programa no decide por ti si
  una estrategia es adecuada.

El programa no se conecta a tu bróker ni ejecuta operaciones. La función
**«Descargar m1»** del módulo **Datos** no descarga un histórico real en esta
versión. Para trabajar con precios reales, importa un CSV que hayas obtenido de
tu plataforma o de una fuente de datos de confianza.

### Cómo moverte por la aplicación

El menú lateral contiene los módulos. En una pantalla estrecha, pulsa el botón
de menú (☰) para abrirlo. Para el trabajo habitual, el recorrido principal es:

**Gestor de datos → Constructor → Banco de estrategias → pruebas de robustez →
Exportar código**.

## 2. El recorrido recomendado

Si es la primera vez, sigue este orden:

1. Importa un CSV desde **Gestor de datos**.
2. Comprueba el símbolo, la temporalidad y el informe de calidad.
3. Abre **Constructor** y genera una tanda con los valores iniciales.
4. Abre el **Banco de estrategias** y entra en las estrategias generadas.
5. Calcula y revisa los veredictos de robustez. Repite pruebas fuera de la
   muestra original cuando tengas datos adecuados.
6. Solo si entiendes los resultados, genera código para MT4 o MT5.
7. Compara el código en MetaTrader usando el probador y, después, una cuenta
   demo. No lo pongas directamente en una cuenta real.

No pasa nada si el Constructor no encuentra estrategias que superen los filtros.
Es preferible obtener cero resultados a aceptar resultados que no cumplen los
criterios elegidos.

## 3. Preparar e importar los datos

### Importar un archivo

1. En el menú, abre **Gestor de datos**.
2. En **Importar a**, elige el símbolo al que corresponden los precios.
3. Elige la temporalidad real de las barras:
   - `m1`: un minuto
   - `m5`: cinco minutos
   - `m15`: quince minutos
   - `m30`: treinta minutos
   - `h1`: una hora
   - `h4`: cuatro horas
   - `d1`: un día
4. Pulsa **Importar CSV** y selecciona el archivo en la ventana de Windows.
5. Espera el mensaje de confirmación y revisa **Calidad del fichero importado**.
6. En la tabla, confirma que aparece el símbolo, la temporalidad, un número de
   barras y el estado **LISTO**.

El archivo debe tener una fila por barra y valores de **fecha/hora, apertura,
máximo, mínimo y cierre**. El volumen es opcional. Se admiten separadores por
coma, punto y coma o tabulador; también son habituales la fecha y la hora en
columnas separadas. Si los decimales se escriben con coma, utiliza punto y coma
para separar las columnas. Ejemplos:

```text
time,open,high,low,close,volume
2024-01-02 09:00:00,1.1000,1.1020,1.0990,1.1010,100
```

```text
Date;Time;Open;High;Low;Close;Tickvol
02.01.2024;09:00:00;1,1000;1,1020;1,0990;1,1015;100
```

Este segundo formato usa punto y coma y coma decimal. Los CSV con cabecera
`<DATE>`, `<TIME>` y columnas separadas por tabuladores también son compatibles.

### Revisar la calidad

- **Barras**: cantidad de observaciones que se pudieron leer.
- **Filas descartadas**: líneas vacías, incompletas, con fecha ilegible o precios
  inválidos. Si se descartan casi todas, revisa el separador, los encabezados y
  el formato de fecha del archivo.
- **Huecos**: intervalos sin barras dentro del histórico. Pueden ser normales
  durante cierres del mercado, pero comprueba que no sean cortes inesperados.
- **Sin volumen**: barras sin dato de volumen; el programa puede seguir, pero
  ciertas lecturas serán menos completas.
- **Precios incoherentes**: valores donde apertura o cierre no encajan entre el
  máximo y el mínimo. Investiga el archivo antes de continuar.

Un CSV con formato válido no garantiza que sus precios sean correctos ni que el
horario coincida con el de tu bróker. Comprueba el símbolo, las fechas y la
temporalidad antes de generar.

### Reutilizar una temporalidad

En la tabla del Gestor de datos puedes elegir una temporalidad mayor y pulsar
**Resamplear**. Esto agrupa barras existentes (por ejemplo, m1 en m5); no crea
información nueva ni sirve para convertir barras grandes en pequeñas.

## 4. Generar estrategias

1. Abre **Constructor**.
2. En **Símbolo**, elige el mismo mercado que importaste.
3. En **Timeframe**, elige la temporalidad que aparece como lista para ese
   símbolo.
4. En **Plantilla base**, deja una de las plantillas disponibles. En esta versión
   son:
   - **Cruce de medias**: compara una media rápida y otra lenta.
   - **Reversión osciladora**: usa niveles del RSI para buscar rebotes.
   - **Ruptura con filtro de tendencia**: busca superar un rango de precios.
5. Deja inicialmente **Estrategias objetivo**, **Operaciones mínimas** y los
   filtros en sus valores predeterminados. Los filtros iniciales son un factor
   de beneficio mínimo de 1,15 y un drawdown máximo del 35 %.
6. Deja activado **Robustez en vez de beneficio** si prefieres ordenar por
   resistencia a las pruebas, no solo por beneficio.
7. Pulsa **Generar estrategias**.

La pantalla de generación muestra el estado, el número de estrategias pasadas y
el registro de actividad. Puedes:

- **Pausar** para detener temporalmente el trabajo.
- **Reanudar** para continuarlo.
- **Detener** para cancelar esa generación.
- **Ver Banco de estrategias** para abrir los resultados guardados.

El progreso se guarda localmente y puede recuperarse si refrescas la ventana. Si
el banco queda vacío, no significa necesariamente que el programa se haya roto:
puede que ninguna candidata cumpliera todos los filtros. Revisa el registro y,
si necesitas una búsqueda más amplia, cambia un criterio cada vez para saber
qué efecto tuvo.

## 5. Entender el Banco y los resultados

### Banco de estrategias

El **Banco de estrategias** reúne las estrategias que superaron los filtros.
Pulsa **Calcular veredictos** para ejecutar las comprobaciones disponibles y
selecciona una fila para abrir su ficha.

### Métricas principales

- **Beneficio neto**: resultado acumulado del backtest bajo los supuestos
  indicados.
- **PF (Profit Factor o factor de beneficio)**: beneficio bruto dividido por
  pérdidas brutas. Un número superior a 1 indica que, en ese histórico, el
  beneficio bruto superó las pérdidas brutas; no mide por sí solo el riesgo.
- **Tasa de acierto**: porcentaje de operaciones que cerraron con beneficio.
  Una tasa alta no garantiza que el resultado total sea bueno: las pérdidas
  pueden ser mayores que las ganancias.
- **Max DD (drawdown máximo)**: mayor caída de la curva desde un máximo previo.
  Cuanto mayor sea, mayor habría sido la caída histórica.
- **Trades**: cantidad de operaciones del backtest. Pocas operaciones ofrecen
  poca evidencia.
- **Sharpe**: compara resultado y variación; el detalle distingue el Sharpe por
  operación del anualizado.
- **Robustez**: puntuación resumida de varias pruebas. Úsala como orientación,
  no como garantía ni como sustituto de revisar cada prueba.

La ficha de una estrategia también muestra una curva de capital (**equity**).
La zona **fuera de muestra** (OOS) representa datos que el motor reserva para
comprobar la regla fuera del tramo principal. Si una estrategia aparece como
**«marcada para revisión»** o **dirty**, no la exportes sin volver a comprobarla.

En la ficha puedes pulsar **Calcular perfil de riesgo** para ver métricas
adicionales como MAE y MFE. En general, compara siempre la curva y el número de
operaciones, no solo el beneficio final.

## 6. Probar si una estrategia es resistente

Estas pruebas responden preguntas diferentes. Un resultado **PASSED** en una
prueba concreta no demuestra que la estrategia vaya a funcionar en el futuro.

### Retesteo

1. Abre **Retesteo**.
2. Marca una o varias estrategias del banco.
3. Elige una temporalidad.
4. Marca los mercados para los que tienes datos y ejecuta el retesteo.

La misma regla se evalúa en los mercados seleccionados. **«Sin datos»** significa
que no se pudo probar en ese mercado; no significa que la estrategia haya
fallado allí.

### Matriz WF (Walk-Forward)

Selecciona una estrategia y pulsa **Correr matriz**. El mapa de calor muestra
cómo cambia la eficiencia fuera de muestra al variar combinaciones de
parámetros. Busca grupos de resultados estables, no una única casilla muy
buena. Si aparece el aviso de muestra insuficiente, los datos no alcanzan para
confiar en esa conclusión.

### Monte Carlo

Elige una estrategia, deja seleccionadas las pruebas que quieres realizar y
pulsa **Correr simulaciones**. Las simulaciones alteran el orden o la composición
de las operaciones para ver cuánto depende el resultado de una secuencia
concreta. Un porcentaje de aprobado es una medida de esas simulaciones, no la
probabilidad de ganar dinero mañana.

### Robustez (SPP/Perf) y Perfil de optimización

- **SPP** cambia piezas o parámetros de la regla para observar si el resultado
  se mantiene o se desploma.
- **Perfil de optimización** dibuja el resultado al cambiar un parámetro. Una
  zona relativamente amplia y estable suele ser más tranquilizadora que un pico
  aislado.

### Optimizador

Selecciona una estrategia y compara **Simple** o **Walk-Forward**. El gráfico
representa combinaciones de parámetros y su factor de beneficio. El mejor punto
no es automáticamente la mejor elección: escoger el máximo absoluto puede
ajustarse demasiado al histórico. Prioriza zonas consistentes y vuelve a probar
fuera de muestra.

## 7. Editar y mejorar una estrategia

### Editor de regla

Desde la ficha de una estrategia, pulsa **Editar regla**. La aplicación permite
editar las tres plantillas disponibles y sus parámetros. Si pulsas **Guardar y
recalcular** en la aplicación de escritorio, la regla se guarda y el motor
vuelve a ejecutar el backtest sobre su dataset. Comprueba las métricas nuevas y
los veredictos; no des por hecho que editarla la mejora.

Si ves **«Vista previa»**, estás en el navegador y no en la aplicación de
escritorio: la regla no se guarda ni se ejecuta en el motor real.

### Refinador

Selecciona una estrategia del banco y pulsa **Mejorar**. El Refinador compara
variantes de su regla con la original. Si ninguna variante la supera, conserva
el original. Revisa siempre si la mejora también aguanta las pruebas de
robustez; una mejora solo en el mismo histórico puede ser sobreajuste.

## 8. Carteras y Laboratorio

### Cartera

1. Abre **Cartera** y selecciona entre 2 y 8 estrategias.
2. Puedes pulsar **Sugerir cartera** para obtener una propuesta y revisar qué
   estrategias se incluyen o descartan.
3. Elige un modelo:
   - **Inverso a volatilidad**: asigna menos peso a las estrategias que más
     fluctúan.
   - **Paridad de riesgo**: intenta repartir de forma más uniforme la
     contribución al riesgo.
   - **Equiponderado**: asigna pesos iguales.
   - **Personalizado**: escribes los pesos que quieres comparar.
4. Pulsa **Analizar cartera** y revisa correlaciones, pesos, drawdown y curva
   combinada.

La cartera combina resultados históricos; no sincroniza órdenes con un bróker ni
gestiona dinero real.

### Laboratorio

El Laboratorio ofrece resumen, veredictos, historial, simulación Monte Carlo de
cartera y una vista orientativa de gestión de capital. Para el resumen de
estrategias, utiliza el banco local. El botón **Importar resultados o CSV** no
convierte automáticamente cualquier fichero externo en estrategias del banco
en esta versión; no lo uses como importador general de datos de mercado.

## 9. Exportar a MetaTrader

1. Abre **Exportar código fuente**.
2. Selecciona una estrategia del banco.
3. Elige **MetaTrader 5** o **MetaTrader 4**.
4. Pulsa **Generar código**.
5. Lee la sección **Verificación antes de fiarte del EA**. Revisa sus supuestos,
   parámetros y checklist.
6. Copia el código mostrado y guárdalo manualmente desde el Bloc de notas:
   - Para MT5, usa una extensión `.mq5`.
   - Para MT4, usa una extensión `.mq4`.
   - En el Bloc de notas, elige **Guardar como tipo: Todos los archivos** para
     evitar que se añada `.txt` al final.
7. Abre MetaTrader y su MetaEditor, coloca el archivo en la carpeta `MQL5\Experts`
   (MT5) o `MQL4\Experts` (MT4), compílalo y pruébalo con el probador de
   estrategias.
8. Compara las operaciones y resultados con el backtest. Si no coinciden,
   detente e investiga la diferencia. Prueba después en demo.

**Limitación conocida:** el botón **Guardar fichero…** de esta versión es una
confirmación simulada y no guarda realmente el archivo. Copiar el código a un
archivo `.mq4` o `.mq5` es el método disponible. SophieQuant exporta código; no
lo instala en MetaTrader ni lo activa en tu cuenta.

El motor tiene tres plantillas disponibles y genera estrategias solo de compra
(largas) en esta versión. Los costes, el tamaño de posición y otros supuestos
del backtest pueden diferir de los de tu bróker. Comprueba la lista que aparece
al generar el EA antes de comparar resultados o plantearte usarlo.

## 10. Qué hace cada módulo

| Módulo | Para qué sirve | ¿Cuándo usarlo? |
| --- | --- | --- |
| **Proyecto** | Inicio: generaciones recientes y resumen del banco. | Al abrir la aplicación. |
| **Constructor** | Buscar estrategias que cumplan filtros. | Primer paso del recorrido. |
| **Banco de estrategias** | Consultar las estrategias generadas. | Después de cada generación. |
| **Retesteo** | Volver a probar estrategias en mercados/temporalidades con datos. | Para comprobar si la regla se generaliza. |
| **Refinador** | Comparar variantes de una regla con la original. | Después de entender una estrategia. |
| **Optimizador** | Comparar parámetros en una cuadrícula. | Como análisis adicional, con cautela ante el sobreajuste. |
| **Matriz WF** | Ver estabilidad al reoptimizar en tramos fuera de muestra. | Para estudiar consistencia. |
| **Monte Carlo** | Alterar operaciones y observar la sensibilidad del resultado. | Para comprobar dependencia de la secuencia histórica. |
| **Robustez (SPP/Perf)** | Ejecutar pruebas de estabilidad de parámetros y perfil. | Después de encontrar una candidata. |
| **Proyectos** | Encadenar pasos de análisis en un workflow sencillo. | Uso avanzado; revisa cada paso y resultado. |
| **Cartera** | Comparar varias estrategias como conjunto. | Cuando ya tengas varias candidatas evaluadas. |
| **Laboratorio** | Ver análisis agregados, historial y simulaciones. | Para consultar las estrategias del banco. |
| **Datos** | Ver símbolos e históricos registrados. | Para comprobar si hay datos disponibles. La descarga mostrada es simulada en esta versión. |
| **Gestor de datos** | Importar CSV, crear temporalidades mayores y exportar datasets. | Para preparar históricos. |
| **Exportar código** | Generar código MetaTrader 4 o 5. | Solo después de validar la estrategia y revisar supuestos. |
| **Ajustes** | Ver y modificar costes y metadatos de mercado, y preferencias. | Solo si sabes los valores de tu bróker. |
| **Paridad / Sistema de diseño** | Pantallas de información y diseño del propio producto. | No son necesarias para el uso de trading. |

## 11. Problemas frecuentes

### «CSV con solo 0 barras útiles» o muchas filas descartadas

El archivo puede tener un formato distinto al esperado. Esta versión reconoce
coma, punto y coma y tabulador, así como encabezados OHLC habituales y fecha/hora
separadas. Comprueba:

1. Que elegiste el archivo de barras históricas, no un informe de operaciones.
2. Que la primera fila contiene columnas para fecha/hora, apertura, máximo,
   mínimo y cierre.
3. Que el símbolo y la temporalidad seleccionados coinciden con el archivo.
4. Que no se han exportado precios con un formato numérico que mezcla miles y
   decimales de forma ambigua.

Si sigue fallando, anota el texto exacto del aviso y consulta el formato de
exportación de tu plataforma. Para pedir ayuda, comparte solo la cabecera y una
línea de ejemplo con valores sustituidos; no envíes el histórico completo ni
datos personales.

### «No hay datos históricos para este símbolo/TF»

Importa primero el CSV en **Gestor de datos**, usando ese mismo símbolo y
temporalidad. Vuelve al Constructor y selecciona la temporalidad que figura como
**LISTO**.

### El banco está vacío

Abre el registro de esa generación. Puede que los filtros sean demasiado
exigentes para el dataset, que haya pocas barras o que las reglas no hayan
encontrado operaciones suficientes. No reduzcas todos los filtros a la vez:
cambia uno y compara el resultado.

### Un análisis muestra «sin datos» o no produce resultado

Algunos análisis necesitan que la estrategia tenga una corrida y un dataset
válidos. Para Retesteo, además, necesitas importar datos para cada mercado que
quieras probar.

### El código exportado no coincide con MetaTrader

Comprueba símbolo, temporalidad, spread, comisión, swaps, tamaño de posición y
las diferencias entre barras y ticks. Consulta el checklist que acompaña al
código. Si hay errores al compilar o resultados distintos, no lo uses en una
cuenta real.

## 12. Glosario sencillo

- **Dataset**: archivo de precios que el programa puede usar para probar reglas.
- **Barra**: resumen del precio durante un intervalo; incluye apertura, máximo,
  mínimo y cierre.
- **Temporalidad**: duración de cada barra, como un minuto (`m1`) o una hora
  (`h1`).
- **Backtest**: simulación de una regla sobre precios pasados.
- **Estrategia**: conjunto de reglas que decide cuándo entrar y salir.
- **Equity**: evolución simulada del capital en el tiempo.
- **OOS / fuera de muestra**: tramo reservado para probar la estrategia con
  datos distintos del tramo principal.
- **Sobreajuste**: cuando una regla parece buena porque se adaptó demasiado al
  histórico concreto, pero no se comporta igual con datos nuevos.
- **Robustez**: capacidad de mantener resultados razonables al cambiar datos,
  parámetros o el orden de las operaciones.
- **EA**: asesor experto; programa de MetaTrader que puede automatizar una
  estrategia.
- **Drawdown**: caída del capital desde un máximo anterior hasta un mínimo
  posterior.
- **MAE / MFE**: excursión adversa máxima (lo peor que fue una operación antes
  de cerrar) y excursión favorable máxima (lo mejor que llegó a estar).

## 13. Privacidad y copia de seguridad

SophieQuant guarda estrategias, ajustes y datos en el ordenador. En Windows, la
carpeta local habitual es:

```text
%APPDATA%\com.sophiequant.app
```

Para hacer una copia de seguridad, cierra SophieQuant, pulsa `Windows + R`,
escribe la ruta anterior y pulsa Intro. Copia la carpeta completa a un disco
externo u otra ubicación segura. **Copia**, no muevas ni borres la carpeta
original. Mantén la copia privada: puede contener tus estrategias y tus
históricos.

No compartas archivos de datos completos para pedir soporte. Los resultados son
locales; esta aplicación no es un servicio de asesoramiento ni un intermediario
de inversión.
