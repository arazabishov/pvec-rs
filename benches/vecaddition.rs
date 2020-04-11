use super::*;
use rayon::prelude::*;

use crate::pvec::core::RbVec;
use crate::pvec::core::RrbVec;
use crate::pvec::PVec;

use criterion::BatchSize::SmallInput;

macro_rules! generate_vec {
    ($new_vec:expr) => {
        |n: usize| {
            let mut vec = $new_vec();

            for i in 0..n {
                vec.push(i);
            }

            vec
        }
    };
}

fn vector_addition_seq(criterion: &mut Criterion) {
    macro_rules! bench {
        ($new_vec:expr) => {
            |(vec_one, vec_two)| {
                vec_one
                    .into_iter()
                    .zip(vec_two)
                    .map(|(e_1, e_2)| e_1 + e_2)
                    .fold($new_vec(), |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
            }
        };
    }

    let mut group = criterion.benchmark_group("vector_addition_with_thread_num_1");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! make_bench {
        ($name:ident, $p:ident, $new_vec:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = generate_vec!($new_vec);

                b.iter_batched(
                    || (generate_vec(*n), generate_vec(*n)),
                    bench!($new_vec),
                    SmallInput,
                );
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, || Vec::new());
        make_bench!(RBVEC, p, || RbVec::new());
        make_bench!(RRBVEC, p, || RrbVec::new());
        make_bench!(PVEC_STD, p, || PVec::new());

        // TODO: verify that RRBVEC is relaxed here.
        make_bench!(PVEC_RRBVEC_RELAXED, p, || PVec::new_with_tree());
    }

    group.finish();
}

// TODO: consider running single thread tests here too!
fn vector_addition_par(criterion: &mut Criterion, num_threads: usize) {
    macro_rules! bench {
        ($new_vec:expr) => {
            |(vec_one, vec_two)| {
                vec_one
                    .into_par_iter()
                    .zip(vec_two)
                    .map(|(e_1, e_2)| e_1 + e_2)
                    .fold($new_vec, |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
                    .reduce($new_vec, |mut vec_1, mut vec_2| {
                        vec_1.append(&mut vec_2);
                        vec_1
                    })
            }
        };
    }

    let mut group =
        criterion.benchmark_group(format!("vector_addition_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    macro_rules! make_bench {
        ($name:ident, $p:ident, $new_vec:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = generate_vec!($new_vec);

                pool.install(|| {
                    b.iter_batched(
                        || (generate_vec(*n), generate_vec(*n)),
                        bench!($new_vec),
                        SmallInput,
                    );
                });
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, || Vec::new());
        make_bench!(RBVEC, p, || RbVec::new());
        make_bench!(RRBVEC, p, || RrbVec::new());
        make_bench!(PVEC_STD, p, || PVec::new());

        // TODO: check that the tree is actually relaxed
        make_bench!(PVEC_RRBVEC_RELAXED, p, || PVec::new_with_tree());
    }

    group.finish();
}

fn vector_addition_1(criterion: &mut Criterion) {
    vector_addition_seq(criterion);
}

fn vector_addition_2(criterion: &mut Criterion) {
    vector_addition_par(criterion, 2);
}

fn vector_addition_4(criterion: &mut Criterion) {
    vector_addition_par(criterion, 4);
}

fn vector_addition_8(criterion: &mut Criterion) {
    vector_addition_par(criterion, 8);
}

fn vector_addition_16(criterion: &mut Criterion) {
    vector_addition_par(criterion, 16);
}

criterion_group!(
    benches,
    vector_addition_1,
    vector_addition_2,
    vector_addition_4,
    vector_addition_8,
    vector_addition_16,
);
