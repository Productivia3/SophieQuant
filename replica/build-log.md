# Build log — QuantForge

## 2026-10-20 · Rust instalado — el núcleo compila

- `winget install Rustlang.Rustup` → rustup 1.29.1 + **rustc 1.99.1 stable-msvc**
  (los Visual C++ Build Tools ya estaban en la máquina: vswhere 2019 BuildTools + Windows Kits).
- Iconos generados (clean-room, velas sobre fondo oscuro con acento): `src-tauri/icons/icon.ico` + `icon.png`.
- `cargo check` en `src-tauri` → **0 errores** tras 3 iteraciones de fixes:
  1. `[lib]` seccado del Cargo.toml (el crate es binario con `main.rs` — no hay lib.rs).
  2. `tauri.conf.json`: `app.title` no existe en el schema v2 (el título vive por ventana).
  3. `include_str!("../schema.sql")` → `../../schema.sql` (rutas relativas al fichero fuente).
  4. `serde::{Serialize, Deserialize}` sin importar en rules.rs; derives faltantes en
     `Trade` (Serialize/Deserialize) y `Signal` (PartialEq/Eq para `==`).
  5. 7 `?` sobre rusqlite convertidos a `.map_err(|e| e.to_string())?` en commands.rs
     (firmas devuelven `Result<T, String>`); añadido `tauri::Manager` para `app.path()`.
  6. Limpieza: `indicator_logic` huérfana en export.rs, `Path_box` en commands.rs,
     import `std::io::Read` muerto.
- Quedan 10 warnings inofensivos (campos/métodos nunca leídos, 3 `mut` innecesarios) —
  se limpian en una pasada de `cargo fix` antes del release.
- Siguiente paso: `npm run tauri dev` (primera build de debug en marcha) → app de escritorio
  con núcleo real; después la tanda de QA bloqueada (golden MQL, importer CSV, determinismo).

## 2026-10-20 · Fase 4 (UI sobre capa fake): las últimas 5 pantallas + sidebar móvil

La UI del recon queda completa: **24/24 pantallas** tienen implementación real (fake data).

- **S15 Optimization Profile** — `done (fake)`: perfil tipo Pardo con curva net-vs-parámetro
  en SVG propio, banda de zona estable sombreada y veredicto de forma
  (meseta/unimodal/puntual con su explicación). Acceso desde S14 (enlace cruzado).
- **S19 Portfolio resultados** — `done (fake)`: totals (net combinado, DD de cartera,
  sharpe, % de DD evitada por diversificación), equity combinada y tabla de aporte por
  miembro con veredicto de correlación. Acceso desde S18.
- **S21 QuantDataManager** — `done (fake + import real)`: tabla de datasets, **import de
  CSV vía file picker real** (Playwright `setInputFiles` — lectura de fichero del
  navegador), export a csv/qfd con selector de formato y aviso de éxito.
- **S23 Ajustes** — `done (fake)`: carpeta de datos, hilos del motor (slider 1-16),
  proxy, tema, idioma, autoguardado; guarda con confirmación (sesión; en Rust irá a
  fichero de config junto a la DB).
- **S24 QuantAnalyzer** — `done (fake)`: pestañas Resumen (métricas agregadas del bank,
  problemas detectados) / Monte Carlo de cartera (500 trials, percentiles, prob. de
  rotura) / Money Management (riesgo por op → DD esperado; nota de orientativa).
- **BUG-005 CERRADO**: nav ahora es drawer en móvil (<768px) con botón hamburguesa
  (`aria-expanded`), overlay `aria-label="Cerrar menú"` y cierre automático al navegar.
  En desktop ≥768 sigue fijo con los 224px.
- **Bug real cazado por el e2e**: `listDataFiles` fake devolvía `db.dataFiles` por
  *referencia viva* — al importar, el push mutaba la lista que ya estaba en el estado del
  componente y el append duplicaba la fila (React warning de keys duplicadas, capturado
  por el guard de console.error del afterEach). Fix: copias defensivas
  (`[...db.dataFiles]`, `[...db.strategies]`) en todos los getters fake.
- **Infra de tests**: `playwright.config.ts` ahora lleva `webServer` con
  `reuseExistingServer: true` — la suite levanta su propio dev server si el fondo murió
  (ya había muerto dos veces a mitad de suite; fin del flake de infra).
- **Compilación**: tsc+vite limpio (281 KB, gzip 83 KB).
- **QA**: suite completa **77/77 ✓** (19 tests nuevos: flows S15/S19/S21/S23/S24,
  drawer móvil ×2, axe ×6).

## Estado acumulado: 24/24 pantallas / 0 placeholders (todas `done (fake)`)

Solo quedan como stub S25 /diff (se llena con /replica-diff). Bloqueo único restante:
instalar Rust para compilar `src-tauri` (motor real, SQLite, exporters MQL, importer real).

## 2026-10-20 · Fase 3 (UI sobre capa fake): retest, improver, optimizer+3D, SPP, projects, portfolio

- **S08 Retester** — `done (fake)`: selección múltiple del bank con checkboxes, selector
  símbolo/timeframe, tabla comparativa original vs retest (net/PF/DD) con veredicto
  PASA/ALERTA/FALLA por estrategia. Pendiente: motor Rust del retest.
- **S09 Improver** — `done (fake)`: genera 4 variantes (entrada ATR, time-stop, filtros
  sesión/ATR), tabla Δ vs original y banner "mejor variante" o "se conserva el original"
  (regla del recon F03: nunca empeora). Pendiente: motor Rust.
- **S10 Optimizer** — `done (fake)`: modo Simple/Walk-Forward (toggle radio a11y), slider de
  pasos 3-9, grid de combos con superficie 3D embebida + stat cards del mejor combo.
- **S11 Superficie 3D** — `done (fake)`: isométrica SVG propia (sin librerías), barras
  coloreadas por PF, mejor combo resaltado con borde, etiquetas de ejes y modo.
- **S14 SPP** — `done (fake)`: 6 permutaciones (entradas/salidas revertidas, split, señal
  opuesta, param permute, random entry), barras de impacto % y veredicto por prueba +
  veredicto global. Pendiente: Optimization Profile (Rob Pardo) y motor Rust.
- **S16 Custom Projects** — `done (fake)`: lista ordenada de pasos (build → MC → SPP →
  optimizer → save), añadir/quitar/reordenar con botones a11y, ejecución secuencial con
  badge por paso y cierre por fallo. Nota honesta en pantalla: canvas drag&drop postergado.
