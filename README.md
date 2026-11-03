# SophieQuant

**Plataforma de escritorio para generar y validar estrategias de trading algorítmico.**
Motor propio en Rust, interfaz en español, todo local: tus datos no salen de tu equipo.

[**Guía de usuario en español**](./GUIA_DE_USUARIO.md) — instrucciones paso a paso
para importar datos, generar y revisar estrategias y exportar a MetaTrader, sin
conocimientos técnicos.

SophieQuant nace de una pregunta incómoda: si una estrategia funciona porque el
motor la encontró con mil intentos y no porque tenga lógica, ¿qué valor tiene?
La respuesta de este producto es **enseñarte las pruebas**, no darte un botón
mágico.

---

## Qué hace

| Módulo | Qué resuelve |
| --- | --- |
| **Constructor** | Genera estrategias con un algoritmo genético determinista, filtrando por **cualquier métrica** del motor y ordenando por **score de robustez**, no por beneficio. |
| **Banco de estrategias** | Todas las estrategias que pasan los filtros, con sus métricas y su score de robustez. |
| **Retesteo** | Comprueba una estrategia en otro símbolo o temporalidad, y en 1, 3 o 5 ventanas out-of-sample: la primera prueba de si sobrevive fuera de donde nació. |
| **Refinador** | Genera variantes de la regla (stop, objetivo, parámetros) y conserva la mejor. |
| **Optimizador** | Búsqueda en rejilla simple y walk-forward, con descuento out-of-sample. |
| **Matriz walk-forward** | Mapa de calor de eficiencia out-of-sample por combinación de parámetros. |
| **Monte Carlo** | Las 9 pruebas estándar (reordenar, saltar, aleatorizar parámetros, variantes de dirección, interpolación OHLC, reconstruir retornos, drawdown, duplicar, omitir periodos). |
| **SPP** | Permutación de las piezas de la estrategia: mide cuánto depende el resultado de una sola decisión. |
| **Perfil de optimización** | Curva net–parámetro: meseta (robusta) o pico (frágil). |
| **Cartera** | Agrupa estrategias, mide correlación, pondera por cuatro modelos y **sugiere qué meter** (explicando cada descarte). |
| **Laboratorio** | Métricas avanzadas, Monte Carlo de cartera y simulación de gestión de capital. |
| **Editor sin código** | Edita la regla (3 plantillas que el motor ejecuta de verdad) y **recalcula la corrida**. |
| **Análisis personalizado** | Ejecuta **tu** script de Python sobre la estrategia y sus barras, con un contrato versionado y salida validada. |
| **Exportar código** | Genera un EA real para MetaTrader 4 y MetaTrader 5. |

### Las tres ideas que la differentiating

1. **El score de robustez es explicable.** Gana la estrategia que aguanta, no la
   que más gana. Los pesos están a la vista: 35 % eficiencia walk-forward, 30 %
   supervivencia Monte Carlo, 20 % estabilidad SPP, 15 % forma del perfil.
2. **Los veredictos se ven con su número.** Una tarjeta por estrategia con
   walk-forward, Monte Carlo, SPP y perfil, cada uno con su umbral (`PASSED`,
   `AVISO`, `FAILED`). Nada de «caja negra».
3. **La zona out-of-sample está marcada.** El corte es el mismo que usa la
   comprobación walk-forward y lo calcula el motor, no una aproximación visual.

---

## Estado

Medido con `parity.py` sobre 51 funcionalidades comparables con el producto de
referencia:

| Métrica | Valor |
| --- | --- |
| Paridad de funcionalidades | **65,8 / 100** (68,8 dentro del alcance definido) |
| Must-haves completos | **10 de 20** |
| Tests Rust | **74 / 74** |
| Tests end-to-end + accesibilidad (axe) | **106 / 106** |
| `cargo audit` | **0 vulnerabilidades** |

Lo que **no** está: más tipos de entrada/salida, más indicadores (MACD, CCI),
patrones de velas, bloques personalizados, backtest a nivel de tick, ejecución
multi-símbolo, modelos de ponderación de cartera y gestión de salidas
escalonada (ATM). El detalle está en `replica/parity.md`.

---

## Requisitos

- **Node.js 18+** y **npm**
- **Rust estable** (para el motor y el empaquetado)
- Windows 10/11 · macOS · Linux (el código es multiplataforma; el binario de esta
  entrega se compila y prueba en Windows)

## Puesta en marcha

```bash
npm install
npm run tauri dev      # app de escritorio con recarga en caliente
```

Para la interfaz web sola (sin motor Rust, con datos de ejemplo):

```bash
npm run dev            # http://localhost:1420
```

## Compilar

```bash
npm run tauri build                # recomendado: binario + instalador (.msi)
# o solo el binario:
npm run build                      # bundle web (dist/)
cargo build --release --manifest-path src-tauri/Cargo.toml
# -> src-tauri/target/release/sophiequant.exe
```

**Importante:** el binario de release **debe** compilarse con el rasgo
`custom-protocol` (viene activo por defecto en `src-tauri/Cargo.toml`). Ese rasgo
es el que hace que la app sirva la interfaz embebida en vez de esperar al
servidor de desarrollo: sin el, el ejecutable arranca y muestra «no se puede
conectar a localhost». `tauri dev` funciona porque arranca con
`--no-default-features`.

