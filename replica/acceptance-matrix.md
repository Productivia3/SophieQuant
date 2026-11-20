# Matriz de aceptación SophieQuant 1.0

Fuente normativa: [ROADMAP_FINAL.md](../ROADMAP_FINAL.md). La guía manual
[PRUEBAS_MANUALES.md](../PRUEBAS_MANUALES.md) conserva los recorridos de UI;
`replica/test-plan.md` conserva los casos automatizados existentes. Esta matriz
une cada tarea de roadmap con la evidencia necesaria y evita marcar como
aprobado un test que aún no se ejecutó en su entorno real.

## Estados

- **Pendiente:** aún no ejecutado o no implementado.
- **Parcial:** hay evidencia, pero falta al menos un criterio o entorno.
- **Bloqueado:** requiere datos, plataforma o decisión aún no disponible.
- **Aprobado:** criterio completo en el entorno indicado y evidencia registrada.
- **Excluido:** fuera del alcance 1.0 aprobado.

El propietario del producto debe aprobar cambios del alcance, umbrales de MT5,
criterios de Demo y excepciones al gate final. Desarrollo/QA prepara y registra
la evidencia.

| ID | Requisito de aceptación | Evidencia automatizada | Evidencia manual / plataforma | Estado inicial Fase 0 |
|---|---|---|---|---|
| R0.1 | Matriz única de alcance, estado y pruebas | Esta matriz + `replica/test-plan.md` | `PRUEBAS_MANUALES.md` | Parcial: matriz creada; falta auditoría funcional completa |
| R0.2 | Entorno y revisión reproducibles, sin usar datos/cuenta personal | Versiones y commit en `replica/phase0-baseline.md` | Perfil Tauri QA aislado y DB sintética verificados; distro Linux/backup pendientes | Parcial |
| R0.3 | Línea base automatizada y manual, con estados inequívocos | Rust, build, Playwright en `replica/phase0-baseline.md` | Tauri QA: B1/B2/B6/C1/C4/C6; C2/C7 parciales; C5 falló (BUG-021); C7 además reveló BUG-022. Resto de Tauri/MT5 pendiente | Parcial |
| R0.4 | Auditoría de gaps de motor, UI, exportación y persistencia | Matriz contra funcionalidades y contratos actuales | Auditoría central y C1 en Tauri; revisión completa de UI pendiente | Parcial |
| R0.5 | Benchmark repetible en Windows y Linux con carga acordada | Dos runs Windows de 10 objetivos/robustez estricta dieron el mismo resumen; el segundo tuvo muestreo de recursos en `replica/phase0-baseline.md` | Importadas 43k barras H1; dos búsquedas completadas en Windows; Linux y presupuesto acordado pendientes | Parcial |
| R0.6 | Dataset GBPUSD H1 2018–2026 y reporte MT5 reproducibles | Import/roundtrip/validación tick pendientes | Export ticks bid/ask y Strategy Tester | Bloqueado: exportación real aún no proporcionada |
| R1.1 | Modelo de operación BUY/SELL completo | Tests de contrato de Trade/order long/short | Ver operación y dirección en detalle/EA | Pendiente |
| R1.2 | Ejecución con costes y bid/ask | Tests de fill/spread/slippage/commission/swap | Comparación de fills | Pendiente |
| R1.3 | SL/TP simétricos, gaps y prioridad de ejecución | Fixtures stop/target/gap para long y short | Casos equivalentes en MT5 Tester | Pendiente |
| R1.4 | Lote fijo/riesgo porcentual, límites de volumen y posiciones | Tests min/max/step/riesgo/posiciones | Configurar y revisar órdenes en Tester | Pendiente |
| R1.5 | Trailing configurable y persistido | Tests de trailing para ambas direcciones | EA Demo/Tester | Pendiente |
| R1.6 | Modo hedging/netting explícito | Tests de contrato por modo | MT5 Tester hedge y netting | Pendiente |
| R2.1 | Importación de ticks bid/ask con metadatos | Tests de parseo, orden y duplicados | Export de referencia de MT5 | Bloqueado por R0.6 |
| R2.2 | Almacenamiento y roundtrip eficiente de ticks | Tests de integridad/roundtrip | Memoria medida en dataset de referencia | Pendiente |
| R2.3 | Backtest tick a tick con costes y órdenes | Escenarios con P&L esperado | Comparación trade a trade en Tester | Pendiente |
| R2.4 | Configuración MT5 completa y persistida | Test de repetición de corrida/config | Comprobar ajustes de símbolo/broker | Pendiente |
| R2.5 | Coincidencia de operaciones/dirección/timestamps/SL/TP | Tests de regresión contra fixture de referencia | Strategy Tester «ticks reales» | Bloqueado por R0.6 |
| R2.6 | Tolerancias cuantificadas y aprobadas | Tests de tolerancias calibradas | Revisar informe y aprobar límites | Bloqueado por R2.5 |
| R2.7 | Fórmulas y métricas alineadas con MT5 | Fixtures por métrica | Informe de referencia de MT5 | Bloqueado por R0.6 |
| R3.1 | Representación común de reglas | Tests de serialización/roundtrip | Crear, guardar, editar y exportar regla | Pendiente |
| R3.2 | Bloques AND/OR de entrada/salida/dirección | Tests de reglas válidas e inválidas | Constructor visual en Tauri | Pendiente |
| R3.3 | SMA, EMA, RSI, ATR, MACD y CCI | Fixtures numéricos/warm-up/no-lookahead por indicador | Selección y resultados en UI | Parcial: validar inventario actual |
| R3.4 | Catálogo de 11 patrones de velas acordados | Fixtures positivos/negativos por patrón | Ver señal/parámetros en UI | Pendiente |
| R3.5 | Señales, SL/TP y trailing seleccionables | Contratos de cada combinación soportada | Flujo completo de creación | Pendiente |
| R3.6 | Motor independiente de símbolo/TF y método elegido | Matriz de integración método × dirección × TF | Runs representativos en Tauri | Pendiente |
| R3.7 | Edición individual con recálculo e invalidación | Test de persistencia y caché/veredicto | Editar y reabrir estrategia | Parcial: editor existe; paridad MT5 pendiente |
| R3.8 | Búsqueda reproducible con semilla, progreso y descarte | Tests deterministas, presupuesto y filtros | Pausa/cancelación/resumen Tauri | Parcial |
| R3.9 | Rendimiento y límites para dataset objetivo | Benchmark reproducible y umbral publicado | Windows/Linux, 43k barras y ticks | Pendiente |
| R4.1 | IS/OOS/holdout aislados de selección | Tests anti-leakage | Revisar segmentos en informes | Pendiente |
| R4.2 | WFE, MC, SPP y perfil con evidencia reproducible | Tests de fórmula/semilla/errores | Revisión Tauri con trades reales | Parcial: suite Rust existente; calibración pendiente |
| R4.3 | Monte Carlo no tautológico | Casos conocidos que deben fallar/pasar | Revisar resultados/percentiles | Parcial: tests existentes; auditoría de suite pendiente |
| R4.4 | Gates individuales y explicables | Contratos pass/warn/fail/pending | Revisar ficha y resumen | Parcial |
| R4.5 | Recomendación robusta sin bloquear exportación por gates | Test exportable vs ranking/etiqueta | Flujo de Banco a exportación | Pendiente |
| R4.6 | Invalidar resultados al cambiar estrategia/dataset/motor/fórmula | Tests de hash y caché | Cambiar config y verificar UI | Parcial: parte de invalidación ya existe |
| R5.1 | Export MQL5 para cada regla soportada | Test de traducción por nodo | Compilar cada fixture en MetaEditor | Parcial: export actual existe; cobertura del alcance pendiente |
| R5.2 | Lotaje, riesgo, límites, SL/TP/trailing y órdenes | Comparación de decisiones motor/EA | MT5 Tester en ambos modos | Pendiente |
| R5.3 | Retirar MT4 del alcance y UI | Test de ausencia de opciones MT4 | Verificar navegación/guía | Pendiente |
| R5.4 | Compilación limpia y Strategy Tester ticks reales | Fixtures y artefactos `.mq5` | MetaEditor + Tester | Bloqueado por R2/R5.1 |
| R5.5 | Hedging y netting en Tester y Demo | Tests de modelo/order management | MT5 Tester/Demo | Pendiente |
| R5.6 | Checklist, hash y supuestos del EA exportado | Test de metadatos/checklist | Revisar export real | Parcial |
| R6.1 | Checklist manual de escritorio completa | Suites automatizadas verdes | Todos los casos aplicables de `PRUEBAS_MANUALES.md` | Parcial |
| R6.2 | 8 semanas y 50 operaciones Demo por estrategia | Validación de evidencia/registro | Monitorización manual en MT5 Demo | Pendiente |
| R6.3 | Cero errores críticos/discrepancias no explicadas en Demo | Test de logs/diff | Revisar Experts/Journal y trades | Pendiente |
| R6.4 | Evidencia Demo anonimizada y reproducible | Test de manifiesto/hash | Informes sin credenciales | Pendiente |
| R6.5 | Cambios de estrategia reinician validaciones | Test de versionado | Repetir flujo después de editar | Pendiente |
| R7.1 | Instaladores locales Windows/Linux sin servidor SophieQuant | Smoke test por plataforma | Instalar en entorno limpio | Pendiente |
| R7.2 | App Linux nativa + MT5 Wine en 8 GB/4 vCPU | Pruebas de instalación y smoke | Linux objetivo/Wine/MT5 | Pendiente: distro por decidir en R0 |
| R7.3 | Backup/restore y migración de SQLite/datasets | Tests de migración/restore | Instalación limpia y recuperación | Pendiente |
| R7.4 | Release GitHub versionada, hashes y documentación | CI verifica artefactos/hashes | Descargar e instalar artefactos | Pendiente |
| R7.5 | Sin S1/S2 abiertos y gates completos | CI completa | Checklist, bugs y excepciones aprobadas | Pendiente |

## Funcionalidades fuera de alcance 1.0

La matriz de comparación completa permanece en `replica/features.csv`; esta
lista es el resumen de alcance aprobado, no un registro de bugs:

- MT4, macOS, ejecución real/broker, acciones/futuros y proveedores integrados.
- Búsqueda batch de todos los símbolos/métodos, datos descargados por la app y
  ejecución de estrategias desde SophieQuant.
- Indicadores y patrones que no aparecen en `ROADMAP_FINAL.md`.

No añadir fuera de alcance como criterio de fallo de 1.0. Reabrirlo requiere una
decisión explícita del propietario del producto.
