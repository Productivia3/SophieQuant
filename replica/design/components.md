## Components — QuantForge (rebuild of StrategyQuant X core)

Base accesible: Radix UI primitives (dialog, popover, tabs, dropdown, tooltip).
Tema oscuro. Tokens de `replica/design/tokens.json` vía `tokens.css` + Tailwind mapping.
Fuente de primitivas a construir primero en `/design` (Storybook) antes que ninguna pantalla.

Button
  variants  primary (accent), secondary (surface-raised + border), ghost (sin fondo), danger
  sizes     sm 28px, md 34px, lg 40px
  states    default, hover (±6% lightness), active (pressed), focus-visible (ring 2px accent), disabled (40% opacity), loading (spinner + label intacto)
  tokens    primary: bg accent / text on-accent / radius-md / text-sm 600; toolbar usa ghost sm
  a11y      <button> real, focus visible, loading no cambia el label (lo lee el lector)
  used on   S01..S24 (principalmente toolbars S03 S04, dialogs S02 S08 S10, S22 export)

Sidebar de módulos
  variants  expandida 224px, colapsada 56px (solo iconos + tooltip)
  states    item hover, item seleccionado (fondo surface-raised + barra lateral accent 2px), badge de builds activos corriendo
  tokens    bg surface, items text-sm text-muted, seleccionado text-base
  a11y      nav con aria-current; expandible con teclado
  used on   S01 (persistente en todas las pantallas)

Toolbar de módulo
  variants  con título + acciones (New/Run/Save/Delete), con filtros inline
  states    acciones disabled cuando no hay selección en el grid
  tokens    height 44, bg bg, border-bottom border
  used on   S04, S08, S09, S16, S18, S20, S24

Grid de datos (TanStack Table + Virtual)
  variants  bank (col tons custom), portfolio, datasets, robustez
  states    sort asc/desc por columna (triángulo), filtrado por columna (input en header), dirty rows (icono danger + tooltip problema), selección simple/multi (checkbox), vacía (empty state con CTA), loading (skeleton filas), scroll virtual miles de filas
  tokens    row 34px, header 32px bg surface, zebra sutil surface/border 1px, números right-aligned en mono
  a11y      grid role, headers asociados, sortable con aria-sort; el foco pasa por celdas (roving tabindex)
  used on   S04, S08, S12, S14, S18, S20, S24

Panel de métricas (stat)
  variants  tarjeta compacta (label xs + valor mono lg), tarjeta con delta vs baseline (▲/▼ success/danger)
  states    default, semáforo de pass/fail (verde/rojo a la izquierda), sin dato ("—")
  tokens    bg surface-raised, radius-md, valor en font-mono
  used on   S05, S09, S10, S13, S19, S24

Dialog / Modal (Radix)
  variants  config técnica (ancho 640, secciones acordeón: Data/Randomizer/Filtro/Robustez), confirmación de borrado (peligro), export (selector plataforma + preview código)
  states    abierto/cerrado con animación base, errores de validación inline bajo campo, scroll interno
  tokens    bg surface-raised, shadow pop, radius-lg, overlay 70% negro
  a11y      focus trap, Esc cierra, título con aria-labelledby
  used on   S02, S05-edit, S08, S09, S10, S13, S14, S15, S22

Configurador multi-sección (acordeón)
  variants  por categoría (Datos, Símbolo/TF, Plantilla, Generación, Filtros, Robustez)
  states    abierto/cerrado, valores modificados marcados (punto amber warning), reset por sección
  tokens    header sm 600, contenido space-4
  used on   S02, S08, S09, S10, S13, S14, S15

Gráfico equity / drawdown (lightweight-charts)
  variants  equity net vs balance, overlay MAE/MFE, múltiples series del build
  states    loading, vacío ("Sin resultados"), crosshair con tooltip (fecha, valor mono), rango de drag para zoom, doble click reset
  tokens    grid chart-grid, up chart-up / down chart-down, axis chart-axis en xs mono
  a11y      canvas con tabla alternativa descargable (csv) y aria-label con resumen
  used on   S05, S09, S10, S13, S19, S24

Candle chart + trades (S06)
  variants  con/sin indicadores overlaid, paneles inferiores (volumen/indicador), marcadores buy/sell
  states    loading, sin datos para el rango, TF selector (chips h1 d1…), iconos pin de indicador
  tokens    velas up/down chart tokens, trades chart-buy/chart-sell, crosshair común
  used on   S06 (y preview en S10/S11 context)

Gráfico 3D de superficie (S11)
  variants  vista isométrica orbitable, rotación con mouse, color por altura (gradiente accent→success→warning→danger vers clúster)
  states    cargando (skeleton wireframe), vacío, render de N×M combos, reset de cámara
  tokens    fondo bg, malla chart-grid, labels xs mono
  a11y      controles de cámara por teclado; export csv de la matriz
  used on   S11

