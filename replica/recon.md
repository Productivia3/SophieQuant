# Recon map: StrategyQuant X (Desktop — Windows/Mac/Linux)

Scope: Clon de StrategyQuant X — Core SQX (Builder/Retester/Improver/Backtester), Robustez avanzada (Monte Carlo, WF Matrix, SPP, Optimization Profile), Optimizador, Custom Projects, Portfolio Master/Composer, Export a 6 plataformas, QuantAnalyzer (análisis+portfolio), QuantDataManager (datos). Fuera: AlgoWizard visual, AI Assistant "Q", Market & Volume Profile, SQ for Business.
For: Producto para vender a traders algo-trading que no quieren pagar $1,290–$2,900 por licencia lifetime.
Date: 2026-10-20

## Reglas clean-room aplicadas

- Solo fuentes públicas. Sin login a cuentas ajenas. Sin decompilar.
- No se copian: su código (Java/MQL), sus datos premium, sus estrategias de regalo, su curso, su marca.
- Target: app de escritorio propia (Tauri/Electron). Motor de análisis propio escrito desde cero.

## Sources

| # | source | URL | notes |
| --- | --- | --- | --- |
| 1 | Home ES | https://strategyquant.com/es/ | posicionamiento, core loop en sus palabras, testimonios |
| 2 | Features | https://strategyquant.com/es/caracteristicas/ | lista completa de módulos + capturas de UI por módulo |
| 3 | Pricing | https://strategyquant.com/es/precios-2/ | qué features gated por tier (Starter/Pro/Ultimate) |
| 4 | Docs / Help | https://strategyquant.com/es/doc/ | NO leída aún — siguiente paso: leer secciones Builder, Pruebas cruzadas, WF Matrix, ATM, Plantillas |
| 5 | Demo interactiva | https://www.appdemostore.com/embed?id=4674799715483648 | flujo Builder click a click (no cargó por fetch; abrir en browser) |
| 6 | Demo "Crear primeras estrategias" | https://www.appdemostore.com/embed?id=6360454782779392 | flujo de generación |
| 7 | Demo "Pruebas de robustez" | https://www.appdemostore.com/embed?id=5292622835351552 | flujo robustez |
| 8 | Roadmap público | https://roadmap.strategyquant.com | qué prioriza el equipo, bugs/solicitud usuarios |
| 9 | Docs data / Dukascopy | https://strategyquant.com/es/suscripcion-de-datos/ | fuentes de datos gated por tier |
| 10 | Foro | https://strategyquant.com/es/forum/ | idiomas, flujos reales, frustraciones |
| 11 | Reviews (pendiente) | G2 / Capterra / Reddit r/algotrading | skill replica-entrepreneur lo hace después |

## Core loop

Configurar un build (mercado, datos, plantilla, filtros) → generar cientos/miles de estrategias automáticamente → el motor las backtestea y somete a pruebas de robustez → las que pasan entran al "Bank of Strategies" → se exportan como código fuente a la plataforma de trading del usuario.

## Screens

