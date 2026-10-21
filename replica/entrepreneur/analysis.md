# Análisis y plan de correcciones — replica-entrepreneur

Entrada: `feedback.md` (ranking automático) + lectura manual de las 52 filas.
Todo lo que sigue se apoya en citas reales con enlace en `feedback.md` / `reviews.csv`.
Las palabras de los revisores son **investigación, no testimoniales**: nada de esto va a la landing como cita publicitaria.

## 1. Qué odian (ordenado por peso real, contrastando el tool con lectura manual)

| # | Tema | Evidencia clave | Impacto en QuantForge |
| --- | --- | --- | --- |
| 1 | **Sobreajuste estructural** — lo que genera no sobrevive fuera de muestra | "The overfitting risk is structural, not incidental." (StatOasis) · "what I keep finding is overfitting … the only metric that has shown meaningful predictive value is drawdown" (ing. con 30 estrategias en vivo, r/mltraders) · "It's essentially a high-powered curve-fitting engine" (r/mltraders) | El problema nº1 del original. QuantForge ya tiene MC×9 + WF Matrix + SPP reales; hay que **hacerlos imposibles de saltarse por accidente**: flujo guiado que obliga a pasar por la validación antes de dar "apta" a una estrategia. |
| 2 | **Rendimiento del generador: resultados mediocres o nada** | "out of 10k strategies not even one passed" (r/algotrading) · "the best profit factor I got … was 1.63" (r/algotrading) · "I tried it and … 12% profit per year, which is not that impressive" (r/algotrading, 2024) | Expectativas: la UI nunca debe vender belleza del backtest; mostrar Δ OOS y probabilidad de supervivencia al lado del PF IS desde el primer listado. |
| 3 | **Complejidad / curva de aprendizaje** | "kinda hard to use (too many options)" · "Steep learning curve (months)" · "The interface still feels dated. Functional, dense, not pretty." · "misreading one [walk-forward matrix] is worse than not running one" | Nuestra baza real: **toda la UI en español + modo guiado**. V1 ya define 24 pantallas; añadir tooltips educativos sobre qué es cada test y qué NO significa. |
| 4 | **Priceado: ediciones cerradas que esconden la validación** | "Starter at $1,290 leaves out the robustness testing … Buy Professional, or buy nothing." (StatOasis) · "It only includes one year update. The real lifetime … $2900." (r/algotrading) · "Transferring your licence: a $300 transfer fee" (StatOasis) | **Anti-patrón explotable en nuestra contra-ficha**: en QuantForge la batería completa de robustez va incluida de serie — ninguna edición se vende sin los frenos; soporte/updates con política clara y prolongable a precio plano, sin canón de transferencia. Directo para /replica-launch. |
| 5 | **Hardware: precios altos + máquina grande** | "I require a good machine to run effetively." (r/algotrading) · "Long runs are slow on a laptop." (StatOasis) | Ventaja del core Rust: correr en portátil normal. Hay que **medir y publicar velocidad** (es una feature, no un detalle técnico). |
| 6 | **WFA/opciones nativas opacas** | "the native WFA felt like a black box … I moved the walk forward work outside the platform… Python" (r/mltraders) · "I don't control the full information inside that plugin — I don't really know how it works" (r/mltraders) | Transparencia radical: documentar cada fórmula (WFE, percentiles, umbrales) en la propia app; permitir exportar CSV/JSON de TODOS los resultados intermedios. Barato de hacer, alto valor percibido. |
| 7 | **Exportaciones que no reproducen resultados** | "when exporting to Metatrader, we noticed that the results of the strategies do not match" (r/algotrading) · "Exports can behave differently elsewhere. … This is the failure mode that costs real money." (StatOasis) · "I can't run my strategies built in SQ on MT5 … 'failed with code 1'" | El exportador MQL ya existe; el diferenciador honesto es un **aviso de verificación** integrado: checklist post-exportación ("regenera y compara en tu MT del broker; desviación esperada X%") + "solo capital real tras verificación en plataforma destino". Documentado también en backend.md. |
| 8 | **Gestión de riesgo/portfolio de nivel superior** | "the biggest drawback is … lacking portfolio level risk management" (r/algotrading) · "so they dont have portfolio level risk management?" (r/algotrading) | Ya en alcance: Portfolio Composer + correlaciones reales. Prioridad de construcción: subirla un nivel (decisión pendiente V1: pesos/ATM — decidir ya con estos datos: pesos + matriz de correlación es lo pedido). |
| 9 | **Privacidad / venta de confianza** | "once StrategyQuant got caught 'calling home' and sending test results back … I'm not paying you, so you can double dip" (r/algotrading) |Producto local (SQLite, sin telemetría) → leverage honesto: "todo corre en tu máquina; cero telemetría". Afirmable y verificable en código. |
| 10 | **Datos: gestión/obtención** (lo MÁS pedido en Reddit) | QuantDataManager aparece como respuesta estándar en ≥8 hilos (descarga de tick/Dukascopy, gratis aunque "slow for downloads") | **Decisión alineada**: si el Dukascopy-deferred se adelanta, ataca el punto de entrada de más usuarios. NOTA: para lanzar, sin QDM propio, el pitch debe integrar CSV import + docs de fuentes gratuitas (Dukascopy manual) — deregación documentada. |
| 11 | **Ruido de afiliados/gurús** | "creators who talk about how profitable the software is … conveniently give you a promotional code" (r/mltraders, ing. 30-estrategias) | Tonos de comunicación: cero códigos de descuento de terceros, cero promesas de rentabilidad. Landing con estructura de "qué hace / qué NO hace" (límite: sin claims de profit). |

