//! engine/ops.rs — robustez y cartera sobre datos reales del núcleo:
//! re-ejecuciones del motor (retest OOS, optimizer/WF matrix, perfil de Pardo,
//! SPP con señales perturbadas), correlación de equities y Monte Carlo
//! bootstrap. Todo PRNG es xorshift con semilla fija → resultados reproducibles.

use rusqlite::{params as q, Connection, OptionalExtension};
use serde_json::{json, Value};

use super::backtest;
use super::datastore::Qfn;
use super::rules::{evaluate, Signal, Template};
use super::types::{BacktestOutput, Bar, EngineCfg, Metrics, Trade};

pub struct StrategyCore {
    pub id: i64,
    pub name: String,
    pub template: Template,
    pub params: [f64; 3],
    pub bars: Vec<Bar>, // vacío si no hay dataset QFN legible para la estrategia
}

impl StrategyCore {
    pub fn has_bars(&self) -> bool { self.bars.len() >= 300 }
}

/// Carga estrategia + barras del dataset QFN asociado a su último backtest_run.
pub fn load_strategy(db: &Connection, id: i64) -> Result<Option<StrategyCore>, String> {
    let row: Option<(String, Option<i64>, Option<String>, Option<String>)> = db
        .query_row(
            "select s.name, s.template_id, s.params,
                    (select df.file_path from backtest_runs b
                       join data_files df on df.id = b.data_file_id
                      where b.strategy_id = s.id order by b.id desc limit 1)
             from strategies s where s.id = ?1",
            q![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some((name, template_id, params_json, df_path)) = row else { return Ok(None) };
    let tmpl = Template::from_id(template_id.unwrap_or(1)).unwrap_or(Template::MaCross);
    let params: [f64; 3] = params_json
        .and_then(|j| serde_json::from_str::<Value>(&j).ok())
        .and_then(|v| v.get("p").and_then(|x| serde_json::from_value::<Vec<f64>>(x.clone()).ok()))
        .and_then(|v| <[f64; 3]>::try_from(v).ok())
        .unwrap_or_else(|| tmpl.default_params());
    let bars = df_path
        .and_then(|p| Qfn::read(std::path::Path::new(&p)).ok())
        .unwrap_or_default();
    Ok(Some(StrategyCore { id, name, template: tmpl, params, bars }))
}

/// Equity almacenada de la última corrida de una estrategia (None si no tiene).
pub fn stored_equity(db: &Connection, strategy_id: i64) -> Option<Vec<f64>> {
    let raw: Option<Option<String>> = db
        .query_row(
            "select equity from backtest_runs where strategy_id = ?1 order by id desc limit 1",
            q![strategy_id],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
        .ok()?;
    let eq_raw: Option<String> = raw.flatten();
    eq_raw
        .and_then(|e| serde_json::from_str::<Vec<f64>>(&e).ok())
        .filter(|e| e.len() > 2)
}

/// Métricas almacenadas de la última corrida.
pub fn stored_metrics(db: &Connection, strategy_id: i64) -> Option<Metrics> {
    let m: Option<Metrics> = db
        .query_row(
            "select net_profit, gross_profit, profit_factor, win_rate, max_dd, rdd,
                    sharpe, sortino, z_score, r_squared, n_trades, avg_trade
             from backtest_runs where strategy_id = ?1 order by id desc limit 1",
            q![strategy_id],
            |r| {
                Ok(Metrics {
                    net_profit: r.get::<_, Option<f64>>(0)?.unwrap_or(0.0),
                    gross_profit: r.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                    profit_factor: r.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    win_rate: r.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                    max_dd: r.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
                    rdd: r.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                    sharpe: r.get::<_, Option<f64>>(6)?.unwrap_or(0.0),
                    sortino: r.get::<_, Option<f64>>(7)?.unwrap_or(0.0),
                    z_score: r.get::<_, Option<f64>>(8)?.unwrap_or(0.0),
                    r_squared: r.get::<_, Option<f64>>(9)?.unwrap_or(0.0),
                    n_trades: r.get::<_, Option<i64>>(10)?.unwrap_or(0) as u32,
                    avg_trade: r.get::<_, Option<f64>>(11)?.unwrap_or(0.0),
                    wfe: 0.0,
                })
            },
        )
        .optional()
        .ok()
        .flatten();
    m.filter(|x| x.n_trades > 0)
}

// ── aritmética de series ────────────────────────────────────────────

pub fn returns_of(eq: &[f64]) -> Vec<f64> {
    eq.windows(2)
        .map(|w| if w[0].abs() > 1e-9 { (w[1] - w[0]) / w[0].abs() } else { 0.0 })
        .collect()
}

pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 3 { return 0.0 }
    let (mut sa, mut sb, mut saa, mut sbb, mut sab) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        sa += a[i]; sb += b[i];
        saa += a[i] * a[i]; sbb += b[i] * b[i]; sab += a[i] * b[i];
    }
    let nf = n as f64;
    let cov = sab / nf - (sa / nf) * (sb / nf);
    let va = saa / nf - (sa / nf) * (sa / nf);
    let vb = sbb / nf - (sb / nf) * (sb / nf);
    if va <= 1e-12 || vb <= 1e-12 { return 0.0 }
    (cov / (va.sqrt() * vb.sqrt())).clamp(-1.0, 1.0)
}

/// Matriz de correlaciones (Pearson sobre retornos de las equities).
pub fn corr_matrix(seqs: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rets: Vec<Vec<f64>> = seqs.iter().map(|e| returns_of(e)).collect();
    let n = rets.len();
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = if i == j { 1.0 } else { pearson(&rets[i], &rets[j]) };
        }
    }
    m
}


/// ── Ponderacion de cartera ────────────────────────────────────────────

/// Modelos de ponderacion disponibles.
pub const MODELS: [&str; 4] = ["equal_weight", "equal_vol", "risk_parity", "custom"];

/// Etiqueta legible de cada modelo.
pub fn model_label(m: &str) -> &'static str {
    match m {
        "equal_weight" => "Equiponderado",
        "equal_vol" => "Inverso a volatilidad",
        "risk_parity" => "Paridad de riesgo",
        "custom" => "Personalizado",
        _ => "Equiponderado",
    }
}

/// Volatilidad de una serie de equity: desviacion tipica de sus retornos.
pub fn volatility(eq: &[f64]) -> f64 {
    let rets = returns_of(eq);
    if rets.len() < 2 { return 0.0 }
    let m = mean(&rets);
    let var = rets.iter().map(|r| (r - m) * (r - m)).sum::<f64>() / (rets.len() as f64 - 1.0);
    var.max(0.0).sqrt()
}

/// Matriz de covarianzas de las series de equity (sobre retornos).
pub fn cov_matrix(seqs: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rets: Vec<Vec<f64>> = seqs.iter().map(|e| returns_of(e)).collect();
    let n = rets.len();
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let (a, b) = (&rets[i], &rets[j]);
            let k = a.len().min(b.len());
            if k < 2 { continue }
            let ma = mean(&a[..k]);
            let mb = mean(&b[..k]);
            let mut s = 0.0;
            for t in 0..k { s += (a[t] - ma) * (b[t] - mb); }
            m[i][j] = s / (k as f64 - 1.0);
        }
    }
    m
}

