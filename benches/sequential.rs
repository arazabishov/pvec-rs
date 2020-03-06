use criterion::BatchSize::SmallInput;
use criterion::*;

#[cfg(feature = "arc")]
use im::Vector as IVec;

#[cfg(not(feature = "arc"))]
use im_rc::Vector as IVec;

use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use pvec::core::RbVec;
use pvec::core::RrbVec;
use pvec::PVec;

use super::*;

fn rnd() -> XorShiftRng {
    XorShiftRng::from_seed([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}

macro_rules! vec_balanced {
    ($n:ident, $vec:ident, $push:ident) => {
        || {
            let mut vec = $vec::new();

            for i in 0..*$n {
                vec.$push(i);
            }

            vec
        }
    };
}

macro_rules! vec_balanced_cloned {
    ($n:ident, $vec:ident, $push:ident) => {
        || {
            let vec_balanced = vec_balanced!($n, $vec, $push);
            let vec = vec_balanced();

            vec.clone()
        }
    };
}

macro_rules! vec_relaxed {
    ($n:ident, $vec:ident, $push:ident, $append:path) => {
        || {
            let mut i = 1;
            let mut vec = $vec::new();

            while i < *$n && (vec.len() + i) <= *$n {
                let mut another_vec = $vec::new();

                for j in 0..i {
                    another_vec.$push(j);
                }

                $append(&mut vec, another_vec);
                i *= 2;
            }

            while vec.len() < *$n {
                vec.$push(i);
            }

            vec
        }
    };
}

mod append {
    use super::*;

    pub fn ivec(vec: &mut IVec<usize>, data: IVec<usize>) {
        vec.append(data);
    }

    pub fn rrbvec(vec: &mut RrbVec<usize>, mut data: RrbVec<usize>) {
        vec.append(&mut data);
    }

    pub fn pvec(vec: &mut PVec<usize>, data: PVec<usize>) {
        // clone() is invoked to ensure that PVec switches its internal
        // representation to RbVec. This piece of code is used only
        // in the setup code and it will not be measured.
        let mut data_clone = data.clone();
        vec.append(&mut data_clone);
    }
}

fn index_sequentially(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident) => {
            |vec| {
                for i in 0..*$n {
                    black_box(vec[i]);
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("index_sequentially");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_relaxed!(n, $vec, $push, $append), bench!(n), SmallInput)
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(vec_balanced_cloned!(n, PVec, push), bench!(n), SmallInput)
        });
    }

    group.finish();
}

fn index_randomly(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $rnd:ident) => {
            |vec| {
                for _ in 0..*$n {
                    let j = ($rnd.next_u32() as usize) % *$n;
                    black_box(vec[j]);
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("index_randomly");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, rnd), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            let mut rnd = rnd();

            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, rnd),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn iterator_next(criterion: &mut Criterion) {
    macro_rules! bench {
        () => {
            |vec| {
                for i in vec.into_iter() {
                    black_box(i);
                }
            }
        };
    }

    let mut group = criterion.benchmark_group("iterator_next");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_relaxed!(n, $vec, $push, $append), bench!(), SmallInput)
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(vec_balanced_cloned!(n, PVec, push), bench!(), SmallInput)
        });
    }

    group.finish();
}

fn push(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($p:ident, $vec:ident, $push:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(vec_balanced!(n, $vec, $push))
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench!(p, Vec, push, STD_VEC);
        bench!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench!(p, RbVec, push, RBVEC);
        bench!(p, PVec, push, PVEC_STD);
    }

    group.finish();
}

fn push_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($p:ident, $vec:ident, $push:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    let mut vec_1 = $vec::new();
                    let mut vec_2 = vec_1.clone();
                    let mut vec_3 = vec_2.clone();

                    for i in 0..*n {
                        vec_3 = vec_1;
                        vec_1 = vec_2;

                        vec_1.$push(i);
                        vec_2 = vec_1.clone();
                    }

                    (vec_1, vec_2, vec_3)
                })
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        bench!(p, Vec, push, STD_VEC);
        bench!(p, IVec, push_back, IM_RS_VECTOR_BALANCED);
        bench!(p, RbVec, push, RBVEC);
        bench!(p, PVec, push, PVEC_RRBVEC_BALANCED);
    }

    group.finish();
}

