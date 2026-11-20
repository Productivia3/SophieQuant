# Hoja de ruta hacia SophieQuant 1.0

**Versión del plan:** 1.0  
**Fecha:** 2026-11-20  
**Objetivo:** una aplicación local para Windows y Linux que permita buscar,
editar, evaluar y exportar estrategias Forex a MT5, y recomendar las que
presenten mejores evidencias de robustez.

Esta hoja de ruta define un punto de cierre verificable. No promete que el
generador encuentre estrategias rentables en cada dataset ni que una estrategia
mantenga resultados futuros. Los filtros de creación son configurables por el
usuario; las pruebas de robustez sirven para explicar y ordenar candidatas, no
para bloquear la exportación de una estrategia que ya superó los filtros de
creación.

## Alcance acordado

- **Instrumentos:** Forex con datos proporcionados por el usuario. CSV/QFD para
  barras y datos tick bid/ask exportados de MT5 para calibración y backtest.
- **Búsqueda:** el usuario elige símbolo, timeframe y método/plantilla en cada
  búsqueda. El motor debe funcionar genéricamente con todas las combinaciones
  admitidas; no se exige una búsqueda batch de todos los símbolos y métodos.
- **Estrategias:** largos y cortos; indicadores y reglas visuales combinables;
  parámetros editables y recalculables en local.
- **Riesgo y órdenes:** lote fijo o riesgo porcentual seleccionable; una
  posición por defecto y límite configurable; SL/TP y trailing stop, con
  validación de tamaños y restricciones del símbolo.
- **Indicadores iniciales:** SMA, EMA, RSI, ATR, MACD y CCI.
- **Patrones iniciales:** doji, martillo, estrella fugaz, engulfing alcista y
  bajista, harami alcista y bajista, morning star, evening star, piercing line
  y dark cloud cover. Se enumeran los 11 casos explícitamente para que no haya
  ambigüedad sobre la opción de catálogo ampliado elegida.
- **Robustez:** WFE/OOS, Monte Carlo, SPP, perfil de optimización y controles de
  overfitting con estados, resultados y motivos visibles. Los filtros de
  creación siguen siendo decisión del usuario.
- **MT5:** exportación MQL5 únicamente; MetaTrader 4 queda fuera del alcance.
  Validación de Strategy Tester y Demo en hedging y netting, priorizando
  hedging.
- **Ejecución:** SophieQuant no se conecta a un broker ni opera en real. El EA
  exportado se prueba en MetaTrader Demo.
- **Plataformas:** aplicación local instalable en Windows y Linux; Linux se
  prueba en un equipo compartido de 8 GB RAM y 4 vCPU. MT5 se ejecuta en Linux
  mediante Wine. No se requiere conexión a un servicio remoto de SophieQuant.
- **Calibración:** GBPUSD H1, 2018–2026, con ticks exportados del mismo broker
  que se usará en el Probador de Estrategias.
- **Prueba demo:** mínimo de 8 semanas y 50 operaciones cerradas por estrategia.
  Si no se alcanza la muestra, la validación queda inconclusa y se prolonga.

### Fuera del alcance de 1.0

- Ejecución real o conexión de órdenes de SophieQuant con brokers.
- Búsqueda batch de todos los símbolos/timeframes disponibles o ejecución
  automática de todos los métodos en una sola búsqueda.
- Acciones, futuros, proveedores de datos integrados y datos descargados por
  la aplicación.
- MetaTrader 4, macOS, publicación en mercados de EAs y garantía de rentabilidad.
- Otros indicadores/patrones no incluidos en el catálogo inicial; podrán
  añadirse en versiones posteriores sin reabrir el cierre de 1.0.

## Reglas de aceptación

1. Los umbrales de creación (PF, drawdown, número de operaciones, etc.) se
   configuran antes de buscar. No se relajan automáticamente para producir
   resultados.
2. Toda estrategia que supera esos filtros se puede exportar a MQL5. La ficha y
   el banco muestran los resultados de cada gate; las recomendaciones ordenan
   según la evidencia, sin llamar «robusta» a una estrategia que tenga un gate
   fallido, pendiente o no evaluable.
