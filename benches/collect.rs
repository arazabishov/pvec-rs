use super::*;
use rayon::prelude::*;

mod util_stdvec {
    use rayon::prelude::*;

    /// Do whatever `collect` does by default.
    pub fn collect<T, PI>(pi: PI) -> Vec<T>
    where
        T: Send,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.collect()
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold<T, PI>(pi: PI) -> Vec<T>
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

    /// Do whatever `collect` does by default.
    pub fn collect<T, PI>(pi: PI) -> RrbVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.collect()
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold<T, PI>(pi: PI) -> RrbVec<T>
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

    /// Do whatever `collect` does by default.
    pub fn collect<T, PI>(pi: PI) -> PVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.collect()
    }

    /// Fold into vectors and then reduce them together.
    pub fn fold<T, PI>(pi: PI) -> PVec<T>
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
    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut vec = Vec::new();
        for i in 0_u32..n {
            vec.push(i);
        }
        vec.into_par_iter()
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        let mut vec = Vec::new();
        for i in 0_u32..n {
            vec.push(i);
        }
        vec.into_par_iter().filter(|_| true)
    }

    pub fn check(v: &[u32], n: u32) {
        assert!(v.iter().cloned().eq(0..n));
    }
}

mod rrbvec {
    use pvec::core::RrbVec;
    use rayon::prelude::*;

    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut pvec = RrbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter()
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        let mut pvec = RrbVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter().filter(|_| true)
    }

    pub fn check(v: &RrbVec<u32>, n: u32) {
        for i in 0..n {
            assert_eq!(*v.get(i as usize).unwrap(), i);
        }
    }
}

mod pvec {
    use pvec::PVec;
    use rayon::prelude::*;

    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        let mut pvec = PVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter()
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        let mut pvec = PVec::new();
        for i in 0_u32..n {
            pvec.push(i);
        }
        pvec.into_par_iter().filter(|_| true)
    }

    pub fn check(v: &PVec<u32>, n: u32) {
        for i in 0..n {
            assert_eq!(*v.get(i as usize).unwrap(), i);
        }
    }
}

macro_rules! make_bench {
    ($generate:ident, $postfix:literal) => {
        fn with_collect(criterion: &mut Criterion, num_threads: usize) {
            let mut group = criterion.benchmark_group(format!(
                "with_collect_{}_with_thread_num_{}",
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
                        let mut vec = None;
                        b.iter(|| vec = Some(util_stdvec::collect(stdvec::$generate(*n))));
                        stdvec::check(&vec.unwrap(), *n);
                    });
                });
                group.bench_with_input(BenchmarkId::new(RRBVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        let mut vec = None;
                        b.iter(|| vec = Some(util_rrbvec::collect(rrbvec::$generate(*n))));
                        rrbvec::check(&vec.unwrap(), *n);
                    });
                });
                group.bench_with_input(BenchmarkId::new(PVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        let mut vec = None;
                        b.iter(|| vec = Some(util_pvec::collect(pvec::$generate(*n))));
                        pvec::check(&vec.unwrap(), *n);
                    });
                });
            }

            group.finish();
        }

        fn with_fold(criterion: &mut Criterion, num_threads: usize) {
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
                        let mut vec = None;
                        b.iter_batched(
                            || stdvec::generate_indexed_iter(*n).with_min_len(2048),
                            |data| vec = Some(util_stdvec::fold(data)),
                            BatchSize::SmallInput,
                        );
                        stdvec::check(&vec.unwrap(), *n);
                    });
                });
                group.bench_with_input(BenchmarkId::new(RRBVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        let mut vec = None;
                        b.iter_batched(
                            || rrbvec::generate_indexed_iter(*n).with_min_len(2048),
                            |data| vec = Some(util_rrbvec::fold(data)),
                            BatchSize::SmallInput,
                        );
                        rrbvec::check(&vec.unwrap(), *n);
                    });
                });
                group.bench_with_input(BenchmarkId::new(PVEC_UNBALANCED, p), p, |b, n| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        let mut vec = None;
                        b.iter_batched(
                            || pvec::generate_indexed_iter(*n).with_min_len(2048),
                            |data| vec = Some(util_pvec::fold(data)),
                            BatchSize::SmallInput,
                        );
                        pvec::check(&vec.unwrap(), *n);
                    });
                });
            }

            group.finish();
        }

        pub fn with_collect_1(criterion: &mut Criterion) {
            with_collect(criterion, 1);
        }

        pub fn with_collect_2(criterion: &mut Criterion) {
            with_collect(criterion, 2);
        }

        pub fn with_collect_4(criterion: &mut Criterion) {
            with_collect(criterion, 4);
        }

        pub fn with_collect_8(criterion: &mut Criterion) {
            with_collect(criterion, 8);
        }

        pub fn with_collect_16(criterion: &mut Criterion) {
            with_collect(criterion, 16);
        }

        pub fn with_collect_32(criterion: &mut Criterion) {
            with_collect(criterion, 32);
        }

        pub fn with_collect_64(criterion: &mut Criterion) {
            with_collect(criterion, 64);
        }

        pub fn with_fold_1(criterion: &mut Criterion) {
            with_fold(criterion, 1);
        }

        pub fn with_fold_2(criterion: &mut Criterion) {
            with_fold(criterion, 2);
        }

        pub fn with_fold_4(criterion: &mut Criterion) {
            with_fold(criterion, 4);
        }

        pub fn with_fold_8(criterion: &mut Criterion) {
            with_fold(criterion, 8);
        }

        pub fn with_fold_16(criterion: &mut Criterion) {
            with_fold(criterion, 16);
        }

        pub fn with_fold_32(criterion: &mut Criterion) {
            with_fold(criterion, 32);
        }

        pub fn with_fold_64(criterion: &mut Criterion) {
            with_fold(criterion, 64);
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
    vec_i::with_collect_1,
    vec_i::with_collect_2,
    vec_i::with_collect_4,
    vec_i::with_collect_8,
    vec_i::with_collect_16,
    vec_i::with_collect_32,
    vec_i::with_collect_64,
    vec_i::with_fold_1,
    vec_i::with_fold_2,
    vec_i::with_fold_4,
    vec_i::with_fold_8,
    vec_i::with_fold_16,
    vec_i::with_fold_32,
    vec_i::with_fold_64,
    vec_i_filtered::with_collect_1,
    vec_i_filtered::with_collect_2,
    vec_i_filtered::with_collect_4,
    vec_i_filtered::with_collect_8,
    vec_i_filtered::with_collect_16,
    vec_i_filtered::with_collect_32,
    vec_i_filtered::with_collect_64,
    vec_i_filtered::with_fold_1,
    vec_i_filtered::with_fold_2,
    vec_i_filtered::with_fold_4,
    vec_i_filtered::with_fold_8,
    vec_i_filtered::with_fold_16,
    vec_i_filtered::with_fold_32,
    vec_i_filtered::with_fold_64,
);