/// Pesos de cartera segun el modelo.
///
/// - `equal_weight`: 1/N.
/// - `equal_vol`: inverso a la volatilidad (riesgo repartido si no hay correlacion).
/// - `risk_parity`: cada miembro aporta el mismo riesgo, resolviendo las
///   contribuciones de riesgo de forma iterativa.
/// - `custom`: los pesos que pasa el usuario (normalizados).
pub fn weights(model: &str, seqs: &[Vec<f64>], custom: &[f64]) -> Vec<f64> {
    let n = seqs.len();
    if n == 0 { return vec![] }
    match model {
        "custom" if custom.len() == n => {
            let neg = custom.iter().any(|w| *w < 0.0);
            let base: Vec<f64> = if neg { vec![0.0; n] } else { custom.to_vec() };
            normalize(&base)
        }
        "equal_vol" => {
            let vol: Vec<f64> = seqs.iter().map(|s| volatility(s)).collect();
            let inv: Vec<f64> = vol.iter().map(|v| if *v > 1e-12 { 1.0 / v } else { 0.0 }).collect();
            let total: f64 = inv.iter().sum();
            if total <= 0.0 { vec![1.0 / n as f64; n] } else { inv.iter().map(|w| w / total).collect() }
        }
        "risk_parity" => risk_parity(seqs),
        _ => vec![1.0 / n as f64; n],
    }
}

fn normalize(w: &[f64]) -> Vec<f64> {
    let total: f64 = w.iter().sum();
    if total <= 0.0 {
        let n = w.len().max(1) as f64;
        return vec![1.0 / n; w.len()];
    }
    w.iter().map(|x| x / total).collect()
}

/// Paridad de riesgo: pesos tales que cada miembro aporta el mismo riesgo.
///
/// Se resuelve con el punto fijo clasico: en cada paso se escala cada peso en
/// funcion de como se aleja su contribucion del objetivo.
fn risk_parity(seqs: &[Vec<f64>]) -> Vec<f64> {
    let n = seqs.len();
    if n == 0 { return vec![] }
    if n == 1 { return vec![1.0]; }
    let cov = cov_matrix(seqs);
    let mut w = vec![1.0 / n as f64; n];
    for _ in 0..2000 {
        // varianza de cartera
        let mut var_p = 0.0;
        for i in 0..n { for j in 0..n { var_p += w[i] * cov[i][j] * w[j]; } }
        let sigma_p = var_p.max(0.0).sqrt();
        if sigma_p <= 1e-12 { break; }
        let target = sigma_p / n as f64;
        let mut w_new = w.clone();
        for i in 0..n {
            let mut c = 0.0;
            for j in 0..n { c += cov[i][j] * w[j]; }
            let rc = w[i] * c;              // contribucion al riesgo de i
            if rc.abs() > 1e-12 {
                w_new[i] = w[i] * (target / rc);
            }
        }
        let w_new = normalize(&w_new);
        let cambio: f64 = w_new.iter().zip(w.iter()).map(|(a, b)| (a - b).abs()).sum();
        w = w_new;
        if cambio < 1e-10 { break; }
    }
    w
}

/// Equity combinada con los pesos dados (se alinea al minimo comun de longitud).
pub fn weighted_equity(seqs: &[Vec<f64>], w: &[f64]) -> Vec<f64> {
    if seqs.is_empty() { return vec![] }
    let n = seqs.iter().map(|s| s.len()).min().unwrap_or(0);
    if n == 0 { return vec![] }
    (0..n)
        .map(|i| seqs.iter().enumerate().map(|(k, s)| s[i] * w.get(k).copied().unwrap_or(0.0)).sum())
        .collect()
}

/// Contribucion al riesgo de cada miembro (en %), con los pesos dados.
pub fn risk_contributions(seqs: &[Vec<f64>], w: &[f64]) -> Vec<f64> {
    let n = seqs.len();
    if n == 0 { return vec![] }
    let cov = cov_matrix(seqs);
    let mut var_p = 0.0;
    for i in 0..n { for j in 0..n { var_p += w[i] * cov[i][j] * w[j]; } }
    let sigma_p = var_p.max(0.0).sqrt();
    if sigma_p <= 1e-12 { return vec![0.0; n]; }
    let mut rc = vec![0.0; n];
    for i in 0..n {
        let mut c = 0.0;
        for j in 0..n { c += cov[i][j] * w[j]; }
        rc[i] = w[i] * c / sigma_p;
    }
    let total: f64 = rc.iter().sum();
    if total.abs() < 1e-12 { return vec![0.0; n]; }
    rc.iter().map(|r| r / total * 100.0).collect()
}

/// Equity combinada = media por índice (alineada al mínimo común de longitud).
pub fn mean_equity(seqs: &[Vec<f64>]) -> Vec<f64> {
    if seqs.is_empty() { return vec![] }
    let n = seqs.iter().map(|s| s.len()).min().unwrap_or(0);
    if n == 0 { return vec![] }
    (0..n)
        .map(|i| seqs.iter().map(|s| s[i]).sum::<f64>() / seqs.len() as f64)
        .collect()
}

/// Máximo drawdown en % del peak.
pub fn max_dd_pct(eq: &[f64]) -> f64 {
    let mut peak = f64::MIN;
    let mut dd = 0.0f64;
    for v in eq {
        if *v > peak { peak = *v }
        if peak > 1e-9 {
            dd = dd.max((peak - v) / peak * 100.0);
        }
    }
    dd
}

pub fn mean(v: &[f64]) -> f64 { if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 } }

pub fn std(v: &[f64]) -> f64 {
    if v.len() < 2 { return 0.0 }
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

pub fn sharpe(rets: &[f64]) -> f64 {
    let s = std(rets);
    if s <= 1e-9 { return 0.0 }
    (mean(rets) / s) * (252.0f64).sqrt()
}

pub fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0 }
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

// ── PRNG determinista (xorshift64*, sembrado) ───────────────────────

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed }) }
    pub fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let x = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (x >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range_i(&mut self, hi: usize) -> usize { (self.next() * hi as f64) as usize % hi.max(1) }
}

/// Monte Carlo bootstrap sobre retornos diarios: 240 pasos resampleados con
/// reemplazo. Ruina = toque de -30% desde el peak en cualquier momento.
pub fn bootstrap_mc(rets: &[f64], trials: usize, seed: u64) -> Value {
    if rets.is_empty() {
        return json!({ "trials": 0, "p5": 0.0, "p50": 0.0, "p95": 0.0, "ruinProb": 0.0 });
    }
    let mut rng = Rng::new(seed);
    let mut finals: Vec<f64> = Vec::with_capacity(trials);
    let mut ruin = 0u32;
    for _ in 0..trials {
        let mut bal = 10_000.0;
        let mut peak = bal;
        let mut busted = false;
        for _ in 0..240 {
            bal *= 1.0 + rets[rng.range_i(rets.len())];
            if bal < peak * 0.70 { busted = true }
            peak = peak.max(bal);
        }
        if busted { ruin += 1 }
        finals.push(bal);
    }
    finals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let t = trials.max(1) as f64;
    json!({
        "trials": trials,
        "p5": round2(pct(&finals, 0.05)),
        "p50": round2(pct(&finals, 0.5)),
        "p95": round2(pct(&finals, 0.95)),
        "ruinProb": round3(ruin as f64 / t),
    })
}

// ── grids de optimización ───────────────────────────────────────────

