# Pruebas manuales de SophieQuant

Guía de aceptación para revisar la aplicación de escritorio en Windows. Cada
prueba indica una tarea y el resultado que debería aparecer. No hace falta que
una estrategia obtenga buenos resultados para que una prueba pase: se comprueba
que el cálculo termina, que los datos son coherentes y que los límites se
explican correctamente.

> **Seguridad:** trabaja con una copia de la base de datos y con históricos de
> prueba. No conectes el programa a una cuenta real ni ejecutes código exportado
> con dinero real. Un backtest o un veredicto PASS no garantiza resultados
> futuros.

## Cómo usar esta guía

Para cada prueba, marca **Pasa**, **Falla**, **Bloqueada** o **No aplica**.
Anota el texto del error y adjunta una captura si falla. No compartas históricos
completos, claves ni datos personales.

- **Pasa:** el resultado observado coincide con el esperado.
- **Falla:** la aplicación se cierra, se bloquea, muestra datos vacíos/`undefined`
  cuando debería haberlos, o el resultado difiere de lo esperado.
- **Bloqueada:** no se pudo ejecutar por falta de datos, dispositivo o
  dependencia. Anota qué faltaba; no la cuentes como aprobada.
- **No aplica:** la función no se ofrece en esta instalación o no se va a usar.

### Datos de la sesión

- Fecha y hora:
- Versión o commit probado:
- Aplicación: ☐ Escritorio con motor Tauri ☐ Navegador/demo
- Símbolo y temporalidad de prueba:
- Nombre local del CSV (no adjuntarlo):
- Filas aproximadas del CSV:
- Copia de seguridad hecha: ☐ Sí ☐ No

**Importante:** las pruebas de cálculo solo son válidas en la ventana de
escritorio. Si el pie de la aplicación dice **«núcleo simulado (navegador)»**,
no uses sus datos ni resultados para validar el motor. Monte Carlo, SPP,
veredictos y otras funciones nativas pueden estar bloqueadas allí por diseño.

## Preparación

1. Cierra la aplicación y haz una copia de seguridad de
   `%APPDATA%\com.sophiequant.app`, o utiliza una base de datos aislada.
2. Prepara un CSV de barras de un símbolo conocido, con fecha/hora, apertura,
   máximo, mínimo y cierre. Usa un histórico de prueba suficientemente largo
   para los análisis; guarda también una copia del CSV.
3. Si vas a probar retesteo entre mercados, prepara un segundo CSV de otro
   símbolo. Confirma la temporalidad real de cada fichero.
4. Abre SophieQuant desde el acceso que quieras validar y confirma que es la
   aplicación de escritorio con motor, no la demo del navegador.
5. Si es posible, prueba sobre una copia de la base habitual para no mezclar los
   resultados de QA con tus estrategias.

## Pruebas

### A. Inicio y navegación

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| A1 | Abrir la aplicación y recorrer las opciones del menú. | La ventana carga sin pantalla negra ni mensajes de error; cada módulo abre su pantalla y el menú indica cuál está seleccionado. | |
| A2 | Minimizar/restaurar la ventana y cambiar entre varias pantallas. | La aplicación sigue respondiendo; los controles y textos no desaparecen ni quedan superpuestos. | |
| A3 | Abrir una ruta inexistente, si se puede escribir o navegar a ella. | Aparece una pantalla propia de «Página no encontrada» y existe una forma clara de volver; no se cierra la aplicación. | |

