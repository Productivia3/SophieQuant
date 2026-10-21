//! engine/verdicts.rs — veredictos consolidados por estrategia.
//!
//! Un único sitio donde se decide si una estrategia "pasa" las cuatro pruebas
//! que el producto enseña al usuario: walk-forward, Monte Carlo, permutación
//! de parámetros (SPP) y perfil de optimización. La tarjeta de veredictos es
//! la respuesta al problema que más se repite en las reseñas del original:
//! "la validación nativa parece una caja negra".
//!
//! Los umbrales son explícitos y se muestran en la interfaz; no hay caja negra.

use serde_json::{json, Value};

use super::ops::{self, StrategyCore};
use super::types::EngineCfg;

/// Fracción in-sample del dataset. Debe coincidir con el corte de
/// `ops::wfe_of` (60 % in-sample / 40 % out-of-sample).
pub const IS_SPLIT: f64 = 0.6;

/// Fracción out-of-sample: la parte final del dataset, la que no se usa para
/// elegir los parámetros.
pub fn oos_fraction() -> f64 {
    1.0 - IS_SPLIT
}

/// Las 9 pruebas estándar de Monte Carlo (las mismas que ofrece la UI).
pub const MC_TESTS: [&str; 9] = [
    "shuffle",
    "skip",
    "randomize_params",
    "use_variants",
    "ohlc_interpolation",
    "simulation_returns",
    "drawdown",
    "duplicate",
    "skip_certain",
];

/// Umbrales (se muestran en la UI para que el usuario sepa qué significa).
pub const WFE_PASS: f64 = 50.0; // % de eficiencia walk-forward
pub const WFE_WARN: f64 = 30.0;
pub const MC_PASS_RATIO: f64 = 0.999; // 9/9 pruebas superadas
pub const MC_WARN_RATIO: f64 = 0.75; // 7/9 o más

fn check(estado: &str, detalle: String, valor: Option<f64>) -> Value {
    json!({
        "estado": estado,
        "detalle": detalle,
        "valor": valor.map(|v| (v * 100.0).round() / 100.0),
    })
}

fn pending(detalle: impl Into<String>) -> Value {
    check("pending", detalle.into(), None)
}

/// Calcula los cuatro veredictos de una estrategia.
/// Devuelve un JSON con las claves `walkForward`, `monteCarlo`, `spp` y `perfil`.
pub fn for_strategy(core: &StrategyCore, cfg: &EngineCfg) -> Value {
    if !core.has_bars() {
        let p = pending("sin dataset cargado para esta estrategia");
        return json!({
            "walkForward": p, "monteCarlo": p, "spp": p, "perfil": p,
        });
    }

    // 1) Walk-forward: eficiencia out-of-sample frente a in-sample.
    let wfe = ops::wfe_of(&core.bars, core.template, core.params, cfg);
    let walk_forward = if wfe >= WFE_PASS {
        check("pass", format!("WFE {wfe:.1} % · umbral {WFE_PASS:.0} %"), Some(wfe))
    } else if wfe >= WFE_WARN {
        check("warn", format!("WFE {wfe:.1} % · umbral {WFE_PASS:.0} %"), Some(wfe))
    } else {
        check("fail", format!("WFE {wfe:.1} % · umbral {WFE_PASS:.0} %"), Some(wfe))
    };

    // 2) Monte Carlo: las 9 pruebas estándar.
    let tests: Vec<String> = MC_TESTS.iter().map(|s| s.to_string()).collect();
    let monte_carlo = match ops::mc_suite(core, cfg, &tests, 60) {
        Ok(v) => {
            let results = v["results"].as_array().cloned().unwrap_or_default();
            let total = results.len();
            let passed = results.iter().filter(|r| r["passed"] == true).count();
            let ratio = if total == 0 { 0.0 } else { passed as f64 / total as f64 };
            let detalle = format!("{passed}/{total} pruebas superadas");
            if ratio >= MC_PASS_RATIO {
                check("pass", detalle, Some(ratio))
            } else if ratio >= MC_WARN_RATIO {
                check("warn", detalle, Some(ratio))
            } else {
                check("fail", detalle, Some(ratio))
            }
        }
        Err(e) => pending(e),
    };

    // 3) SPP: permutación de las piezas de la estrategia.
    let spp = match ops::spp(core, cfg) {
        Ok(v) => {
            let overall = v["overall"].as_str().unwrap_or("fail");
            let results = v["results"].as_array().cloned().unwrap_or_default();
            let worst = results
                .iter()
                .filter_map(|r| r["impactPct"].as_f64())
                .fold(0.0_f64, f64::max);
            let detalle = format!("impacto máximo {worst:.1} %");
            match overall {
                "pass" => check("pass", detalle, Some(worst)),
                _ => check("fail", detalle, Some(worst)),
            }
        }
        Err(e) => pending(e),
    };

    // 4) Perfil de optimización: forma de la curva net-param.
    let perfil = match ops::opt_profile(core, cfg) {
        Ok(v) => {
            let shape = v["shape"].as_str().unwrap_or("puntual");
            let from = v["stableFrom"].as_f64().unwrap_or(0.0);
            let to = v["stableTo"].as_f64().unwrap_or(0.0);
            let detalle = match shape {
                "meseta" => format!("meseta · zona estable {from:.1}–{to:.1}"),
                "unimodal" => format!("unimodal · zona estable {from:.1}–{to:.1}"),
                _ => format!("puntual · sin meseta estable ({from:.1}–{to:.1})"),
            };
            match shape {
                "meseta" => check("pass", detalle, Some(to - from)),
                "unimodal" => check("warn", detalle, Some(to - from)),
                _ => check("fail", detalle, Some(to - from)),
            }
        }
        Err(e) => pending(e),
    };

    json!({
        "walkForward": walk_forward,
        "monteCarlo": monte_carlo,
        "spp": spp,
        "perfil": perfil,
    })
}