3. La paridad con MT5 se mide con el mismo símbolo, ticks, periodo, zona horaria,
   spread, comisión, swap, divisa de cuenta, apalancamiento y reglas de fill.
   Las tolerancias de precio y métricas se fijan **después** de la calibración
   inicial y se guardan por escrito antes de declarar aprobado el motor.
4. Operaciones y dirección BUY/SELL deben coincidir exactamente. Las horas
   deben quedar dentro de un tick o de pocos segundos, según lo que permita el
   feed. SL/TP teóricos deben coincidir al tick del símbolo. Las desviaciones
   aceptables de fills y métricas se cuantifican en la tarea de calibración; no
   se inventan antes de medir.
5. Una prueba que no dispone de datos, muestra suficiente o acceso a MT5 queda
   **bloqueada/inconclusa**, nunca aprobada por defecto.
6. La rentabilidad futura no es un criterio que el software pueda garantizar.
   Demo comprueba ejecución y comportamiento observado; no certifica resultados
   futuros.

## Fases y tareas

Tamaños relativos: **S** pequeña, **M** mediana, **L** grande, **XL** muy
grande. No son fechas ni compromisos de duración; se revisarán tras inspeccionar
el código al iniciar cada fase. No se empieza una fase mientras no se cumplan
sus dependencias.

### Fase 0 — Congelar alcance y establecer una línea base

**Dependencias:** ninguna.  
**Tamaño estimado:** M.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R0.1 | Convertir `PRUEBAS_MANUALES.md`, la matriz de funcionalidades y esta hoja de ruta en una matriz única con ID, prioridad, estado, prueba y responsable. | Cada requisito de 1.0 tiene al menos un test automatizado o un caso manual; todo lo fuera de alcance está etiquetado y no bloquea la versión. |
| R0.2 | Registrar el commit, versiones de Rust/Node/Tauri/MT5, SO, configuración del símbolo y copias aisladas de DB/datasets de QA. | Otra persona puede reconstruir el entorno de QA sin usar la base habitual ni credenciales del usuario. |
| R0.3 | Ejecutar la suite existente y los recorridos manuales aplicables en Windows; clasificar cada caso como pasa/falla/bloqueado/no aplica. | Cero estados ambiguos; cada fallo tiene pasos, evidencia y bug ID. No se cuenta ningún bloqueado como aprobado. |
| R0.4 | Auditar los huecos actuales de motor, UI, exportación y persistencia contra el alcance acordado. | Inventario con archivo/módulo, cambio necesario, dependencias y tests; ningún requisito se declara terminado solo por aparecer en la UI. |
| R0.5 | Medir el uso actual de CPU, RAM y tiempo para 43.000 barras H1 y búsquedas repetibles en Windows y en el Linux compartido. | Benchmark reproducible con dataset/configuración y hardware documentados; acordar presupuesto de recursos y tiempos antes de optimizar. |
| R0.6 | Confirmar que se pueden obtener y reproducir los ticks bid/ask de GBPUSD H1 2018–2026 en el mismo MT5/broker, junto con el reporte del Strategy Tester. | Dataset, intervalo, especificación del símbolo y ajustes de tester están disponibles y tienen hash/identificador reproducible; si no, se detiene R2 y se elige otro conjunto de referencia. |

**Gate de fase:** alcance y línea base aprobados; no hay requisitos sin prueba
asignada.

**Seguimiento inicial (2026-11-20):** matriz de aceptación en
[`replica/acceptance-matrix.md`](replica/acceptance-matrix.md) y evidencia/
pendientes en [`replica/phase0-baseline.md`](replica/phase0-baseline.md).
Automatización Windows inicial: Rust 94/94, Playwright 115/115 y build frontend
correctos. R0.1, R0.2, R0.3, R0.4 y R0.5 siguen parciales; R0.6 está bloqueado
hasta disponer del tick export y del reporte reproducible de MT5. La Fase 0 no
está cerrada.

