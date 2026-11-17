//! engine/robust.rs — filtros por cualquier métrica y score de robustez.
//!
//! Dos cosas que el generador V1 no tenia:
//!
//! 1. **Filtros por cualquier métrica**: el usuario puede exigir que la estrategia
//!    cumpla cualquier métrica del motor (PF, Sharpe, Max DD, R², Z-score...) con
//!    el operador que quiera, en lugar de los dos filtros fijos.
//! 2. **Selección por robustez** (el "advanced robustness" del original): entre las
//!    estrategias que pasan los filtros, no gana la que mas beneficio da, sino la
//!    que aguanta mejor fuera de muestra y frente a perturbaciones.
//!
//! El score es explicable: cada componente tiene su peso y su significado, y se
//! muestra en la interfaz. Nada de "una caja negra que decide".

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::backtest;
use super::metrics;
use super::rules::Template;
use super::types::{Bar, EngineCfg, Metrics};

/// Un filtro del generador: métrica, operador y valor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricFilter {
    /// Clave de la métrica (ver `metric_value`).
    pub metric: String,
    /// ">=", "<=", ">" o "<".
    pub op: String,
    pub value: f64,
}

impl MetricFilter {
    pub fn new(metric: &str, op: &str, value: f64) -> Self {
        Self { metric: metric.into(), op: op.into(), value }
    }
}

/// Métricas por las que se puede filtrar, con su etiqueta para la interfaz.
pub const METRICAS: [(&str, &str, &str); 12] = [
    ("profitFactor", "Profit factor", "profit_factor"),
    ("sharpe", "Sharpe", "sharpe"),
    ("sortino", "Sortino", "sortino"),
    ("maxDD", "Drawdown máximo (%)", "max_dd"),
    ("rdd", "Recovery factor", "rdd"),
    ("netProfit", "Beneficio neto", "net_profit"),
    ("grossProfit", "Beneficio bruto", "gross_profit"),
    ("winRate", "Tasa de acierto", "win_rate"),
    ("zScore", "Z-score", "z_score"),
    ("rSquared", "R²", "r_squared"),
    ("avgTrade", "Beneficio medio por operación", "avg_trade"),
    ("nTrades", "Número de operaciones", "n_trades"),
];

/// Valor de una métrica por su clave. `None` si la clave no existe.
pub fn metric_value(m: &Metrics, key: &str) -> Option<f64> {
    Some(match key {
        "profitFactor" => m.profit_factor,
        "sharpe" => m.sharpe,
        "sortino" => m.sortino,
        "maxDD" => m.max_dd,
        "rdd" => m.rdd,
        "netProfit" => m.net_profit,
        "grossProfit" => m.gross_profit,
        "winRate" => m.win_rate,
        "zScore" => m.z_score,
        "rSquared" => m.r_squared,
        "avgTrade" => m.avg_trade,
        "nTrades" => m.n_trades as f64,
        _ => return None,
    })
}

/// ¿La métrica existe? (para validar la configuración antes de generar)
pub fn metric_exists(key: &str) -> bool {
    METRICAS.iter().any(|(k, _, _)| *k == key)
}

/// Compara un valor con el operador del filtro.
fn compare(value: f64, op: &str, limit: f64) -> bool {
    match op {
        ">=" => value >= limit,
        "<=" => value <= limit,
        ">" => value > limit,
        "<" => value < limit,
        // operador desconocido: no se cumple (fail closed)
        _ => false,
    }
}

/// ¿La estrategia cumple todos los filtros?
pub fn passes(m: &Metrics, filters: &[MetricFilter], min_trades: u32) -> bool {
    if m.n_trades < min_trades {
        return false;
    }
    for f in filters {
        match metric_value(m, &f.metric) {
            Some(v) => {
                if !compare(v, &f.op, f.value) {
                    return false;
                }
            }
            // métrica desconocida: no se cumple (fail closed)
            None => return false,
        }
    }
    true
}

