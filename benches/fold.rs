use super::*;
use rayon::prelude::*;

use crate::pvec::core::{RbVec, RrbVec};
use crate::pvec::PVec;

use criterion::BatchSize::SmallInput;

macro_rules! vec_balanced {
    ($vec:ident) => {
        |n: usize| {
            let mut vec = $vec::new();

            for i in 0..n {
                vec.push(i);
            }

            vec
        }
    };
}

macro_rules! vec_balanced_cloned {
    ($vec:ident) => {
        |n: usize| {
            let generate_vec = vec_balanced!($vec);
            generate_vec(n).clone()
        }
    };
}

fn map_fold_seq(criterion: &mut Criterion) {
    macro_rules! bench {
        ($vec:ident) => {
            |vec| {
                vec.into_iter()
                    .map(|it| Some(it))
                    .fold($vec::new(), |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
            }
        };
    }

    let mut group = criterion.benchmark_group("map_fold_with_thread_num_1");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! make_bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = vec_balanced!($vec);
                b.iter_batched(|| generate_vec(*n), bench!($vec), SmallInput);
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, Vec);
        make_bench!(RBVEC, p, RbVec);
        make_bench!(RRBVEC, p, RrbVec);
        make_bench!(PVEC_STD, p, PVec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            let generate_vec = vec_balanced_cloned!(PVec);
            b.iter_batched(|| generate_vec(*n), bench!(PVec), SmallInput);
        });
    }

    group.finish();
}

fn map_fold_par(criterion: &mut Criterion, num_threads: usize) {
    macro_rules! bench {
        ($vec:ident) => {
            |vec| {
                vec.into_par_iter()
                    .map(|it| Some(it))
                    .fold($vec::new, |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
                    .reduce($vec::new, |mut vec_1, mut vec_2| {
                        vec_1.append(&mut vec_2);
                        vec_1
                    })
            }
        };
    }

    let mut group = criterion.benchmark_group(format!("map_fold_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    macro_rules! make_bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = vec_balanced!($vec);

                pool.install(|| {
                    b.iter_batched(|| generate_vec(*n), bench!($vec), SmallInput);
                });
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, Vec);
        make_bench!(RBVEC, p, RbVec);
        make_bench!(RRBVEC, p, RrbVec);
        make_bench!(PVEC_STD, p, PVec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            let generate_vec = vec_balanced_cloned!(PVec);

            pool.install(|| {
                b.iter_batched(|| generate_vec(*n), bench!(PVec), SmallInput);
            });
        });
    }

    group.finish();
}

pub fn map_fold_1(criterion: &mut Criterion) {
    map_fold_seq(criterion);
}

pub fn map_fold_2(criterion: &mut Criterion) {
    map_fold_par(criterion, 2);
}

pub fn map_fold_4(criterion: &mut Criterion) {
    map_fold_par(criterion, 4);
}

pub fn map_fold_8(criterion: &mut Criterion) {
    map_fold_par(criterion, 8);
}

pub fn map_fold_16(criterion: &mut Criterion) {
    map_fold_par(criterion, 16);
}

fn filter_fold_seq(criterion: &mut Criterion) {
    macro_rules! bench {
        ($vec:ident) => {
            |vec| {
                vec.into_iter()
                    .filter(|it| it % 2 == 0)
                    .fold($vec::new(), |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
            }
        };
    }

    let mut group = criterion.benchmark_group("filter_fold_with_thread_num_1");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! make_bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = vec_balanced!($vec);
                b.iter_batched(|| generate_vec(*n), bench!($vec), SmallInput);
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, Vec);
        make_bench!(RBVEC, p, RbVec);
        make_bench!(RRBVEC, p, RrbVec);
        make_bench!(PVEC_STD, p, PVec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            let generate_vec = vec_balanced_cloned!(PVec);
            b.iter_batched(|| generate_vec(*n), bench!(PVec), SmallInput);
        });
    }

    group.finish();
}

fn filter_fold_par(criterion: &mut Criterion, num_threads: usize) {
    macro_rules! bench {
        ($vec:ident) => {
            |vec| {
                vec.into_par_iter()
                    .filter(|it| it % 2 == 0)
                    .fold($vec::new, |mut vec_1, x| {
                        vec_1.push(x);
                        vec_1
                    })
                    .reduce($vec::new, |mut vec_1, mut vec_2| {
                        vec_1.append(&mut vec_2);
                        vec_1
                    })
            }
        };
    }

    let mut group =
        criterion.benchmark_group(format!("filter_fold_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    macro_rules! make_bench {
        ($name:ident, $p:ident, $module:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let generate_vec = vec_balanced!($vec);

                pool.install(|| {
                    b.iter_batched(|| generate_vec(*n), bench!($vec), SmallInput);
                });
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        make_bench!(STD_VEC, p, stdvec, Vec);
        make_bench!(RBVEC, p, rbvec, RbVec);
        make_bench!(RRBVEC, p, rrbvec, RrbVec);
        make_bench!(PVEC_STD, p, pvec, PVec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            let generate_vec = vec_balanced_cloned!(PVec);

            pool.install(|| {
                b.iter_batched(|| generate_vec(*n), bench!(PVec), SmallInput);
            });
        });
    }

    group.finish();
}

pub fn filter_fold_1(criterion: &mut Criterion) {
    filter_fold_seq(criterion);
}

pub fn filter_fold_2(criterion: &mut Criterion) {
    filter_fold_par(criterion, 2);
}

pub fn filter_fold_4(criterion: &mut Criterion) {
    filter_fold_par(criterion, 4);
}

pub fn filter_fold_8(criterion: &mut Criterion) {
    filter_fold_par(criterion, 8);
}

pub fn filter_fold_16(criterion: &mut Criterion) {
    filter_fold_par(criterion, 16);
}

fn create_criterion() -> Criterion {
    Criterion::default().configure_from_args().sample_size(10)
}

criterion_group!(
    name = benches;
    config = create_criterion();
    targets = 
        map_fold_1,
        map_fold_2,
        map_fold_4,
        map_fold_8,
        map_fold_16,
        filter_fold_1,
        filter_fold_2,
        filter_fold_4,
        filter_fold_8,
        filter_fold_16,
);