**Actualización Fase 0 (2026-11-20):** se ejecutó Tauri con el perfil aislado
`com.sophiequant.qa`. En fixtures sintéticas pasaron B1/B2/B6/C1; la exportación
CSV fue verificada en disco y se importaron 43.000 barras H1 sin filas
descartadas ni alertas de calidad. Dos búsquedas de 10 objetivos con robustez
estricta guardaron 10/10 y dieron el mismo resumen; la segunda incluyó muestreo
de recursos aproximadamente cada segundo. En el Banco, C4/C6 pasaron sus
criterios; C5 falló por dirección inicial de orden (BUG-021) y C7 completó
20/20 con resultados FAILED, aunque la ficha no cargó los veredictos previos
(BUG-022).
La línea
base y los pendientes están en
[`replica/phase0-baseline.md`](replica/phase0-baseline.md); la matriz refleja el
avance. R0.5 aún requiere benchmark Linux y acordar el presupuesto con mediciones
comparables. R0.6 sigue bloqueado hasta disponer del tick feed y el reporte de
MT5 en ubicación de QA.

### Fase 1 — Modelo de órdenes y motor long/short

**Dependencias:** R0.4.  
**Tamaño estimado:** XL.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R1.1 | Representar explícitamente BUY/SELL, apertura/cierre, tamaño, precio, tiempo, SL/TP y motivo de salida en el modelo de operación. | Tests unitarios cubren apertura, cierre, reversión y datos inválidos para ambos sentidos; no hay operaciones short registradas como long. |
| R1.2 | Implementar el ciclo de vida de órdenes en barras y ticks, con fill, spread bid/ask, slippage, comisión, swap y restricciones del símbolo. | Fixtures conocidos producen fills reproducibles; cada coste aparece en el detalle del trade y en el P&L. |
| R1.3 | Implementar SL/TP simétricos para long/short y la prioridad de ejecución cuando una barra/tick toca más de un nivel. | Casos de stop, target, gap y señal de salida tienen resultado esperado explícito y tests para ambas direcciones. |
| R1.4 | Añadir lote fijo y riesgo porcentual, normalizando volumen a min/max/step del símbolo; admitir una posición por defecto y límite configurable. | Tests para tamaño mínimo, máximo, step, balance insuficiente, riesgo cero/negativo y límites de posiciones; no se emite una orden fuera de los límites del instrumento. |
| R1.5 | Implementar trailing stop configurable y persistir la configuración usada en cada corrida. | Tests de movimiento del trailing en long/short; resultados históricos conservan los parámetros de ejecución que los produjeron. |
| R1.6 | Definir el comportamiento netting/hedging en el modelo de posiciones y mantener una capa común de reglas. | Tests de contrato de órdenes pasan en ambos modos; las diferencias deliberadas quedan documentadas. Hedging se valida primero. |

**Gate de fase:** los tests de contrato del motor pasan para long y short; costes,
tamaño y modo de cuenta no tienen valores silenciosos por defecto.

### Fase 2 — Datos tick y fidelidad del backtest

**Dependencias:** R0.6 y R1.1–R1.3.  
**Tamaño estimado:** XL.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R2.1 | Diseñar/importar ticks bid/ask de MT5 con timestamp, orden, símbolo y metadatos de broker. | Importar un archivo de referencia reproduce cantidad, primer/último timestamp y precios; duplicados, orden inválido y huecos se reportan y no se ocultan. |
| R2.2 | Añadir validación y almacenamiento eficiente de ticks, sin reemplazar el flujo CSV/QFD de barras. | Roundtrip de importación/exportación preserva los datos; prueba del dataset de referencia no supera el presupuesto RAM acordado en R0.5. |
| R2.3 | Implementar ejecución tick-a-tick en bid/ask con spreads variables, stops, gaps, slippage y swaps. | Un conjunto pequeño de escenarios manualmente calculados coincide con cada trade y su P&L; no se usa el cierre de barra como sustituto silencioso del tick. |
| R2.4 | Replicar y guardar los ajustes relevantes de MT5: símbolo, divisa de cuenta, balance, apalancamiento, volumen, comisión, swap, spread y tipo de fill. | Una corrida se puede repetir desde la configuración persistida; faltantes o discrepancias de configuración detienen la comparación con mensaje explícito. |
| R2.5 | Comparar trade por trade con Strategy Tester para GBPUSD H1 de referencia y generar informe de diferencias. | 100% de coincidencia en número de operaciones y dirección; timestamps dentro de la tolerancia acordada; SL/TP teóricos al tick; diferencias de fills y métricas informadas por trade/métrica. |
| R2.6 | Calibrar tolerancias numéricas de precios de ejecución y métricas y aprobarlas con el usuario antes de fijarlas como gates de release. | Documento de calibración con error absoluto/relativo, percentiles y máximo para cada campo; las tolerancias aprobadas se convierten en tests de regresión. |
| R2.7 | Documentar fórmulas y convenciones para PF, net/gross, DD, win rate, recovery, Sharpe/Sortino y operaciones. | Para el informe MT5 de referencia, cada métrica queda dentro de su tolerancia aprobada; cada diferencia tiene causa conocida o mantiene la fase bloqueada. |