fn push_relaxed(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $push:ident) => {
            |mut vec| {
                for i in 0..*$n {
                    vec.$push(i);
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("push_relaxed");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, $push), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, $push),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, push),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn push_clone_relaxed(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $push:ident) => {
            |vec| {
                let mut vec_1 = vec;
                let mut vec_2 = vec_1.clone();
                let mut vec_3 = vec_2.clone();

                for i in 0..*$n {
                    vec_3 = vec_1;
                    vec_1 = vec_2;

                    vec_1.$push(i);
                    vec_2 = vec_1.clone();
                }

                (vec_1, vec_2, vec_3)
            }
        };
    }

    let mut group = criterion.benchmark_group("push_clone_relaxed");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, $push), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, $push),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, push),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn update(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident) => {
            |mut vec| {
                for i in 0..*$n {
                    (*vec.get_mut(i).unwrap()) += 1;
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("update");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_relaxed!(n, $vec, $push, $append), bench!(n), SmallInput)
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(vec_balanced_cloned!(n, PVec, push), bench!(n), SmallInput)
        });
    }

    group.finish();
}

fn update_randomly(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $rnd:ident) => {
            |mut vec| {
                for _ in 0..*$n {
                    let j = ($rnd.next_u32() as usize) % *$n;
                    (*vec.get_mut(j).unwrap()) += 1;
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("update_randomly");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, rnd), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            let mut rnd = rnd();

            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, rnd),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn update_clone(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident) => {
            |vec| {
                let mut vec_1 = vec;
                let mut vec_2 = vec_1.clone();
                let mut vec_3 = vec_2.clone();

                for i in 0..*$n {
                    vec_3 = vec_1;
                    vec_1 = vec_2;

                    (*vec_1.get_mut(i).unwrap()) += 1;
                    vec_2 = vec_1.clone();
                }

                (vec_1, vec_2, vec_3)
            }
        };
    }

    let mut group = criterion.benchmark_group("update_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_relaxed!(n, $vec, $push, $append), bench!(n), SmallInput)
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(vec_balanced_cloned!(n, PVec, push), bench!(n), SmallInput)
        });
    }

    group.finish();
}