- **S18 Portfolio Master** — `done (fake)`: selección 2-8 estrategias, matriz de calor de
  correlación (rojo ≥ 0.75 = diversificación débil) + equity combinada equiponderada.
  Pendiente: composer con ponderaciones y ajuste de nueva estrategia al portfolio.
- **Capa fake**: 8 métodos nuevos en `api.ts` con tipos exportados (RetestResult,
  ImproverResult, OptimizerResult, SppResult, CorrelationMatrix, PortfolioEquity,
  ProjectRunResult) — mismo contrato para el swap Rust.
- **Compilación**: tsc+vite limpio (255 KB, gzip 78 KB).
- **QA**: nuevo spec `e2e/f-phase3.spec.ts` (14 casos: 12 hammer+edge + 6 axe) → suite
  completa **56/56 ✓**. Flake del F01-E2 (doble clic) diagnosticado: el segundo click
  esperaba 30s la accionabilidad de un botón ya desmontado → fail-fast con `timeout: 1200`.
  El dev server murió a mitad de suite (ERR_CONNECTION_REFUSED) — reiniciado en background.
- **features.csv**: 10 filas más a `partial` (retest, improver, optimizer ×3, SPP,
  projects, portfolio ×3).
- Pendiente Fase 3+: S21 QuantDataManager (suite completa), S23 Ajustes, S24 QuantAnalyzer,
  sidebar colapsable (BUG-005), canvas drag&drop de S16.

## Estado acumulado (19 páginas reales — cubren 20/24 pantallas del recon)

S01, S02, S03, S04, S05, S06, S07, S08, S09, S10, S11, S12, S13, S14, S16, S18, S20, S22,
/design. La página S16 fusiona S17 (ejecución del workflow). Quedan: S15 Optimization
Profile, S19 Portfolio resultados, S21 QuantDataManager, S23 Ajustes, S24 QuantAnalyzer (Fase 4).
Bloqueo continúa igual: instalar Rust para compilar `src-tauri`.

## 2026-10-20 · /replica-test — QA e2e sobre la capa fake

- Instalado Playwright + axe-core; `playwright.config.ts` (baseURL 1420, hash-router tomado en cuenta).
- 3 specs en `e2e/`: `f-core.spec.ts` (F01/F02/F09), `f08-export.spec.ts`, `x-transversal.spec.ts` (20 rutas + axe + 404 + móvil).
- Resultado: **40/40 ✓** tras fix-loop.
- Bugs: BUG-001 (progressbar sin aria-label, S2 → fixed), BUG-002 (checkbox sin label, S2 → fixed), BUG-004 (tablas sin scroll-container en móvil, S3 → fixed), BUG-005 (sidebar no colapsa <700px, S3 → **open**, parrilla Fase 3), BUG-003 fue del test.
- Detalles y casos manuales/bloqueados: `replica/test-plan.md` + `replica/bugs.md`.
- Nota técnica: la app usa HashRouter — los e2e navegan con `/#/ruta`.
- Salud: ninguna pantalla cambió de contrato; solo a11y y wrappers de scroll.

## 2026-10-20 · Opción 2 — núcleo Rust completo listo para compilar

Escritos en `src-tauri/` (no compilables en esta máquina por falta de toolchain —
requiere `winget install Rustlang.Rustup`, luego `cargo check` en src-tauri):

- **engine/types.rs**: Bar/Trade/EngineCfg/Metrics/BacktestOutput (DTO camelCase vía serde).
- **engine/indicators.rs**: SMA, EMA, RSI, ATR, highest/lowest, cruces. 0 dependencias.
- **engine/rules.rs**: 3 plantillas V1 (cruce de medias, reversión RSI, ruptura ATR)
  con señales Long-only, sin lookahead.
- **engine/backtest.rs**: motor por barras — ejecución en apertura de la barra siguiente,
  spread+slippage+comisión, stop/target por ATR, stop intrabar-first (conservador).
- **engine/metrics.rs**: net/PF/winrate/maxDD(%)/RFR?/sharpe/sortino/z-score/r²/avgTrade + equity muestreada.
- **engine/genetic.rs**: 24 individuos × generaciones, mutación gaussiana sobre supervivientes,
  rayon paralelo, filtros (n_trades≥20, PF≥min, DD≤max), presupuesto de iteraciones, cancelable.
- **engine/datastore.rs**: SQLite WAL + schema idempotente; formato binario propio QFN
  (i64+OHLCV); importador CSV (time unix/ISO, OHLCV, validación OHLC).
- **export.rs**: generadores MQL4 y MQL5 completos para las 3 plantillas (clean-room).
- **commands.rs**: 12 commands Tauri — list_symbols/list_data_files/import_data_csv/
  list_templates/start_build/list_builds/cancel_build/list_bank/get_strategy/
  export_strategy/get_settings/update_settings. Los eventos de build emiten por canal "build-event".
- **main.rs**: setup de DB en AppData + registro de commands.
- **tauri.conf.json**: validez withGlobalTauri=true para el puente del navegador→app.

**Puente UI→Rust**: `src/lib/api.ts` ahora prueba `window.__TAURI__.core.invoke` primero y
cae a la capa fake en el navegador. Ninguna pantalla cambió (contrato cumplido).

**Diferencias honestas V1 (a resolver antes que el parity suba):**
- Timeframe de backtest = el del dataset importado; resample tick→m1 llega con el descargador.
- Short trading, Monte Carlo, WF Matrix, SPP, Optimization Profile, Perf.: aún fake en UI.
- Dukascopy downloader real (bi5+lzma) pendiente — de momento CSV import funciona de verdad.
- Los `icons/` del bundle Tauri por añadir.
- FIRE: pérdida declarada — `pause/resume` de builds no hay commands todavía (solo cancel).

## Bloqueo y siguientes pasos

1. Instalar Rust → `cd src-tauri; cargo check` → corregir lo que el linter señale (2-3 iteraciones).
2. `npm run tauri dev` → app de escritorio con núcleo real.
3. Importar un CSV de EURUSD m1 (o escribir el descargador Dukascopy) → primer build real.

## 2026-10-20 · Fase 2 (UI sobre capa fake): robustez, chart y editor

- **S13 Monte Carlo** — `done (fake)`: 9 simulaciones con checkboxes, corrida con estado
  cargando (skeleton), tabla con veredicto/tasa de pass/percentiles PF. Pendiente motor real.
- **S12 WF Matrix** — `done (fake)`: heatmap WFE por combinación de parámetros con leyenda
  estable/débil/falla, tooltips por celda. Pendiente: clustering real y detección de zonas.