### B. Datos e históricos

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| B1 | En **Gestor de datos**, importar el CSV preparado al símbolo y temporalidad correctos. | Se confirma la importación; el fichero figura como **LISTO**, con el símbolo, timeframe y una cantidad de barras coherente con el archivo. | |
| B2 | Revisar el informe de calidad del fichero importado. | Se muestran las barras leídas, el desglose de filas descartadas, intervalos temporales sin registros, volumen ausente y precios incoherentes. Los intervalos pueden corresponder a cierres de mercado y no prueban por sí solos que falten datos. | |
| B3 | Importar otra copia de prueba con separador punto y coma o tabulador, si tienes un CSV así. | Se reconocen las columnas OHLC y la fecha; las filas válidas se importan. La prueba no debe reemplazar ni borrar el primer histórico. | |
| B4 | En **Datos**, comprobar que aparecen el símbolo y el histórico recién importados. | El dataset se puede localizar por símbolo/timeframe y coincide con el estado del Gestor de datos. No aparece como listo si la importación falló. | |
| B5 | En el Gestor, resamplear una copia de datos desde una temporalidad menor a otra mayor (por ejemplo m1 a m5). | Se crea la temporalidad mayor con menos barras y OHLC agregados; la fuente original permanece intacta. No se ofrece convertir de una temporalidad mayor a otra menor. | |
| B6 | Pulsar **Exportar**, elegir carpeta y nombre en el diálogo de Windows; repetir con CSV y QFD si procede. | Se abre «Guardar como»; tras confirmar, el archivo existe en la ruta elegida, su tamaño es mayor que cero y el programa confirma esa ruta. Si se cancela, informa que no guardó nada. El dataset original permanece. | |
| B7 | Tras actualizar la aplicación, volver a importar el CSV y revisar el mensaje de intervalos temporales. | El programa identifica esos valores como intervalos estimados, avisa que puede contar cierres de mercado y no los confunde con filas descartadas. Para validar lagunas reales, comparar timestamps con sesiones esperadas de ese símbolo/timeframe. |  |

### C. Constructor y Banco

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| C1 | Abrir **Constructor** y alternar entre un timeframe con datos y otro sin datos para el símbolo elegido. | Con datos se muestran plantilla, volumen, filtros y robustez, y se puede iniciar la generación. Sin datos se conserva el aviso y el enlace a Datos, se ocultan esas opciones y se deshabilita Generar estrategias. | |
| C2 | Ejecutar una generación de prueba con objetivo 10, filtros personalizados y **Exigir robustez estricta** activado; probar también un filtro deliberadamente exigente. | Se muestra estado y progreso durante la ejecución; al terminar se marca como completada y queda un resumen persistente con pasadas/objetivo, mejores métricas observadas durante la búsqueda, descartes por cada filtro, mínimo de operaciones y pruebas concretas de robustez fallidas. Los conteos por criterio pueden solaparse. SPP estricto exige que pase la perturbación de parámetros; señales invertidas/aleatorias son controles diagnósticos y no condiciones de aprobación. Cero resultados es válido si ninguna supera las condiciones. |  |
| C3 | Si el proceso permite pausar, pausar y reanudar; después, iniciar otra generación de prueba y detenerla. | Pausar detiene temporalmente el avance; reanudar lo continúa; detener cancela y deja el estado final claro. No se duplica una generación por doble clic. | |
| C4 | Abrir el Banco tras una generación. | Se muestran las estrategias guardadas y sus métricas; las celdas vacías se identifican como no disponibles, no como valores ficticios. | |
| C5 | Ordenar el Banco pulsando una columna dos veces. | El primer clic ordena ascendente y el segundo descendente; los valores de la columna y sus filas siguen correspondiéndose. | |
| C6 | Seleccionar una estrategia y abrir su ficha. | La ficha muestra regla/parametría, métricas, curva de equity y tramo OOS cuando exista. No aparecen `undefined`, `NaN` o datos de otra estrategia. | |
| C7 | Calcular veredictos para todas las estrategias, si hay candidatas. | Se ve el progreso, cuántas se completaron y cuántas faltan; al acabar, los estados y resultados corresponden a cada estrategia. Un fallo se informa y no se presenta como PASS. | |
| C8 | Si el Banco queda vacío tras C2, repetir con filtros menos estrictos solo para comprobar el flujo; identificar claramente que esta segunda generación no demuestra robustez. | El Constructor guarda únicamente las que pasan los filtros elegidos. Desactivar robustez estricta no debe etiquetar por sí solo las estrategias como robustas. | |