| ID | screen | route / how to reach | purpose | key components | states seen |
| --- | --- | --- | --- | --- | --- |
| S01 | Ventana principal / panel proyecto | arranque app | hub: lista de builds/proyectos, tabs del módulo | sidebar de módulos, tabla de proyectos, toolbar | vacío (sin proyectos), cargando, con builds activos |
| S02 | Builder — configuración | S01 → botón "New build" | configurar generación: símbolo, TF, datos, plantilla, nº de estrategias, filtros | formularios con secciones (Data/Randomizer/Filtering), dropdowns, checkbox robustez | configuración mínima, perfil guardado |
| S03 | Builder — ejecución y cola | S02 → Start | progreso de generación en vivo, pausa/stop | barra progreso, log en vivo, contadores strategies passed/failed, cola de builds | corriendo, pausado, completado, fallido |
| S04 | Bank of Strategies / databank | S01 → tab | tabla de estrategias generadas con métricas | tabla sortable, filtros en columnas, paginación, columnas custom (Net Profit, Sharpe, DD, PF, WFE...) | vacía, poblada, filtrada, dirty (estrategias inválidas detectadas) |
| S05 | Detalle de estrategia — resultados | S04 → doble click | métricas completas + gráfico equity + stats | gráfico equity/MAE/MFE, panel de stats, tabs por test | con resultados, con fallo de robustez |
| S06 | Gráfico de operaciones | S05 → tab Chart | velas + indicadores + trades marcados | candle chart, superposición indicadores, marcadores de trades | con/sin indicadores visibles |
| S07 | Editor de estrategia (no-AlgoWizard, tabla/lógica) | S05 → Edit | ver/editar lógica: condiciones, indicadores, parámetros | árbol de reglas, lista condiciones, dropdown de indicadores | vista simple, editada, como plantilla |
| S08 | Retester — config + resultados | S01 → módulo | re-test de estrategias en otro mercado/TF/config | selector de estrategias del bank, params, tabla resultado | nuevamente pruebas de robustez on/off |
| S09 | Improver | S05 → Improve | optimizar una parte de la estrategia (entrada o salida) con ML | selector de pieza a mejorar, config, resultados comparativos | solo entrada, solo salida, ambas |
| S10 | Optimizador — config/run | S05 → Optimize | optimize Simple o Walk-Forward sobre parámetros | grilla de parámetros y rangos, botón run, resultados 2D/3D | sin rango = error, resultado vacío |
| S11 | Gráficos 3D de optimización | S10 → results | vista 3D de superficie de combos WF | gráfico 3D interactivo, rotación | plano, clusterizado, disperso |
| S12 | Walk-Forward Matrix | S05 → WF Matrix | clustering de reoptimizaciones para detectar zonas estables de parámetros | tabla por rango parámetro, heatmap, gráficos | clusters estables vs dispersión |
| S13 | Monte Carlo | S05 → Monte Carlo | simulations (trades shuffle, skipped trades, randomize params,…) con métricas p-valor | selector de testeos (9+ tipos), tabla de resultados con confianza | pasó/no pasó, config por defecto |
| S14 | SPP (System Parameter Permutation) | S05 → Robustez | permutación de parámetros: prueba estabilidad vía re-test con params alterados | config de tests, tabla resultados | in/out rojo/verde |
| S15 | Optimization Profile | S05 → Robustez | perfil de optimización tipo Pardo: tendencia estable vs pico azaroso | gráfico rendimiento por valor de param | estable, unimodal, plana, puntual |
| S16 | Custom Projects — editor de workflow | S01 → Projects | crear flujo multi-paso (build→test→filter→optimizer→…) | canvas de bloques conectados, palette de steps, config por step | vacío, plantilla,taus recientes 
| S17 | Custom Projects — ejecución | S16 → Run | pipeline corre pasos encadenados con outputs entre steps | progresión de pasos, log, resultados por paso | corriendo, abortado, completado |
| S18 | Portfolio Master / Composer | S01 → Portfolio | agrupar estrategias en cartera con modelos de ponderación | selector bank, lista PDF, config sin/correlation, equity combinado | 1 estrategia, 4 (límite Pro), ilimitado (Ultimate) |
| S19 | Portfolio — resultados | S18 → Results | equity de cartera, métricas combinadas, análisis de riesgo | gráfico equity de portfolio, tabla por estrategia, correlación | correlacionadas vs no |
| S20 | Data Manager — fuentes y descarga | S01 → Data | descarga histórica: Dukascopy (forex/CFD) gratis, Yahoo (stocks) gratis, premium negociado | tabla de símbolos, source, TF, rango fechas | sin datos, descargando (progreso), data corrupta |
| S21 | QuantManager — import/export datos | S20 → tab | gestionar tick/min/EOD, importar fuentes externas, exportar a formato backtest | file picker, lista datasets, conversor | import ok, error formato |
| S22 | Export — diálogo plataforma | S05 → Export Code | elegir plataforma (MT4/MT5/Ninja/TS/MultiCharts/JForex) y generar código fuente | selector plataforma, opciones de código, preview, guardar fichero | generado ok, error en librería requerida |
| S23 | Ajustes globales de app | S01 → Settings | dirs de datos, proxy, tema, lenguaje, licencia/licencia chips | prefs panel, tabs por categoría | primer arranque (wizard no queda claro; flag: investigar) |
| S24 | QuantAnalyzer — dashboard análisis | producto separado, mismo bundle | analiza backtests: métricas avanzadas, Monte Carlo portfolio, correlación, money management | tabs por analytics, file import .sqx/.mtm | sin archivos, con varias estrategias cargadas |

## Flows