Heatmap de WF Matrix (S12)
  variants  filas = param A, cols = param B, celda pas/fail + color intensidad de WFE
  states    hover cell (tooltip con métricas), clusters resaltados con borde success, leyenda
  tokens    celda radius-sm, borde grid, textos xs mono
  used on   S12

Badges de estado de test
  variants  pass (success), fail (danger), running (warning, spinner xs), pending (gris)
  states    con conteo "MC 8/10 pass", clickable para ver details
  tokens    pill radius, bg success/danger al 15% con texto del color a full
  used on   S03, S04, S05, S12, S13, S14, S15, S17

Progress bar + live log (dock inferior)
  variants  indeterminada (descarga), determinada (build %), log con niveles (info/warn/error)
  states    running/paused/done/cancelled/error, auto-scroll con toggle, copiar log
  tokens    barra 4px accent, log en mono xs sobre surface-raised, dock border-top
  a11y      role progressbar con aria-valuenow, log region aria-live polite (resumen)
  used on   S03, S09, S17, S20, S24

Canvas de workflow (S16, custom projects)
  variants  nodos por step (icono por kind: build/test/filter/optimizer/analysis), aristas direccionales
  states    hover node, seleccionado (ring accent), conectando (drag), error de conexión (rojo y rechazo), plantilla de arranque, paleta lateral searchable
  tokens    node bg surface-raised radius-md border, eliminar con teclado Delete
  a11y      alternativa en lista ordenada (steps 1..n) navegable por teclado
  used on   S16, S17

Selector de símbolo / TF
  variants  dropdown searchable con favoritos (estrella), chips de TF (m1…d1), indicador de datos disponibles (● success / ○ warning "sin datos")
  states    vacío (empty con CTA al Data Manager S20), descargando (spinner inline)
  used on   S02, S08, S09, S18, S20, S24

Selector de indicador/bloque
  variants  dropdown agrupado por familia (osciladores, tendencias, velas, estadístico), buscador
  states    con extensión custom instalada (tag "custom"), placeholder aleatorio marcado con warning
  used on   S02 plantillas, S07 editor

Editor de lógica (S07)
  variants  árbol de reglas (si/lógica difusa/entonces), filas de condición = [Left] [Op] [Right] con dropdowns
  states    añadir/quitar condición, grupo con indentación y líneas conectoras, validación al vuelo (error inline), guardar como plantilla
  tokens    filas 32px surface, dropdowns mono sm
  used on   S07, y configuración del Improver S09

Export dialog (S22)
  variants  por plataforma: MQL4, MQL5 (hedge/netting), NinjaTrader, TradeStation, MultiCharts, JForex
  states    selección de plataforma (radio cards), opciones (librerías, precio en pips…), generación (spinner), resultado con preview de código (mono sobre surface) y "Guardar…" (file picker SO)
  tokens    code block bg bg, border, sin resaltado agresivo: 2 tokens (palabras clave accent, números success)
  used on   S22

Toasts (Notificación)
  variants  success (data descargada, export guardado), error (fallo de build con "ver log"), info
  states    4s auto-dismiss, apiladas, clickable para ir al módulo
  tokens    bg surface-raised shadow pop, barra lateral 3px del color del estado
  used on   todo

Inputs (texto / número / rango)
  variants  text, number con steppers y unidad (pips, $), range slider doble para optimización (min–max/step), file path picker, toggle
  states    default, focus (border-input→accent), error (danger + mensaje), disabled, readOnly
  tokens    h 32, radius-md, border-input, números en mono right-align
  used on   S02, S08, S09, S10, S13, S14, S15, S16

Tabs
  variants  módulo (Results/Chart/Edit/Improve/Optimize/Robustez en detalle de estrategia), pestañas de docks
  states    activo (border-bottom accent 2px), hover, overflow scroll con flechas
  used on   S05, S13, S20, S24

Statusbar
  content   versión, nº workers activos, RAM, proyectos abierto, indicador offline (accent dot), atajos
  tokens    h 24, xs muted, bg surface
  used on   global

Empty states (escritos nuevos, nunca copia del original)
  bank vacío:   "Aún no hay estrategias. Ejecuta un build para generar la primera." + CTA New build
  sin datos:    "Descarga el histórico de {símbolo} para empezar a investigar." + CTA al Data Manager
  robustez no ejecutada: botón con los tests sugeridos por defecto
  portafolio vacío:  "Añade estrategias del bank para combinarlas."
  custom projects vacío:  "Crea un proyecto para encadenar build → test → filtro."

Iconografía
  set: Lucide (MIT) — sola familia, 16px en UI, 14 en tablas
  prohibido: assets del original (su SVG de velas, ilustraciones, fotos de la web)

Marketing (landing del skill replica-launch)
  dark boarding page coherente: bg bg, display xl, mismo sistema; ilustraciones propias de charts