### D. Retesteo y análisis de parámetros

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| D1 | En **Retesteo**, elegir una estrategia, timeframe y mercado con datos; ejecutar. | El análisis termina y muestra resultado/veredicto por estrategia. Sin datos debe indicar «sin datos» o explicar el bloqueo; no debe inventar resultados. | |
| D2 | En **Retesteo**, seleccionar un segundo mercado con su propio histórico, si está disponible. | Los resultados distinguen cada mercado; los mercados sin datos no se cuentan como pruebas aprobadas ni fallidas. | |
| D3 | En **Optimizador**, ejecutar el modo **Simple**. | Aparece el mapa de calor con filas y columnas identificadas. Cada combinación muestra PF y drawdown; la tarjeta de resumen y el borde destacado se corresponden con la combinación seleccionada. No hay `undefined`/`NaN`. | |
| D4 | Cambiar a **Walk-Forward** y ejecutar de nuevo. | Aparece el mapa WFE, drawdown y veredicto por combinación. Se explica la leyenda y la combinación seleccionada; no se muestra PF donde corresponde WFE ni quedan casillas vacías por una métrica inexistente. | |
| D5 | Cambiar estrategia o número de pasos después de ejecutar el optimizador. | El resultado anterior se limpia y no se presenta como si correspondiera a la nueva selección. Al ejecutar, el mapa se vuelve a generar con sus valores. | |
| D6 | Abrir el **Mapa de optimización** desde una ficha de estrategia, si está disponible. | Se carga el análisis para la estrategia seleccionada. Si falla el cálculo, se ve un mensaje de error recuperable en lugar de una pantalla vacía. | |
| D7 | En **Matriz WF**, seleccionar estrategia y parámetros y ejecutar. | Se muestra una matriz de combinaciones y sus veredictos; los avisos de muestra insuficiente se hacen visibles. No se afirma que un resultado pendiente sea PASS. | |
| D8 | En **Monte Carlo**, ejecutar las pruebas disponibles con la estrategia seleccionada. | El control muestra que está trabajando y vuelve a estar disponible al terminar. Hay métricas/unidades e iteraciones; cada error se explica y permite reintentar. No se interpreta el porcentaje de una muestra como probabilidad de ganar en el futuro. | |
| D9 | En **Robustez (SPP/Perf)**, ejecutar SPP y el perfil en la aplicación de escritorio. | SPP lista sus pruebas, impactos y veredicto; el perfil muestra cada parámetro activo. Una advertencia, error, muestra insuficiente o resultado pendiente no se presenta como calificación estricta aprobada. | |

### E. Edición, Refinador y Proyectos

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| E1 | Abrir el editor desde una ficha, cambiar un parámetro y guardar/recalcular en la copia de prueba. | La nueva regla se guarda, se recalcula y se muestran métricas actualizadas. La estrategia queda marcada para revisión o pierde sus resultados de robustez antiguos cuando corresponda. | |
| E2 | Abrir el Refinador y ejecutar una comparación de variantes. | Se muestra la variante y sus métricas frente al original; si ninguna mejora, se indica que se conserva el original. No se afirma que una mejora histórica garantice robustez. | |
| E3 | En **Proyectos**, ejecutar el workflow inicial y otro con un paso opcional. | Los pasos avanzan con estado; al terminar se informa si el workflow completó o se detuvo y qué paso produjo el resultado/error. | |
| E4 | En el workflow, quitar y reordenar un paso antes de ejecutar. | La lista refleja el orden nuevo y el paso quitado no se ejecuta. | |

