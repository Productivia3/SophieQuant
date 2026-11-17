//! engine/genetic.rs — generador V1: búsqueda aleatoria + evolución simple
//! (mutación gaussiana sobre los supervivientes) paralelizada con rayon.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::backtest;
use super::robust::{self, MetricFilter, Robustness};
use super::rules::Template;
use super::types::{BacktestOutput, Bar, EngineCfg};

pub trait BuildReporter: Send + Sync {
    /// llamado desde hilos del build; cancel = true para abortar
    fn report(&self, level: &str, msg: &str, passed: u32, total: u32, progress: f64);
    fn cancelled(&self) -> bool;
    /// pausa: el bucle espera sin gastar CPU hasta que se reanude
    fn paused(&self) -> bool { false }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildConfig {
    pub kind: String,
    pub symbol_id: i64,
    pub timeframe: String,
    pub template_id: i64,   // 1|2|3
    pub n_target: u32,
    /// Filtros por cualquier metrica (antes: PF minimo y DD maximo fijos).
    pub filters: Vec<MetricFilter>,
    /// Numero minimo de operaciones para considerar valida una estrategia.
    pub min_trades: u32,
    /// Exigir validaciones estrictas y ordenar por score de robustez.
    pub por_robustez: bool,
    pub monte_carlo: bool,
    pub n_iterations: u32,  // presupuesto de backtests
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            kind: "builder".into(),
            symbol_id: 1, timeframe: "m1".into(), template_id: 1,
            n_target: 100,
            filters: vec![
                MetricFilter::new("profitFactor", ">=", 1.15),
                MetricFilter::new("maxDD", "<=", 35.0),
            ],
            min_trades: 20,
            por_robustez: true,
            monte_carlo: true, n_iterations: 4000,
        }
    }
}

pub struct GenStrat {
    pub tmpl: Template,
    pub params: [f64; 3],
    pub name: String,
    pub result: BacktestOutput,
    /// Componentes del score de robustez (0-100) y su score final.
    pub robust: Robustness,
}

pub struct BuildOutcome {
    pub strategies: Vec<GenStrat>,
    pub summary: String,
}

