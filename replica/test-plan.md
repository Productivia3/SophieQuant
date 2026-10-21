# Test plan: QuantForge

Build: sin commit (repo no iniciado) · Date: 2026-10-20 · Env: local, capa fake (seed data), dev server http://localhost:1420

## Alcance de esta pasada

- **Automatizado (e2e, capa fake)**: Playwright 40 casos + axe — navegación completa, formularios S02, ranura S20/S03/S04/S05/S22, accesibilidad. Suite verde: **40 ✓ / 0 ✘** después del fix-loop (ver `bugs.md`).
- **Manual / bloqueado**: lo que requiere el binario Tauri (builds reales, import CSV real, export a disco, eventos nativos). Marcadas `block`.

Leyenda: e2e = automatizado (Playwright + axe) · manual = checklist humano · block = requiere Rust/binario

| case | flow | type | steps | expected | auto | result |
| --- | --- | --- | --- | --- | --- | --- |
| F01-H1 | generar estrategias | happy | S01 → "Nuevo build" → plantilla/TF → "Correr build" → S03 → "Ver Bank" | S03 con progreso + bank visible | e2e | ✓ |
| F01-E1 | generar estrategias | edge: TF sin dataset | elegir d1 (solo m1 en seed) | aviso "No hay datos históricos" + enlace a Datos navega | e2e | ✓ |
| F01-E2 | generar estrategias | edge: doble clic en "Correr build" | doble submit rápido | sin crash, aterriza en S03 | e2e | ✓ |
| F01-E3 | generar estrategias | edge: refresh mid-build | F5 en S03 durante run | badge correcto tras reload | manual | pendiente |
| F01-E4 | generar estrategias | edge: objetivo fuera de rango | nTarget=9 | input clampa a min 10, corre igual | e2e | ✓ |
| F01-N1 | generar estrategias | negative: build inexistente | /#/build/run/9999 | sin crash (badge pending, 0 pasadas) | e2e | ✓ |
| F02-H1 | revisar resultados | happy | S04 → fila → S05 | 6 statcards + equity, sin NaN/undefined | e2e | ✓ |
| F02-E1 | revisar resultados | edge: estrategia dirty | abrir dirty | aviso de problemas visible | manual | pendiente (seed sin dirty) |
| F02-E2 | revisar resultados | edge: estrategia sin runs | /#/strategy/9999 | pantalla de carga, sin crash | e2e | ✓ |
| F03-H1 | improver | happy | S05 → S09 | stub Fase 3 | manual | block (pantalla stub) |
| F04-H1 | optimizar + 3D | happy | S10 → S11 | stubs | manual | block |
| F05-H1 | robustez MC integrada | happy | S02 checkbox MC ON | checkbox accesible (BUG-002 fixed) | e2e | ✓ (persistencia real = block) |
| F06-H1 | custom project | happy | S16 canvas | Fase 3 | manual | block |
| F07-H1 | portfolio | happy | S18 → S19 | Fase 3 | manual | block |
| F08-H1 | exportar | happy | S22: estrategia → MT5 → "Generar código" | pre con código | e2e | ✓ |
| F08-E1 | exportar | edge: sin estrategia | click Generar sin selección | botón disabled | e2e | ✓ |
| F08-E2 | exportar | edge: las 6 plataformas | generar en cada una | código coherente por plataforma | e2e | ✓ |
| F08-N1 | exportar | negative: id inexistente | api directa id=999 | error controlado | manual | pendiente (con Rust) |
| F09-H1 | data manager | happy | S20 → "Descargar m1" → toast | dataset badge + toast | e2e | ✓ |
| F09-E1 | data manager | edge: descarga simultánea | dos símbolos | solo 1 activa, resto disabled | e2e | ✓ |
| F09-E2 | data manager | edge: CSV con ruido/gaps | importer CSV (Rust) | descartadas con `skipped`, informe claro | block | **BUG-batch pendiente con Rust** |
| F10-H1 | retest | happy | S04 multi → S08 | Fase 3 | manual | block |
| F11-H1 | analyzer | happy | S24 | Fase 4 | manual | block |
| X-H1 | transversal | navegación completa | 20 rutas del Shell | sin console errors ni 5xx | e2e | ✓ (20/20) |
| X-H2 | transversal | axe | 6 pantallas clave | 0 violaciones | e2e | ✓ (tras BUG-001/002) |
| X-404 | transversal | negative | /#/no-existe | 404 propio | e2e | ✓ |
| X-H1b | transversal | mobile 420px | /#/bank con viewport 420 | sin crash; overflow recortado | e2e | ✓ con BUG-005 abierto (S3) |
| X-E2 | transversal | keyboard only | tab por S02 | foco visible en todo | manual | pendiente |

## Bugs encontrados y su estado

- BUG-001 (S2) progressbar sin aria-label → **fixed**
- BUG-002 (S2) checkbox sin label → **fixed**
- BUG-003 (test) selector ambiguo → **fixed in test**
- BUG-004 (S3) tablas sin contenedor de scroll en móvil → **fixed**
- BUG-005 (S3) sidebar no colapsa en móvil → **open** (parrilla Fase 3)
- Detalles en `replica/bugs.md`. No hay S1/S2 abiertos → nada bloquea continuar por esta vía.

## Bloqueos para la siguiente capa de QA (todo con Rust compilado)

1. Golden-file tests exporters MQL4/MQL5 (Rust unit).
2. Tests del importer CSV: fila mala, timestamps ms/seg, ISO con/sin T.
3. Determinismo del genetic (mismo seed → mismo output).
4. E2E del build real: import CSV → build → bank con métricas reales.
5. Cero pueda = `pause/resume` cuando se añadan los commands reales.
