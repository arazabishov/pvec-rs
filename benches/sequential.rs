use criterion::*;

#[cfg(feature = "arc")]
use im::Vector as IVec;

#[cfg(not(feature = "arc"))]
use im_rc::Vector as IVec;

use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use pvec::core::RrbVec;
use pvec::PVec;

fn push(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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

    let mut group = criterion.benchmark_group("push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16000, 32000, 64000,
    ];

    for p in params.iter() {
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

fn unbalanced_push(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal, $append:expr) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..128 {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
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

    let mut group = criterion.benchmark_group("unbalanced_push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16000, 32000, 64000,
    ];

    for p in params.iter() {
        make_bench!(
            group,
            p,
            Vec,
            push,
            "std",
            |vec: &mut Vec<usize>, mut data| { vec.append(&mut data) }
        );
        make_bench!(
            group,
            p,
            IVec,
            push_back,
            "im",
            |vec: &mut IVec<usize>, data| { vec.append(data) }
        );
        make_bench!(
            group,
            p,
            RrbVec,
            push,
            "rrbvec",
            |vec: &mut RrbVec<usize>, mut data| { vec.append(&mut data) }
        );
        make_bench!(
            group,
            p,
            PVec,
            push,
            "pvec",
            |vec: &mut PVec<usize>, mut data| { vec.append(&mut data) }
        );
    }

    group.finish();
}

fn push_clone(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter(|| {
                    let mut vec = $vec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..*n {
                        vec.$op(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                })
            });
        };
    }

    let mut group = criterion.benchmark_group("push_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![10, 20, 50, 100, 500, 1000, 5000, 10000, 20000];
    for p in params.iter() {
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

fn unbalanced_push_clone(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal, $append:expr) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || {
                        let mut vec = $vec::new();

                        for i in 0..128 {
                            let mut another_vec = $vec::new();

                            for j in 0..i {
                                another_vec.$op(j);
                            }

                            $append(&mut vec, another_vec);
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

                        drop(vec_one);
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let mut group = criterion.benchmark_group("unbalanced_push_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![10, 20, 50, 100, 500, 1000, 5000, 10000, 20000];
    for p in params.iter() {
        make_bench!(
            group,
            p,
            Vec,
            push,
            "std",
            |vec: &mut Vec<usize>, mut data| { vec.append(&mut data) }
        );
        make_bench!(
            group,
            p,
            IVec,
            push_back,
            "im",
            |vec: &mut IVec<usize>, data| { vec.append(data) }
        );
        make_bench!(
            group,
            p,
            RrbVec,
            push,
            "rrbvec",
            |vec: &mut RrbVec<usize>, mut data| { vec.append(&mut data) }
        );
        make_bench!(
            group,
            p,
            PVec,
            push,
            "pvec",
            |vec: &mut PVec<usize>, mut data| { vec.append(&mut data) }
        );
    }

    group.finish();
}

fn pop(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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
    let mut group = criterion.benchmark_group("pop");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16000, 32000, 64000,
    ];

    for p in params.iter() {
        make_bench!(group, p, Vec, push, pop, "std");
        make_bench!(group, p, IVec, push_back, pop_back, "im");
        make_bench!(group, p, RrbVec, push, pop, "rrbvec");
        make_bench!(group, p, PVec, push, pop, "pvec");
    }

    group.finish();
}

fn pop_clone(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $push:ident, $pop:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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
    let mut group = criterion.benchmark_group("pop_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![100, 500, 1000, 5000, 10000, 20000];
    for p in params.iter() {
        make_bench!(group, p, Vec, push, pop, "std");
        make_bench!(group, p, IVec, push_back, pop_back, "im");
        make_bench!(group, p, RrbVec, push, pop, "rrbvec");
        make_bench!(group, p, PVec, push, pop, "pvec");
    }

    group.finish();
}

fn index_sequentially(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("index_sequentially");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000,
        40000, 60000, 80000, 100000, 200000, 400000, 600000, 800000, 1000000,
    ];

    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut vec = Vec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
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

        group.bench_with_input(BenchmarkId::new("im-rs-rbtree", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut vec = IVec::new();

                    for i in 0..*n {
                        vec.push_back(i * 2);
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

        group.bench_with_input(BenchmarkId::new("im-rs-rrbtree", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut i = 1;
                    let mut vec = IVec::new();

                    while i < *n && (vec.len() + i) <= *n {
                        let mut another_vec = IVec::new();

                        for j in 0..i {
                            another_vec.push_back(j);
                        }

                        vec.append(another_vec);
                        i *= 2;
                    }

                    while vec.len() < *n {
                        vec.push_back(i);
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

        group.bench_with_input(BenchmarkId::new("rbvec", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut vec = RrbVec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
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

        group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut i = 1;
                    let mut vec = RrbVec::new();

                    while i < *n && (vec.len() + i) <= *n {
                        let mut another_vec = RrbVec::new();

                        for j in 0..i {
                            another_vec.push(j);
                        }

                        vec.append(&mut another_vec);
                        i *= 2;
                    }

                    while vec.len() < *n {
                        vec.push(i);
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

        group.bench_with_input(BenchmarkId::new("pvec-rbtree", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut vec = PVec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
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

        group.bench_with_input(BenchmarkId::new("pvec-rrbtree", p), p, |b, n| {
            b.iter_batched(
                || {
                    let mut i = 1;
                    let mut vec = PVec::new();

                    while i < *n && (vec.len() + i) <= *n {
                        let mut another_vec = PVec::new();

                        for j in 0..i {
                            another_vec.push(j);
                        }

                        vec.append(&mut another_vec);
                        i *= 2;
                    }

                    while vec.len() < *n {
                        vec.push(i);
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
    }

    group.finish();
}

fn index_randomly(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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
                    |vec| {
                        for _ in 0..*n {
                            let j = (rng.next_u32() as usize) % *n;
                            black_box(vec[j]);
                        }
                    },
                    BatchSize::SmallInput,
                )
            });
        };
    }

    let params = vec![
        10, 20, 30, 50, 100, 200, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000,
    ];

    let mut group = criterion.benchmark_group("index_randomly");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for p in params.iter() {
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im-rs");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

fn append(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $vec:ident, $op:ident) => {
            || {
                let mut input = Vec::new();
                for _ in 0..16 {
                    let mut vec = $vec::new();

                    for i in 0..*$n {
                        vec.$op(i);
                    }

                    input.push(vec)
                }
                input
            }
        };
    }

    let mut group = criterion.benchmark_group("append");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];
    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
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
        group.bench_with_input(BenchmarkId::new("im-rs", p), p, |b, n| {
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
        group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
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
        group.bench_with_input(BenchmarkId::new("pvec", p), p, |b, n| {
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

fn append_clone(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $vec:ident, $op:ident) => {
            || {
                let mut vec = $vec::new();
                for i in 0..*$n {
                    vec.$op(i);
                }
                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("append_clone");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000];
    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, Vec, push),
                |data| {
                    let mut vec = Vec::new();

                    for _ in 0..16 {
                        vec.append(&mut data.clone());
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("im-rs", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, IVec, push_back),
                |data| {
                    let mut vec = IVec::new();

                    for _ in 0..16 {
                        vec.append(data.clone());
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, RrbVec, push),
                |data| {
                    let mut vec = RrbVec::new();

                    for _ in 0..16 {
                        vec.append(&mut data.clone());
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("pvec", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, PVec, push),
                |data| {
                    let mut vec = PVec::new();

                    for _ in 0..16 {
                        vec.append(&mut data.clone());
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn append_push(criterion: &mut Criterion) {
    macro_rules! create_input {
        ($n:ident, $vec:ident, $op:ident) => {
            || {
                let mut vec = $vec::new();
                for i in 0..*$n {
                    vec.$op(i);
                }
                vec
            }
        };
    }

    let mut group = criterion.benchmark_group("append_push");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![100, 500, 1000, 5000, 10000, 50000];
    for p in params.iter() {
        group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, Vec, push),
                |data| {
                    let mut vec = Vec::new();

                    for i in 0..1024 {
                        if i % 2 == 0 {
                            vec.push(i);
                        } else {
                            vec.append(&mut data.clone());
                        }
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("im-rs", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, IVec, push_back),
                |data| {
                    let mut vec = IVec::new();

                    for i in 0..1024 {
                        if i % 2 == 0 {
                            vec.push_back(i);
                        } else {
                            vec.append(data.clone());
                        }
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, RrbVec, push),
                |data| {
                    let mut vec = RrbVec::new();

                    for i in 0..1024 {
                        if i % 2 == 0 {
                            vec.push(i);
                        } else {
                            vec.append(&mut data.clone());
                        }
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("pvec", p), p, |b, n| {
            b.iter_batched(
                create_input!(n, PVec, push),
                |data| {
                    let mut vec = PVec::new();

                    for i in 0..1024 {
                        if i % 2 == 0 {
                            vec.push(i);
                        } else {
                            vec.append(&mut data.clone());
                        }
                    }

                    drop(vec)
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn iterator_next(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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

    let mut group = criterion.benchmark_group("iterator_next");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        10, 20, 30, 50, 100, 200, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000,
    ];
    for p in params.iter() {
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im-rs");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

fn iterator_next_back(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
            $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
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

    let mut group = criterion.benchmark_group("iterator_next_back");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let params = vec![
        10, 20, 30, 50, 100, 200, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000,
    ];
    for p in params.iter() {
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im-rs");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

fn split_off(criterion: &mut Criterion) {
    macro_rules! make_bench {
        ($group:ident, $p:ident, $vec:ident, $op:ident, $name:literal) => {
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
        make_bench!(group, p, Vec, push, "std");
        make_bench!(group, p, IVec, push_back, "im-rs");
        make_bench!(group, p, RrbVec, push, "rrbvec");
        make_bench!(group, p, PVec, push, "pvec");
    }

    group.finish();
}

criterion_group!(
    benches,
    push,
    unbalanced_push,
    push_clone,
    unbalanced_push_clone,
    pop,
    pop_clone,
    index_sequentially,
    index_randomly,
    iterator_next,
    iterator_next_back,
    append,
    append_clone,
    append_push,
    split_off
);