Acceso directo al Escritorio (copia portable, sobrevive a un `cargo clean`):

```bash
powershell -ExecutionPolicy Bypass -File scripts\crear-acceso-directo.ps1
# -> %LOCALAPPDATA%\Programs\SophieQuant\SophieQuant.exe
```

Tambien hay instalador MSI generado en
`src-tauri/target/release/bundle/msi/SophieQuant_0.1.0_x64_en-US.msi`
(3,9 MB) y el script lo puede instalar: `-Instalador <ruta.msi>`.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml     # motor (40 tests)
npx playwright test                                 # interfaz (88 tests)
python .claude/skills/replica-design/contrast.py replica/design/tokens.json   # contraste AA
```

## Seguridad y privacidad

- **Los datos no salen del equipo.** Sin telemetría, sin cuentas, sin licencias,
  sin red: la app funciona sin conexión.
- La base de datos y los datasets viven en la carpeta de datos local de la
  aplicación (`%APPDATA%\com.sophiequant.app` en Windows; configurable en
  Ajustes).
- `cargo audit --deny warnings` da **0 vulnerabilidades** sobre 437
  dependencias. Los dos avisos informativos aceptados (`glib`,
  `proc-macro-error`) vienen solo del stack GTK/Linux de Tauri y no se compilan
  en la entrega de Windows; están documentados en `src-tauri/audit.toml`.
- Exportar código genera un EA **real** para MetaTrader: en MQL4 y MQL5, y solo
  esas dos plataformas (no se anuncian las que no hay generador).

---

## Cómo está montado

```
src/                    interfaz (React + TypeScript + Tailwind)
  lib/api.ts            puente: comandos reales en Tauri; datos simulados solo
                        en el navegador de desarrollo. Los errores no activan fallbacks.
  components/           piezas reutilizables (gráficos SVG propios, tabla de
                        veredictos, gráfico de equity)
  pages/                una pantalla por módulo
src-tauri/              motor en Rust
  src/engine/           backtest, métricas, indicadores, reglas, generador
                        genético, robustez, veredictos, datastore
  src/commands.rs       comandos Tauri (la API del motor)
  src/export.rs         generadores de código MQL4/MQL5
  schema.sql            esquema SQLite (13 tablas + migraciones)
e2e/                    tests de interfaz con Playwright + axe
replica/                documentación de trabajo: mapa de referencia, arquitectura,
                        diseño, marca, paridad, registro de construcción
.claude/skills/         el kit de herramientas con el que se construyó (recon,
                        arquitectura, diseño, build, test, backend, marca,
                        entrepreneur, diff): documenta el método, no el producto
```

El motor es **determinista**: la misma semilla y la misma configuración dan
exactamente el mismo resultado. Sin lookahead, con spread, comisión y slippage
en el cálculo.

---

## Lo que no se sube al repositorio (y por qué)

`.gitignore` deja fuera tres cosas a propósito:

| Qué | Por qué |
| --- | --- |
| `replica/entrepreneur/raw-*.txt` y `reviews.csv` | Volcados literales de reseñas de terceros (Reddit, Trustpilot, un blog). Se quedan en tu disco: no es material nuestro para redistribuir. El análisis derivado sí está en el repo. |
| `replica/screens/ORIGINAL-*.png` | Capturas de marketing del producto de referencia: propiedad suya. |
| `replica/e2e-report/` | Informe generado, se regenera con los tests. |

La carpeta `replica/` **sí** se sube: es el mapa de referencia, la
arquitectura, el sistema de diseño, la guía de marca, el informe de paridad y
el registro de construcción. Documenta el proceso completo, incluida la
investigación de qué·echan de menos los usuarios y por qué este producto
difiere.

---

## Procedencia

Proyecto **clean-room**: construido desde documentación pública, páginas de
ayuda y el análisis de reseñas de usuarios. No hay código, texto, marcas ni
recursos gráficos del producto de referencia. Los nombres de las funcionalidades
son descriptivos del dominio; la identidad (SophieQuant, paleta «Noche Violeta»,
icono) es propia. Único trabajo derivado: los datos de mercado que tú importas.

Si algún día se publica, hay dos decisiones pendientes: la licencia del
repositorio y el aviso legal de exención de responsabilidad (el producto trata
de rentabilidades).

---

## Documentación

| Documento | Qué contiene |
| --- | --- |
| `replica/recon.md` | Mapa de la referencia: pantallas, flujos, modelo de datos |
| `replica/architecture.md` | Decisiones de arquitectura y contrato de comandos |
| `replica/schema.sql` | Esquema de base de datos |
| `replica/backend.md` | Checklist de backend y huecos conocidos |
| `replica/design/` | Sistema de diseño: tokens, componentes, contraste |
| `replica/brand.md` | Nombre, paleta, brief de logo, voz de marca |
| `replica/entrepreneur/` | Investigación de usuarios: qué·echan de menos y por qué |
| `replica/features.csv` | Matriz de 59 funcionalidades con su estado real |
| `replica/parity.md` | Informe de paridad, layout y plan de evolución |
| `replica/export-verificacion.md` | Qué asume el export y cómo comprobar el EA en MetaTrader |
| `replica/custom-analysis.md` | Contrato de los scripts de Python (Custom Analysis) |
| `replica/build-log.md` | Registro de todo el trabajo, fase a fase |