/// Eje de un parámetro: n valores lineales del 40% al 220% del valor actual.
pub fn axis(base: f64, n: usize) -> Vec<f64> {
    let n = n.max(3).min(9);
    let denom = (n - 1) as f64;
    (0..n)
        .map(|i| round1((base * (0.4 + 1.8 * i as f64 / denom)).max(1.0)))
        .collect()
}

/// Walk-forward simple: train en el 60% inicial, eval OOS en el 40% final.
/// WFE = 100 × PF(OOS) / PF(IS), con tope 150%.
pub fn wfe_of(bars: &[Bar], tmpl: Template, params: [f64; 3], cfg: &EngineCfg) -> f64 {
    let cut = (bars.len() * 6 / 10).max(60);
    if bars.len().saturating_sub(cut) < 60 { return 0.0 }
    let is = backtest::run(&bars[..cut], tmpl, params, cfg);
    let oos = backtest::run(&bars[cut..], tmpl, params, cfg);
    if is.metrics.profit_factor <= 0.01 { return 0.0 }
    (100.0 * oos.metrics.profit_factor / is.metrics.profit_factor).min(150.0)
}

/// Grid p1 × p2 con PF/maxDD (optimizer) o WFE + veredicto (WF matrix).
pub fn run_grid(
    core: &StrategyCore,
    cfg: &EngineCfg,
    steps: usize,
    kind: &str, // "simple" | "wf"
) -> Result<Value, String> {
    if !core.has_bars() {
        return Err(format!("Estrategia {} sin dataset QFN cargable — importa el CSV primero", core.id));
    }
    let n = steps.max(3).min(9);
    let a_vals = axis(core.params[0], n);
    let b_vals = axis(core.params[1], n);
    let mut cells: Vec<Value> = Vec::new();
    let mut best: Option<Value> = None;
    let mut best_score = f64::MIN;
    for a in &a_vals {
        for b in &b_vals {
            let p = [round1(*a).max(2.0), round1(*b).max(2.0), core.params[2]];
            let out = backtest::run(&core.bars, core.template, p, cfg);
            let (cell, score) = if kind == "wf" {
                let wfe = round1(wfe_of(&core.bars, core.template, p, cfg));
                let s = wfe_of(&core.bars, core.template, p, cfg) - out.metrics.max_dd * 0.3;
                let verdict = if wfe >= 50.0 { "pass" } else if wfe >= 35.0 { "warn" } else { "fail" };
                (json!({ "a": a, "b": b, "wfe": wfe, "verdict": verdict }), s)
            } else {
                (json!({ "a": a, "b": b, "pf": round2(out.metrics.profit_factor), "maxDD": round1(out.metrics.max_dd) }),
                 out.metrics.profit_factor)
            };
            if score > best_score {
                best_score = score;
                best = Some(match kind {
                    "wf" => cell.clone(), // el mejor es el propio cell (wfe + verdict)
                    _ => json!({ "a": a, "b": b, "pf": round2(out.metrics.profit_factor), "maxDD": round1(out.metrics.max_dd) }),
                });
            }
            cells.push(cell);
        }
    }
    Ok(json!({ "aValues": a_vals, "bValues": b_vals, "cells": cells, "best": best }))
}

/// Perfil de optimización estilo Pardo: barrido de p1 con net por valor y
/// clasificación de la forma (meseta / unimodal / puntual).
pub fn opt_profile(core: &StrategyCore, cfg: &EngineCfg) -> Result<Value, String> {
    if !core.has_bars() {
        return Err(format!("Estrategia {} sin dataset QFN cargable — importa el CSV primero", core.id));
    }
    let n = 12;
    let base = core.params[0].max(4.0);
    let vals: Vec<f64> = (0..n)
        .map(|i| round1(base * (0.3 + 2.2 * i as f64 / (n - 1) as f64)).max(2.0))
        .collect();
    let mut points: Vec<Value> = Vec::new();
    let mut nets: Vec<f64> = Vec::with_capacity(n);
    for v in &vals {
        let out = backtest::run(&core.bars, core.template, [round1(*v), core.params[1], core.params[2]], cfg);
        let net = round2(out.metrics.net_profit);
        nets.push(net);
        points.push(json!({ "param": v, "netProfit": net }));
    }
    let ib = nets.iter().cloned().enumerate()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|x| x.0).unwrap_or(0);
    let best_net = nets[ib].max(1.0);
    let ok: Vec<f64> = vals.iter().zip(nets.iter())
        .filter(|(_, nt)| **nt >= best_net * 0.8)
        .map(|(v, _)| *v).collect();
    let (stable_from, stable_to) = if ok.is_empty() {
        (vals[0], vals[0])
    } else {
        (ok.iter().cloned().fold(f64::MAX, f64::min), ok.iter().cloned().fold(f64::MIN, f64::max))
    };
    // forma: anchura de la franja estable respecto del rango total del barrido
    let width = stable_to - stable_from;
    let span = vals.last().copied().unwrap_or(1.0) - vals[0];
    let shape = if span <= 0.0 { "puntual" }
        else if width >= span * 0.5 { "meseta" }
        else if width >= span * 0.2 { "unimodal" }
        else { "puntual" };
    Ok(json!({
        "param": format!("p1 · {}", core.template.param_names()[0]),
        "points": points,
        "stableFrom": stable_from, "stableTo": stable_to,
        "shape": shape, "bestParam": vals[ib],
    }))
}


/// ── Editor sin codigo: guardar la regla y recalcular la corrida ──────────
///
/// Persiste la regla editada (plantilla + parametros + logico) y vuelve a correr
/// el backtest sobre el dataset de la estrategia, guardando la corrida nueva con
/// sus metricas, su equity y su corte out-of-sample. Si la regla no produce
/// operaciones, no se toca nada: se devuelve el error y la estrategia sigue como
/// estaba.
pub fn save_logic(
    db: &Connection,
    dataset_path: &std::path::Path,
    data_file_id: i64,
    cfg: &EngineCfg,
    strategy_id: i64,
    template: Template,
    p: [f64; 3],
    logic: &serde_json::Value,
) -> Result<Value, String> {
    let bars = crate::engine::datastore::Qfn::read(dataset_path)
        .map_err(|e| format!("no se pudo leer el dataset: {e}"))?;
    if bars.len() < 200 {
        return Err(format!("dataset demasiado pequeno ({} barras)", bars.len()));
    }

    // correr la regla nueva ANTES de persistir
    let out = backtest::run(&bars, template, p, cfg);
    if out.trades.is_empty() {
        return Err("esta regla no genera operaciones en este dataset: revisa los parametros".into());
    }
    let oos_frac = oos_frac_of(&bars, &out.trades);

    db.execute(
        "update strategies set template_id = ?1, params = ?2, logic = ?3, status = 'active'
         where id = ?4",
        rusqlite::params![
            template_id_of(template),
            serde_json::json!({ "p": p }).to_string(),
            logic.to_string(),
            strategy_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    db.execute(
        "insert into backtest_runs (strategy_id, data_file_id, engine_cfg, build_id,
           net_profit, gross_profit, profit_factor, win_rate, max_dd, rdd, sharpe,
           sortino, z_score, r_squared, n_trades, avg_trade, wfe, equity, oos_frac, status)
         values (?1, ?2, ?3, null,
           ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 'done')",
        rusqlite::params![
            strategy_id,
            data_file_id,
            serde_json::to_string(cfg).unwrap_or_default(),
            out.metrics.net_profit, out.metrics.gross_profit, out.metrics.profit_factor,
            out.metrics.win_rate, out.metrics.max_dd, out.metrics.rdd, out.metrics.sharpe,
            out.metrics.sortino, out.metrics.z_score, out.metrics.r_squared,
            out.metrics.n_trades as i64, out.metrics.avg_trade, 0.0,
            serde_json::to_string(&out.equity).unwrap_or_default(),
            oos_frac,
        ],
    )
    .map_err(|e| e.to_string())?;

    let name: String = db
        .query_row("select name from strategies where id = ?1", rusqlite::params![strategy_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "strategyId": strategy_id,
        "nombre": name,
        "plantilla": template.label(),
        "params": { "p": p },
        "metrics": metrics_json(&out.metrics),
        "equity": out.equity,
        "oosFrac": oos_frac,
        "nTrades": out.metrics.n_trades,
    }))
}

