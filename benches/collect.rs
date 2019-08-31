use criterion::*;
use rayon::prelude::*;

mod util_stdvec {
    use rayon::prelude::*;
    use std::collections::LinkedList;

    /// Do whatever `collect` does by default.
    pub fn collect<T, PI>(pi: PI) -> Vec<T>
    where
        T: Send,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.collect()
    }

    /// Collect a linked list of vectors intermediary.
    pub fn linked_list_collect_vec<T, PI>(pi: PI) -> Vec<T>
    where
        T: Send,
        PI: ParallelIterator<Item = T> + Send,
    {
        let list: LinkedList<Vec<_>> = pi
            .fold(Vec::new, |mut vec, elem| {
                vec.push(elem);
                vec
            })
            .collect();
        list.into_iter().fold(Vec::new(), |mut vec, mut sub| {
            vec.append(&mut sub);
            vec
        })
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
    use std::collections::LinkedList;
    use std::fmt::Debug;

    /// Do whatever `collect` does by default.
    pub fn collect<T, PI>(pi: PI) -> RrbVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        pi.collect()
    }

    /// Collect a linked list of vectors intermediary.
    pub fn linked_list_collect_vec<T, PI>(pi: PI) -> RrbVec<T>
    where
        T: Send + Sync + Clone + Debug,
        PI: ParallelIterator<Item = T> + Send,
    {
        let list: LinkedList<RrbVec<_>> = pi
            .fold(
                || RrbVec::new(),
                |mut vec, elem| {
                    vec.push(elem);
                    vec
                },
            )
            .collect();
        list.into_iter().fold(RrbVec::new(), |mut vec, mut sub| {
            vec.append(&mut sub);
            vec
        })
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

mod stdvec {
    use rayon::prelude::*;
    pub fn generate_indexed_iter(n: u32) -> impl IndexedParallelIterator<Item = u32> {
        (0_u32..n).into_par_iter()
    }

    pub fn generate_filter(n: u32) -> impl ParallelIterator<Item = u32> {
        (0_u32..n).into_par_iter().filter(|_| true)
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

macro_rules! make_bench {
    ($generate:ident, $postfix:literal) => {
        pub fn with_collect(criterion: &mut Criterion) {
            let mut group = criterion.benchmark_group("with_collect".to_owned() + $postfix);
            group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

            let params = vec![10, 30, 50, 100, 500, 1000, 10000, 50000, 100000, 500000];
            for p in params.iter() {
                group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| vec = Some(util_stdvec::collect(stdvec::$generate(*n))));
                    stdvec::check(&vec.unwrap(), *n);
                });
                group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| vec = Some(util_rrbvec::collect(rrbvec::$generate(*n))));
                    rrbvec::check(&vec.unwrap(), *n);
                });
            }

            group.finish();
        }

        pub fn with_linked_list_collect_vec(criterion: &mut Criterion) {
            let mut group =
                criterion.benchmark_group("with_linked_list_collect_vec".to_owned() + $postfix);
            group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

            let params = vec![10, 30, 50, 100, 500, 1000, 10000, 50000, 100000, 500000];
            for p in params.iter() {
                group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| {
                        vec = Some(util_stdvec::linked_list_collect_vec(stdvec::$generate(*n)))
                    });
                    stdvec::check(&vec.unwrap(), *n);
                });
                group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| {
                        vec = Some(util_rrbvec::linked_list_collect_vec(rrbvec::$generate(*n)))
                    });
                    rrbvec::check(&vec.unwrap(), *n);
                });
            }

            group.finish();
        }

        pub fn with_fold(criterion: &mut Criterion) {
            let mut group = criterion.benchmark_group("with_fold".to_owned() + $postfix);
            group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

            let params = vec![10, 30, 50, 100, 500, 1000, 10000, 50000, 100000, 500000];
            for p in params.iter() {
                group.bench_with_input(BenchmarkId::new("std", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| vec = Some(util_stdvec::fold(stdvec::$generate(*n))));
                    stdvec::check(&vec.unwrap(), *n);
                });
                group.bench_with_input(BenchmarkId::new("rrbvec", p), p, |b, n| {
                    let mut vec = None;
                    b.iter(|| vec = Some(util_rrbvec::fold(rrbvec::$generate(*n))));
                    rrbvec::check(&vec.unwrap(), *n);
                });
            }

            group.finish();
        }
    };
}

mod vec_i {
    use super::*;

    pub fn with_collect_into_vec(criterion: &mut Criterion) {
        macro_rules! bench {
            ($group:ident, $p:ident, $module:ident, $name:literal) => {
                $group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                    let mut vec = None;
                    b.iter(|| {
                        let mut v = vec![];
                        $module::generate_indexed_iter(*n).collect_into_vec(&mut v);
                        vec = Some(v);
                    });
                    stdvec::check(&vec.unwrap(), *n);
                });
            };
        }
        let mut group = criterion.benchmark_group("with_collect_into_vec");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

        let params = vec![
            8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16000, 32000, 64000,
        ];

        for p in params.iter() {
            bench!(group, p, stdvec, "std");
            bench!(group, p, rrbvec, "rrbvec");
        }

        group.finish();
    }

    // ToDo: prefix benchmarks to make them unique
    make_bench!(generate_indexed_iter, "iter");
}

mod vec_i_filtered {
    use super::*;

    // ToDo: prefix benchmarks to make them unique
    make_bench!(generate_filter, "filtered");
}

criterion_group!(
    benches,
    vec_i::with_collect_into_vec,
    vec_i::with_collect,
    vec_i::with_linked_list_collect_vec,
    vec_i::with_fold,
    vec_i_filtered::with_collect,
    vec_i_filtered::with_linked_list_collect_vec,
    vec_i_filtered::with_fold,
);
