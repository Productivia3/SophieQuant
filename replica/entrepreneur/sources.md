# Fuentes de la investigación — replica-entrepreneur

Fecha de recolección: 20-10-2026. Producto clasificado: StrategyQuant X (SQX, app de escritorio de trading algorítmico).

## Resumen

- **52 filas** en `reviews.csv`: 30 comentarios de Reddit + 6 reseñas de Trustpilot + 16 frases verbatim de 1 reseña experta (StatOasis).
- **Todas las citas son textuales** y están extraídas por script desde los JSON/markdown guardados, no transcritas a mano. Cada fila lleva su enlace.

## Fuentes alcanzadas

| Fuente | Método | Resultado |
| --- | --- | --- |
| Reddit | API PullPush (archivo público de API; la API oficial de Reddit devuelve 403 sin OAuth) | Búsqueda general `strategyquant` (100 comentarios), 4 hilos completos: "Does anyone have to try Strategyquant? It is worth it?", "Thoughts on strategyquant and quantconnect", "Your opinion on Strategy Quant", "StrategyQuant X in Live Trading: … Curve-Fitting Generator?" |
| Trustpilot | Lectura puntual de la página pública vía proxy r.jina.ai (trustpilot.com devuelve 403 directo) | 8 reseñas totales (4,3/5). 6 recuperadas, 2 no aparecían en la versión servida. **Nota del autor StatOasis: Trustpilot también devuelve 403 a su revisión automática.** |
| Blogs de reseña experta | DuckDuckGo HTML + fetch | StatOasis (Ali Casey): reseña profunda con precios verificados (ago-2026), Build 144, y límites honestos. Disclosure: es afiliado pagado y testimonial en la página de precios de SQX — la usamos como **investigación, no como recomendación**. |

## Fuentes no alcanzables (documentadas, sin inventar nada)

- **Hacker News (API Algolia oficial)**: 4 resultados, ninguno aplicable («strategy quant» como puesto de trabajo). Fuente vacía.
- **ForexPeaceArmy** (reseñas de usuarios): 403, bloqueado para acceso automatizado.
- **Reddit directo (reddit.com/old.reddit)**: 403. Se usó PullPush en su lugar.
- **G2 / Capterra / Product Hunt**: no localizados para SQX (producto de nicho de escritorio, sin listados de tiendas).
- **App Store / Google Play**: SQX no tiene listado (es de escritorio). No aplicable.
- **Board oficial de feature requests**: no existe un Canny público (`strategyquant.canny.io` → 404).

## Muestra y límites honestos

- Muestra **pequeña pero verificable**: 3 fuentes, todas con enlace. No cumple el objetivo orientativo de 100+ reseñas; el volumen real disponible para un producto así es limitado (Trustpilot tiene 8 reseñas en total).
- Reddit: el comentario con más peso (5 puntos) es `nnwei46`; la mayoría de comentarios valen 1-2 puntos o están moderados/borrados.
- **Spam excluido** (no son reseñas): Silly-Cup-1477 y LimpDiscipline9665 (reventa de licencias), AFRANK97 y HaveGunsWillTravl (reventa), Lopsided-Ship6875 (spam de cursos con código promocional). Relevancia de mercado: hay mercado secundario de licencias revendidas y spam de afiliados persistente alrededor del producto.
- Reseñas de Trustpilot sin estrellas individuales verificables en el HTML leído → `rating` en blanco (el global es 4,3).

## Artefactos

- `reviews.csv` — fuente de datos de reviews.py.
- `themes-sqx.json` — temas adaptados al dominio (el genérico del pack mezclaba "support" con reseñas positivas).
- `feedback.md` — salida de `reviews.py` (ranking de temas con citas y enlaces).
- `raw-pullpush-comments-search.txt`, `raw-thread-1l2zqqz.txt` — basura cruda archivada como evidencia.