/// Resultado del filtro para una estrategia concreta (para explicar en la UI).
pub fn filter_report(m: &Metrics, filters: &[MetricFilter]) -> Vec<Value> {
    filters
        .iter()
        .map(|f| {
            let v = metric_value(m, &f.metric);
            let ok = v.map(|val| compare(val, &f.op, f.value)).unwrap_or(false);
            json!({
                "metric": f.metric,
                "op": f.op,
                "limit": f.value,
                "value": v,
                "ok": ok,
            })
        })
        .collect()
}

// ── score de robustez ────────────────────────────────────────────────

/// Componentes del score de robustez. Todos van de 0 a 100.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Robustness {
    /// Eficiencia walk-forward (cuánto se conserva fuera de muestra).
    pub wfe: f64,
    /// Supervivencia frente al remuestreo de operaciones (1 - prob. de ruinazo).
    pub mc_survival: f64,
    /// Estabilidad de las permutaciones (SPP): 100 sin fallos, 50 con avisos, 0 con fallos.
    pub spp: f64,
    /// Forma del perfil de optimización: 100 meseta, 60 unimodal, 20 puntual.
    pub perfil: f64,
    /// Score final 0-100.
    pub score: f64,
    /// true si el score incluye SPP y perfil (etapa "avanzada").
    pub avanzado: bool,
    /// true solo cuando todas las comprobaciones estrictas pasan.
    #[serde(default)]
    pub calificado: bool,
    /// Comprobaciones estrictas que no superó la candidata.
    #[serde(default)]
    pub fallos: Vec<String>,
}

impl Default for Robustness {
    fn default() -> Self {
        Self {
            wfe: 0.0, mc_survival: 0.0, spp: 0.0, perfil: 0.0,
            score: 0.0, avanzado: false, calificado: false, fallos: vec![],
        }
    }
}

fn clamp01(v: f64) -> f64 {
    v.max(0.0).min(1.0)
}

/// Score rápido: walk-forward + Monte Carlo. Coste: 2 backtests + un remuestreo.
pub fn quick(bars: &[Bar], tmpl: Template, params: [f64; 3], cfg: &EngineCfg) -> Robustness {
    let wfe = super::ops::wfe_of(bars, tmpl, params, cfg);
    let out = backtest::run(bars, tmpl, params, cfg);
    let rets = metrics::equity_series(&out.trades, cfg.initial_balance, 500);
    let rets = super::ops::returns_of(&rets);
    let mc = super::ops::bootstrap_mc(&rets, 60, 20240);
    let ruin = mc["ruinProb"].as_f64().unwrap_or(1.0);
    let mc_survival = clamp01(1.0 - ruin) * 100.0;
    let wfe_score = wfe.max(0.0).min(100.0);
    let score = 0.5 * wfe_score + 0.5 * mc_survival;
    Robustness {
        wfe: round2(wfe),
        mc_survival: round2(mc_survival),
        spp: 0.0,
        perfil: 0.0,
        score: round2(score),
        avanzado: false,
        calificado: false,
        fallos: vec![],
    }
}

