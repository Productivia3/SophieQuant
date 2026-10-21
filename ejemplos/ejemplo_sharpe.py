"""Ejemplo de Custom Analysis para SophieQuant.

Uso (lo que hace la app por ti):
    python ejemplo_sharpe.py --input <contrato.json> --output <salida.json>

Calcula el Sharpe anualizado sobre las barras del dataset y devuelve un
veredicto con el contrato de replica/custom-analysis.md.
"""

import csv
import json
import math
import statistics
import sys


def argumentos() -> dict:
    argv = sys.argv[1:]
    return {argv[i]: argv[i + 1] for i in range(0, len(argv) - 1, 2)}


def main() -> int:
    args = argumentos()
    if "--input" not in args or "--output" not in args:
        print("uso: python ejemplo_sharpe.py --input <json> --output <json>", file=sys.stderr)
        return 2

    with open(args["--input"], encoding="utf-8") as fh:
        contrato = json.load(fh)

    closes = []
    csv_path = contrato.get("datos", {}).get("csv", "")
    if csv_path:
        with open(csv_path, newline="", encoding="utf-8") as fh:
            for fila in csv.DictReader(fh):
                closes.append(float(fila["close"]))

    if len(closes) < 3:
        salida = {
            "veredicto": "fail",
            "etiqueta": "no hay barras suficientes",
            "metricas": {"barras": len(closes)},
            "notas": "el dataset tiene menos de 3 barras",
        }
    else:
        retornos = [closes[i] / closes[i - 1] - 1 for i in range(1, len(closes))]
        desv = statistics.pstdev(retornos) if len(retornos) > 1 else 0.0
        sharpe = (statistics.mean(retornos) / desv) * math.sqrt(252) if desv > 0 else 0.0
        veredicto = "pass" if sharpe > 1.0 else "warn" if sharpe > 0 else "fail"
        salida = {
            "veredicto": veredicto,
            "etiqueta": f"sharpe anualizado {sharpe:.2f}",
            "metricas": {
                "sharpe_anual": round(sharpe, 3),
                "barras": len(closes),
                "retorno_medio": round(statistics.mean(retornos), 6),
            },
            "notas": "calculo propio sobre las barras del dataset (252 dias de trading al ano)",
        }

    with open(args["--output"], "w", encoding="utf-8") as fh:
        json.dump(salida, fh, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
