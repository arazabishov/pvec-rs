// ToDo: import rest of the benchmars from rayon/pythagoras

use num::Integer;

use rayon::prelude::*;
use rayon::range::Iter;

use criterion::{criterion_group, Benchmark, Criterion};

use std::ops::Add;
use std::usize;

mod stdvec {
    use super::*;

    /// Same as par_euclid, without using rayon.
    pub fn euclid() -> u32 {
        (1u32..2000)
            .map(|m| {
                (1..m)
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .fold(0, Add::add)
            })
            .fold(0, Add::add)
    }

    /// Use Euclid's formula to count Pythagorean triples
    ///
    /// https://en.wikipedia.org/wiki/Pythagorean_triple#Generating_a_triple
    ///
    /// For coprime integers m and n, with m > n and m-n is odd, then
    ///     a = m²-n², b = 2mn, c = m²+n²
    ///
    /// This is a coprime triple.  Multiplying by factors k covers all triples.
    pub fn par_euclid<FM, M, FN, N>(map_m: FM, map_n: FN) -> u32
    where
        FM: FnOnce(Iter<u32>) -> M,
        M: ParallelIterator<Item = u32>,
        FN: Fn(Iter<u32>) -> N + Sync,
        N: ParallelIterator<Item = u32>,
    {
        map_m((1u32..2000).into_par_iter())
            .map(|m| -> u32 {
                map_n((1..m).into_par_iter())
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .sum()
            })
            .sum()
    }

    /// Same as par_euclid, without tweaking split lengths
    pub fn par_euclid_weightless() -> u32 {
        (1u32..2000)
            .into_par_iter()
            .map(|m| -> u32 {
                (1..m)
                    .into_par_iter()
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .sum()
            })
            .sum()
    }
}

mod rrbvec {
    use super::*;
    use pvec::core::iter::RrbVecParIter;
    use pvec::core::RrbVec;

    /// Same as par_euclid, without using rayon.
    pub fn euclid() -> u32 {
        let mut source = RrbVec::new();
        for i in 1u32..2000 {
            source.push(i);
        }

        source
            .into_iter()
            .map(|m| {
                (1..m)
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .fold(0, Add::add)
            })
            .fold(0, Add::add)
    }

    /// Use Euclid's formula to count Pythagorean triples
    ///
    /// https://en.wikipedia.org/wiki/Pythagorean_triple#Generating_a_triple
    ///
    /// For coprime integers m and n, with m > n and m-n is odd, then
    ///     a = m²-n², b = 2mn, c = m²+n²
    ///
    /// This is a coprime triple.  Multiplying by factors k covers all triples.
    pub fn par_euclid<FM, M, FN, N>(map_m: FM, map_n: FN) -> u32
    where
        FM: FnOnce(RrbVecParIter<u32>) -> M,
        M: ParallelIterator<Item = u32>,
        FN: Fn(RrbVecParIter<u32>) -> N + Sync,
        N: ParallelIterator<Item = u32>,
    {
        let mut source = RrbVec::new();
        for i in 1u32..2000 {
            source.push(i);
        }

        map_m(source.into_par_iter())
            .map(|m| -> u32 {
                let mut other = RrbVec::new();
                for i in 1..m {
                    other.push(i);
                }

                map_n(other.into_par_iter())
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .sum()
            })
            .sum()
    }

    /// Same as par_euclid, without tweaking split lengths
    pub fn par_euclid_weightless() -> u32 {
        let mut source = RrbVec::new();
        for i in 1u32..2000 {
            source.push(i);
        }
        source
            .into_par_iter()
            .map(|m| -> u32 {
                (1..m)
                    .into_par_iter()
                    .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                    .map(|n| 4000000 / (m * m + n * n))
                    .sum()
            })
            .sum()
    }
}

fn euclid_serial(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("euclid_serial");
    group.bench_function("std", |b| {
        let count = stdvec::euclid();
        b.iter(|| assert_eq!(stdvec::euclid(), count))
    });
    group.bench_function("rrbvec", |b| {
        let count = rrbvec::euclid();
        b.iter(|| assert_eq!(rrbvec::euclid(), count))
    });
    group.finish();
}

fn euclid_faux_serial(criterion: &mut Criterion) {
    use pvec::core::iter::RrbVecParIter;
    criterion.bench(
        "euclid_faux_serial",
        Benchmark::new("std", |bencher| {
            let count = stdvec::euclid();
            let serial = |r: Iter<u32>| r.with_min_len(usize::MAX);
            bencher.iter(|| assert_eq!(stdvec::par_euclid(&serial, &serial), count))
        })
        .with_function("rrbvec", |bencher| {
            let count = stdvec::euclid();
            let serial = |r: RrbVecParIter<u32>| r.with_min_len(usize::MAX);
            bencher.iter(|| assert_eq!(rrbvec::par_euclid(&serial, &serial), count))
        }),
    );
}

fn euclid_parallel_weightless(criterion: &mut Criterion) {
    criterion.bench(
        "euclid_parallel_weightless",
        Benchmark::new("std", |bencher| {
            let count = stdvec::euclid();
            bencher.iter(|| assert_eq!(stdvec::par_euclid_weightless(), count))
        })
        .with_function("rrbvec", |bencher| {
            let count = stdvec::euclid();
            bencher.iter(|| assert_eq!(rrbvec::par_euclid_weightless(), count))
        }),
    );
}

fn euclid_parallel_one(criterion: &mut Criterion) {
    criterion.bench(
        "euclid_parallel_one",
        Benchmark::new("std", |bencher| {
            let count = stdvec::euclid();
            bencher.iter(|| assert_eq!(stdvec::par_euclid(|m| m, |n| n), count))
        })
        .with_function("rrbvec", |bencher| {
            let count = stdvec::euclid();
            bencher.iter(|| assert_eq!(rrbvec::par_euclid(|m| m, |n| n), count))
        }),
    );
}

fn euclid_parallel_outer(criterion: &mut Criterion) {
    use pvec::core::iter::RrbVecParIter;
    criterion.bench(
        "euclid_parallel_outer",
        Benchmark::new("std", |bencher| {
            let count = stdvec::euclid();
            let parallel = |r: Iter<u32>| r.with_max_len(1);
            bencher.iter(|| assert_eq!(stdvec::par_euclid(&parallel, |n| n), count))
        })
        .with_function("rrbvec", |bencher| {
            let count = stdvec::euclid();
            let parallel = |r: RrbVecParIter<u32>| r.with_max_len(1);
            bencher.iter(|| assert_eq!(rrbvec::par_euclid(&parallel, |n| n), count))
        }),
    );
}

fn euclid_parallel_full(criterion: &mut Criterion) {
    use pvec::core::iter::RrbVecParIter;
    criterion.bench(
        "euclid_parallel_full",
        Benchmark::new("std", |bencher| {
            let count = stdvec::euclid();
            let parallel = |r: Iter<u32>| r.with_max_len(1);
            bencher.iter(|| assert_eq!(stdvec::par_euclid(&parallel, &parallel), count));
        })
        .with_function("rrbvec", |bencher| {
            let count = stdvec::euclid();
            let parallel = |r: RrbVecParIter<u32>| r.with_max_len(1);
            bencher.iter(|| assert_eq!(rrbvec::par_euclid(&parallel, &parallel), count));
        }),
    );
}

criterion_group!(
    benches,
    euclid_serial,
    euclid_faux_serial,
    euclid_parallel_weightless,
    euclid_parallel_one,
    euclid_parallel_outer,
    euclid_parallel_full,
);