/// id de plantilla en la DB (mismo mapeo que `Template::from_id`).
fn template_id_of(t: Template) -> i64 {
    match t {
        Template::MaCross => 1,
        Template::RsiReversion => 2,
        Template::AtrBreakout => 3,
    }
}

/// Fraccion de la serie de equity que corresponde a la zona out-of-sample.
///
/// La serie de equity tiene un punto por operacion (mas el saldo inicial) y se
/// re-muestrea; el corte walk-forward se define sobre las BARRAS, asi que se
/// cuenta cuantas operaciones se abren antes del corte y se proyecta ese punto
/// sobre la serie. Es el mismo corte que usa `ops::wfe_of` (60 % in-sample).
pub fn oos_frac_of(bars: &[Bar], trades: &[Trade]) -> f64 {
    let cut = (bars.len() * 6 / 10).max(60);
    let cut_time = bars.get(cut).map(|b| b.time).unwrap_or(i64::MAX);
    if trades.is_empty() {
        return 1.0 - crate::engine::verdicts::IS_SPLIT;
    }
    let before = trades.iter().filter(|t| t.open_time < cut_time).count();
    let total = trades.len() as f64 + 1.0; // + saldo inicial
    (before as f64 / total).clamp(0.0, 1.0)
}


/// Retesteo en varios periodos out-of-sample.
///
/// El dataset se parte en 70 % in-sample + 30 % out-of-sample (igual que el
/// retesteo de una sola ventana) y ese 30 % se divide a su vez en `k` ventanas
/// consecutivas, cada una evaluada por separado. Asi se ve si la estrategia
/// aguanta de forma *consistente* o solo por suerte en un tramo.
///
/// `k = 1` reproduce exactamente el retesteo de siempre.
pub fn retest_periods(
    bars: &[Bar],
    tmpl: Template,
    params: [f64; 3],
    cfg: &EngineCfg,
    k: usize,
) -> Result<Value, String> {
    let cut = (bars.len() * 7 / 10).max(100);
    let oos = &bars[cut..];
    if oos.len() < 60 {
        return Err("dataset demasiado corto para separar OOS".into());
    }
    let k = k.clamp(1, 12);
    let ventana = oos.len() / k;
    if ventana < 20 {
        return Err(format!(
            "el dataset no da para {k} periodos OOS (cada uno necesita 20 barras)"
        ));
    }
    let mut periodos: Vec<Value> = Vec::new();
    let mut pass = 0usize;
    let mut peor_pf = f64::MAX;
    for i in 0..k {
        let ini = i * ventana;
        let fin = if i == k - 1 { oos.len() } else { ini + ventana };
        let trozo = &oos[ini..fin];
        let out = backtest::run(trozo, tmpl, params, cfg);
        let m = &out.metrics;
        let (verdict, nota) = if m.n_trades == 0 {
            ("fail", "la regla no se activa en este periodo".to_string())
        } else if m.profit_factor >= 1.15 && m.net_profit > 0.0 {
            ("pass", String::new())
        } else if m.profit_factor >= 0.95 {
            ("warn", "apenas positivo".to_string())
        } else {
            ("fail", "pierde dinero aqui".to_string())
        };
        if verdict == "pass" { pass += 1; }
        peor_pf = peor_pf.min(m.profit_factor);
        periodos.push(json!({
            "indice": i + 1,
            "desde": trozo.first().map(|b| b.time).unwrap_or(0),
            "hasta": trozo.last().map(|b| b.time).unwrap_or(0),
            "barras": trozo.len(),
            "verdict": verdict,
            "nota": nota,
            "metrics": metrics_json(m),
        }));
    }
    let (verdict, detalle) = if pass == k {
        ("pass", format!("rentable en los {k} periodos OOS"))
    } else if pass > 0 {
        ("warn", format!("rentable solo en {pass} de {k} periodos OOS"))
    } else {
        ("fail", format!("no rentable en ninguno de los {k} periodos OOS"))
    };
    Ok(json!({
        "periodos": periodos,
        "consistencia": {
            "verdict": verdict,
            "detalle": detalle,
            "pasados": pass,
            "total": k,
            "peorPF": if peor_pf.is_finite() { (peor_pf * 100.0).round() / 100.0 } else { 0.0 },
        },
    }))
}

// ── retest OOS ──────────────────────────────────────────────────────


pub fn metrics_json(m: &Metrics) -> Value {
    json!({
        "netProfit": round2(m.net_profit), "grossProfit": round2(m.gross_profit),
        "profitFactor": round2(m.profit_factor), "winRate": round2(m.win_rate),
        "maxDD": round2(m.max_dd), "rdd": round1(m.rdd),
        "sharpe": round2(m.sharpe), "sortino": round2(m.sortino),
        "zScore": round2(m.z_score), "rSquared": round3(m.r_squared),
        "nTrades": m.n_trades, "avgTrade": round2(m.avg_trade), "wfe": round1(m.wfe),
    })
}

// ── Monte Carlo (suite de tests sobre trades recomputados) ──────────

/// Genera una variación de la lista de trades según el tipo de test.
/// Cada test usa su propia semilla → determinista.
pub fn mc_perturb(base: &[Trade], test: &str, seed: u64) -> Vec<Trade> {
    let mut rng = Rng::new(seed);
    let mut out: Vec<Trade> = base.to_vec();
    match test {
        "shuffle" => {
            // Fisher–Yates con el PRNG sembrado
            for i in (1..out.len()).rev() {
                let j = rng.range_i(i + 1);
                out.swap(i, j);
            }
        }
        "skip" | "skip_certain" => {
            out.retain(|_| rng.next() > 0.10);
        }
        "duplicate" => {
            let picked: Vec<Trade> = out.iter().filter(|_| rng.next() > 0.88).cloned().collect();
            out.extend(picked);
        }
        "slippage" => {
            // +1 pip de coste extra por trade (≈ $5 con 0.1 lots en EURUSD)
            for t in &mut out {
                t.profit -= 5.0;
            }
        }
        _ => {
            // mapeo V1 de los tests de reconstrucción: jitter de P&L ±8%
            for t in &mut out {
                let j = 1.0 + (rng.next() - 0.5) * 0.16;
                t.profit *= j;
            }
        }
    }
    out
}