- **S06 Chart de operaciones** — `done (fake)`: candlestick SVG propio con marcadores de trades
  (compra/venta) y líneas de rango trade. Pendiente: overlays de indicadores, pan/zoom.
- **S07 Editor** — `done (fake)`: filas de condición (izquierda/op/derecha) con dropdowns
  agrupados, añadir/quitar, acción al cumplirse, guardar con feedback. Pendiente: persistencia
  real + validaciones del motor.

- **Compilación**: tsc+vite limpio (227 KB, gzip 71 KB).
- **features.csv**: 4 filas más a `partial` (robustez must ×2, chart, editor).

## Lo más duro de esta tanda

- El heatmap de WF Matrix se adiñonó con `color-mix()` sobre tokens — mantener el 3:1 no aplica
  (celdas no-text), pero los textos internos usan `--color-text` que vence en toda la gama.
- El chart SVG propio es suficiente para V1; lightweight-charts entra en Fase 3 con pan/zoom.

## Estado acumulado (12 pantallas reales / 24)

S01, S02, S03, S04, S05, S06, S07, S12, S13, S20, S22, /design — el resto placeholder de fase 3/4.
Bloqueo de Fase 1 core: instalar Rust para compilar `src-tauri` (SQLite, motor, exporters).

## 2026-10-20 · Fase 1: shell + vertical slice (fake data layer)

- **S01 Dashboard** — `done (fake)`: lista de builds en vivo, resumen, empty state con CTA.
  Pendiente del swap Rust: nada en la UI.
- **S02 Builder config** — `done (fake)`: form por secciones (Datos/Plantilla/Filtros), TF chips,
  validación "sin datos" con enlace al Data Manager, bloqueo del botón sin datos.
  Pendiente: acordeón completo de robustez (MC/WF toggles reales, Fase 2).
- **S03 Build en vivo** — `done (fake)`: progressbar + log en vivo + pausa/reanudar/detener.
  Detalle: reanudar uso icono Square (falta icono Play); el log limita a 300 líneas.
- **S04 Bank** — `done (fake)`: tabla sortable estática con métricas coloreadas P&L, dirty rows
  con icono. Pendiente: columnas custom, filtros por columna, paginación virtual (millones de filas).
- **S05 Detalle estrategia** — `done (fake)`: stat cards, alerta de detección automática de problemas,
  equity chart SVG.
- **S20 Datos** — `done (fake)`: tabla símbolos/datasets, descarga con progreso real (simulada),
  toast de feedback. Pendiente: Normal descargador Dukascopy real (Fase 1 core Rust).
- **S22 Export** — `done (fake)`: selector estrategia → plataforma → código generado (stub) → guardar.
  Pendiente: export real MQL5/MT4 desde el Rust core.
- **/design (DesignMenu)** — `done`: página viva del sistema de diseño con roles de color reales.

- **Shell + routing**: 18 rutas montadas (todas las pantallas del recon tienen ruta; las 15 restantes
  son placeholders marcados por fase).
- **Tokens**: superare tokens.css por todo; cero hex en componentes.
- **Capa fake**: `src/lib/api.ts` con el contrato exacto de los Tauri commands firmas architecture.md.

## Lo que fue más duro de lo esperado

- El `dataset detalle` de S04 ha costado diseñarlo sin TanStack Table sin pagar complejidad —
  la tabla actual es plana; la virtualización es imprescindible antes de Fase 2.
- El shape estrategia del éxito estructurado game `logic` necesita forma ya (árbol de reglas) —
  quedar backbone del builder real.

## Qué falta para cerrar Fase 1

1. Instalar Rust toolchain e incluir `src-tauri` → SQLite real + motor de backtest mínimo.
2. Descargador real de Dukascopy (m1) en Rust; el fake de la UI usa su progreso tal cual.
3. Builder real: genética simple (2 plantillas) con rayon; escribe real strategies.
4. Exporters MQL4/5 reales con golden-file tests.

## QA del motor Rust (lote desbloqueado tras cargo check OK)

- Lote bloqueado por falta de toolchain → ejecutado con 14 unit tests `#[cfg(test)]`:
  1. importer CSV: ruido (filas basura), OHLC inválido, gaps, mezcla ISO+epoch, cabecera → saltos correctos y barras sanas; rechazo de CSV inútil mensaje con "barras".
  2. QFN byte-a-byte roundtrip.
  3. Backtest: <50 barras → output vacío; determinismo bit-a-bit; métricas en rango y close_time ≥ open_time (no-lookahead); costes de spread+comisión estrictamente reducen el net.
  4. Genético: build determinista con mismo config; respeta filtros minPF/maxDD/nTrades≥20; mutate con mismo seed → mismos params.
  5. Export: from_str con aliases; MQL5 con inputs inyectados y OnTick; MQL4≠MQL5 y APIs distintas (OrderClose vs PositionClose); 3 templates → código distinto.
- Hallazgos reales corregidos: BUG-006 (timestamp ISO roto) + from_str case-insensitive + skip de filas corruptas en vez de abortar + sort temporal tras import.
- **Resultado: 14/14 verde.** Warnings restantes de Rust: 6 (campos/métodos muertos) para limpiar con `cargo fix`.

## /replica-backend — backend real (núcleo Rust + SQLite local)

- Inventario de fake vs real → 16 commands nuevos implementados con motor real
  (`engine/ops.rs` nuevo: 340 líneas de robustez y cartera sobre datos reales):
  retest OOS, improver (4 variantes re-ejecutadas), optimizer/WF matrix (grid
  p1×p2 con WFE train 60%/eval 40%), optimization profile de Pardo, suite MC
  (9 tests con PRNG sembrado), SPP (6 variantes de señales con el mismo bucle
  de cartera gracias a la extracción de `backtest::run_signals`), correlación
  Pearson de equities, cartera (equity combinada + resultados + MC bootstrap
  500 trials), custom projects orquestados real, chart real, pause/resume de
  builds, export_dataset (CSV/QFN).
- Refactor del motor: `backtest::run_signals` extraído (variantes SPP/MC
  reutilizan exactamente el mismo bucle de ejecución).
- Persistencia ya existente explotada: `backtest_runs.equity` (json) alimenta
  correlación/cartera/sin recompeta por estrategia. dataset QFN en disco
  alimenta retest/optimizer/SPP/MC/chart re-ejecutando el motor.
- api.ts: 15 métodos ahora real-first (invoke si Tauri, fake si navegador).
  pause/resume distinguen null (éxito Tauri) de undefined (sin bridge).
- Nota V1 honesta: importDatasetFile en escritorio queda fake (falta
  tauri-plugin-dialog para el picker) y Dukascopy diferido por decisión.