## 2. Qué piden y que no existe (huecos reales del original)

1. **Validación del proceso completo, no solo de la estrategia elegida**: "It's important to evaluate not only the robustness of the final strategy but the entire research/search process." (r/mltraders, EAForexLab). SQX da filtros por estrategia; cualitativamente nadie cubre "te dejo ver por qué descartamos"— si QuantForge añade panel "por qué esta quedó fuera" (historial de filtros por run), sería único.
2. **Meseta/paridad en vivo**: "execution drift or discrepancy between SQX's simulated order fills and your actual paper/live trading fills" (pregunta abierta en r/mltraders). Fuera de alcance V1, pero citable como roadmap.
3. **Enlace/CLI Python**: "SQX is a powerful tool … if they linked to python… wow" (r/algotrading). Nuestro core Rust exporta CSV/JSON con API Tauri; en V1 el CLI de exportación ya da esto en bruto. Citable como roadmap.
4. **Gestión de dinero en cripto**: "It's just that it's the only part of strategyquant that I don't understand how to use for cryptocurrencies." (r/algotrading) — soporte/config por clase de activo con guía en español.
5. **Export final a móvil/no-code**: "❌ No mobile access" + telemetría evitada — fuera de alcance, anotado como rechazo consciente.

## 3. Plan de correcciones para QuantForge (priorizado, con estimación V1)

| Prioridad | Corrección | Dónde | Coste |
| --- | --- | --- | --- |
| P0 (antes de lanzar) | Flujo guiado que **obligue** a pasar por validación (Estado "sin validar" visible hasta MC+WF+SPP) | Módulo Builder/Retester, guard en UI | Bajo (JS) |
| P0 | Transparencia total de fórmulas: tooltips + página "cómo se calcula" (WFE, drawdown p95, umbrales) | Docs + app | Bajo (MD) |
| P0 | Posicionamiento y precio contra complaints: robustesa incluida de serie, precio único, sin transfer fee, soporte/updates ≥1 año incluido | /replica-launch (pricing + landing) | 0 (texto) |
| P1 (V1 si hay tiempo; V1.1 si no) | Portfolio Composer con pesos + matriz de correlación + equity portfolio (ya reales) | Módulo Portfolio | Medio (ya existe backend; falta UI de pesos) |
| P1 | Aviso de verificación de export + checklist documentado | Exportador MQL → docs + UI warn | Bajo |
| P1 | Benchmarks de velocidad publicados del core Rust | README/landing | Bajo (medir) |
| P2 | "Por qué descartamos" (historial de filtros por estrategia en WK Proyecto) | Custom Projects | Medio — enorme diferenciador |
| P2 | Triple split (train/validación/test) además de WFA | Motor | Medio |
| P2 | Enlace Python (CLI estable + docs) | Renderer | Ya existe básico |
| Decision para el usuario | ¿Adelantar Dukascopy/QDM analog? (datos = puerta de entrada) | Data Manager | Alto (integración real) |

## 4. Ángulo de posicionamiento (para /replica-launch)

**Diagnóstico:** las críticas fuertes no son "el motor es lento" o "falta X"; son (a) te venden un generador y te cobran aparte los frenos, (b) sin validación ni hipótesis, el resultado es un fabricante de ilusiones, (c) la comunidad/afiliados agrava el ruido con promesas de rentabilidad.

**Ángulo:** *QuantForge es el taller con los frenos de serie.* En vez de "genera estrategias rentables sin programar" (promesa implícita del original, desmentida por sus propios usuarios), el pitch honesto es:

- **Todo el arsenal de validación incluido desde el primer día** — sin ediciones que desbloqueen la verdad a +$200.
- **En español, de tu máquina, sin telemetría** — respuesta directa a "calling home", a la curva de aprendizaje (UI/educación en español) y al miedo creciente a servicios en la nube.
- **Rust = velocidad para portátil normal** — razonamiento contra "necesitas una bestia" y contra el stack Java lento del original.
- **No prometemos rentabilidad ninguna; prometemos que cada número que veas es reproducible y auditable.**

Frase candidata (fecha para revisar una vez defina marca): de momento en casting:
1. "La validación que otros cobran aparte, incluida de serie."
2. "Menos promesas, más pruebas. En español."
3. "Un Motor de verificación, no un vendedor de soñadores."

Notas para /replica-launch: comparativa de precios contra los datos públicos verificados 19-08-2026 (Starter $1,290 / Professional $1,490 / Ultimate $2,900 + $300/año + transfer $300 + AlgoCloud $8-82/mes, fuente StatOasis) pero **verificado por nosotros** en la página de precios propia antes de publicar nada (los precios rotan; no citar cifras de terceros ni de memoria).