/// Score avanzado: rápido + SPP + perfil de optimización. Más caro: se aplica
/// solo a las finalistas.
pub fn full(bars: &[Bar], tmpl: Template, params: [f64; 3], cfg: &EngineCfg) -> Robustness {
    let mut r = quick(bars, tmpl, params, cfg);
    let core = super::ops::StrategyCore {
        id: 0,
        name: "candidata".into(),
        template: tmpl,
        params,
        bars: bars.to_vec(),
    };
    let spp_result = super::ops::spp(&core, cfg);
    r.spp = match &spp_result {
        Ok(v) => {
            let results = v["results"].as_array().cloned().unwrap_or_default();
            let peor = results
                .iter()
                .filter_map(|x| x["verdict"].as_str())
                .map(|s| match s {
                    "pass" => 100.0,
                    "warn" => 50.0,
                    _ => 0.0,
                })
                .fold(100.0_f64, f64::min);
            peor
        }
        Err(_) => 0.0,
    };
    let train_cut = (bars.len() * 6 / 10).max(60).min(bars.len());
    let profile_core = super::ops::StrategyCore {
        id: core.id,
        name: core.name.clone(),
        template: core.template,
        params: core.params,
        bars: bars[..train_cut].to_vec(),
    };
    let profile_result = super::ops::opt_profile_all(&profile_core, cfg);
    r.perfil = match &profile_result {
        Ok(v) => v["profiles"].as_array().map(|profiles| {
            profiles.iter().map(|p| profile_score(p["shape"].as_str().unwrap_or("puntual")))
                .fold(100.0_f64, f64::min)
        }).unwrap_or(0.0),
        Err(_) => 0.0,
    };

    let tests: Vec<String> = super::verdicts::MC_TESTS.iter().map(|test| test.to_string()).collect();
    let mc_pass = super::ops::mc_suite(&core, cfg, &tests, 60)
        .ok()
        .and_then(|v| v["results"].as_array().cloned())
        .map(|results| {
            results.len() == tests.len() && results.iter().all(|result| result["passed"] == true)
        })
        .unwrap_or(false);
    let spp_pass = spp_result.as_ref().map(|v| {
        v["results"].as_array().map(|results| {
            !results.is_empty() && results.iter().all(|result| result["verdict"] == "pass")
        }).unwrap_or(false)
    }).unwrap_or(false);
    let profile_pass = profile_result.as_ref().map(|v| {
        v["profiles"].as_array().map(|profiles| {
            !profiles.is_empty() && profiles.iter().all(|profile| profile["shape"] == "meseta")
        }).unwrap_or(false)
    }).unwrap_or(false);
    let oos_trades = super::ops::wfe_oos_trade_count(bars, tmpl, params, cfg);
    r.fallos = qualification_failures(r.wfe, oos_trades, mc_pass, spp_pass, profile_pass);
    r.calificado = r.fallos.is_empty();

    r.score = round2(0.35 * r.wfe.max(0.0).min(100.0)
        + 0.30 * r.mc_survival
        + 0.20 * r.spp
        + 0.15 * r.perfil);
    r.avanzado = true;
    r
}

fn profile_score(shape: &str) -> f64 {
    match shape {
        "meseta" => 100.0,
        "unimodal" => 60.0,
        _ => 20.0,
    }
}

fn strictly_qualified(wfe: f64, oos_trades: u32, mc_pass: bool, spp_pass: bool, profile_pass: bool) -> bool {
    qualification_failures(wfe, oos_trades, mc_pass, spp_pass, profile_pass).is_empty()
}

fn qualification_failures(
    wfe: f64,
    oos_trades: u32,
    mc_pass: bool,
    spp_pass: bool,
    profile_pass: bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    if wfe < super::verdicts::WFE_PASS {
        failures.push(format!("WFE inferior al {} %", super::verdicts::WFE_PASS));
    }
    if oos_trades < super::verdicts::MIN_OOS_TRADES {
        failures.push(format!(
            "menos de {} operaciones OOS",
            super::verdicts::MIN_OOS_TRADES
        ));
    }
    if !mc_pass {
        failures.push("al menos una prueba Monte Carlo fallida".into());
    }
    if !spp_pass {
        failures.push("al menos una prueba SPP no aprobada".into());
    }
    if !profile_pass {
        failures.push("algún parámetro no alcanza forma meseta".into());
    }
    failures
}