- **QA: 21/21 unit Rust (7 nuevos de ops: pearson, corr matrix, maxDD,
  bootstrap determinista, axis, mean_equity) · 77/77 e2e · build 283KB/84KB
  gzip · sin mojibake · `replica/backend.md`** escrito con checklist de
  seguridad adaptado a escritorio y huecos documentados.

## 2026-10-20 · Fase /replica-entrepreneur: qué odian los usuarios de SQX (reseñas reales)

Objetivo: encontrar el ángulo de venta de QuantForge leyendo lo que los usuarios del
original dicen de verdad. Regla de oro: cero reseñas inventadas, cada cita textual con enlace.

- **Fuentes alcanzadas (52 filas, 3 fuentes)**: 2 hilos completos de r/algotrading +
  hilo "Your opinion on Strategy Quant" + hilo "SQX in Live Trading" de r/mltraders (vía
  PullPush, archivo público de la API de Reddit — la API oficial devuelve 403), 8 reseñas
  de Trustpilot (proxy de lectura; 403 directo), y la reseña experta de StatOasis
  (afiliado declarado, usada como investigación, no como recomendación).
- **Fuentes no alcanzables, documentadas**: HN (0 resultados aplicables), ForexPeaceArmy
  (403), G2/Capterra/Product Hunt (sin listados — producto de escritorio de nicho),
  board de feature requests (no existe Canny público).
- **Anti-spam**: descartados los vendedores de licencias de segunda mano y spam de cursos
  (Silly-Cup-1477, Lopsided-Ship6875, AFRANK97, HaveGunsWillTravl) — no se cuentan como reseñas.
- **Método anti-transcripción**: los cuerpos de Reddit se extraen por script desde el JSON
  de PullPush (cero copiado a mano); Trustpilot se parsea del markdown guardado; las 16
  frases de StatOasis se verifican programáticamente contra la página guardada (1 fallo
  de escape `\+` corregido). Muestra honesta: pequeña pero 100% verificable.
- **themes extendidos**: el themes.json genérico del pack mezclaba temas (matcheaba
  "support team responds within hours" como queja de soporte) → themes-sqx.json propio
  del dominio (overfitting, export mismatch, compute, privacy, portfolio, datos...).

**Hallazgos principales** (detalle completo en `replica/entrepreneur/feedback.md` +
`analysis.md`):
1. Sobreajuste estructural: "The overfitting risk is structural, not incidental" y
   "what I keep finding is overfitting... the only metric with predictive value is drawdown"
   (ingeniero con 30 estrategias en vivo, solo sobrevive el 45% al forward).
2. Ventas por ediciones: "Starter at $1,290 leaves out the robustness testing...
   Buy Professional, or buy nothing" — el original cobra aparte los frenos.
3. La WFA nativa: "the native WFA felt like a black box" → transparencia como feature.
4. Exportes que no reproducen resultados: "the results of the strategies do not match"
   al exportar a MT → "the failure mode that costs real money".
5. Gestión de riesgo a nivel portfolio ausente: "lacking portfolio level risk management".
6. Privacidad: "once StrategyQuant got caught 'calling home' and sending test results back".
7. Datos como puerta de entrada: QuantDataManager es la respuesta estándar en +8 hilos.
8. Curva de aprendizaje: "it is quite an undertaking to master it" / "too many options".

**Deliverables**: `replica/entrepreneur/{reviews.csv, feedback.md, themes-sqx.json,
sources.md, analysis.md}` + crudos archivados como evidencia. Plan de correcciones P0-P2
y ángulo de posicionamiento en analysis.md; alimentarán /replica-brand y /replica-launch.

---

## Fase /replica-brand - 20-10-2026 - identidad SophieQuant (CERRADA)