fn update_clone_randomly(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $rnd:ident) => {
            |vec| {
                let mut vec_1 = vec;
                let mut vec_2 = vec_1.clone();
                let mut vec_3 = vec_2.clone();

                for _ in 0..*$n {
                    vec_3 = vec_1;
                    vec_1 = vec_2;

                    let j = ($rnd.next_u32() as usize) % *$n;
                    (*vec_1.get_mut(j).unwrap()) += 1;

                    vec_2 = vec_1.clone();
                }

                (vec_1, vec_2, vec_3)
            }
        };
    }

    let mut group = criterion.benchmark_group("update_clone_randomly");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, rnd), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back);
        bench_relaxed!(IM_RS_VECTOR_RELAXED, p, IVec, push_back, append::ivec);

        bench_balanced!(RBVEC, p, RbVec, push);
        bench_relaxed!(RRBVEC, p, RrbVec, push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            let mut rnd = rnd();

            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, rnd),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn pop(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $pop:ident) => {
            |mut vec| {
                for _ in 0..*$n {
                    black_box(vec.$pop());
                }

                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("pop");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, $pop), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, $pop),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push, pop);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back, pop_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            IVec,
            push_back,
            pop_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, RbVec, push, pop);
        bench_relaxed!(RRBVEC, p, RrbVec, push, pop, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push, pop);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, pop, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, pop),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn pop_clone(criterion: &mut Criterion) {
    macro_rules! bench {
        ($n:ident, $pop:ident) => {
            |vec| {
                let mut vec_1 = vec;
                let mut vec_2 = vec_1.clone();
                let mut vec_3 = vec_2.clone();

                for _ in 0..*$n {
                    vec_3 = vec_1;
                    vec_1 = vec_2;

                    black_box(vec_1.$pop());
                    vec_2 = vec_1.clone();
                }

                (vec_1, vec_2, vec_3)
            }
        };
    }

    let mut group = criterion.benchmark_group("pop_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench_balanced {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(n, $pop), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $vec, $push, $append),
                    bench!(n, $pop),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, Vec, push, pop);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, IVec, push_back, pop_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            IVec,
            push_back,
            pop_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, RbVec, push, pop);
        bench_relaxed!(RRBVEC, p, RrbVec, push, pop, append::rrbvec);

        bench_balanced!(PVEC_STD, p, PVec, push, pop);
        bench_relaxed!(PVEC_RRBVEC_RELAXED, p, PVec, push, pop, append::pvec);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_BALANCED, p), p, |b, n| {
            b.iter_batched(
                vec_balanced_cloned!(n, PVec, push),
                bench!(n, pop),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn append(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $vec:ident, $push:ident) => {
            || {
                let mut input = Vec::new();
                let mut input_len = 0;
                let mut i = 1;

                while i < *$n && (input_len + i) <= *$n {
                    let mut vec = $vec::new();

                    for j in 0..i {
                        vec.$push(j);
                    }

                    input_len += vec.len();
                    input.push(vec);

                    i *= 2;
                }

                let mut vec = $vec::new();
                let mut j = 0;

                while input_len < *$n {
                    vec.$push(j);

                    input_len += 1;
                    j += 1;
                }

                input.push(vec);
                input
            }
        };
    }

    macro_rules! create_input_cloned {
        ($n:ident, $vec:ident, $push:ident) => {
            || {
                let create_input = create_input!($n, $vec, $push);

                let mut input = create_input();
                let mut input_cloned = Vec::new();

                for vec in input.iter_mut() {
                    // force transition to RrbVec by cloning the vector
                    input_cloned.push(vec.clone());
                }

                input_cloned
            }
        };
    }

    macro_rules! bench {
        ($n:ident, $vec:ident) => {
            |data| {
                let mut vec = $vec::new();

                for mut input in data.into_iter() {
                    vec.append(&mut input);
                }

                vec
            }
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    let mut group = criterion.benchmark_group("append");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new(STD_VEC, p), p, |b, n| {
            b.iter_batched(create_input!(n, Vec, push), bench!(n, Vec), SmallInput)
        });
        group.bench_with_input(BenchmarkId::new(IM_RS_VECTOR_RELAXED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, IVec, push_back),
                |data| {
                    let mut vec_two = IVec::new();

                    for input in data.into_iter() {
                        vec_two.append(input);
                    }

                    vec_two
                },
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(RRBVEC, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, RrbVec, push),
                bench!(n, RrbVec),
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(RBVEC, p), p, |b, n| {
            b.iter_batched(create_input!(n, RbVec, push), bench!(n, RbVec), SmallInput)
        });
        group.bench_with_input(BenchmarkId::new(PVEC_STD, p), p, |b, n| {
            b.iter_batched(create_input!(n, PVec, push), bench!(n, PVec), SmallInput)
        });
        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            b.iter_batched(
                create_input_cloned!(n, PVec, push),
                bench!(n, PVec),
                SmallInput,
            )
        });
    }

    group.finish();
}

fn split_off(criterion: &mut Criterion) {
    macro_rules! bench {
        () => {
            |mut vec| {
                while vec.len() > 64 {
                    vec = vec.split_off(64)
                }

                vec
            }
        };
    }

    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $push:ident, $name:ident) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $vec, $push), bench!(), SmallInput)
            });
        };
    }

    let mut group = criterion.benchmark_group("split_off");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        128, 512, 768, 1024, 2048, 4096, 10000, 20000, 30000, 40000, 60000, 80000, 100000, 200000,
    ];
    for p in params.iter() {
        make_bench!(group, p, Vec, push, STD_VEC);
        make_bench!(group, p, RbVec, push, RBVEC);
        make_bench!(group, p, RrbVec, push, RRBVEC);
        make_bench!(group, p, IVec, push_back, IM_RS_VECTOR_RELAXED);
        make_bench!(group, p, PVec, push, PVEC_STD);

        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            b.iter_batched(vec_balanced_cloned!(n, PVec, push), bench!(), SmallInput)
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    index_sequentially,
    index_randomly,
    iterator_next,
    push,
    push_clone,
    push_relaxed,
    push_clone_relaxed,
    update,
    update_randomly,
    update_clone,
    update_clone_randomly,
    pop,
    pop_clone,
    append,
    split_off
);