**Gate de fase:** informe de paridad aprobado en la muestra congelada; no se declara
tick-accurate mientras existan trades omitidos/fantasma o una métrica fuera de
tolerancia.

### Fase 3 — Lenguaje de reglas, métodos y búsqueda

**Dependencias:** R1 y R2.  
**Tamaño estimado:** XL.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R3.1 | Diseñar una representación de regla compartida por generador, editor, backtest y exportador. | Una regla serializada se carga, edita, recalcula y exporta sin traducirla a una lógica distinta; test de contrato cubre cada nodo soportado. |
| R3.2 | Añadir bloques AND/OR con condiciones de entrada/salida y reglas de dirección. | UI crea, valida, guarda y vuelve a abrir reglas; condiciones incompletas se rechazan con error legible; tests cubren combinaciones anidadas permitidas. |
| R3.3 | Implementar SMA, EMA, RSI, ATR, MACD y CCI con contratos de valores, warm-up y señales. | Para cada indicador hay fixtures numéricos de referencia y pruebas de índices iniciales, NaN, datos insuficientes y uso long/short. |
| R3.4 | Implementar el catálogo inicial de patrones de velas acordado. | Cada uno de los 11 casos enumerados en alcance tiene fixtures positivos y negativos, criterios explícitos de OHLC y pruebas de no-lookahead. |
| R3.5 | Conectar entradas y salidas por señal con SL/TP/trailing y los parámetros que controla el usuario. | Toda combinación visible se puede ejecutar; combinación no soportada se deshabilita o explica, nunca se ignora en silencio. |
| R3.6 | Mantener la selección del método por búsqueda, pero hacer común el motor para todos los símbolos/timeframes Forex admitidos. | Matriz automatizada método × dirección × timeframe valida que no hay ramas codificadas a un par específico; cambiar par no requiere editar código. |
| R3.7 | Permitir edición individual de parámetros y creación de una copia de prueba recalculada. | El cambio persiste, muestra métricas nuevas y marca como obsoletas las métricas/veredictos anteriores. |
| R3.8 | Añadir diagnóstico de diversidad, presupuesto, semilla, progreso, pausa/cancelación y razones de descarte. | Una búsqueda determinista se reproduce con la misma semilla; el total evaluado coincide con el presupuesto/lotes reportados; no se guardan estrategias que incumplen filtros de creación. |
| R3.9 | Validar escalado en los límites acordados en R0.5, incluyendo el dataset de barras de referencia y el tick dataset. | No hay OOM ni bloqueo de UI; CPU/RAM/tiempo medidos y comparados con el presupuesto publicado. Si no alcanza, optimización queda como tarea antes de pasar. |

**Gate de fase:** todos los bloques, métodos, parámetros y métricas de búsqueda
tienen un único significado entre edición, cálculo y exportación.

### Fase 4 — Robustez, overfitting y selección informada

