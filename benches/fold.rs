use super::*;
use rayon::prelude::*;

use crate::pvec::core::RbVec;
use crate::pvec::core::RrbVec;
use crate::pvec::PVec;

mod stdvec {
    pub fn generate_vec(n: u32) -> Vec<u32> {
        let mut vec = Vec::new();
        for i in 0_u32..n {
            vec.push(i);
        }
        vec
    }
}

mod rbvec {
    use pvec::core::RbVec;

    pub fn generate_vec(n: u32) -> RbVec<u32> {
        let mut pvec = RbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec
    }
}

mod rrbvec {
    use pvec::core::RrbVec;

    pub fn generate_vec(n: u32) -> RrbVec<u32> {
        let mut pvec = RrbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec
    }
}

mod pvec {
    use pvec::PVec;

    pub fn generate_vec(n: u32) -> PVec<u32> {
        let mut pvec = PVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec
    }
}

fn map_fold_seq(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("map_fold_with_thread_num_1");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($name:ident, $p:ident, $module:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || $module::generate_vec(*n),
                    |data| {
                        data.into_iter()
                            .map(|it| it + 1)
                            .fold($vec::new(), |mut vec1, x| {
                                vec1.push(x);
                                vec1
                            })
                    },
                    BatchSize::SmallInput,
                );
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, stdvec, Vec);
        bench!(PVEC_UNBALANCED, p, pvec, PVec);
        bench!(RRBVEC_BALANCED, p, rbvec, RbVec);
        bench!(RRBVEC_UNBALANCED, p, rrbvec, RrbVec);
    }

    group.finish();
}

fn map_fold_par(criterion: &mut Criterion, num_threads: usize) {
    let mut group = criterion.benchmark_group(format!("map_fold_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    macro_rules! bench {
        ($name:ident, $p:ident, $module:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                pool.install(|| {
                    b.iter_batched(
                        || $module::generate_vec(*n),
                        |data| {
                            data.into_par_iter()
                                .map(|it| it + 1)
                                .fold($vec::new, |mut vec1, x| {
                                    vec1.push(x);
                                    vec1
                                })
                                .reduce($vec::new, |mut vec1, mut vec2| {
                                    vec1.append(&mut vec2);
                                    vec1
                                })
                        },
                        BatchSize::SmallInput,
                    );
                });
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, stdvec, Vec);
        bench!(PVEC_UNBALANCED, p, pvec, PVec);
        bench!(RRBVEC_BALANCED, p, rbvec, RbVec);
        bench!(RRBVEC_UNBALANCED, p, rrbvec, RrbVec);
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
    let mut group = criterion.benchmark_group("filter_fold_with_thread_num_1");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($name:ident, $p:ident, $module:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || $module::generate_vec(*n),
                    |data| {
                        data.into_iter().filter(|it| it % 2 == 0).fold(
                            $vec::new(),
                            |mut vec1, x| {
                                vec1.push(x);
                                vec1
                            },
                        )
                    },
                    BatchSize::SmallInput,
                );
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, stdvec, Vec);
        bench!(PVEC_UNBALANCED, p, pvec, PVec);
        bench!(RRBVEC_BALANCED, p, rbvec, RbVec);
        bench!(RRBVEC_UNBALANCED, p, rrbvec, RrbVec);
    }

    group.finish();
}

fn filter_fold_par(criterion: &mut Criterion, num_threads: usize) {
    let mut group =
        criterion.benchmark_group(format!("filter_fold_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    macro_rules! bench {
        ($name:ident, $p:ident, $module:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                pool.install(|| {
                    b.iter_batched(
                        || $module::generate_vec(*n),
                        |data| {
                            data.into_par_iter()
                                .filter(|it| it % 2 == 0)
                                .fold($vec::new, |mut vec1, x| {
                                    vec1.push(x);
                                    vec1
                                })
                                .reduce($vec::new, |mut vec1, mut vec2| {
                                    vec1.append(&mut vec2);
                                    vec1
                                })
                        },
                        BatchSize::SmallInput,
                    );
                });
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, stdvec, Vec);
        bench!(PVEC_UNBALANCED, p, pvec, PVec);
        bench!(RRBVEC_BALANCED, p, rbvec, RbVec);
        bench!(RRBVEC_UNBALANCED, p, rrbvec, RrbVec);
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

criterion_group!(
    benches,
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
