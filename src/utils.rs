use rand::{RngCore, Rng};
use std::cmp::{Ordering};

pub fn normalize_pdf(pdf: &mut [f32]) {
    let inv_sum = 1. / pdf.iter().sum::<f32>();
    for prob in pdf.iter_mut() {
        (*prob) *= inv_sum;
    }
}

/*
pub fn normalize_log_pdf(log_pdf: &mut [f32]) {
    let max = log_pdf.iter().fold(-f32::INFINITY, |x, &y| f32::max(x, y));
    let sum_exp = log_pdf.iter().map(|x| (x - max).exp()).sum::<f32>();
    let normalizer = max + sum_exp.ln();
    log_pdf.iter_mut().for_each(|x| *x -= normalizer);
}
*/

pub fn exp_normalize_log_pdf(log_pdf: &mut [f32]) {
    let max = log_pdf.iter().fold(-f32::INFINITY, |x, &y| f32::max(x, y));
    let sum_exp = log_pdf.iter().map(|x| (x - max).exp()).sum::<f32>();
    let normalizer = max + sum_exp.ln();
    log_pdf.iter_mut().for_each(|x| *x = (*x - normalizer).exp());
}


pub fn sample_pdf(rng: &mut dyn RngCore, pdf: &[f32]) -> usize {
    let sample = rng.gen::<f32>();
    let mut partial_sum = 0.;
    for (i, prob) in pdf.iter().enumerate() {
        partial_sum += prob;
        if partial_sum >= sample { return i }
    }
    return 0;
}

/// Computes the value of binomial distribution: what is the probability that exactly k
/// events from n are successes, if the probability of success is p?
pub fn binom_pdf(n: usize, k: usize, p: f32) -> f32 {
    binom(n, k) * p.powi(k as i32) * (1. - p).powi((n - k) as i32)
}

/// Computes the value of binomial coefficient n over k: how many ways there are to select
/// k objects from n objects?
pub fn binom(n: usize, k: usize) -> f32 {
    (1..=k).map(|i| (n - k + i) as f32 / k as f32).product()
}

/// Computes log(a + b) from log a and log b. This calculation should preserve numerical
/// precision better than the naive version and should correctly handle negative infinity.
pub fn log_add(log_a: f32, log_b: f32) -> f32 {
    // Let log_x be the larger and log_y the smaller value
    let (log_x, log_y) = 
        if log_a > log_b { (log_a, log_b) }
        else { (log_b, log_a) };

    // We have: log(x + y) = log(x * (1 + y/x)) = log x + log(1 + y/x)
    // where y/x = exp(log y) / exp(log x) = exp(log y - log x)
    log_x + (log_y - log_x).exp().ln_1p()
}

/*
/// Computes log(n!).
pub fn log_factorial(n: usize) -> f32 {
    (2..=n).into_iter().map(|k| (k as f32).ln()).sum::<f32>()
}
*/

pub fn compare_f32(x: f32, y: f32) -> Ordering {
    if x < y {
        Ordering::Less
    } else if x > y {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}
