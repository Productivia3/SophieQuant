## 2026-10-28 · Bloque 1 CERRADO — validación walk-forward + PBO + reglas de parada

Se completan los sub-bloques pendientes del Bloque 1:

- **1.1 Walk‑forward rolling**: matrix WFE + veredictos por ventana (ya implementado en Blocks 1.1‑1.3).
- **1.2 PBO y deflated Sharpe**: `sharpeAnual × (1‑PBO)` calculado y expuesto en la tarjeta de veredictos.
- **1.3 Mínimo de muestra y regla de parada** (item 1.4):
  - `insufficientOos`: bandera si < 30 trades OOS o < 1 año de datos OOS → UI muestra advertencia "poca data OOS, resultados no confiables".
  - `stopped`: bandera si el drawdown OOS supera el 80 % del peak IS → UI muestra advertencia "estrategia detenida por regla de parada".
  - Ambos campos incluidos en la respuesta JSON de `run_wfm` y en la interfaz `WfMatrixResult` del API TypeScript.
  - Build Rust (`cargo build`) limpio: 68 tests pasando, 0 errores.
  - Build TypeScript/Vite (`npm run build`) limpio.

**Nuevos campos en la UI (fix aplicado y verificado)**:
```json
{
  "insufficientOos": false,
  "oosTradesApprox": 45,
  "oosYearsApprox": 2,
  "stopped": false
}
```
La UI usa `&lt;30`, `&lt;1` y `&gt;80%`. La capa fake avisa que no puede acreditar una muestra suficiente; el backend Rust calcula trades OOS, duración y drawdown a partir de la mejor celda y de la configuración real del símbolo, sin inferirlos del profit factor.

**Verificación de cierre (2026-10-29)**: `npm run build`, `cargo build` y `cargo test` (74 tests) correctos; suite Playwright + axe: 106/106 correctos. Los smoke tests Tauri verificaron importación CSV sintética, persistencia al refrescar, Banco y exportación. El importador también acepta CSV separados por coma, punto y coma o tabulador, con columnas de fecha/hora separadas y decimal europeo. La importación del CSV concreto del usuario queda pendiente de confirmación.

---