//! The family corrections over one request's predicates
//! (`research.md` §1.5). Max-T is counted with the permutations
//! (`engine`); these read the raw p-values alone.

/// The predicates' p-values with their indices, ascending, ties kept in
/// the request's order.
fn ascending(p: &[f64]) -> Vec<(usize, f64)> {
    let mut order: Vec<(usize, f64)> = p.iter().copied().enumerate().collect();
    order.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    order
}

/// Values given with their predicates' indices, back in the request's
/// order.
pub(crate) fn in_request_order(mut values: Vec<(usize, f64)>) -> Vec<f64> {
    values.sort_by_key(|&(index, _)| index);
    values.into_iter().map(|(_, value)| value).collect()
}

/// The family's size as a float.
#[allow(clippy::cast_precision_loss)]
fn size(p: &[f64]) -> f64 {
    p.len() as f64
}

/// Bonferroni: each p-value times the family's size, capped at 1. Only
/// reported as the bound Holm improves on.
pub(crate) fn bonferroni(p: &[f64]) -> Vec<f64> {
    let m = size(p);
    p.iter().map(|p| (p * m).min(1.0)).collect()
}

/// Holm's step-down (*Scand. J. Statist.* 6, 1979): the `i`th smallest
/// times `m − i + 1`, made non-decreasing in that order. It holds the
/// familywise error rate under any dependence.
pub(crate) fn holm(p: &[f64]) -> Vec<f64> {
    let m = size(p);
    let mut running: f64 = 0.0;
    let adjusted = ascending(p)
        .into_iter()
        .enumerate()
        .map(|(rank, (index, p))| {
            #[allow(clippy::cast_precision_loss)]
            let factor = m - rank as f64;
            running = running.max((p * factor).min(1.0));
            (index, running)
        })
        .collect();
    in_request_order(adjusted)
}

/// Benjamini–Hochberg q-values (*J. R. Statist. Soc. B* 57, 1995), each
/// scaled by `scale` (1 for BH; the harmonic sum for BY): the `i`th
/// smallest times `m / i`, made non-increasing from the largest down.
fn step_up(p: &[f64], scale: f64) -> Vec<f64> {
    let m = size(p);
    let mut running: f64 = 1.0;
    let adjusted = ascending(p)
        .into_iter()
        .enumerate()
        .rev()
        .map(|(rank, (index, p))| {
            #[allow(clippy::cast_precision_loss)]
            let i = rank as f64 + 1.0;
            running = running.min((p * m / i * scale).min(1.0));
            (index, running)
        })
        .collect();
    in_request_order(adjusted)
}

/// Benjamini–Hochberg, which controls the false discovery rate under
/// positive dependence.
pub(crate) fn benjamini_hochberg(p: &[f64]) -> Vec<f64> {
    step_up(p, 1.0)
}

/// Benjamini–Yekutieli (*Ann. Statist.* 29, 2001): BH scaled by
/// `1 + 1/2 + … + 1/m`, which holds under any dependence.
pub(crate) fn benjamini_yekutieli(p: &[f64]) -> Vec<f64> {
    let harmonic: f64 = (1..=p.len())
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let i = i as f64;
            1.0 / i
        })
        .sum();
    step_up(p, harmonic)
}
