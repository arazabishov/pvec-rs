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
    ($n:ident, $new_vec:expr, $push:ident) => {
        || {
            let mut vec = $new_vec();

            for i in 0..*$n {
                vec.$push(i);
            }

            vec
        }
    };
}

macro_rules! vec_relaxed {
    ($n:ident, $new_vec:expr, $push:ident, $append:path) => {
        || {
            let mut i = 1;
            let mut vec = $new_vec();

            while i < *$n && (vec.len() + i) <= *$n {
                let mut another_vec = $new_vec();

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

    pub fn pvec(vec: &mut PVec<usize>, mut data: PVec<usize>) {
        vec.append(&mut data);
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $new_vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
                    bench!(n),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $new_vec, $push), bench!(), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
                    bench!(),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
    }

    group.finish();
}

fn push(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($p:ident, $new_vec:expr, $push:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(vec_balanced!(n, $new_vec, $push))
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000, 40000,
        60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        bench!(p, || Vec::new(), push, STD_VEC);
        bench!(p, || IVec::new(), push_back, IM_RS_VECTOR_BALANCED);
        bench!(p, || RbVec::new(), push, RBVEC);
        bench!(p, || PVec::new(), push, PVEC_STD);
    }

    group.finish();
}

fn push_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("push_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    macro_rules! bench {
        ($p:ident, $new_vec:expr, $push:ident, $name:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    let mut vec_1 = $new_vec();
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
        bench!(p, || Vec::new(), push, STD_VEC);
        bench!(p, || IVec::new(), push_back, IM_RS_VECTOR_BALANCED);
        bench!(p, || RbVec::new(), push, RBVEC);
        bench!(p, || PVec::new_with_tree(), push, PVEC_RRBVEC_BALANCED);
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, $push),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, $push),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $new_vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
                    bench!(n),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
                    bench!(n, rnd),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $new_vec, $push), bench!(n), SmallInput)
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
                    bench!(n),
                    SmallInput,
                )
            });
        };
    }

    let params = vec![
        20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
    ];

    for p in params.iter() {
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, rnd),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                let mut rnd = rnd();

                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push);

        bench_balanced!(IM_RS_VECTOR_BALANCED, p, || IVec::new(), push_back);
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, $pop),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $pop:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push, pop);

        bench_balanced!(
            IM_RS_VECTOR_BALANCED,
            p,
            || IVec::new(),
            push_back,
            pop_back
        );
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            pop_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push, pop);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, pop, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push, pop);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push, pop);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            pop,
            append::pvec
        );
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
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $pop:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_balanced!(n, $new_vec, $push),
                    bench!(n, $pop),
                    SmallInput,
                )
            });
        };
    }

    macro_rules! bench_relaxed {
        ($name:ident, $p:ident, $new_vec:expr, $push:ident, $pop:ident, $append:path) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    vec_relaxed!(n, $new_vec, $push, $append),
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
        bench_balanced!(STD_VEC, p, || Vec::new(), push, pop);

        bench_balanced!(
            IM_RS_VECTOR_BALANCED,
            p,
            || IVec::new(),
            push_back,
            pop_back
        );
        bench_relaxed!(
            IM_RS_VECTOR_RELAXED,
            p,
            || IVec::new(),
            push_back,
            pop_back,
            append::ivec
        );

        bench_balanced!(RBVEC, p, || RbVec::new(), push, pop);
        bench_relaxed!(RRBVEC, p, || RrbVec::new(), push, pop, append::rrbvec);

        bench_balanced!(PVEC_STD, p, || PVec::new(), push, pop);
        bench_balanced!(PVEC_RRBVEC_BALANCED, p, || PVec::new_with_tree(), push, pop);
        bench_relaxed!(
            PVEC_RRBVEC_RELAXED,
            p,
            || PVec::new_with_tree(),
            push,
            pop,
            append::pvec
        );
    }

    group.finish();
}

fn append(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $new_vec:expr, $push:ident) => {
            || {
                let mut input = Vec::new();
                let mut input_len = 0;
                let mut i = 1;

                while i < *$n && (input_len + i) <= *$n {
                    let mut vec = $new_vec();

                    for j in 0..i {
                        vec.$push(j);
                    }

                    input_len += vec.len();
                    input.push(vec);

                    i *= 2;
                }

                let mut vec = $new_vec();
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

    macro_rules! bench {
        ($n:ident, $new_vec:expr) => {
            |data| {
                let mut vec = $new_vec();

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
            b.iter_batched(
                create_input!(n, || Vec::new(), push),
                bench!(n, || Vec::new()),
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(IM_RS_VECTOR_RELAXED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, || IVec::new(), push_back),
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
                create_input!(n, || RrbVec::new(), push),
                bench!(n, || RrbVec::new()),
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(RBVEC, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, || RbVec::new(), push),
                bench!(n, || RbVec::new()),
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(PVEC_STD, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, || PVec::new(), push),
                bench!(n, || PVec::new()),
                SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new(PVEC_RRBVEC_RELAXED, p), p, |b, n| {
            b.iter_batched(
                create_input!(n, || PVec::new_with_tree(), push),
                bench!(n, || PVec::new_with_tree()),
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
        ($group:ident, $p:ident, $new_vec:expr, $push:ident, $name:ident) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(vec_balanced!(n, $new_vec, $push), bench!(), SmallInput)
            });
        };
    }

    let mut group = criterion.benchmark_group("split_off");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        128, 512, 768, 1024, 2048, 4096, 10000, 20000, 30000, 40000, 60000, 80000, 100000, 200000,
    ];
    for p in params.iter() {
        make_bench!(group, p, || Vec::new(), push, STD_VEC);
        make_bench!(group, p, || RbVec::new(), push, RBVEC);
        make_bench!(group, p, || RrbVec::new(), push, RRBVEC);
        make_bench!(group, p, || IVec::new(), push_back, IM_RS_VECTOR_RELAXED);
        make_bench!(group, p, || PVec::new(), push, PVEC_STD);
        make_bench!(
            group,
            p,
            || PVec::new_with_tree(),
            push,
            PVEC_RRBVEC_RELAXED
        );
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