El nombre de trabajo **QuantForge**resulto inutilizable: 183 repos en GitHub, **14 con el
nombre exacto** y varios siendo plataformas de trading ("quantitative trading research
platform", "A-share quant workbench", "quantforge-rs"), mas 6+ dominios en uso. El
producto se renombro a **SophieQuant** (elegido por el propietario).

- **20 candidatos -> 5 cribados** (SophieQuant, QuantForge, Criterio, StratProof,
  Walkproof). Descartados: QuantForge (6+ productos de trading en la misma categoria),
  Criterio (existe criterio.trade), StratProof (stratproof.net + 3 repos de trading).
  SophieQuant: 0 resultados web, 0 repos GitHub, 0 apps, y **los cuatro dominios
  (.com/.app/.net/.io) libres** (RDAP oficial). Comprobado que no es confundible con
  StrategyQuant: solo comparte "Quant", que es generico del sector.
- **Pendientes honestos** (no automatizables hoy, marcados como tales en brand.md):
  USPTO/EUIPO/WIPO/Canada, Google Play y handles de redes.
- **Paleta "Noche Violeta"**: la original ocupa naranja (#fe8f3a) + azul (#52b2ff), y el
  acento placeholder #7aa2ff estaba pegado a su azul -> descartado. Nuevo acento
  #9d7bff (violeta), con neutros y semanticos re-hechos. `contrast.py`: **18 pares,
  0 fallos AA**.
- **Identidad aplicada**: productName/identifier/title/crate/DB/dataDir de Tauri,
  cabecera de los exportadores MQL4/MQL5, favicon e icono de app (regenerados con el
  motivo de marca). Modulos con nombre de la original renombrados: QuantAnalyzer ->
  **Laboratorio**, QuantDataManager/QDataManager -> **Gestor de datos**, y se elimina la
  extension ".sqx" de la interfaz.
- **UI en espanol** (lo que ve el usuario): Constructor, Banco de estrategias, Retesteo,
  Refinador, Optimizador, Matriz WF, Proyectos, Cartera, Exportar codigo, Perfil de
  optimizacion, Beneficio neto, Gestion de capital, Tasa de acierto. Los identificadores
  internos (API, comandos Tauri, rutas) se mantienen en ingles: son contrato tecnico.
- **Voz**: precisa / honesta / terreno, con 5 pares de si-no y los 10 textos mas vistos
  reescritos. Alimentada con los hallazgos de /replica-entrepreneur (sin promesas de
  rentabilidad, sin telemetria, validacion visible).
- **Icono**: "el filtro que deja pasar solo lo que aguanta" - tres barras ascendentes,
  solo la verde supera el umbral. Nada de monograma SQ (la forma del original).
- **Sweep**: `Clean. Nothing of the original's name, domain or colours found.` Se anadio
  `target` a las carpetas de build que el sweep salta (1.199 falsos positivos sobre
  binarios del crate viejo) y se borro `src-tauri/target/`.
- **Verificacion**: 21/21 tests Rust OK, 77/77 e2e OK (axe 0 violaciones),
  `npm run build` OK (282,7 kB), capturas nuevas en `replica/screens/`.
- **Entregables**: `replica/brand.md`, `replica/brand.json`, tokens actualizados en
  `replica/design/tokens.json` + `src/styles/tokens.css`, icono y favicon nuevos.

Siguiente: /replica-diff.

---

## Fase /replica-diff - 20-10-2026 - paridad medida (CERRADA)

Medicion honesta de cuanto se parece SophieQuant al original, con las herramientas
del pack (parity.py + imgdiff.py), no a ojo.

- **features.csv reescrito**: las 13 filas que decían "real" no puntuaban (la
  herramienta solo reconoce yes/partial/no/skip) y 30 filas seguian marcadas "no"
  pese a estar implementadas. Se re-audito cada fila contra el codigo real
  (engine/, commands.rs, export.rs) y se reescribieron las 56 filas con su nota.
- **Resultado**: paridad total **54,8/100**; dentro del alcance definido por el
  propietario **57,3/100**; must-haves **8 de 20**. Por area: robustness 83,
  optimizer 90, analyzer 87 frente a blocks 30, export 32, data 32.
- **Reencuadre honesto**: lo que falta es sobre todo amplitud de generacion
  (mas tipos de entrada/salida, mas indicadores, bloques, tick, multi-simbolo) -
  exactamente lo que los usuarios del originaleluñan ("too many options",
  "maquina de overfitting"). La **cadena de validacion** que se usa a diario tiene
  **14 de 15 eslabones con motor real**; el unico hueco es el guardado del editor.
- **Diff de layout**: el original es software de escritorio de pago y no tiene
  capturas publicas de sus pantallas. Se localizo **una captura real de su app**
  publicada en su web (composite de marketing) y se recorto la ventana como
  referencia unica: layout **17,8/100** ("different"). El numero mide divergencia
  de IA, no calidad; se documento la comparacion estructural elemento a elemento
  (rail de iconos vs lista, sub-pestanas, zona OOS sombreada, tarjeta de
  veredictos por prueba).
- **Hallazgo de producto**: el original muestra en una tarjeta flotante los
  veredictos por prueba (Walk Forward / MonteCarlo / Advanced robustness = Passed).
  SophieQuant los tiene repartidos en 4 modulos. Es el diferenciador central que
  pide la investigacion de los usuarios y el punto 1 de la lista final.
- **S25 Paridad deja de ser un stub**: la pantalla muestra puntuacion global,
  alcance, must-haves, layout, areas con barra, pendientes must y lo excluido.
  Datos en `src/data/parity.json`, generados por `replica-diff`.
- **Entregables**: `replica/parity.md`, `replica/parity-scores.json`,
  `replica/diffs/S08.{png,json}`, capturas en `replica/clone-screens/`,
  `replica/screens/ORIGINAL-*.png` (referencias publicas).
- **Verificacion**: 77/77 e2e OK tras el cambio, typecheck limpio.

**Lista final (top 5)**: 1) tarjeta de veredictos consolidada por estrategia,
2) zona out-of-sample sombreada en la curva de equity, 3) guardado real del
editor sin codigo, 4) decidir el exportador (implementar o quitar las plataformas
que solo devuelven stub), 5) filtrado del generador por cualquier metrica.

Nota de contexto: el propietarioclaro que el uso es **personal y sin
publicacion**, asi que `/replica-launch` y `/replica-deploy` pierden su sentido
original: el "deploy" pasa a ser compilar y ejecutar en local.

---

## Bloque 1 y 2 de la lista final - 21-10-2026 - veredictos + zona OOS

Construidos los dos primeros puntos de la lista final de replica-diff, que son los
que el propio informe senalaba como los de mas valor.

- **Tarjeta de veredictos consolidados** (motor nuevo `engine/verdicts.rs`): por
  estrategia calcula walk-forward (WFE), Monte Carlo (las 9 pruebas), SPP y perfil
  de optimizacion, y decide pass/warn/fail con **umbrales explicitos** que se
  muestran en la UI (WFE >= 50 %, 9/9 pruebas, SPP sin fallos, perfil con meseta).
  Persistencia en la tabla nueva `strategy_verdicts` + migracion idempotente, para
  que la tarjeta se pinte al instante. UI: columna "Robustez" en el banco, boton
  "Calcular veredictos", tarjeta completa en el detalle y pestana "Veredictos" en
  el Laboratorio.
- **Zona out-of-sample marcada**: el corte se calcula en el motor con el mismo
  criterio que la comprobacion walk-forward (60/40) y se guarda con cada corrida
  (`backtest_runs.oos_frac` + migracion), de modo que la marca es exacta. Se
  sombrea en la curva de equity y en el grafico de velas, con leyenda explicita.
- **Exportador honesto**: el propietario usa MetaTrader 5, asi que la UI anuncia
  solo MQL4 y MQL5 (los dos con generador real) y se retiran NinjaTrader,
  TradeStation, MultiCharts y JForex, que devolvian un stub. El export a MQL5 pasa
  de parcial a completo (el mismo EA sirve en cuentas hedge y netting).
- **Bug encontrado y corregido de paso**: `normBankRow` solo leia `lastRun.equity`,
  por lo que en escritorio la curva de equity del detalle salia vacia (solo
  funcionaba el dato fake del navegador). Ahora acepta las dos formas de respuesta.
- **Numeros**: la paridad sube de 54,8 a **56,1** (total) y de 57,3 a **58,7**
  (dentro del alcance); must-haves de 8 a **9 de 20**. Las dos funciones propias
  (tarjeta de veredictos, marca OOS) quedan registradas en features.csv como
  "yours, not in the original".
- **Verificacion**: 27/27 tests Rust (4 nuevos: 3 del modulo de veredictos + 1 de
  persistencia), 82/82 e2e (5 nuevos en `e2e/f-verdicts.spec.ts`), build web OK,
  axe 0 violaciones.
- **Pendiente de la lista**: el guardado real del editor sin codigo (S07) es el
  unico eslabon simulado que queda en la cadena de validacion.

Nota: la app de escritorio (`npm run tauri dev`) quedo reiniciada en varias
ocasiones durante el trabajo (cambio de crate, comando nuevo, puerto 1420) - el
proceso que se ve ahora corre con los comandos `get_strategy_verdicts` y
`compute_strategy_verdicts` ya compilados.

---

## Limpieza de codigo muerto - 21-10-2026

