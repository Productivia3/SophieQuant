# Custom Analysis: contrato con scripts de Python

Fecha: 21-10-2026 · Versión del contrato: **1** · Implementación: `engine/analysis.rs`

## Qué es

La vía para que **tus** análisis养护 (en Python) hablen con SophieQuant. El
producto no sabe qué va a hacer tu script: solo le pasa la estrategia y sus
datos, y lee un veredicto.

Por qué existe: el usuario tipo de estas herramientas acaba montando su propio
pipeline en Python para validar lo que el software no valida. En vez de dejarle
un muro, se le da un contrato.

## Cómo se ejecuta

1. En el detalle de la estrategia: **Analisis personalizado (Python)** →
   «Elegir script» → se abre el selector nativo.
2. «Ejecutar»:
   - la app escribe `<carpeta de datos>/custom_analysis/strategy_<id>_input.json`
     con el contrato de abajo;
   - escribe `<...>/strategy_<id>_bars.csv` con las barras
     (`time,open,high,low,close,volume`);
   - lanza `python <tu script> --input <json> --output <json>`;
   - lee `<...>/strategy_<id>_output.json` y normaliza el resultado.

## Salvaguardas

- Se invoca **sin shell**: no hay concatenación de comandos.
- **Tiempo límite de 60 s**: si el script se cuelga, se le mata y se avisa.
- El script **lo elige el usuario**, nunca la app.
- La salida se **valida antes de trustarla**: solo se leen `veredicto`,
  `etiqueta`, `metricas` (solo números) y `notas`. Cualquier otro campo se ignora.
- El `veredicto` se acepta en cualquier combinación de mayúsculas: `pass`,
  `passed`, `ok` → `pass`; `warn` → `warn`; `fail`, `ko` → `fail`; cualquier otra
  cosa → `pending`.

## El JSON de entrada

```json
{
  "contrato": "sophiequant.custom-analysis",
  "version": 1,
  "estrategia": {
    "id": 12,
    "nombre": "Cruce de medias·20·50",
    "regla": "Cruce de medias",
    "plantilla": "MaCross",
    "params": [20.0, 50.0, 0.0]
  },
  "datos": {
    "csv": "C:\\...\\strategy_12_bars.csv",
    "formato": "time,open,high,low,close,volume",
    "barras": 0
  },
  "salida_esperada": {
    "veredicto": "pass | warn | fail",
    "etiqueta": "texto corto que resuma el analisis",
    "metricas": { "clave": "valor numerico" },
    "notas": "texto opcional con el razonamiento"
  }
}
```

## El JSON de salida

```json
{
  "veredicto": "warn",
  "etiqueta": "el setup aguanta en 2 de 3 regimes",
  "metricas": { "sharpe": 1.12, "max_dd": 18.4 },
  "notas": "el tramo de 2023 rompe la estrategia; conviene filtrar por volatilidad"
}
```

Lo que devuelva el script se guarda en `custom_analyses` y aparece en el
historial de la estrategia.

## Script mínimo de ejemplo

```python
import json, sys, csv, statistics

args = dict(zip(sys.argv[1::2], sys.argv[1::2][1:]))
with open(args["--input"], encoding="utf-8") as fh:
    contrato = json.load(fh)

# las barras vienen en el csv que indica el contrato
closes = []
with open(contrato["datos"]["csv"], newline="", encoding="utf-8") as fh:
    for fila in csv.DictReader(fh):
        closes.append(float(fila["close"]))

rets = [closes[i] / closes[i-1] - 1 for i in range(1, len(closes))] if len(closes) > 1 else []
sharpe = (statistics.mean(rets) / statistics.pstdev(rets)) * (252 ** 0.5) if len(rets) > 1 and statistics.pstdev(rets) else 0.0

veredicto = "pass" if sharpe > 1.0 else "warn" if sharpe > 0 else "fail"
salida = {
    "veredicto": veredicto,
    "etiqueta": f"sharpe anualizado {sharpe:.2f}",
    "metricas": {"sharpe_anual": round(sharpe, 3), "barras": len(closes)},
    "notas": "calculo propio sobre las barras del dataset",
}
with open(args["--output"], "w", encoding="utf-8") as fh:
    json.dump(salida, fh)
```

## Ideas de uso

- Validar la estrategia con **tus** criterios (filtro por régimen de volatilidad,
  por horario, por noticia).
- Calcular métricas que el motor no tiene (Sharpe anualizado, Omega, CPD).
- Probar la regla con **datos de otra fuente**: el script puede ignorar el CSV del
  contrato y usar el suyo, siempre que cite en `notas` de dónde salen los datos.
- Convertir un script en una **estrategia**: si el JSON de salida incluye
  `params` con una configuración que el motor entiende, se puede guardar como
  estrategia nueva (pendiente:Gap de implementación, anotado aquí para que no se
  pierda).