```
F01 Generar estrategias (core loop)
    S01 -> S02 -> S03 (corre) -> S04 (bank) -> S05
    happy path clicks: ~8 + horas de cómputo
    edge: sin datos descargados para el símbolo, plantilla inválida, filtro elimina todas
F02 Revisar resultados de una estrategia
    S04 -> S05 -> S06 (chart) -> S04
    happy path clicks: 4
    edge: estrategia marcada con problema de detección automática
F03 Escalar → Builded in version P: mejorar estrategia (Improver)
    S04 -> S05 -> S09 -> S05
    clicks: 5
    edge: peor resultado que el original (mantener mejor)
F04 Optimizar (Simple / Walk-Forward) + leer 3D
    S05 -> S10 -> S11
    clicks: 4
    edge: rango de parámetro inválido, WF Efficiency bajo
F05 Pruebas de robustez automáticas en un build
    S02 (checkbox on) -> S03 (corre Monte Carlo + SPP + Opt Profile integrado) -> S04 (filtrado automático)
    edge: todas eliminadas por Monte Carlo
F06 Crear workflow custom (Custom Project)
    S01 -> S16 -> S17 -> S04
    clicks: 10
    edge: paso sin output esperado, loop infinito mal armado
F07 Construir portfolio
    S04 (selección) -> S18 -> S19
    clicks: 6
    edge: estrategias correlacionadas al 90%+, límite 4 en tier free
F08 Exportar estrategia a plataforma
    S05 -> S22 -> guardar .mq4/.mq5/.cs/...
    clicks: 4
    edge: librería de runtime sin instalar (NinjaTrader), síntaxis no soportada
F09 Preparar datos (Data Manager)
    S20 -> descargar Dukascopy -> S21 (import extra) -> S02 disponible
    clicks: 5
    edge: fuente caída, gap en datos, resample interminable
F10 Retest en otros mercados
    S04 (multi select) -> S08 -> S04
    clicks: 5
    edge: estrategia multi-simbolica → hay que elegir nuevos inputs
F11 Análisis profundo QuantAnalyzer
    S24 -> importar .sqx/strategies -> Monte Carlo de cartera, correlación, MM
    >> usar esto para portfolio decisions pre-live
```

## Components

| component | variants | states | used on |
| --- | --- | --- | --- |
| Sidebar de navegación de módulos | collapse, labels+icons | hover, selected | S01, todas |
| Toolbar de módulo | botones: New/Run/Save/Delete | enabled, disabled (sin selección) | S04, S08, S16, S18, S20 |
| Tabla de datos (grid) | databank, portfolio, data manager | sortable, filtrable por columna, paginada, columnas custom, dirty rows | S04, S08, S12, S14, S18, S20, S24 |
| Panel de stats | tarjeta con métrica | verde/rojo vs baseline | S05, S10, S13, S19 |
| Gráfico equity/balance/drawdown | en línea, con sombreado condicional | loading, empty, con trades | S05, S09, S13, S19, S24 |
| Candle chart + overlay indicadores | TF selector, símbolo | loading, sin datos | S06 |
| Gráfico 3D de superficie | rotación, color escala | cargando, vacío | S11 |
| Heatmap de matriz (WF Matrix) | filas = param A, cols = param B | cluster en verde, deconcentrado en rojo | S12 |
| Dialog modal | config, export, confirm delete | ok, error, cancel | S02, S08, S09, S10, S22 |
| Form config multi-sección | acordeón por categoría | corriente, validados | S02, S08, S09, S10, S13 |
| Progress bar + live log | linear, indeterminado | running, paused, done, error | S03, S17, S20, S24 |
| Selector de símbolo/TF | dropdown con búsqueda, favoritos | vacío, con datos, sin datos (🚫) | S02, S08, S18, S20, S24 |
| Selector de indicador/bloque | dropdown agrupado, searchable | vacío, con custom (snippet) | S02, S07 |
| Badge de estado de test | verde (pasó), rojo (falló), gris (corriendo) | disallow run | S03, S04, S12, S13, S14, S15 |
| Drag&drop del canvas de WF | nodes conectados | hover, seleccionado, cuadro de conexión | S16 |
| Bottom dockable log | stdout-like, niveles | info/warn/error | S03, S17, S24 |
| Toolbar de chart | pan/zoom/crosshair/Mejoras | on/off | S06, S11, S15 |

## Inferred data model