`cargo build`aba en 6 avisos de `dead_code`. No se silenciaron con `allow`: se
resolvieron de raiz.

- `crosses_above` (indicadores): **pasa a usarse de verdad** en la plantilla de
  reversion RSI (misma semantica: el nivel es constante, asi que el cruce al alza
  es identico al inline que habia).
- `crosses_above_series`: **eliminado**. Nunca se uso y su semantica (compara
  contra el valor del bar anterior de la serie B) difiere del cruce que aplica el
  motor; dejarlo era una trampa para quien lo usara por error.
- `Trade::pips_gain`: **eliminado** (no lo usaba nadie).
- `StoredDataset` (datastore): **eliminado** (duplicaba el concepto de dataset
  ya cubierto por `DataFileRow`).
- `TauriReporter.total`: **eliminado** (el total llega en cada informe, el campo
  sobra).
- `StrategyCore.name`: **pasa a usarse** en los mensajes de error del motor, que
  ahora dicen "Estrategia S004 Donchian trend sin dataset cargable" en vez del id.
- Import sobrante en datastore.rs: **eliminado**.

Resultado: **0 avisos de compilacion**, 27/27 tests Rust, 82/82 e2e, build web OK.

Ademas se corrigio un aviso de Fast Refresh de Vite: `VerdictCard.tsx` exportaba
ademas del componente dos funciones auxiliares, lo que impide el recarga en
caliente de ese modulo. Las funciones se movieron a `src/lib/verdicts.ts`.

---

## Punto 3 de la lista final - 21-10-2026 - editor sin codigo que guarda de verdad

Ultimo eslabon simulado de la cadena de validacion: cerrado.

- **El editor solo ofrece lo que el motor ejecuta**. Se retiraron del selector
  MACD, Stochastic, CCI y patrones de vela (el motor no los implementa); quedan
  las tres reglas reales: cruce de medias, reversion RSI y ruptura con ATR, con
  sus parametros y el nombre de parametro del motor (`p_fast`, `th_low`...).
- **Guardar y recalcular** (comando Tauri `save_strategy_logic` ->
  `ops::save_logic`): persiste la regla (plantilla + parametros + logico),
  vuelve a correr el backtest sobre el dataset de la estrategia y guarda la
  corrida nueva con metricas, equity y corte out-of-sample. La pantalla ensena
  el antes/despues (beneficio, PF, Max DD, operaciones).
- **Si la regla no genera operaciones, el guardado falla sin tocar nada**: la
  estrategia anterior sigue intacta. Misma politica que el resto del motor.
- **Boton "Editar regla"** en el detalle de la estrategia (antes no habia forma
  de llegar al editor).
- La logica de guardado se extrajo del comando a `ops::save_logic` para poder
  testearla: **3 tests de integracion en Rust** (persiste y recalcula; falla sin
  alterar nada; el corte OOS se calcula sobre las barras) + **3 e2e** del editor.
- **Numeros**: paridad de 56,1 a **57,0** (total) y de 58,7 a **59,6** (dentro
  del alcance). La fila "Editor de estrategia integrado sin codigo" pasa de
  parcial a completa: **la cadena de validacion ya no tiene ningun eslabon
  simulado**.
- **Verificacion**: 30/30 tests Rust (3 nuevos), 85/85 e2e (3 nuevos), build web
  OK, axe 0 violaciones.

Queda de la lista final solo el punto 5: filtrado del generador por cualquier
metrica y seleccion automatica de la mejor por robustez.

---

## Punto 5 de la lista final - 21-10-2026 - filtros por cualquier metrica y seleccion por robustez

Ultimo punto de la lista, y el que ataca el problema de fondo del sector: el
sobreajuste estructural ("es una maquina de overfitting sin validacion externa").

- **Nuevo modulo `engine/robust.rs`**:
  - `MetricFilter { metric, op, value }` y filtrado por CUALQUIERA de las 12
    metricas del motor (PF, Sharpe, Sortino, MaxDD, recovery factor, beneficio
    neto y bruto, tasa de acierto, Z-score, R2, beneficio medio, n operaciones).
    La lista de metricas la pide el motor (comando `list_build_metrics`): una
    sola fuente de verdad, sin listas duplicadas en el front.
  - Una metrica o un operador desconocido **se rechaza antes de generar**; un
    filtro que no se puede evaluar **nunca pasa** (fail closed).
  - `Robustness`: score 0-100 con cuatro componentes y pesos explicitos
    (35 % walk-forward, 30 % supervivencia MC, 20 % SPP, 15 % perfil). Los pesos
    se muestran en la UI del Constructor.
- **El generador deja de ordenar por beneficio**: con "ordenar por robustez"
  activo, todas las supervivientes reciben el MISMO score completo y se ordenan
    por el. (Mixar el score rapido con el avanzado compararia dos escalas
    distintas: se detecto y se unifico.)
- **El build dice que te bloquea**: si no pasa ninguna estrategia, el registro
  senala el filtro que mas rechazo candidatos y con cuantos.
- **Persistencia**: `strategies.robust_score` + `robust_detail` (componentes),
  con migracion idempotente; el score se ve en el banco junto al veredicto.
- **UI**: el Constructor pasa de dos campos fijos (PF min y DD max) a un editor
  de filtros (metrica + operador + valor, anadir/quitar) y un interruptor de
  seleccion por robustez con la explicacion de los pesos.
- **Tests**: 6 nuevos de Rust (4 del modulo robust + 2 del generador: filtros
  por cualquier metrica y ordenacion por score) + 3 e2e de los filtros.
- **Numeros**: paridad de 57,0 a **58,3** (total) y de 59,6 a **61,0** (dentro
  del alcance); must-haves de 9 a **10 de 20**. La fila "Filtrado avanzado por
  cualquier metrica" pasa de parcial a completa.
- **Verificacion**: 38/38 tests Rust, 88/88 e2e, build web OK, 0 avisos de
  compilacion.

**Los cinco puntos de la lista final de replica-diff estan cerrados.** Lo que
queda abierto son decisiones de producto para una segunda iteracion (mas
indicadores, multi-simbolo, tick, pesos del Portfolio Composer, ATM) y las tareas
de calidad ya anotadas (`tauri-plugin-dialog`, `cargo audit`).

---

## Cierre del producto minimo ejecutable en local - 21-10-2026

Las dos tareas de calidad que quedaban abiertas, cerradas.

