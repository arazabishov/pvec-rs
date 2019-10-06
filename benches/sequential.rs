use criterion::*;

#[cfg(feature = "arc")]
use im::Vector as IVec;

#[cfg(not(feature = "arc"))]
use im_rc::Vector as IVec;

use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use pvec::core::RrbVec;
use pvec::PVec;

use super::*;

fn index_sequentially(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("index_sequentially");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |data| {
                        for i in 0..*n {
                            black_box(data[i]);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |data| {
                        for i in 0..*n {
                            black_box(data[i]);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn index_randomly(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("index_randomly");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rng =
                    XorShiftRng::from_seed([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |data| {
                        for _ in 0..*n {
                            let j = (rng.next_u32() as usize) % *n;
                            black_box(data[j]);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rng =
                    XorShiftRng::from_seed([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |data| {
                        for _ in 0..*n {
                            let j = (rng.next_u32() as usize) % *n;
                            black_box(data[j]);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);
    }

    group.finish();
}

fn iterator_next(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("iterator_next");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |data| {
                        for i in data.into_iter() {
                            black_box(i);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |data| {
                        for i in data.into_iter() {
                            black_box(i);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn iterator_next_back(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("iterator_next_back");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |data| {
                        for i in data.into_iter().rev() {
                            black_box(i);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |data| {
                        for i in data.into_iter().rev() {
                            black_box(i);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn push(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($p:ident, $vec:ident, $op:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    let mut vec = $vec::new();
                    for i in 0..*n {
                        vec.$op(i);
                    }
                    vec
                })
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(p, Vec, push, STD_VEC);
        bench_balanced!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench_balanced!(p, RrbVec, push, RRBVEC_BALANCED);
        bench_balanced!(p, PVec, push, PVEC_BALANCED);
    }

    group.finish();
}

fn push_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($p:ident, $vec:ident, $op:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_with_large_drop(|| {
                    let mut vec = $vec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..*n {
                        vec.$op(i);
                        vec_one = vec.clone();
                    }

                    vec_one
                })
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    for p in params.iter() {
        bench_balanced!(p, Vec, push, STD_VEC);
        bench_balanced!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench_balanced!(p, RrbVec, push, RRBVEC_BALANCED);
        bench_balanced!(p, PVec, push, PVEC_BALANCED);
    }

    group.finish();
}

fn push_unbalanced(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push_unbalanced");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        for i in 0..*n {
                            data.$op(i);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |mut data| {
                        for i in 0..*n {
                            data.$op(i);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn push_clone_unbalanced(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push_clone_unbalanced");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |data| {
                        let mut vec = data;
                        let mut vec_one = vec.clone();

                        for i in 0..*n {
                            vec.$op(i);
                            vec_one = vec.clone();
                        }

                        vec_one
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |data| {
                        let mut vec = data;
                        let mut vec_one = vec.clone();

                        for i in 0..*n {
                            vec.$op(i);
                            vec_one = vec.clone();
                        }

                        vec_one
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn update(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("update");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($p:ident, $vec:ident, $op:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                        }
                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(p, Vec, push, STD_VEC);
        bench_balanced!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench_balanced!(p, RrbVec, push, RRBVEC_BALANCED);
        bench_balanced!(p, PVec, push, PVEC_BALANCED);
    }

    group.finish();
}

fn update_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("update_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($p:ident, $vec:ident, $op:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        let mut data_one = data.clone();

                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                            data_one = data.clone();
                        }

                        data_one
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    for p in params.iter() {
        bench_balanced!(p, Vec, push, STD_VEC);
        bench_balanced!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench_balanced!(p, RrbVec, push, RRBVEC_BALANCED);
        bench_balanced!(p, PVec, push, PVEC_BALANCED);
    }

    group.finish();
}

fn update_unbalanced(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("update_unbalanced");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                        }
                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |mut data| {
                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn update_clone_unbalanced(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("update_clone_unbalanced");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        let mut data_one = data.clone();

                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                            data_one = data.clone();
                        }

                        data_one
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $op:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$op(i);
                        }

                        vec
                    },
                    |mut data| {
                        let mut data_one = data.clone();

                        for i in 0..*n {
                            (*data.get_mut(i).unwrap()) += 1;
                            data_one = data.clone();
                        }

                        data_one
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_unbalanced!(IM_RS_VECTOR_UNBALANCED, p, IVec, push_back, append_ivec);

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, append_pvec);
    }

    group.finish();
}

fn pop(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("pop");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$push(i * 2);
                        }

                        vec
                    },
                    |mut vec| {
                        for _ in 0..*n {
                            black_box(vec.$pop());
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$push(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$push(i);
                        }

                        vec
                    },
                    |mut vec| {
                        for _ in 0..*n {
                            black_box(vec.$pop());
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push, pop);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back, pop_back);
        bench_unbalanced!(
            IM_RS_VECTOR_UNBALANCED,
            p,
            IVec,
            push_back,
            pop_back,
            append_ivec
        );

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push, pop);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, pop, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push, pop);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, pop, append_pvec);
    }

    group.finish();
}

fn pop_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("pop_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$push(i * 2);
                        }

                        vec
                    },
                    |vec| {
                        let mut vec_one = vec.clone();
                        let mut vec_two = vec_one.clone();
                        for _ in 0..*n {
                            let _ = vec_one.$pop();
                            vec_two = vec_one.clone();
                        }

                        drop(vec_two);
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    macro_rules! bench_unbalanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $append:expr) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut i = 1;
                        let mut vec = $vec::new();

                        while i < *n && (vec.len() + i) <= *n {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$push(j);
                            }

                            $append(&mut vec, another_vec);
                            i *= 2;
                        }

                        while vec.len() < *n {
                            vec.$push(i);
                        }

                        vec
                    },
                    |vec| {
                        let mut vec_one = vec.clone();
                        let mut vec_two = vec_one.clone();
                        for _ in 0..*n {
                            let _ = vec_one.$pop();
                            vec_two = vec_one.clone();
                        }

                        drop(vec_two);
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    let append_ivec = |vec: &mut IVec<usize>, data| vec.append(data);
    let append_pvec = |vec: &mut PVec<usize>, mut data| vec.append(&mut data);
    let append_rrbvec = |vec: &mut RrbVec<usize>, mut data| vec.append(&mut data);

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push, pop);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back, pop_back);
        bench_unbalanced!(
            IM_RS_VECTOR_UNBALANCED,
            p,
            IVec,
            push_back,
            pop_back,
            append_ivec
        );

        bench_balanced!(RRBVEC_BALANCED, p, RrbVec, push, pop);
        bench_unbalanced!(RRBVEC_UNBALANCED, p, RrbVec, push, pop, append_rrbvec);

        bench_balanced!(PVEC_BALANCED, p, PVec, push, pop);
        bench_unbalanced!(PVEC_UNBALANCED, p, PVec, push, pop, append_pvec);
    }

    group.finish();
}

fn append(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $vec:ident, $op:ident) => {
            || {
                let mut input = Vec::new();
                let mut input_len = 0;
                let mut i = 1;

                while i < *$n && (input_len + i) <= *$n {
                    let mut another_vec = $vec::new();

                    for j in 0..i {
                        another_vec.$op(j);
                    }

                    input_len += another_vec.len();
                    input.push(another_vec);

                    i *= 2;
                }

                let mut another_vec = $vec::new();
                let mut j = 0;

                while input_len < *$n {
                    another_vec.$op(j);
                    input_len += 1;
                    j += 1;
                }

                input.push(another_vec);
                input
            }
        };
    }

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    let mut group = criterion.benchmark_group("append");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new(STD_VEC, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, Vec, push),
                |mut data| {
                    let mut vec_two = Vec::new();

                    for mut input in data.iter_mut() {
                        vec_two.append(&mut input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(IM_RS_VECTOR_UNBALANCED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, IVec, push_back),
                |data| {
                    let mut vec_two = IVec::new();

                    for input in data.into_iter() {
                        vec_two.append(input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(RRBVEC_UNBALANCED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, RrbVec, push),
                |mut data| {
                    let mut vec_two = RrbVec::new();

                    for mut input in data.iter_mut() {
                        vec_two.append(&mut input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(PVEC_UNBALANCED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, PVec, push),
                |mut data| {
                    let mut vec_two = PVec::new();

                    for mut input in data.iter_mut() {
                        vec_two.append(&mut input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn split_off(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:ident) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..*n {
                            vec.$op(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        while data.len() > 64 {
                            data = data.split_off(64)
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let mut group = criterion.benchmark_group("split_off");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        32, 64, 128, 512, 768, 1024, 2048, 4096, 10000, 20000, 40000, 50000, 100000, 500000,
    ];

    for p in params.iter() {
        make_bench!(group, p, Vec, push, STD_VEC);
        make_bench!(group, p, IVec, push_back, IM_RS_VECTOR_UNBALANCED);
        make_bench!(group, p, RrbVec, push, RRBVEC_UNBALANCED);
        make_bench!(group, p, PVec, push, PVEC_UNBALANCED);
    }

    group.finish();
}

criterion_group!(
    benches,
    index_sequentially,
    index_randomly,
    iterator_next,
    iterator_next_back,
    update,
    update_clone,
    update_unbalanced,
    update_clone_unbalanced,
    push,
    push_unbalanced,
    push_clone,
    push_clone_unbalanced,
    pop,
    pop_clone,
    append,
    split_off
);