/// Suite Monte Carlo: para cada test elegido, n trials + percentiles del net
/// y passRate. V1 ejecuta las perturbaciones sobre los trades del motor
/// (recomputados del dataset real); "drawdown" reporta percentiles de maxDD.
pub fn mc_suite(core: &StrategyCore, cfg: &EngineCfg, tests: &[String], trials: usize) -> Result<Value, String> {
    if !core.has_bars() {
        return Err(format!("Estrategia {} sin dataset cargable", core.name));
    }
    let base_out = backtest::run(&core.bars, core.template, core.params, cfg);
    if base_out.trades.is_empty() {
        return Err("la estrategia no tiene trades que perturbar en este dataset".into());
    }
    let mut results: Vec<Value> = Vec::new();
    for t in tests {
        let mut nets: Vec<f64> = Vec::with_capacity(trials);
        // randomize_params es la única que re-ejecuta señales (jitter ±10% en p1/p2)
        if t == "randomize_params" {
            for k in 0..trials {
                let mut rr = Rng::new(9000 + k as u64);
                let p = [
                    round2(core.params[0] * (1.0 + (rr.next() - 0.5) * 0.2)).max(2.0),
                    round2(core.params[1] * (1.0 + (rr.next() - 0.5) * 0.2)).max(2.0),
                    core.params[2],
                ];
                let o = backtest::run(&core.bars, core.template, p, cfg);
                nets.push(o.metrics.net_profit);
            }
            results.push(mc_value(t, nets));
        } else if t == "drawdown" {
            // distribución de maxDD al reordenar: pass = p95 del DD no empeora >30%
            for k in 0..trials {
                let pert = mc_perturb(&base_out.trades, t, 1000 + k as u64);
                let eq = super::metrics::equity_series(&pert, cfg.initial_balance, 500);
                nets.push(max_dd_pct(&eq));
            }
            results.push(mc_dd_value(t, nets, base_out.metrics.max_dd));
        } else {
            for k in 0..trials {
                let pert = mc_perturb(&base_out.trades, t, 1000 + k as u64);
                let eq = super::metrics::equity_series(&pert, cfg.initial_balance, 500);
                let m = super::metrics::compute(&pert, &eq, cfg.initial_balance);
                nets.push(m.net_profit);
            }
            results.push(mc_value(t, nets));
        }
    }
    Ok(json!({ "results": results }))
}

fn mc_value(test: &str, nets: Vec<f64>) -> Value {
    let mut s = nets.clone();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pass_rate = nets.iter().filter(|n| **n > 0.0).count() as f64 / nets.len().max(1) as f64;
    json!({
        "test": test, "passed": pass_rate >= 0.75,
        "passRate": round2(pass_rate * 100.0) / 100.0,
        "percentiles": { "p5": round2(pct(&s, 0.05)), "p50": round2(pct(&s, 0.5)), "p95": round2(pct(&s, 0.95)) },
    })
}

fn mc_dd_value(test: &str, dds: Vec<f64>, base_dd: f64) -> Value {
    let mut s = dds.clone();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95 = pct(&s, 0.95);
    let ok = p95 <= base_dd * 1.3 + 5.0;
    let pass_rate = dds.iter().filter(|d| **d <= base_dd * 1.3 + 5.0).count() as f64 / dds.len().max(1) as f64;
    json!({
        "test": test, "passed": ok, "passRate": round2(pass_rate * 100.0) / 100.0,
        "percentiles": { "p5": round2(pct(&s, 0.05)), "p50": round2(pct(&s, 0.5)), "p95": round2(p95) },
    })
}

// ── SPP: señales perturbadas re-ejecutadas en el motor ──────────────

/// Seis variantes de robustez sobre las señales de la estrategia. Todas
/// ejecutan el MISMO bucle de cartera (backtest::run_signals) — sin azar,
/// salvo la de entradas aleatorias (sembrada).
pub fn spp(core: &StrategyCore, cfg: &EngineCfg) -> Result<Value, String> {
    if !core.has_bars() {
        return Err(format!("Estrategia {} sin dataset cargable", core.name));
    }
    let base_sig = evaluate(core.template, core.params, &core.bars, cfg);
    let atr_vals = super::indicators::atr(&core.bars, 14);
    let base_out = backtest::run_signals(&core.bars, &base_sig, &atr_vals, cfg);
    let net0 = base_out.metrics.net_profit;

    // variantes de señales; None = se calcula por otra vía (post-proceso o re-eval)
    let variants: Vec<(&str, &str, Option<Vec<Signal>>)> = vec![
        ("entry_reversal", "Entradas revertidas", Some(base_sig.iter().map(|s| match s {
            Signal::EnterLong => Signal::None,
            Signal::ExitLong => Signal::EnterLong,
            Signal::None => Signal::None,
        }).collect())),
        ("exit_reversal", "Salidas revertidas", Some(base_sig.iter().map(|s| match s {
            Signal::EnterLong => Signal::ExitLong, // también sale en la próxima señal de entrada
            s => *s,
        }).collect())),
        ("trade_split", "Operaciones divididas", None),
        ("opposite", "Señal opuesta", Some(base_sig.iter().map(|s| match s {
            Signal::EnterLong => Signal::ExitLong,
            Signal::ExitLong => Signal::EnterLong,
            Signal::None => Signal::None,
        }).collect())),
        ("param_permute", "Permutación de parámetros", None),
        ("random_entry", "Entradas aleatorias", None),
    ];

    let mut results = Vec::with_capacity(variants.len());
    for (key, label, sigs) in &variants {
        let net = match (*key, sigs) {
            ("trade_split", _) => {
                // dividir cada trade en dos mitades (mismo valor repartido)
                let mut two: Vec<Trade> = Vec::with_capacity(base_out.trades.len() * 2);
                for t in &base_out.trades {
                    for _ in 0..2 {
                        two.push(Trade { direction: t.direction, open_time: t.open_time, open_price: t.open_price, close_time: t.close_time, close_price: t.close_price, profit: t.profit / 2.0, bars: t.bars });
                    }
                }
                let eq = super::metrics::equity_series(&two, cfg.initial_balance, 500);
                super::metrics::compute(&two, &eq, cfg.initial_balance).net_profit
            }
            ("param_permute", _) => {
                let p = [core.params[1], core.params[0], core.params[2]];
                let sigs2 = evaluate(core.template, p, &core.bars, cfg);
                backtest::run_signals(&core.bars, &sigs2, &atr_vals, cfg).metrics.net_profit
            }
            ("random_entry", _) => {
                let mut rr = Rng::new(31337);
                let sigs3: Vec<Signal> = base_sig.iter().map(|s| match s {
                    Signal::None if rr.next() < 0.04 => Signal::EnterLong,
                    s => *s,
                }).collect();
                backtest::run_signals(&core.bars, &sigs3, &atr_vals, cfg).metrics.net_profit
            }
            (_, Some(sigs_custom)) => {
                backtest::run_signals(&core.bars, sigs_custom, &atr_vals, cfg).metrics.net_profit
            }
            (_, None) => net0,
        };
        let impact = (100.0 * (net - net0).abs() / net0.abs().max(1.0)).min(999.0);
        let verdict = if impact < 10.0 { "pass" } else if impact < 25.0 { "warn" } else { "fail" };
        results.push(json!({ "key": key, "label": label, "impactPct": round1(impact), "verdict": verdict }));
    }
    let overall = if results.iter().all(|r| r["verdict"] != "fail") { "pass" } else { "fail" };
    Ok(json!({ "results": results, "overall": overall }))
}