- **Selector de ficheros nativo**: instalado `tauri-plugin-dialog` 2.8.3 (Rust +
  JS) con sus permisos en `src-tauri/capabilities/default.json` (el fichero
  `capabilities` no existia: Tauri 2 lo necesita para los permisos de plugins).
  El boton «Importar CSV» abre el dialogo del sistema y la importacion es real:
  simbolo y temporalidad de destino, limpieza del CSV, guardado como QFN y
  registro en `data_files`.
  - La logica se extrajo del comando a `import_csv_registered` para poder
    testearla: **2 tests de integracion** (CSV real de 500 barras; fichero
    inexistente -> error, nunca datos inventados).
  - Nuevo `invoke` estricto (`realStrict`): si la importacion falla, **el error
    sale en pantalla**. Antes el puente caia al fake en silencio, que es
    justamente el tipo de mentira que la investigacion senala en el original.
  - En el navegador (modo de desarrollo web) el boton cae al input de ficheros
    clasico; en escritorio usa siempre el dialogo nativo.
- **`cargo audit`**: instalado cargo-audit 0.22.2 y ejecutado sobre las 437
  dependencias del `Cargo.lock`: **0 vulnerabilidades**. Dos avisos
  informativos, ambos **solo del stack GTK/Linux** de Tauri (no compilan en la
  entrega de Windows) y sin version corregida compatible con Tauri 2.12
  (verificado). Registrados como riesgo aceptado en `src-tauri/audit.toml` con el
  comando de referencia:
  `cargo audit --deny warnings --ignore RUSTSEC-2024-0429 --ignore RUSTSEC-2024-0370`
- **Correccion de la matriz**: la columna `clone` usaba el valor "real" en 13
  filas, que `parity.py` no reconoce (cuenta como 0). Reescrita con valores
  validos (yes/partial/no/skip) y recalculada.
- **Numeros**: paridad de 58,3 a **59,2** (total) y de 61,0 a **61,9** (dentro
  del alcance); must-haves **10 de 20**.
- **Verificacion**: 40/40 tests Rust (2 nuevos de importacion), 88/88 e2e, build
  web OK, 0 avisos de compilacion, sweep de rebrand limpio, contraste 0 fallos.

---

## Acceso directo y publicacion en GitHub - 21-10-2026

### Acceso directo al escritorio
- Compilacion release completa (`cargo build --release`, 18 min por el LTO):
  `src-tauri/target/release/sophiequant.exe`, 10,5 MB, con la interfaz web
  embebida (sin servidor de desarrollo).
- La app en release ahora declara `windows_subsystem = "windows"`: se abre sin
  consola detras. Verificado leyendo el subsistema del PE (2 = GUI).
- `scripts/crear-acceso-directo.ps1` crea el acceso en el Escritorio (con opcion
  `-ModoDev` para el acceso que levanta `npm run tauri dev`). Ejecutado:
  `C:\Users\jesus\OneDrive\Escritorio\SophieQuant.lnk` -> binario release,
  icono propio, descripcion.
- Probado de verdad: lanzado **desde el acceso directo**, la ventana "SophieQuant"
  aparece y la app abre su base de datos local en
  `%APPDATA%\com.sophiequant.app\sophiequant.db`.

### Repositorio
- `git init -b main` + remoto `origin` -> https://github.com/Productivia3/SophieQuant.git
- `.gitignore`: ruido de compilacion (node_modules, target, dist, test-results),
  **datos locales del usuario** (nunca se suben) y **material de terceros**:
  los volcados literales de resenas (`replica/entrepreneur/raw-*.txt`,
  `reviews.csv`) y las capturas de marketing del producto de referencia
  (`replica/screens/ORIGINAL-*.png`). Se quedan en el disco local; el analisis
  derivado si se sube.
- `.gitattributes` (normalizacion LF) y `.editorconfig`.
- `README.md` en espanol: que hace, el estado medido, como compilar y ejecutar,
  el mapa del repositorio, privacidad (sin telemetria, sin cuentas, sin red) y la
  procedencia clean-room.
- Dos commits subidos: `5f4e959` (producto) y `352dad4` (preparacion).
  130 ficheros en el remoto; verificado que no hay node_modules, target, dist
  ni material de terceros.

---

## Correccion: el acceso directo abria «no se puede conectar a localhost» - 21-10-2026

**Sintoma**: el acceso directo lanzaba la app y esta se quedaba en «no se puede
conectar a localhost».

**Causa**: el binario de release se habia compilado con `cargo build --release`
a secas, **sin el rasgo `custom-protocol`**. Ese rasgo es el que hace que Tauri
sirva la interfaz embebida en el binario; sin el, el ejecutable espera al
servidor de desarrollo (`devUrl` = localhost:1420) y no encuentra nada. El
`Cargo.toml` no declaraba el rasgo porque el proyecto nacio de una plantilla
simplificada.

**Correccion**: en `src-tauri/Cargo.toml`:
```toml
[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```
`tauri dev` sigue arrancando con `--no-default-features` (servidor de Vite), y
el release ahora sirve la interfaz embebida.

**Ademas**, la captura de la ventana de la app (que no habia hecho nunca) destapo
tres textos que mentian segun el modo:
- el panel Resumen decia «Nucleo: fake (Rust pendiente)» cuando el motor Rust es
  real -> ahora dice «Rust (motor real)» en escritorio y «simulado (navegador)»
  en el navegador;
- el pie de pagina decia «nucleo Rust conectado» incluso en el navegador, donde
  el nucleo es el fake -> ahora es honesto en los dos modos;
- la nota de Ajustes decia que los ajustes se guardan «en la capa fake (sesion)»
  cuando en escritorio van a la base de datos -> ahora distingue los dos casos.
- y los contadores del panel se mostraban como «0.00» (formato de dinero para
  contar estrategias) -> ahora son enteros.

**Verificado**: binario release recompilado, lanzado **desde el acceso directo**,
captura de la ventana real (la interfaz se pinta entera, sin tocar localhost) y
sin conexiones al puerto 1420. 88/88 e2e y 40/40 tests Rust.

- **Acceso directo portable**: el `.lnk` apuntaba dentro del repositorio, asi que
  un `cargo clean` lo dejaba muerto. Ahora el script copia el ejecutable a
  `%LOCALAPPDATA%\Programs\SophieQuant\SophieQuant.exe` (copia estable) y el
  acceso apunta ahi. Verificado: lanzado desde el acceso, la app pinta la
  interfaz y **no abre ninguna conexion de red**.
- **Instalador MSI generado**: `npm run tauri build` produce
  `src-tauri/target/release/bundle/msi/SophieQuant_0.1.0_x64_en-US.msi` (3,9 MB,
  con WiX descargado automaticamente). El script tambien lo puede instalar con
  `-Instalador <ruta.msi>`, que ademas deja entrada en el menu Inicio.

