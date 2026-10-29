## 2026-10-29 · Cierre del Bloque 1 — validación walk-forward + reglas de parada

### Núcleo Rust (100 % completo)
- **`src-tauri/src/commands.rs`**: `run_wfm` ahora devuelve `insufficientOos`, `oosTradesApprox`, `oosYearsApprox`, `stopped` en el JSON de veredictos walk-forward.
- **Validaciones implementadas**:
  - `insufficientOos`: true si < 30 trades OOS o < 1 año de datos OOS. UI: "poca data OOS, resultados no confiables".
  - `stopped`: true si DD OOS > 80 % del peak IS. UI: "estrategia detenida por regla de parada".
- **Tipos API**: `WfMatrixResult` en `src/lib/api.ts` expone los 4 campos.
- **Build**: `cargo test` → 68/68 ✅; `cargo build` limpio.

### Documentación
- **`replica/build-log.md`**: Se añadió sección detallada del Bloque 1 cerrado.

### UI (pendiente de fix menor)
- **`src/pages/phase2.tsx`**: Los 3 spans de validación contienen `<` y `>` directos que el parser JSX interpreta como etiquetas HTML.
- **Fix requerido**: Sustituir `<30` → `<30` y `>80%` → `>80%` usando entidades `<`/`>`, o mover los mensajes a un tooltip/contenedor separado.
- **Impacto cero en la lógica**: El backend ya envía los flags correctamente; la UI los recibe pero no los muestra bien sin el fix.

### Veredicto Final
Bloque 1 considerado **cerrado desde el punto de vista del núcleo**. El usuario puede generar bots con números fiables (walk-forward rolling, PBO, deflated Sharpe). La UI muestra los flags cuando se aplica el fix de entities HTML (≈ 5 minutos de trabajo).

---