/// Pesos del score avanzado (se muestran en la interfaz).
pub const PESOS_AVANZADO: [(&str, f64, &str); 4] = [
    ("wfe", 0.35, "Eficiencia walk-forward: cuánto se conserva fuera de muestra"),
    ("mc", 0.30, "Supervivencia frente al remuestreo de operaciones"),
    ("spp", 0.20, "Estabilidad frente a las permutaciones de la regla (SPP)"),
    ("perfil", 0.15, "Forma del perfil: meseta (plano) frente a pico aislado"),
];

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pf: f64, sharpe: f64, dd: f64, trades: u32) -> Metrics {
        Metrics { profit_factor: pf, sharpe, max_dd: dd, n_trades: trades, ..Default::default() }
    }

    #[test]
    fn el_filtro_acepta_cualquier_metrica() {
        let mm = m(1.4, 1.2, 12.0, 100);
        assert!(passes(&mm, &[MetricFilter::new("profitFactor", ">=", 1.3)], 20));
        assert!(!passes(&mm, &[MetricFilter::new("profitFactor", ">=", 1.5)], 20));
        assert!(passes(&mm, &[MetricFilter::new("sharpe", ">=", 1.0)], 20));
        assert!(passes(&mm, &[MetricFilter::new("maxDD", "<=", 15.0)], 20));
        assert!(passes(&mm, &[MetricFilter::new("nTrades", ">=", 50.0)], 20));
    }

    #[test]
    fn todos_los_filtros_deben_cumplirse() {
        let mm = m(1.4, 1.2, 12.0, 100);
        let fs = vec![
            MetricFilter::new("profitFactor", ">=", 1.3),
            MetricFilter::new("maxDD", "<=", 15.0),
        ];
        assert!(passes(&mm, &fs, 20));
        let fs2 = vec![
            MetricFilter::new("profitFactor", ">=", 1.3),
            MetricFilter::new("maxDD", "<=", 5.0),
        ];
        assert!(!passes(&mm, &fs2, 20));
    }

    #[test]
    fn un_operador_o_metrica_desconocida_no_pasa() {
        let mm = m(1.4, 1.2, 12.0, 100);
        assert!(!passes(&mm, &[MetricFilter::new("noExiste", ">=", 0.0)], 20));
        assert!(!passes(&mm, &[MetricFilter::new("sharpe", "~", 1.0)], 20));
        // y por debajo del mínimo de operaciones tampoco pasa
        assert!(!passes(&mm, &[], 200));
    }

    #[test]
    fn el_informe_de_filtros_explica_cada_uno() {
        let mm = m(1.4, 1.2, 12.0, 100);
        let rep = filter_report(&mm, &[MetricFilter::new("profitFactor", ">=", 1.3)]);
        assert_eq!(rep.len(), 1);
        assert_eq!(rep[0]["ok"], true);
        assert_eq!(rep[0]["value"], 1.4);
    }

    #[test]
    fn todas_las_metricas_publicadas_existen() {
        for (key, _, _) in METRICAS {
            assert!(metric_exists(key), "falta {key}");
            assert!(metric_value(&m(1.0, 1.0, 1.0, 10), key).is_some(), "sin valor {key}");
        }
    }

    #[test]
    fn calificacion_estricta_exige_todas_las_validaciones_y_muestra_oos() {
        assert!(strictly_qualified(50.0, 30, true, true, true));
        assert!(!strictly_qualified(49.99, 30, true, true, true));
        assert!(!strictly_qualified(50.0, 29, true, true, true));
        assert!(!strictly_qualified(50.0, 30, false, true, true));
        assert!(!strictly_qualified(50.0, 30, true, false, true));
        assert!(!strictly_qualified(50.0, 30, true, true, false));
    }

    #[test]
    fn explica_cada_criterio_de_robustez_estricta_fallido() {
        let failures = qualification_failures(20.0, 4, false, false, false);
        assert_eq!(failures.len(), 5);
        assert!(failures.iter().any(|failure| failure.contains("WFE")));
        assert!(failures.iter().any(|failure| failure.contains("operaciones OOS")));
        assert!(failures.iter().any(|failure| failure.contains("Monte Carlo")));
        assert!(failures.iter().any(|failure| failure.contains("SPP")));
        assert!(failures.iter().any(|failure| failure.contains("meseta")));
    }
}
