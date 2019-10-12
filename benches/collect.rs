use super::*;
use rayon::prelude::*;

mod util_stdvec {
    use rayon::prelude::*;

    // TODO: sequential iter does not have fold or reduce
    // TODO: also, it doesn't make much sense to split and combine
    //   stuff without changing it in any way, fuck.

    /// Fold into vectors and then reduce them together.
    pub fn fold_seq<T, I>(i: I) -> Vec<T>
    where
        I: Iterator<Item = T>,
    {
        i.fold(Vec::new(), |mut vec, x| {
            vec.push(x);
            vec
        })
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold_par<T, PI>(pi: PI) -> Vec<T>
    where
        T: Send,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.fold(Vec::new, |mut vec, x| {
            vec.push(x);
            vec
        })
        .reduce(Vec::new, |mut vec1, mut vec2| {
            vec1.append(&mut vec2);
            vec1
        })
    }
}

mod util_rrbvec {
    use pvec::core::RrbVec;
    use rayon::prelude::*;
    use std::fmt::Debug;

    pub fn fold_seq<T, I>(i: I) -> RrbVec<T>
    where
        T: Clone + Debug,
        I: Iterator<Item = T>,
    {
        i.fold(RrbVec::new(), |mut vec, x| {
            vec.push(x);
            vec
        })
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold_par<T, PI>(pi: PI) -> RrbVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.fold(RrbVec::new, |mut vec, x| {
            vec.push(x);
            vec
        })
        .reduce(RrbVec::new, |mut vec1, mut vec2| {
            vec1.append(&mut vec2);
            vec1
        })
    }
}

mod util_pvec {
    use pvec::PVec;
    use rayon::prelude::*;
    use std::fmt::Debug;

    /// Fold into vectors and then reduce them together.
    pub fn fold_seq<T, I>(i: I) -> PVec<T>
    where
        T: Clone + Debug,
        I: Iterator<Item = T>,
    {
        i.fold(PVec::new(), |mut vec, x| {
            vec.push(x);
            vec
        })
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold_par<T, PI>(pi: PI) -> PVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.fold(PVec::new, |mut vec, x| {
            vec.push(x);
            vec
        })
        .reduce(PVec::new, |mut vec1, mut vec2| {
            vec1.append(&mut vec2);
            vec1
        })
    }
}

mod stdvec {
    use rayon::prelude::*;

    pub fn generate_iter(n: u32) -> impl Iterator<Item = u32> {
        let mut vec = Vec::new();
        for i in 0_u32..n {
            vec.push(i);
        }
        vec.into_iter().map(|x| x + 1)
    }

    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut vec = Vec::new();
        for i in 0_u32..n {
            vec.push(i);
        }
        vec.into_par_iter().map(|x| x + 1)
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        generate_indexed_iter(n).filter(|_| true)
    }
}

mod rrbvec {
    use pvec::core::RrbVec;
    use rayon::prelude::*;

    pub fn generate_iter(n: u32) -> impl Iterator<Item = u32> {
        let mut pvec = RrbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_iter().map(|x| x + 1)
    }

    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut pvec = RrbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter().map(|x| x + 1)
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        generate_indexed_iter(n).filter(|_| true)
    }
}

mod pvec {
    use pvec::PVec;
    use rayon::prelude::*;

    pub fn generate_iter(n: u32) -> impl Iterator<Item = u32> {
        let mut pvec = PVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_iter().map(|x| x + 1)
    }

    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut pvec = PVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter().map(|x| x + 1)
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        generate_indexed_iter(n).filter(|_| true)
    }
}

// TODO: add benchmark which supports filtered iterator
// TODO: add benchmark which measures split_time as well
// TODO: add benchmark which evaluates naive append as well
// TODO: stop using modules here, because they introduce additional overhead

macro_rules! make_bench {
    ($generate:ident, $postfix:literal) => {
        fn with_fold_seq(criterion: &mut Criterion) {
            let mut group = criterion.benchmark_group(format!("with_fold_seq_{}", $postfix));
            group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

            let params = vec![
                10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000,
                20000, 40000,
            ];

            for p in params.iter() {
                group.bench_with_input(BenchmarkId::new(STD_VEC, p), p, |b, n| {
                    b.iter(|| {
                        let data = stdvec::generate_iter(*n);
                        util_stdvec::fold_seq(data)
                    });
                });
                group.bench_with_input(BenchmarkId::new(RRBVEC_UNBALANCED, p), p, |b, n| {
                    b.iter(|| {
                        let data = rrbvec::generate_iter(*n);
                        util_rrbvec::fold_seq(data)
                    });
                });
                group.bench_with_input(BenchmarkId::new(PVEC_UNBALANCED, p), p, |b, n| {
                    b.iter(|| {
                        let data = pvec::generate_iter(*n);
                        util_pvec::fold_seq(data)
                    });
                });
            }

            group.finish();
        }

        fn with_fold_par(criterion: &mut Criterion, num_threads: usize) {
            let mut group = criterion.benchmark_group(format!(
                "with_fold_{}_with_thread_num_{}",
                $postfix, num_threads
            ));
            group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

            let params = vec![
                10, 20, 40, 60, 80, 100, 200, 400, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000,
                20000, 40000,
            ];
            for p in params.iter() {
                group.bench_with_input(BenchmarkId::new(STD_VEC, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        b.iter(|| {
                            let data = stdvec::generate_indexed_iter(*n).with_max_len(2048);
                            util_stdvec::fold_par(data)
                        });
                    });
                });
                group.bench_with_input(BenchmarkId::new(RRBVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        b.iter(|| {
                            let data = rrbvec::generate_indexed_iter(*n).with_max_len(2048);
                            util_rrbvec::fold_par(data)
                        });
                    });
                });
                group.bench_with_input(BenchmarkId::new(PVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        b.iter(|| {
                            let data = pvec::generate_indexed_iter(*n).with_max_len(2048);
                            util_pvec::fold_par(data);
                        });
                    });
                });
            }

            group.finish();
        }

        pub fn with_fold_1(criterion: &mut Criterion) {
            with_fold_seq(criterion);
        }

        pub fn with_fold_2(criterion: &mut Criterion) {
            with_fold_par(criterion, 2);
        }

        pub fn with_fold_4(criterion: &mut Criterion) {
            with_fold_par(criterion, 4);
        }

        pub fn with_fold_8(criterion: &mut Criterion) {
            with_fold_par(criterion, 8);
        }

        pub fn with_fold_16(criterion: &mut Criterion) {
            with_fold_par(criterion, 16);
        }
    };
}

mod vec_i {
    use super::*;

    make_bench!(generate_indexed_iter, "iter");
}

mod vec_i_filtered {
    use super::*;

    make_bench!(generate_filter, "filtered");
}

criterion_group!(
    benches,
    vec_i::with_fold_1,
    vec_i::with_fold_2,
    vec_i::with_fold_4,
    vec_i::with_fold_8,
    vec_i::with_fold_16,
    // vec_i_filtered::with_fold_1,
    // vec_i_filtered::with_fold_2,
    // vec_i_filtered::with_fold_4,
    // vec_i_filtered::with_fold_8,
    // vec_i_filtered::with_fold_16,
);