**Dependencias:** R2.5–R2.7 y R3.  
**Tamaño estimado:** L.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R4.1 | Separar IS, OOS y holdout final no utilizado para escoger parámetros. | Tests demuestran que búsqueda/optimización no lee el holdout; interfaz identifica cada segmento y su tamaño. |
| R4.2 | Revisar WFE, walk-forward, Monte Carlo, SPP y perfil con datos/ticks y cálculos reproducibles. | Cada prueba tiene fórmula, muestra, semilla/configuración y resultado persistido; errores o muestra insuficiente quedan como pendientes, no PASS. |
| R4.3 | Revisar las pruebas MC para que cada perturbación mida una hipótesis definida y no sea tautológica. | Tests con secuencias conocidas detectan el fallo esperado; cada resultado identifica test, iteraciones, métrica y percentiles. |
| R4.4 | Mantener gates independientes y explicar los fallos por candidato. | Cada estrategia muestra pass/warn/fail/pending por gate y evidencia; el agregado coincide con el detalle y sus conteos pueden solaparse. |
| R4.5 | Implementar recomendación/ranking transparente por evidencias de robustez, sin bloquear exportación a quien supera filtros de creación. | Test prueba que un candidato con filtro base aprobado pero gate de robustez fallido se puede exportar, pero no se etiqueta como robusto ni se recomienda por encima de uno mejor validado sin explicación. |
| R4.6 | Invalidar resultados al cambiar estrategia, dataset, configuración del motor o versión de fórmula. | Un cambio en cada uno de esos campos invalida score/veredicto; no se reutiliza evidencia perteneciente a otra corrida. |

**Gate de fase:** overfitting, falta de datos y gates fallidos no producen falsos
PASS; la recomendación no se confunde con garantía de rentabilidad.

### Fase 5 — MT5: exportación y ejecución equivalente

**Dependencias:** R1, R2.6–R2.7 y R3.  
**Tamaño estimado:** XL.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R5.1 | Generar MQL5 desde la representación común para todos los indicadores, patrones, long/short y salidas del alcance. | Test de contrato comprueba que cada nodo de regla tiene traducción; funciones no traducibles impiden exportar e identifican el nodo. |
| R5.2 | Implementar lotaje fijo/riesgo porcentual, límites de volumen, SL/TP, trailing y gestión de órdenes. | EA y motor producen la misma decisión para escenarios de referencia; se incluyen hedging y netting, priorizando hedging. |
| R5.3 | Quitar MT4 del flujo de usuario y de los claims/documentación de 1.0. | No hay selección MT4 en UI ni instrucciones que prometan soporte MT4; la regresión confirma que exportar produce solo `.mq5`. |
| R5.4 | Compilar cada fixture exportado con MetaEditor y ejecutar el EA en Strategy Tester con ticks reales. | Cero errores y warnings en el código generado; dirección/operaciones y comportamiento de órdenes comparados con los resultados de SophieQuant. Cualquier warning de herramienta/dependencia se documenta y justifica. |
| R5.5 | Probar hedge y netting en MT5 Tester y Demo con el mismo conjunto de órdenes. | Casos de entrada, cierre, reversión, SL, TP, trailing, rechazo por volumen y reinicio del EA cumplen los resultados esperados en ambos modos. |
| R5.6 | Incorporar una verificación del EA y una checklist de supuestos antes de exportar. | Cada export incluye hash de estrategia/configuración, versión del motor, símbolo, timeframe y advertencias; no afirma que el bot vaya a ser rentable. |

**Gate de fase:** todo `.mq5` del alcance compila y replica las reglas soportadas;
no hay diferencias críticas de dirección, tamaño ni gestión de órdenes.

### Fase 6 — Validación pre-puesta en Demo

**Dependencias:** Fases 4 y 5.  
**Tamaño estimado:** L de ingeniería + mínimo 8 semanas de observación.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R6.1 | Ejecutar el checklist manual de escritorio y registrar versión, configuración, evidencia y estado de cada caso. | Todos los casos aplicables pasan; los no aplicables tienen motivo; cero casos bloqueados se contabilizan como aprobados. |
| R6.2 | Correr el EA de referencia en Demo MT5 durante al menos 8 semanas y 50 operaciones cerradas por estrategia. | Por estrategia, el periodo y las 50 operaciones mínimas están acreditados. Si faltan trades, la prueba es inconclusa y continúa; no se extrapola. |
| R6.3 | Comparar señales, órdenes, lote, SL/TP, cierres y errores entre EA y logs de MT5 Demo. | Cero errores críticos y cero discrepancias de regla/gestión no explicadas; cualquier desviación de fill se evalúa con las tolerancias aprobadas en R2.6. |
| R6.4 | Guardar los informes completos del Tester y Demo sin credenciales ni datos de cuenta sensibles. | Evidencia reproducible asociada a la versión exacta del EA, hash de estrategia, broker/símbolo y configuración anonimizada. |
| R6.5 | Revisar resultados desfavorables sin cambiar umbrales a posteriori para hacerlos pasar. | Todo cambio de regla crea una nueva versión de estrategia y reinicia sus validaciones afectadas. |