```
Symbol      name, exchange, asset_class (forex|cfd|stock|future|crypto)
            evidence: S20 data manager, S02 selector
            confidence: high

DataFile    symbol_id, timeframe, type (tick|m1|eod), compression, date_from, date_to, source (Dukascopy|Yahoo|file), size_mb, checksum
            evidence: S20 descarga, S21 gestión
            confidence: high

Strategy    id, name, market (tick), timeframe, created_at, source (builder|improver|import/exported|random), 
            template_id, logic (tree de bloques: entry, exit, stop, position_sizing, filters),
            params[], marketplace_code (referencia a export), notes, status (raw|filtered|invalid-reasons[])
            evidence: S04 tabla, S07 editor, S22 export
            confidence: high (estructura lógica inferida de UI: árbol de bloques)

BacktestRun strategy_id, data_file_id, engine_config (spread, slippage, initial_balance, position sizing),
            metrics (net_profit, gross_profit, sharpe, sortino, profit_factor, win_rate, max_dd, rdd, trades, avg_trade, z-score, r_squared, WFE...)
            series (equity[], mae[], mfe[], daily_equity[])
            problems[] (auto-detección de fallos comunes)
            evidence: S05 tablas métricas, features page
            confidence: high

RobustnessRun strategy_id, type (monte_carlo|wf_matrix|spp|opt_profile|oos_check),
            config JSON, verdict (pass|fail), detail per trial
            evidence: S12-S15, features "pruebas de robustez"
            confidence: high

Build       id, type (builder|retester|improver|optimizer|custom_project),
            config JSON, status, progress, log[], results (n strategies passed, bank ref)
            evidence: S03, S08, S09, S10, S17
            confidence: high

WorkflowStep* order, type (build|test|filter|optimizer|monte_carlo|quant_analysis|python_call|custom_analysis), 
            inputs_from_previous_outputs, config
            evidence: S16 canvas + "Custom Analysis (external scripts Python)"
            confidence: medium (estructura concreta del evento la confirman los docs oficiales)

Portfolio   strategies[], weighting model (equal_vol|risk_parity|custom), combination mode, 
            metrics (equity, correlación matrix, max DD conjunto, rebalance)
            evidence: S18, S19, features "Portfolio Master"
            confidence: high

Export      strategy_id, platform (MQL4|MQL5|NinjaScript|EasyLanguage|MultiCharts|JForex), 
            code (string), generated_at, includes (indicators custom, sniffer de libs)
            evidence: S22, features "Envía tu estrategia…"
            confidence: high

LicensePlan tier (starter|professional|ultimate), payment (single|12-installment), features_gated[], 
            data_subscription (forex/stocks/futures), ai_credits (300/1200/2400/6000)
            evidence: Página de precios íntegra
            confidence: high (solo como referencia del qué gates; en el clon se decide otro modelo)

Templates   id, name, type (standard|fuzzy|multi_market|trading_system_custom), placeholders[],
            evidence: features "Generate your own template style"
            confidence: medium
```

Relaciones:
- Symbol 1-n DataFile
- DataFile n-n BacktestRun (una estrategia opera sobre un DataFile concreto)
- Strategy 1-n BacktestRun, 1-n RobustnessRun, 1-n Export
- Build 1-n Strategy (outputs del bank)
- Portfolio n-n Strategy
- WorkflowStep pertenece a Project (n-m con outputs/inputs)
- Template 1-n Strategy

## Feature matrix

See `features.csv`. Must: {{n}}, should: {{n}}, could: {{n}}, skip: {{n}} — ver CSV.

## Out of scope (cannot or should not be cloned)

- **SQ for Business / MQL Market publishing** — red de distribución de terceros (MQL5.com). No es parte del producto, es un canal.
- **Paquete EDUCACIÓN** (e-book, curso 56 lecciones, estrategias de regalo) — contenido intelectual propio de StrategyQuant. Skip irreducible.
- **Datos Premium de stocks/futuros (30 años)** — son suscripciones a proveedores. El clon integrará fuentes públicas propias, nunca su feed.
- **Marca StrategyQuant, logos, screenshots de marketing, copy de la web** — todo se reescribe.
- **AI Assistant "Q" + créditos IA** — fuera de scope según el acuerdo.
- **Market & Volume Profile (v144)** — fuera de scope.
- **Licencia de por vida, FastSpring, soporte Skype** — modelo comercial suyo.
- **Código fuente de su motor genético y backtesting** (Java/c++) — líneas de implementación propias, se rediseña desde cero.

## Size (honesto)

- Screens: ~24 contando paneles propios + QuantAnalyzer
- Flows: 11
- Entities: 10
- **Hard parts (compatibilidad y precisión, no solo UX):**
  1. Motor de backtesting con precisión de tick multi-símbolo/multi-TF (hay que diseñarlo desde cero; velocidad "miles de backtests/seg" es un objetivo de ingeniería serio)
  2. Generación genética con plantillas + placeholders e exportadores de código para 6 plataformas con código fuente completo
  3. Suite de robustez (Monte Carlo 9 variantes, WF Matrix clustering, SPP, Optimization Profile) — es matemática, no CRUD
  4. Persistencia eficiente de millones de backtests (banks + filtros en vivo)
- **Tamaño: XL — rescope obligatorio.** El `replica-architect` deberá cortar en vertical slices:
  - Fase 1 (MVP funcional): motor backtick + builder básico + bank + export MQL5 → clonare habitable
  - Fase 2: robustez (MC + WF), improver, optimizer
  - Fase 3: custom projects, portfolio, QuantAnalyzer, retester multi-market, export resto plataformas
  - Fase 4: data manager nuevo + QuantDataManager + QuantAnalyzer avanzado

## Notas de riesgo de términos

- Términos SQ de la licencia prohíben usar la VERSIÓN PROPIA del software para construir competencia. El usuario NO tiene cuenta → no hay riesgo de violación de TOS propia. Recon solo fuentes públicas. Bien.
- MQL/Ninja/Tradestation son marcas y plataformas de terceros: el clon puede "exportar código" para ellas pero no debe usar sus nombres en UI final hasta rebrand (la skill replica-brand se encarga).