### F. Cartera y Laboratorio

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| F1 | En **Cartera**, seleccionar al menos dos estrategias y probar equiponderado. | El análisis termina; los pesos son coherentes y suman 100% (salvo redondeo); se ven correlaciones, drawdown y curva combinada. | |
| F2 | Probar inverso a volatilidad y paridad de riesgo; luego probar pesos personalizados. | Cada modelo indica el modelo seleccionado y devuelve pesos/resultados. Los pesos personalizados se normalizan o se rechazan con una explicación clara. | |
| F3 | Dejar solo una estrategia seleccionada e intentar analizar. | La acción se deshabilita o informa que se requieren al menos dos estrategias; no aparece un resultado de cartera falso. | |
| F4 | En **Laboratorio**, recorrer resumen, veredictos, historial y las pestañas de análisis disponibles. | Las tablas y filtros corresponden a la pestaña elegida; el historial muestra las ejecuciones realmente realizadas. No aparecen análisis SPP/veredictos inventados al abrirlo desde un navegador. | |
| F5 | Ejecutar Monte Carlo de cartera si está disponible en escritorio. | Se muestran número de simulaciones, percentiles y probabilidad de ruina con valores numéricos; al fallar, aparece un error explícito. | |

### G. Exportación, ajustes y pantallas informativas

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| G1 | En **Exportar código**, generar código para MT5 desde una estrategia de prueba. | Aparece código no vacío y la verificación explica los supuestos y comprobaciones pendientes. El código no se instala ni se activa por sí mismo. | |
| G2 | Repetir G1 para MT4. | Se genera el formato correspondiente a MT4, distinto del de MT5, y se puede copiar/guardar manualmente con extensión adecuada. | |
| G3 | Abrir el fichero en MetaEditor y compilarlo, si MetaTrader está instalado. | La compilación no tiene errores. Si hay advertencias, se anotan; después se compara en el probador con supuestos equivalentes. No ejecutar en una cuenta real. | |
| G4 | En **Ajustes**, consultar los metadatos de un símbolo y modificar un valor solo en la copia de prueba. | Los campos visibles corresponden al símbolo; el valor se valida y persiste tras volver a abrir la pantalla. Valores incorrectos se rechazan o notifican. | |
| G5 | Abrir **Paridad** y **Sistema de diseño**. | Ambas pantallas cargan sin errores de interfaz, desbordamientos ni valores rotos. Si una sección informa que la comparación no está disponible, lo expresa claramente. | |
| G6 | Cambiar el tamaño de la ventana y navegar con teclado por algunos controles. | El contenido sigue siendo legible, no aparece desbordamiento horizontal inesperado y el foco de teclado es visible. | |

### H. Persistencia y errores recuperables

| ID | Tarea | Resultado esperado | Estado / notas |
|---|---|---|---|
| H1 | Cerrar y volver a abrir SophieQuant después de importar datos y guardar estrategias en la base de prueba. | Los datos y estrategias aparecen de nuevo; no se duplican por el simple hecho de reiniciar. | |
| H2 | Actualizar la ventana durante una generación pausada, si es seguro en la copia de QA. | El estado/progreso se recupera o la interfaz informa claramente de que el proceso terminó; no muestra un estado activo falso. | |
| H3 | Provocar una acción inválida inocua (por ejemplo, entrar en un detalle con un ID inexistente si se puede navegar). | La aplicación informa que el recurso no está disponible o presenta un estado vacío; no se cierra ni enseña datos de otra estrategia. | |
| H4 | Repetir una acción de análisis después de un error recuperable. | El botón vuelve a habilitarse y permite reintentar; el error anterior no se presenta como resultado correcto. | |

## Cierre de la sesión

- Pruebas pasadas:
- Pruebas fallidas:
- Pruebas bloqueadas/no aplicables:
- Errores exactos y pasos para reproducir:
- Capturas tomadas (sin datos sensibles):
- ¿Se comprobó en la ventana Tauri y no solo en el navegador?: ☐ Sí ☐ No
- ¿Se probó MetaTrader de forma aislada y nunca con dinero real?: ☐ Sí ☐ No ☐ No aplica

Una sesión completa de pruebas manuales **no certifica** que la estrategia sea
rentable ni que no existan errores. Significa únicamente que los recorridos
comprobados se comportaron como se esperaba con la configuración y los datos de
esa sesión.