/// Genera `n_target` estrategias que pasan los filtros, con un presupuesto máximo de iteraciones.
pub fn build(
    bars: &[Bar],
    cfg: &EngineCfg,
    bcfg: &BuildConfig,
    reporter: &dyn BuildReporter,
) -> BuildOutcome {
    let mut passed: Vec<GenStrat> = Vec::new();
    let n_target = bcfg.n_target.min(2000);
    let mut iterations_left: i64 = bcfg.n_iterations.max(200) as i64;
    let search_bars = if bcfg.por_robustez {
        let cut = (bars.len() * 6 / 10).max(60);
        if cut < 300 {
            return BuildOutcome {
                strategies: passed,
                summary: "No se pudo ejecutar: la robustez estricta requiere al menos 300 barras de entrenamiento; importa un dataset más largo.".into(),
            };
        }
        &bars[..cut]
    } else {
        bars
    };

    // población inicial: variantes de la plantilla elegida con params aleatorios
    let templates = [Template::MaCross, Template::RsiReversion, Template::AtrBreakout];
    let seeds: Vec<(Template, [f64; 3])> = (0..24)
        .map(|i| {
            let t = if bcfg.template_id > 0 {
                Template::from_id(bcfg.template_id).unwrap_or(templates[i % 3])
            } else {
                templates[i % 3]
            };
            (t, mutate(t.default_params(), 0.35, i as u64))
        })
        .collect();

    let mut current_gen: Vec<(Template, [f64; 3])> = seeds;

    // contador de rechazos por filtro: al final se dice cual esta bloqueando
    let rechazos: Vec<std::sync::atomic::AtomicU32> =
        (0..bcfg.filters.len().max(1)).map(|_| std::sync::atomic::AtomicU32::new(0)).collect();
    let rechazos_min_trades = std::sync::atomic::AtomicU32::new(0);
    let mut candidatos_evaluados = 0u32;
    let mut pasaron_filtros = 0u32;

    for gen in 0.. {
        if passed.len() >= n_target as usize || iterations_left <= 0 || reporter.cancelled() { break }
        // pausa: esperar sin gastar CPU hasta reanudar o cancelar
        while reporter.paused() && !reporter.cancelled() {
            std::thread::sleep(std::time::Duration::from_millis(150));
        }

        let batch: Vec<GenStrat> = current_gen
            .par_iter()
            .filter_map(|(t, p)| {
                let r = backtest::run(search_bars, *t, *p, cfg);
                let filter_results = robust::filter_report(&r.metrics, &bcfg.filters);
                for (idx, result) in filter_results.iter().enumerate() {
                    if result["ok"] == false {
                        rechazos[idx].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                let enough_trades = r.metrics.n_trades >= bcfg.min_trades;
                if !enough_trades {
                    rechazos_min_trades.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                if enough_trades && filter_results.iter().all(|result| result["ok"] == true) {
                    Some(GenStrat {
                        tmpl: *t,
                        params: *p,
                        name: format!("{} {}", t.label(), gen_label(*t, p)),
                        result: r,
                        robust: Robustness::default(),
                    })
                } else {
                    None
                }
            })
            .collect();

        candidatos_evaluados += current_gen.len() as u32;
        pasaron_filtros += batch.len() as u32;
        iterations_left -= current_gen.len() as i64;
        passed.extend(batch.into_iter()
            .take(n_target as usize - passed.len().min(n_target as usize)));

        let p = if bcfg.n_iterations > 0 {
            (bcfg.n_iterations - iterations_left.max(0) as u32) as f64 / bcfg.n_iterations as f64
        } else { 1.0 };
        let progress = p;
        reporter.report(
            "info",
            &format!("generación {}: {} pasadas de {}", gen, passed.len(), bcfg.n_target),
            passed.len() as u32,
            bcfg.n_target,
            progress,
        );

        // nueva generación: mutación de TODOS los seeds (ya que no mutamos sobre los pasados
        // hasta tener variedad); intercalamos mutaciones de los survivors para converger
        current_gen = (0..24)
            .map(|i| {
                let base = if !passed.is_empty() && i % 2 == 1 {
                    let pp = &passed[i % passed.len()];
                    (pp.tmpl, pp.params)
                } else if bcfg.template_id > 0 {
                    (Template::from_id(bcfg.template_id).unwrap_or(templates[i % 3]), templates[i % 3].default_params())
                } else {
                    (templates[i % 3], templates[i % 3].default_params())
                };
                (base.0, mutate(base.1, 0.18, gen as u64 * 97 + i as u64))
            })
            .collect();
    }

    // ── seleccion ──────────────────────────────────────────────────────
    // Con el modo estricto solo se conservan candidatos que pasan todas las
    // validaciones; el score ordena los que ya han quedado calificados.
    let mut robust_evaluated = 0usize;
    let mut robust_qualified = 0usize;
    let mut robust_summary = None;
    if bcfg.por_robustez && !passed.is_empty() {
        let evaluated = passed.len();
        robust_evaluated = evaluated;
        reporter.report(
            "info",
            &format!("evaluando robustez estricta de {evaluated} candidatos..."),
            0,
            bcfg.n_target,
            0.95,
        );
        passed = passed
            .into_par_iter()
            .map(|mut s| {
                s.robust = robust::full(bars, s.tmpl, s.params, cfg);
                if !bcfg.monte_carlo {
                    s.robust.fallos.push("Monte Carlo integrado desactivado".into());
                    s.robust.calificado = false;
                }
                if s.robust.calificado {
                    s.result = backtest::run(bars, s.tmpl, s.params, cfg);
                }
                s
            })
            .collect();
        let qualified = passed.iter().filter(|s| s.robust.calificado).count();
        let mut robust_failures = std::collections::BTreeMap::<String, u32>::new();
        for candidate in &passed {
            for failure in &candidate.robust.fallos {
                *robust_failures.entry(failure.clone()).or_default() += 1;
            }
        }
        let detail = robust_failures.iter()
            .map(|(reason, count)| format!("{reason}: {count}"))
            .collect::<Vec<_>>()
            .join("; ");
        passed.retain(|s| s.robust.calificado);
        if qualified < evaluated {
            reporter.report(
                "warn",
                &format!("robustez estricta: {qualified}/{evaluated} candidatos calificados; fallos por criterio (pueden solaparse): {detail}"),
                qualified as u32, bcfg.n_target, 1.0,
            );
        }
        robust_qualified = qualified;
        robust_summary = Some(format!(
            "Robustez estricta: {qualified} de {evaluated} candidatos superaron todas las pruebas.{}",
            if detail.is_empty() { String::new() } else { format!(" Fallos por criterio (pueden solaparse): {detail}.") }
        ));
        passed.sort_by(|a, b| {
            b.robust.score.partial_cmp(&a.robust.score).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        // orden: mejor PF primero
        passed.sort_by(|a, b| b.result.metrics.profit_factor
            .partial_cmp(&a.result.metrics.profit_factor).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mut summary = vec![format!(
        "Evaluación base: {candidatos_evaluados} candidatos; {pasaron_filtros} superaron todos los filtros y el mínimo de operaciones."
    )];
    if rechazos_min_trades.load(std::sync::atomic::Ordering::Relaxed) > 0
        || rechazos.iter().any(|count| count.load(std::sync::atomic::Ordering::Relaxed) > 0)
    {
        summary.push("Los descartes por criterio pueden solaparse; un candidato puede incumplir varios.".into());
    }
    let min_trades_rejected = rechazos_min_trades.load(std::sync::atomic::Ordering::Relaxed);
    if min_trades_rejected > 0 {
        summary.push(format!(
            "Mínimo de operaciones (< {}): {min_trades_rejected} candidatos descartados.",
            bcfg.min_trades
        ));
    }
    for (idx, filter) in bcfg.filters.iter().enumerate() {
        let rejected = rechazos[idx].load(std::sync::atomic::Ordering::Relaxed);
        if rejected > 0 {
            let label = robust::METRICAS.iter()
                .find(|(key, _, _)| *key == filter.metric)
                .map(|(_, label, _)| *label)
                .unwrap_or(&filter.metric);
            summary.push(format!(
                "Filtro {label} {} {}: {rejected} candidatos descartados.",
                filter.op, filter.value
            ));
        }
    }
    if bcfg.por_robustez {
        let message = if pasaron_filtros == 0 {
            "Robustez estricta: no se evaluaron finalistas porque ningún candidato superó los filtros base.".to_string()
        } else {
            robust_summary.unwrap_or_else(|| format!(
                "Robustez estricta: {robust_qualified} de {robust_evaluated} candidatos superaron todas las pruebas."
            ))
        };
        summary.push(message);
    }
    let result_count = passed.len() as u32;
    if reporter.cancelled() {
        summary.push(format!("Generación cancelada; quedaron {result_count} estrategias calificadas."));
    } else if result_count >= n_target {
        summary.push(format!("Objetivo alcanzado: {result_count}/{n_target} estrategias."));
    } else {
        summary.push(format!(
            "No se alcanzó el objetivo de {n_target} estrategias: solo {result_count} superaron todos los criterios con el presupuesto de búsqueda configurado."
        ));
    }

    BuildOutcome { strategies: passed, summary: summary.join("\n") }
}

fn gen_label(t: Template, p: &[f64; 3]) -> String {
    match t {
        Template::MaCross => format!("{}·{}", p[0], p[1]),
        Template::RsiReversion => format!("{}·{}", p[0], p[1]),
        Template::AtrBreakout => format!("{}·{}", p[0], p[1]),
    }
}

/// mutación gaussiana determinística por seed (mismo seed → mismos params)
fn mutate(base: [f64; 3], strength: f64, seed: u64) -> [f64; 3] {
    let mut s = xorshift(seed);
    base.map(|v| {
        if v == 0.0 { return 0.0 }
        let noise = (rnd(&mut s) - 0.5) * 2.0 * strength * v.abs() + (rnd(&mut s) - 0.5) * strength;
        let n = v + noise;
        n.max(2.0).min(400.0)
    })
}

fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13; x ^= x >> 7; x ^= x << 17; x
}
fn rnd(state: &mut u64) -> f64 {
    *state = xorshift(*state);
    (*state % 10_000) as f64 / 10_000.0
}

// ── tests del generador ───────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// Serie sintética con cruces (igual que la de backtest; aquí duplicada
    /// porque los mod test no se ven entre sí).
    fn serie(n: usize) -> Vec<Bar> {
        let t0 = 1_700_000_000i64;
        (0..n).map(|i| {
            let w = i as f64 / 24.0;
            let base = 1.1000 + i as f64 * 0.00002;
            let onda = (w * core::f64::consts::TAU).sin() * 0.0008;
            let o = base + onda;
            let c = base + (w * core::f64::consts::TAU + 0.35).sin() * 0.0008;
            Bar {
                time: t0 + (i as i64) * 60,
                open: o,
                high: o.max(c) + 0.0003,
                low: o.min(c) - 0.0003,
                close: c,
                volume: 100.0,
            }
        }).collect()
    }

    /// Reporter mudo para los tests.
    struct SilentReporter;

    impl BuildReporter for SilentReporter {
        fn report(&self, _l: &str, _m: &str, _p: u32, _t: u32, _g: f64) {}
        fn cancelled(&self) -> bool { false }
    }

    #[test]
    fn build_es_deterministico_mismo_config_mismo_resultado() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 4;
        bcfg.n_iterations = 500;
        bcfg.template_id = 1;   // MaCross

        let r1 = build(&bars, &cfg, &bcfg, &SilentReporter);
        let r2 = build(&bars, &cfg, &bcfg, &SilentReporter);

        // mismo equilibrio de supervivientes y mismos params en el mismo orden
        assert_eq!(r1.strategies.len(), r2.strategies.len(), "mismo nº de pasadas");
        for (a, b) in r1.strategies.iter().zip(r2.strategies.iter()) {
            assert_eq!(a.params, b.params, "params idénticos");
            assert_eq!(a.tmpl, b.tmpl);
            assert_eq!(a.result.metrics.profit_factor, b.result.metrics.profit_factor);
        }
    }

    #[test]
    fn build_respeta_los_filtros_por_cualquier_metrica() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 10;
        bcfg.n_iterations = 600;
        bcfg.template_id = 1;
        bcfg.filters = vec![
            MetricFilter::new("profitFactor", ">=", 1.0),
            MetricFilter::new("maxDD", "<=", 20.0),
        ];

        let out = build(&bars, &cfg, &bcfg, &SilentReporter);
        for s in out.strategies.iter() {
            assert!(s.result.metrics.profit_factor >= 1.0);
            assert!(s.result.metrics.max_dd <= 20.0, "DD {} > 20", s.result.metrics.max_dd);
            assert!(s.result.metrics.n_trades >= bcfg.min_trades);
        }
        assert!(out.strategies.len() as u32 <= bcfg.n_target);
    }

    #[test]
    fn un_filtro_por_sharpe_tambien_se_respeta() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 6;
        bcfg.n_iterations = 600;
        bcfg.template_id = 1;
        bcfg.filters = vec![MetricFilter::new("sharpe", ">=", 0.5)];

        let out = build(&bars, &cfg, &bcfg, &SilentReporter);
        for s in out.strategies.iter() {
            assert!(s.result.metrics.sharpe >= 0.5, "sharpe {}", s.result.metrics.sharpe);
        }
    }

    #[test]
    fn build_resume_el_objetivo_y_cuantos_candidatos_bloqueo_cada_filtro() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 10;
        bcfg.n_iterations = 200;
        bcfg.template_id = 1;
        bcfg.min_trades = 10_000;
        bcfg.por_robustez = false;
        bcfg.filters = vec![
            MetricFilter::new("profitFactor", ">=", 1_000_000_000.0),
            MetricFilter::new("maxDD", "<=", -1.0),
        ];

        let outcome = build(&bars, &cfg, &bcfg, &SilentReporter);

        assert!(outcome.strategies.is_empty());
        assert!(outcome.summary.contains("Evaluación base:"));
        assert!(outcome.summary.contains("Mínimo de operaciones (< 10000)"));
        assert!(outcome.summary.contains("Filtro Profit factor >= 1000000000"));
        assert!(outcome.summary.contains("Filtro Drawdown máximo (%) <= -1"));
        assert!(outcome.summary.contains("pueden solaparse"));
        assert!(outcome.summary.contains("No se alcanzó el objetivo de 10 estrategias"));
    }

    #[test]
    fn por_robustez_las_estrategias_vienen_ordenadas_por_score() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 8;
        bcfg.n_iterations = 1200;
        bcfg.template_id = 1;
        bcfg.filters = vec![MetricFilter::new("profitFactor", ">=", 0.9)];
        bcfg.por_robustez = true;

        let out = build(&bars, &cfg, &bcfg, &SilentReporter);
        // El modo estricto puede devolver cero: nunca debe incluir una candidata
        // que no haya superado cada criterio.
        for s in out.strategies.iter() {
            assert!(s.robust.calificado, "candidato no calificado: {}", s.name);
            assert!(s.robust.wfe <= 150.0 && s.robust.mc_survival >= 0.0);
            assert!(s.robust.mc_survival <= 100.0);
        }
        // y van ordenadas de mas a menos score
        for w in out.strategies.windows(2) {
            assert!(
                w[0].robust.score >= w[1].robust.score,
                "orden roto: {} vs {}",
                w[0].robust.score,
                w[1].robust.score
            );
        }
        // todas llevan el score avanzado (SPP + perfil).
        for s in out.strategies.iter() {
            assert!(s.robust.avanzado, "falta el score avanzado en {}", s.name);
        }
    }

    #[test]
    fn sin_robustez_se_ordena_por_profit_factor() {
        let bars = serie(800);
        let cfg = EngineCfg::default();
        let mut bcfg = BuildConfig::default();
        bcfg.n_target = 8;
        bcfg.n_iterations = 1200;
        bcfg.template_id = 1;
        bcfg.filters = vec![MetricFilter::new("profitFactor", ">=", 0.9)];
        bcfg.por_robustez = false;

        let out = build(&bars, &cfg, &bcfg, &SilentReporter);
        for w in out.strategies.windows(2) {
            assert!(
                w[0].result.metrics.profit_factor >= w[1].result.metrics.profit_factor,
                "orden por PF roto"
            );
        }
    }

    #[test]
    fn mutate_mismo_seed_mismos_params() {
        let base = [14.0, 45.0, 2.0];
        assert_eq!(mutate(base, 0.2, 42), mutate(base, 0.2, 42));
        assert_ne!(mutate(base, 0.2, 42), mutate(base, 0.2, 43));
    }
}
