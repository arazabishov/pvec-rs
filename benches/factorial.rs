use criterion::*;

use num::{BigUint, One};
use pvec::core::RrbVec;
use rayon::prelude::*;
use std::ops::Mul;

/// Compute the Factorial using a plain iterator.
fn factorial(n: u32) -> BigUint {
    (1..=n).map(BigUint::from).fold(BigUint::one(), Mul::mul)
}

fn factorial_par_iter(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($factorial:ident, $data:ident) => {
            assert_eq!(
                $data
                    .into_par_iter()
                    .map(BigUint::from)
                    .reduce_with(Mul::mul)
                    .unwrap(),
                $factorial
            );
        };
    }
    let mut group = criterion.benchmark_group("factorial_par_iter");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        10, 20, 30, 50, 100, 200, 500, 1000, 2000, 4000, 8000, 12000, 16000,
    ];

    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
            let factorial = factorial(*n);
            b.iter_batched(
                || (1..*n + 1),
                |data| make_bench!(factorial, data),
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
            let factorial = factorial(*n);
            b.iter_batched(
                || {
                    let mut vec = RrbVec::new();

                    for i in 1..(*n + 1) {
                        vec.push(i);
                    }

                    vec
                },
                |data| make_bench!(factorial, data),
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, factorial_par_iter);
