// Power iteration t <- (1 - alpha) * C^T t + alpha * p on dense vectors.
//
// The trust vector becomes dense after a few iterations, and a dense lookup makes each row
// dot product O(nnz(row)). Sums use Kahan-Babuska-Neumaier compensation in a fixed order,
// so results are reproducible bit for bit, with or without threads.

use super::matrix::{CsrMatrix, Entry};
use crate::EigenTrustError;

pub(crate) struct Converged {
    pub(crate) scores: Vec<f64>,
    pub(crate) iterations: usize,
    pub(crate) residual: f64,
}

// `c` is row-stochastic (or close to it) and `p` a distribution, both of size n > 0.
pub(crate) fn iterate(
    c: &CsrMatrix,
    p: &[f64],
    alpha: f64,
    epsilon: f64,
    max_iterations: usize,
) -> Result<Converged, EigenTrustError> {
    let n = p.len();
    let ct = c.transpose();
    let ap: Vec<f64> = p.iter().map(|v| v * alpha).collect();
    let mut t1 = p.to_vec();
    let mut prev = t1.clone();
    let mut t2 = vec![0.0; n];
    let mut residual = f64::INFINITY;
    let mut iterations = 0;

    log::info!(
        "EigenTrust started: peers={} nnz={} alpha={} epsilon={}",
        n,
        ct.rows.iter().map(Vec::len).sum::<usize>(),
        alpha,
        epsilon
    );

    loop {
        if iterations >= 1 {
            residual = delta_norm2(&t1, &prev);
            prev.copy_from_slice(&t1);
            log::trace!("iteration={} residual={}", iterations, residual);
            if residual <= epsilon {
                break;
            }
        }
        if iterations == max_iterations {
            return Err(EigenTrustError::NotConverged {
                iterations,
                residual,
            });
        }

        mul(&ct, &t1, &mut t2);
        for (x, ap_i) in t2.iter_mut().zip(&ap) {
            *x = *x * (1.0 - alpha) + ap_i;
        }
        std::mem::swap(&mut t1, &mut t2);
        iterations += 1;
    }

    log::info!(
        "EigenTrust finished: iterations={} residual={}",
        iterations,
        residual
    );
    Ok(Converged {
        scores: t1,
        iterations,
        residual,
    })
}

// ||t - prev||2
fn delta_norm2(t: &[f64], prev: &[f64]) -> f64 {
    let mut sum = KbnSum::default();
    for (x, y) in t.iter().zip(prev) {
        let d = x - y;
        if d != 0.0 {
            sum.add(d * d);
        }
    }
    sum.value().sqrt()
}

// out = m * v
fn mul(m: &CsrMatrix, v: &[f64], out: &mut [f64]) {
    let row_dot = |row: &[Entry]| {
        let mut sum = KbnSum::default();
        for entry in row {
            let x = v[entry.index];
            if x != 0.0 {
                sum.add(entry.value * x);
            }
        }
        sum.value()
    };

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        out.par_iter_mut()
            .zip(m.rows.par_iter())
            .with_min_len(1024)
            .for_each(|(o, row)| *o = row_dot(row));
    }

    #[cfg(not(feature = "parallel"))]
    for (o, row) in out.iter_mut().zip(&m.rows) {
        *o = row_dot(row);
    }
}

// Kahan-Babuska-Neumaier compensated summation.
#[derive(Default)]
struct KbnSum {
    sum: f64,
    compensation: f64,
}

impl KbnSum {
    fn add(&mut self, value: f64) {
        let (more, less) = if self.sum.abs() < value.abs() {
            (value, self.sum)
        } else {
            (self.sum, value)
        };
        self.sum += value;
        self.compensation += less - (self.sum - more);
    }

    fn value(&self) -> f64 {
        self.sum + self.compensation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kbn_recovers_lost_precision() {
        let mut s = KbnSum::default();
        for v in [1.0, 1e100, 1.0, -1e100] {
            s.add(v);
        }
        assert_eq!(s.value(), 2.0);
    }

    // Reference result carried over from the original implementation: a raw (not
    // re-normalized) 8x8 matrix, alpha 0.5, epsilon 1.25e-7.
    #[test]
    fn matches_reference_result() {
        let seventh = 0.14285714285714285;
        let full = || (0..7).map(|i| Entry::new(i, seventh)).collect::<Vec<_>>();
        let c = CsrMatrix {
            rows: vec![
                vec![Entry::new(3, 1.0)],
                full(),
                vec![Entry::new(3, 1.0)],
                full(),
                vec![Entry::new(1, 1.0)],
                full(),
                vec![Entry::new(5, 1.0)],
                full(),
            ],
        };
        let mut p = vec![seventh; 7];
        p.push(0.0);

        let out = iterate(&c, &p, 0.5, 1.25e-7, 10_000).unwrap();
        assert_eq!(
            out.scores,
            vec![
                0.11111110842697292,
                0.16666666867977029,
                0.11111110842697292,
                0.22222222893256766,
                0.11111110842697292,
                0.16666666867977029,
                0.11111110842697292,
                0.0,
            ]
        );
    }

    #[test]
    fn stops_at_max_iterations() {
        // 0 <-> 1 with no teleport oscillates forever
        let c = CsrMatrix {
            rows: vec![vec![Entry::new(1, 1.0)], vec![Entry::new(0, 1.0)]],
        };
        let err = iterate(&c, &[1.0, 0.0], 0.0, 1e-9, 50).err().unwrap();
        assert!(matches!(
            err,
            EigenTrustError::NotConverged { iterations: 50, .. }
        ));
    }
}