/// Resumen agregado: peor estado entre los cuatro checks.
pub fn overall(checks: &Value) -> &'static str {
    // de peor a mejor
    let order = ["fail", "pending", "warn", "pass"];
    let mut worst_idx = order.len() - 1; // "pass"
    for k in ["walkForward", "monteCarlo", "spp", "perfil"] {
        let estado = checks[k]["estado"].as_str().unwrap_or("pending");
        let idx = order.iter().position(|s| *s == estado).unwrap_or(1);
        if idx < worst_idx {
            worst_idx = idx;
        }
    }
    order[worst_idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rules::Template;
    use crate::engine::types::Bar;

    /// Serie sintética determinista: sirve para exercising el motor completo.
    fn serie(n: usize) -> Vec<Bar> {
        (0..n)
            .map(|i| {
                let base = 1.10 + ((i as f64) * 0.0007).sin() * 0.01 + (i as f64) * 0.00002;
                Bar {
                    time: 1_700_000_000 + i as i64 * 60,
                    open: base,
                    high: base * 1.001,
                    low: base * 0.999,
                    close: base * 1.0004,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    fn core_con_datos() -> StrategyCore {
        StrategyCore {
            id: 1,
            name: "test".into(),
            template: Template::MaCross,
            params: [20.0, 50.0, 0.0],
            bars: serie(600),
        }
    }

    #[test]
    fn oos_es_el_40_por_ciento_final() {
        assert!((oos_fraction() - 0.4).abs() < 1e-9);
    }

    #[test]
    fn hay_nueve_pruebas_de_monte_carlo() {
        assert_eq!(MC_TESTS.len(), 9);
        assert!(MC_TESTS.contains(&"drawdown"));
    }

    #[test]
    fn overall_toma_el_peor_estado() {
        let c = json!({
            "walkForward": {"estado": "pass"},
            "monteCarlo": {"estado": "warn"},
            "spp": {"estado": "pass"},
            "perfil": {"estado": "fail"},
        });
        assert_eq!(overall(&c), "fail");
        let todo_pass = json!({
            "walkForward": {"estado": "pass"},
            "monteCarlo": {"estado": "pass"},
            "spp": {"estado": "pass"},
            "perfil": {"estado": "pass"},
        });
        assert_eq!(overall(&todo_pass), "pass");
    }

    #[test]
    fn veredictos_sin_dataset_quedan_pendientes() {
        let core = StrategyCore {
            id: 1,
            name: "sin datos".into(),
            template: Template::MaCross,
            params: [20.0, 50.0, 0.0],
            bars: vec![],
        };
        let v = for_strategy(&core, &EngineCfg::default());
        for k in ["walkForward", "monteCarlo", "spp", "perfil"] {
            assert_eq!(v[k]["estado"], "pending", "falta {k}");
        }
    }

    #[test]
    fn los_cuatro_veredictos_se_calculan_con_datos_reales() {
        let core = core_con_datos();
        let v = for_strategy(&core, &EngineCfg::default());
        for k in ["walkForward", "monteCarlo", "spp", "perfil"] {
            let estado = v[k]["estado"].as_str().unwrap_or("");
            assert!(
                ["pass", "warn", "fail", "pending"].contains(&estado),
                "estado invalido en {k}: {estado}"
            );
            assert!(
                !v[k]["detalle"].as_str().unwrap_or("").is_empty(),
                "falta detalle en {k}"
            );
        }
        // el WFE siempre se calcula con barras suficientes
        assert!(v["walkForward"]["valor"].is_number());
    }
}