**Gate de fase:** se puede marcar una estrategia como «validada en Demo» solo con
8 semanas, 50 cierres, cero errores críticos y discrepancias dentro de tolerancia.
Un resultado negativo es un resultado válido de QA, no un fallo del test.

### Fase 7 — Instaladores Windows/Linux y cierre 1.0

**Dependencias:** Fases 0–6.  
**Tamaño estimado:** L.

| ID | Tarea | Criterio medible de salida |
|---|---|---|
| R7.1 | Crear builds instalables locales Windows y Linux; SophieQuant no depende de una API o servidor remoto propio. | Instalar, iniciar, importar un dataset, ejecutar una búsqueda y recuperar resultados en ambos sistemas sin entorno de desarrollo. |
| R7.2 | Probar la app nativa Linux y MT5 bajo Wine en el hardware de 8 GB/4 vCPU. | Smoke test completo; no hay errores críticos de Wine/MT5; recursos y límites publicados según R0.5/R3.9. |
| R7.3 | Verificar migración/backup/restore de SQLite y datasets. | Restaurar una copia en una instalación limpia reproduce banco, datos y resultados; migraciones preservan datos de la versión anterior. |
| R7.4 | Publicar artefactos versionados en GitHub con hashes, requisitos, guía y limitaciones conocidas. | Releases Windows/Linux instalables; hash de descarga verificable; guía explica importar ticks, configurar MT5 y ejecutar Demo con seguridad. |
| R7.5 | Cerrar defectos y fijar soporte de 1.0. | Suite automatizada completa verde, cero bugs S1/S2 abiertos, checklist manual aplicable cerrada y matriz de funcionalidades de 1.0 completa o con excepción firmada. |

**Gate de salida 1.0:** se cumplen las reglas de aceptación, los instaladores
Windows/Linux pasan smoke tests, MT5 compile/test/demo tiene evidencia y las
limitaciones quedan publicadas.

## Dependencias principales

```text
Fase 0
  └─ Fase 1 (órdenes long/short y costes)
       └─ Fase 2 (ticks y paridad MT5)
            ├─ Fase 3 (reglas y búsqueda)
            └─ Fase 4 (robustez y overfitting) ← también necesita Fase 3
Fases 1 + 2.6/2.7 + 3
  └─ Fase 5 (EA MQL5)
Fases 4 + 5
  └─ Fase 6 (Tester/Demo)
Fases 0–6
  └─ Fase 7 (release 1.0)
```

## Criterios globales de cierre

- `cargo test`, build frontend, suite Playwright/accesibilidad y builds Tauri
  Windows/Linux pasan en el commit de release.
- Cero bugs S1/S2 abiertos; todo bug restante tiene workaround, prioridad y
  decisión de incluir/excluir registrada.
- Cada caso manual aplicable de `PRUEBAS_MANUALES.md` tiene evidencia y estado;
  ningún caso bloqueado se declara aprobado.
- El motor no da falsos PASS en filtros, resultados OOS, Monte Carlo, SPP,
  perfiles, métricas o ejecución de órdenes.
- La comparación con MT5 satisface los umbrales aprobados en calibración; los
  umbrales son reproducibles y versionados.
- El EA solo se marca «validado en Demo» al completar su gate temporal y de
  muestra.
- La matriz de funcionalidades identifica explícitamente lo implementado,
  excluido o aplazado. «Versión final» significa alcance 1.0 cerrado, no paridad
  ilimitada con otros productos ni rentabilidad garantizada.

## Decisiones que se congelan en Fase 0

1. Distribución Linux objetivo (distribución/versión y formato instalable).
2. Presupuesto de CPU/RAM/tiempo en el equipo compartido y tamaño máximo
   operativo de datasets tick.
3. Métricas exactas del reporte MT5 a comparar y tolerancias numéricas tras la
   primera calibración.
4. Reglas de horario/timezone y sesiones del broker para el dataset de
   referencia.
5. Duración/configuración de Demo por estrategia y el mecanismo de evidencia
   que no exponga identificadores ni credenciales de cuenta.