/// Retesteo de la MISMA regla en OTRO mercado.
///
/// `bars` son los datos del mercado de destino (otro simbolo y/o temporalidad).
/// `None` en `verdict` no se usa: si el mercado no tiene datos, quien llama lo
/// marca como "sin datos" — que no es lo mismo que un fallo.
pub fn retest_on_market(
    bars: &[Bar],
    tmpl: Template,
    params: [f64; 3],
    cfg: &EngineCfg,
) -> Result<Value, String> {
    if bars.len() < 200 {
        return Err(format!("dataset demasiado corto ({} barras)", bars.len()));
    }
    let out = backtest::run(bars, tmpl, params, cfg);
    let m = &out.metrics;
    let (verdict, nota) = if m.n_trades == 0 {
        ("fail", "la regla no se activa en este mercado".to_string())
    } else if m.profit_factor >= 1.15 && m.net_profit > 0.0 {
        ("pass", String::new())
    } else if m.profit_factor >= 0.95 {
        ("warn", "apenas positivo".to_string())
    } else {
        ("fail", "pierde dinero aqui".to_string())
    };
    Ok(json!({
        "retest": metrics_json(m),
        "verdict": verdict,
        "nota": nota,
        "nBars": bars.len(),
    }))
}

/// Veredicto global de una estrategia en varios mercados.
/// `mercados` = [(nombre, veredicto)] con "sin datos" excluido del computo.
pub fn verdict_mercados(mercados: &[(String, String)]) -> Value {
    let con_datos: Vec<&(String, String)> = mercados.iter().filter(|(_, v)| v != "sin datos").collect();
    let pass = con_datos.iter().filter(|(_, v)| v.as_str() == "pass").count();
    let fail = con_datos.iter().filter(|(_, v)| v.as_str() == "fail").count();
    let warn = con_datos.iter().filter(|(_, v)| v.as_str() == "warn").count();
    let (verdict, detalle) = if con_datos.is_empty() {
        ("pending", "ningun mercado con datos".to_string())
    } else if fail == 0 && warn == 0 {
        ("pass", format!("sobrevive en {pass} de {}", con_datos.len()))
    } else if pass > 0 {
        ("warn", format!("sobrevive en {pass} de {}", con_datos.len()))
    } else {
        ("fail", format!("falla en {} de {}", fail + warn, con_datos.len()))
    };
    json!({
        "verdict": verdict,
        "detalle": detalle,
        "mercadosConDatos": con_datos.len(),
        "pass": pass,
        "warn": warn,
        "fail": fail,
    })
}

// ── improver: variantes reales de cfg/params evaluadas ──────────────

/// Variantes de mejora con re-ejecución real del motor: cambios de stop/target
/// y ajuste fino de parámetros (V1; la construcción de reglas nuevas llega en
/// una fase posterior).
pub fn improver(core: &StrategyCore, cfg: &EngineCfg) -> Result<Value, String> {
    if !core.has_bars() {
        return Err(format!("Estrategia {} sin dataset cargable", core.name));
    }
    let base = backtest::run(&core.bars, core.template, core.params, cfg);
    let mk = |name: &str, out: &BacktestOutput| -> Value {
        json!({
            "name": name,
            "metrics": metrics_json(&out.metrics),
            "improvement": round2(out.metrics.net_profit - base.metrics.net_profit),
        })
    };
    let mut c1 = cfg.clone(); c1.sl_atr_mult = 1.5;
    let mut c2 = cfg.clone(); c2.tp_atr_mult = 5.0;
    let mut c3 = cfg.clone(); c3.tp_atr_mult = 0.0; // solo stop
    let p1 = core.params[0] * 1.3;
    let run1 = backtest::run(&core.bars, core.template, core.params, &c1);
    let run2 = backtest::run(&core.bars, core.template, core.params, &c2);
    let run3 = backtest::run(&core.bars, core.template, core.params, &c3);
    let run4 = backtest::run(&core.bars, core.template, [p1, core.params[1], core.params[2]], cfg);
    let variants = vec![
        mk("Stop más ajustado (SL ×1.5 ATR)", &run1),
        mk("Objetivo ampliado (TP ×5 ATR)", &run2),
        mk("Solo stop, sin objetivo", &run3),
        mk("Periodo de entrada ×1.3", &run4),
    ];
    let best = variants.iter()
        .filter(|v| v["improvement"].as_f64().unwrap_or(f64::MIN) > 0.5)
        .max_by(|a, b| a["improvement"].as_f64().unwrap_or(-1e9)
            .partial_cmp(&b["improvement"].as_f64().unwrap_or(-1e9))
            .unwrap_or(std::cmp::Ordering::Equal));
    Ok(json!({
        "original": metrics_json(&base.metrics),
        "variants": variants,
        "bestVariant": best.and_then(|b| b["name"].as_str()).map(|s| s.to_string()),
    }))
}

// ── helpers numéricos ───────────────────────────────────────────────

fn round1(v: f64) -> f64 { (v * 10.0).round() / 10.0 }
fn round2(v: f64) -> f64 { (v * 100.0).round() / 100.0 }
fn round3(v: f64) -> f64 { (v * 1000.0).round() / 1000.0 }