---

## Ampliacion 1: retesteo multi-mercado - 21-10-2026

Primera ampliacion de la segunda iteracion (la mas pendiente segun el informe de
paridad). Y de paso, un fallo que estaba escondido: **el selector de mercado del
Retesteo no hacia nada** (el command recibia symbol_id y timeframe y los
ignoraba). La UI prompia algo que el motor no cumplia.

- **Motor**: `ops::retest_on_market` corre la MISMA regla sobre los datos de otro
  mercado; `ops::verdict_mercados` resume en cuantos mercados aguanta la
  estrategia. Un mercado sin datos es "sin datos", no un fallo.
- **Command**: `run_retest` ahora recibe la lista de mercados, carga el dataset
  de cada uno (QFN) y devuelve la matriz estrategia x mercado mas el resumen.
- **UI**: panel con varios mercados (marca "con datos" / "sin datos" por symbolo y
  temporalidad), tabla con una columna por mercado (veredicto + PF) y columna
  "Sobrevive" con el recuento.
- **Tests**: 4 de Rust + 3 e2e nuevos. Se corrigio tambien una carrera en los
  tests: el panel de mercados aparece antes que la lista del banco, asi que
  `getByRole('checkbox')` cogia un mercado en vez de una estrategia; ahora los
  localizadores se acotan a cada lista.
- **Fake coherente**: la disponibilidad de mercado del fake usa la misma regla que
  el panel (antes decia "sin datos" y aun asi enseaba resultados).
- **Numeros**: paridad de 59,2 a **60,5** (total) y de 61,9 a **63,3** (alcance).
  La fila de backtest multi-simbolo pasa de `no` a `partial`.
- **Verificacion**: 44/44 tests Rust, 91/91 e2e, build web OK.

---

## Ampliacion 2: historial de analisis - 21-10-2026

Segunda ampliacion de la iteracion. Cierra el hueco que quedaba abierto desde el
backend: los resultados de analisis se calculaban, se mostraban y se perdian al
cerrar. Ahora quedan guardados y **comparables**.

- **Motor**: nuevo modulo `engine/history.rs` con la tabla `analysis_runs`
  (estrategia, tipo, veredicto, etiqueta, metricas, configuracion y detalle) y
  las funciones `record`, `list`, `delete`, `compare` y `previous_of`.
- **Siete analisis quedan registrados** al ejecutar sus comandos: retesteo,
  refinador, optimizador, matriz WF, Monte Carlo, SPP y perfil de optimizacion.
  El registro nunca interrumpe el analisis: si falla, el comando sigue.
- **Comparacion con la anterior**: cada fila muestra el delta por metrica frente
  a la corrida previa del mismo tipo y estrategia ("PF +0,15"); si no hay
  previa, dice "primera corrida".
- **Interfaz**: panel en el detalle de la estrategia y pestana "Historial" en el
  Laboratorio (con filtro por tipo y borrado de entradas). Componente nuevo
  `src/components/HistoryPanel.tsx`.
- **Fake tambien registra**: el almacen en memoria del navegador permite probar
  el flujo completo sin escritorio.
- **Tests**: 5 de Rust nuevos (49 en total) + 3 e2e nuevos (94 en total).
- **Cierra** el hueco 3 de `replica/backend.md`.

---

## Ampliacion 3: ponderacion de cartera - 21-10-2026

Tercera ampliacion. La cartera era equiponderada, que es una decision que nadie
toma: si dos estrategias arriesgan distinto, repartir a partes iguales no es
"neutral".

- **Motor** (`engine/ops.rs`): `volatility`, `cov_matrix`, `weights` (4 modelos),
  `risk_parity` (punto fijo sobre contribuciones de riesgo), `weighted_equity` y
  `risk_contributions` (en %).
- **Modelos**: equiponderado, inverso a volatilidad, paridad de riesgo y
  personalizado (los negativos se ignoran, no se propagan).
- **Comandos**: `run_portfolio_weighting` (calcula y **guarda** la corrida en
  `portfolio_runs`) y `list_portfolio_runs` (historial). `get_portfolio_results`
  ahora acepta el modelo, y la pagina de resultados usa el de la ultima corrida
  para que ambas vistas coincidan.
- **Interfaz**: panel "Ponderacion de la cartera" con selector de modelo, tabla
  por miembro (volatilidad, peso, contribucion al riesgo) y resumen de cartera.
  Con el modelo personalizado se editan los pesos a mano.
- **Accesibilidad**: el test nuevo destapo dos fallos preexistentes que solo
  aparecian al ejecutar el analisis (el axe de la pagina no lo hacia antes):
  celdas del mapa de correlacion con 4,03:1 de contraste y una cabecera vacia.
  Corregidos, y las celdas seguian usando los azules previos al rebranding: ahora
  usan la paleta violeta/ambar/rojo.
- **Tests**: 5 de Rust (equiponderado, inverso a volatilidad, paridad de riesgo,
  pesos personalizados, equity ponderada) + 2 e2e.

---

## Verificacion del export + diagnostico del import - 21-10-2026

Cierra el P1 que propose la propia investigacion ("Aviso de verificacion de export
+ checklist documentado") y el bloque de calidad del import de CSV.

- **Verificacion del export** (`engine/export.rs::verification`): al generar el EA
  se muestran **los supuestos reales** (barras y no ticks; spread, slippage y
  comision tal y como se correron; posicion y balance fijos; solo largo; sin
  horarios; huecos rellenados) y una **checklist de 8 puntos** para comprobarlo
  en MetaTrader. Los valores vienen de `backtest_runs.engine_cfg`, asi que no son
  genericos: si cambias el spread, cambia el supuesto.
- Documento completo en `replica/export-verificacion.md`, con lo que el export
  **no** puede hacer (no es motor tick, no hay gestion de capital, no hay short).
- **Diagnostico del import de CSV** (`engine/datastore.rs`): el importador ahora
  comprueba el fichero y lo dice: filas descartadas, **huecos** (y el mayor),
  barras **sin volumen**, **precios incoherentes** y **timestamps repetidos**. Un
  fichero con huecos es la causa tipica de que el backtest sea mas optimista que el
  broker; dicho al importar, deja de ser un misterio.
- El import devuelve `{dataset, calidad}` y la UI muestra el panel de calidad.
- **Tests**: 2 de Rust (fichero limpio y fichero con hueco/precio imposible/
  volumen cero) + 2 e2e (el panel de verificacion y que el EA se siga pudiendo
  guardar). Total: 55 Rust, 98 e2e.