// ── tests ───────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;



    /// El retesteo multi-periodo divide el OOS en ventanas y mide consistencia.
    #[test]
    fn el_retesteo_multi_periodo_mide_la_consistencia() {
        let cfg = EngineCfg::default();
        let barras = serie_con_ciclos(1200);
        let r = retest_periods(&barras, Template::MaCross, [20.0, 50.0, 0.0], &cfg, 3)
            .expect("el retesteo multi-periodo debe funcionar");
        let periodos = r["periodos"].as_array().unwrap();
        assert_eq!(periodos.len(), 3);
        for (i, p) in periodos.iter().enumerate() {
            assert_eq!(p["indice"], (i + 1) as i64);
            assert!(p["verdict"].is_string());
            assert!(p["metrics"]["profitFactor"].is_number());
        }
        let c = &r["consistencia"];
        assert_eq!(c["total"], 3);
        assert!(c["pasados"].as_u64().unwrap() <= 3);
        assert!(c["detalle"].as_str().unwrap().contains("periodos OOS"));
    }

    /// Con una sola ventana se reproduce el retesteo clasico.
    #[test]
    fn una_sola_ventana_da_un_unico_periodo() {
        let cfg = EngineCfg::default();
        let barras = serie_con_ciclos(1200);
        let r = retest_periods(&barras, Template::MaCross, [20.0, 50.0, 0.0], &cfg, 1).unwrap();
        assert_eq!(r["periodos"].as_array().unwrap().len(), 1);
        assert_eq!(r["consistencia"]["total"], 1);
    }

    /// Serie con ciclos claros: hace que el cruce de medias genere operaciones.
    fn serie_con_ciclos(n: usize) -> Vec<Bar> {
        (0..n)
            .map(|i| {
                let t = i as f64;
                // ciclo de 120 barras: sube 60, baja 60
                let fase = (t % 120.0) / 120.0;
                let base = 1.10 + if fase < 0.5 { fase * 0.06 } else { (1.0 - fase) * 0.06 };
                Bar {
                    time: 1_700_000_000 + i as i64 * 60,
                    open: base,
                    high: base * 1.002,
                    low: base * 0.998,
                    close: base * 1.0005,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    /// Guardar una regla debe: persistir la regla, recalcular la corrida y
    /// dejar una nueva fila en backtest_runs con las metricas nuevas.
    #[test]
    fn guardar_regla_persiste_y_recalcula_la_corrida() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();
        // la plantilla que se guarda referencia strategy_templates
        conn.execute(
            "insert into strategy_templates (id, name, source, logic)
             values (1, 'Cruce de medias', 'built_in', '{}')",
            [],
        )
        .unwrap();

        // dataset QFN temporal
        let path = std::env::temp_dir().join(format!("qf_save_test_{}.qfn", std::process::id()));
        let bars = serie_con_ciclos(600);
        crate::engine::datastore::Qfn::write(&path, &bars).unwrap();

        conn.execute(
            "insert into data_files (symbol_id, timeframe, storage_kind, file_path, n_bars, status)
             values (?1, 'm1', 'local_qfd', ?2, ?3, 'ready')",
            rusqlite::params![sym, path.to_string_lossy().to_string(), bars.len() as i64],
        )
        .unwrap();
        let df = conn.last_insert_rowid();

        conn.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, params, status)
             values ('S1', ?1, 'm1', 'builder', '{}', '{\"p\":[20,50,0]}', 'active')",
            rusqlite::params![sym],
        )
        .unwrap();
        let sid = conn.last_insert_rowid();

        let logic = serde_json::json!({ "regla": "Cruce de medias" });
        let out = save_logic(
            &conn,
            &path,
            df,
            &EngineCfg::default(),
            sid,
            Template::MaCross,
            [10.0, 30.0, 0.0],
            &logic,
        )
        .expect("save_logic deberia funcionar");

        assert!(out["nTrades"].as_u64().unwrap_or(0) > 0, "debe generar operaciones");
        assert!(out["metrics"]["netProfit"].is_number());
        assert!(out["oosFrac"].as_f64().unwrap_or(1.0) > 0.0);

        // la regla queda guardada
        let (tpl, params): (i64, String) = conn
            .query_row(
                "select template_id, params from strategies where id = ?1",
                rusqlite::params![sid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(tpl, 1);
        assert!(params.contains("10") && params.contains("30"), "params: {params}");

        // hay una corrida nueva, con estado done
        let (runs, net): (i64, f64) = conn
            .query_row(
                "select count(*), coalesce(max(net_profit), 0.0) from backtest_runs where strategy_id = ?1",
                rusqlite::params![sid],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(runs, 1);
        // la DB guarda el valor crudo; el JSON que vuelve va redondeado
        let esperado = out["metrics"]["netProfit"].as_f64().unwrap();
        assert!((net - esperado).abs() < 0.01, "net en DB {net} vs {esperado}");

        let _ = std::fs::remove_file(&path);
    }

    /// Una regla que no genera operaciones debe fallar SIN tocar lo guardado.
    #[test]
    fn guardar_una_regla_sin_operaciones_no_altera_nada() {
        let conn = Connection::open_in_memory().unwrap();
        crate::engine::datastore::init_schema(&conn).unwrap();
        conn.execute(
            "insert into symbols (name, asset_class) values ('EURUSD', 'forex')",
            [],
        )
        .unwrap();
        let sym = conn.last_insert_rowid();
        // la plantilla que se guarda referencia strategy_templates
        conn.execute(
            "insert into strategy_templates (id, name, source, logic)
             values (1, 'Cruce de medias', 'built_in', '{}')",
            [],
        )
        .unwrap();

        // serie plana: ninguna regla genera operaciones
        let path = std::env::temp_dir().join(format!("qf_save_plano_{}.qfn", std::process::id()));
        let bars: Vec<Bar> = (0..600)
            .map(|i| Bar {
                time: 1_700_000_000 + i as i64 * 60,
                open: 1.10, high: 1.10, low: 1.10, close: 1.10, volume: 1000.0,
            })
            .collect();
        crate::engine::datastore::Qfn::write(&path, &bars).unwrap();

        conn.execute(
            "insert into data_files (symbol_id, timeframe, storage_kind, file_path, n_bars, status)
             values (?1, 'm1', 'local_qfd', ?2, ?3, 'ready')",
            rusqlite::params![sym, path.to_string_lossy().to_string(), bars.len() as i64],
        )
        .unwrap();
        let df = conn.last_insert_rowid();

        conn.execute(
            "insert into strategies (name, symbol_id, timeframe, origin, logic, params, status)
             values ('S1', ?1, 'm1', 'builder', '{}', '{\"p\":[20,50,0]}', 'active')",
            rusqlite::params![sym],
        )
        .unwrap();
        let sid = conn.last_insert_rowid();

        let err = save_logic(
            &conn, &path, df, &EngineCfg::default(), sid, Template::MaCross,
            [20.0, 50.0, 0.0], &serde_json::json!({}),
        );
        assert!(err.is_err(), "debe fallar si no hay operaciones");

        // nada cambio: ni params ni corridas
        let params: String = conn
            .query_row("select params from strategies where id = ?1", rusqlite::params![sid], |r| r.get(0))
            .unwrap();
        assert!(params.contains("20") && params.contains("50"), "params: {params}");
        let runs: i64 = conn
            .query_row("select count(*) from backtest_runs where strategy_id = ?1", rusqlite::params![sid], |r| r.get(0))
            .unwrap();
        assert_eq!(runs, 0);

        let _ = std::fs::remove_file(&path);
    }

    /// El corte out-of-sample se calcula sobre las barras, no sobre las operaciones.
    #[test]
    fn el_corte_oos_usa_el_60_por_ciento_de_las_barras() {
        let bars = serie_con_ciclos(600);
        let trades: Vec<Trade> = Vec::new();
        // sin operaciones cae al 40 % por defecto
        assert!((oos_frac_of(&bars, &trades) - 0.4).abs() < 1e-9);
    }


    /// Retesteo en otro mercado: la MISMA regla, otros datos.
    #[test]
    fn retest_en_otro_mercado_devuelve_metricas_y_veredicto() {
        let cfg = EngineCfg::default();
        let barras = serie_con_ciclos(600);
        let r = retest_on_market(&barras, Template::MaCross, [20.0, 50.0, 0.0], &cfg)
            .expect("el retesteo en otro mercado debe funcionar");
        assert!(r["verdict"].is_string());
        let m = &r["retest"];
        assert!(m["netProfit"].is_number());
        assert!(m["profitFactor"].is_number());
        assert_eq!(r["nBars"], 600);
    }

    /// Un mercado con muy pocos datos es un error claro, no un veredicto falso.
    #[test]
    fn retest_con_dataset_minimo_da_error() {
        let cfg = EngineCfg::default();
        let err = retest_on_market(&serie_con_ciclos(50), Template::MaCross, [20.0, 50.0, 0.0], &cfg);
        assert!(err.is_err());
    }

    /// El resumen por mercados: cuenta solo los que tienen datos.
    #[test]
    fn el_resumen_cuenta_solo_los_mercados_con_datos() {
        let mercados = vec![
            ("EURUSD".to_string(), "pass".to_string()),
            ("GBPUSD".to_string(), "fail".to_string()),
            ("USDJPY".to_string(), "sin datos".to_string()),
        ];
        let r = verdict_mercados(&mercados);
        assert_eq!(r["mercadosConDatos"], 2);
        assert_eq!(r["pass"], 1);
        assert_eq!(r["fail"], 1);
        // uno pasa y otro falla: ni pasa ni falla del todo -> alerta
        assert_eq!(r["verdict"], "warn");
        assert!(r["detalle"].as_str().unwrap().contains("1 de 2"));
    }

    /// Si ningun mercado tiene datos, el veredicto queda pendiente (no fallido).
    #[test]
    fn sin_mercados_con_datos_el_veredicto_queda_pendiente() {
        let mercados = vec![("EURUSD".to_string(), "sin datos".to_string())];
        let r = verdict_mercados(&mercados);
        assert_eq!(r["verdict"], "pending");
        assert_eq!(r["mercadosConDatos"], 0);
    }


    /// El equiponderado reparte igual y los pesos suman 1.
    #[test]
    fn el_equiponderado_reparte_igual() {
        let eq = vec![serie_equity(200, 0.01), serie_equity(200, 0.02)];
        let w = weights("equal_weight", &eq, &[]);
        assert_eq!(w.len(), 2);
        assert!((w[0] - 0.5).abs() < 1e-9);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    /// El inverso a volatilidad da mas peso a la serie menos volatil.
    #[test]
    fn el_inverso_a_volatilidad_pesa_mas_a_la_serie_estable() {
        let tranquila = serie_equity(300, 0.002);
        let nerviosa = serie_equity(300, 0.05);
        let eq = vec![tranquila.clone(), nerviosa];
        let w = weights("equal_vol", &eq, &[]);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!(w[0] > w[1], "la serie tranquila debe pesar mas: {w:?}");
    }

    /// La paridad de riesgo iguala la contribucion al riesgo de cada miembro.
    #[test]
    fn la_paridad_de_riesgo_iguala_contribuciones() {
        let eq = vec![serie_equity(400, 0.01), serie_equity(400, 0.03)];
        let w = weights("risk_parity", &eq, &[]);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        let rc = risk_contributions(&eq, &w);
        // las contribuciones se reparten (no tiene que ser exacto, pero si parecidas)
        assert!((rc[0] - rc[1]).abs() < 25.0, "contribuciones: {rc:?}");
        assert!((rc[0] + rc[1] - 100.0).abs() < 1.0, "deben sumar 100: {rc:?}");
    }

    /// Los pesos personalizados se normalizan y no aceptan negativos.
    #[test]
    fn los_pesos_personalizados_se_normalizan() {
        let eq = vec![serie_equity(100, 0.01), serie_equity(100, 0.01)];
        let w = weights("custom", &eq, &[2.0, 2.0]);
        assert!((w[0] - 0.5).abs() < 1e-9);
        // negativos -> se ignoran (reparto equiponderado en vez de pesos invalidos)
        let w2 = weights("custom", &eq, &[-1.0, 1.0]);
        assert!(w2.iter().all(|x| *x >= 0.0), "pesos negativos: {w2:?}");
    }

    /// La equity combinada con pesos no es la media simple cuando hay pesos.
    #[test]
    fn la_equity_ponderada_respeta_los_pesos() {
        let a = serie_equity(100, 0.0);           // plana
        let b = serie_equity(100, 0.0);
        let mut b2 = b.clone();
        for v in b2.iter_mut() { *v *= 2.0; }
        let eq = vec![a, b2];
        let combinada = weighted_equity(&eq, &[0.5, 0.5]);
        let media = mean_equity(&eq);
        assert!((combinada[50] - media[50]).abs() < 1e-9);
        // con todo el peso en la primera, la combinada es esa serie
        let solo_a = weighted_equity(&eq, &[1.0, 0.0]);
        assert!((solo_a[50] - eq[0][50]).abs() < 1e-9);
    }

    /// Serie de equity sintetica con la volatilidad pedida.
    fn serie_equity(n: usize, vol: f64) -> Vec<f64> {
        let mut out = Vec::with_capacity(n);
        let mut v = 10_000.0;
        let mut s = 12345u64;
        for i in 0..n {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let r = ((s >> 33) as f64 / 2_000_000_000.0) - 0.5;
            v *= 1.0 + r * vol;
            out.push(v);
        }
        out
    }

    #[test]
    fn pearson_detecta_correlacion_e_independencia() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b_same = vec![1.0, 2.0, 3.0, 4.0, 5.5];
        let b_inv = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!(pearson(&a, &b_same) > 0.95);
        assert!(pearson(&a, &b_inv) < -0.95);
        let noise = Rng::new(9).next();
        assert!(noise >= 0.0 && noise < 1.0);
    }

    #[test]
    fn pearson_con_serie_plana_da_cero() {
        let flat = vec![1.0, 1.0, 1.0, 1.0];
        let other = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(pearson(&flat, &other), 0.0);
    }

    #[test]
    fn corr_matrix_diagonal_uno_y_simetrica() {
        // escala ×2 exacta → retornos idénticos → correlación ≈ 1
        let seqs = vec![
            vec![10_020.0, 10_100.0, 10_050.0, 10_250.0, 10_080.0],
            vec![10_020.0, 10_100.0, 10_050.0, 10_250.0, 10_080.0],
            vec![10_010.0, 10_015.0, 10_300.0, 10_020.0, 10_500.0], // otra forma
        ];
        let m = corr_matrix(&seqs);
        for i in 0..3 {
            assert!((m[i][i] - 1.0).abs() < 1e-9);
            for j in 0..3 { assert!((m[i][j] - m[j][i]).abs() < 1e-12); }
        }
        assert!((m[0][1] - 1.0).abs() < 1e-6, "series idénticas → 1");
        assert!((-1.0..=1.0).contains(&m[0][2]), "el restante queda dentro de [-1,1]");
    }

    #[test]
    fn max_dd_pct_caso_conocido() {
        // 100 → 120 (peak) → 90: dd = 30/120 = 25%
        assert!((max_dd_pct(&[100.0, 120.0, 90.0]) - 25.0).abs() < 1e-9);
    }

    #[test]
    fn bootstrap_mc_determinista_y_coherente() {
        let mut rr = Rng::new(11);
        let rets: Vec<f64> = (0..400).map(|_| (rr.next() - 0.48) * 0.01).collect();
        let a = bootstrap_mc(&rets, 300, 4242);
        let b = bootstrap_mc(&rets, 300, 4242);
        assert_eq!(a, b, "misma semilla → mismo resultado");
        assert!(a["p5"].as_f64().unwrap() <= a["p50"].as_f64().unwrap());
        assert!(a["p50"].as_f64().unwrap() <= a["p95"].as_f64().unwrap());
        let ruin = a["ruinProb"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&ruin));
    }

    #[test]
    fn axis_produce_n_valores_masivos_y_ordenados() {
        let a = axis(20.0, 5);
        assert_eq!(a.len(), 5);
        for w in a.windows(2) { assert!(w[1] > w[0], "ordenado"); }
        assert!((a[0] - 8.0).abs() < 0.5, "40% de 20 ≈ 8");
    }

    #[test]
    fn mean_equity_alinea_al_minimo() {
        let s1 = vec![10.0, 11.0, 12.0, 13.0];
        let s2 = vec![10.0, 12.0, 14.0];
        let m = mean_equity(&[s1, s2]);
        assert_eq!(m.len(), 3);
        assert!((m[1] - 11.5).abs() < 1e-9);
    }
}
