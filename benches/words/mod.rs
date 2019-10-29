use criterion::*;

use pvec::core::RbVec;
use pvec::core::RrbVec;
use pvec::PVec;
use rayon::prelude::*;
use std::fs;

use super::*;

fn is_palindrome(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    let mut chars_f = word.chars();
    let mut chars_b = word.chars().rev();

    let mut i = 0;
    let mut j = word.len() - 1;

    while j >= i {
        let ch_f = match chars_f.next() {
            Some(ch) => ch,
            None => break,
        };

        let ch_b = match chars_b.next() {
            Some(ch) => ch,
            None => break,
        };

        if ch_f != ch_b {
            return false;
        }

        i += 1;
        j -= 1;
    }

    true
}

fn words_map_seq(criterion: &mut Criterion) {
    *criterion = Criterion::default().with_plots().sample_size(50);

    let mut group = criterion.benchmark_group(format!("words_map_with_thread_num_1"));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let file = fs::read_to_string("benches/words/words.txt").expect("Oops, something went wrong");
    let lines = file.lines();

    macro_rules! bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || lines.clone().take(*n).collect::<$vec<&str>>(),
                    |words| {
                        words
                            .into_iter()
                            .map(|it| (it, is_palindrome(&it)))
                            .collect::<$vec<(&str, bool)>>()
                    },
                    BatchSize::SmallInput,
                );
            });
        };
    }

    let params = vec![
        10000, 20000, 40000, 60000, 80000, 100000, 200000, 300000, 370103,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, Vec);
        bench!(PVEC_UNBALANCED, p, PVec);
        bench!(RRBVEC_BALANCED, p, RbVec);
        bench!(RRBVEC_UNBALANCED, p, RrbVec);
    }

    group.finish();
}

fn words_map_par(criterion: &mut Criterion, num_threads: usize) {
    *criterion = Criterion::default().with_plots().sample_size(50);

    let mut group = criterion.benchmark_group(format!("words_map_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let file = fs::read_to_string("benches/words/words.txt").expect("Oops, something went wrong");
    let lines = file.lines();

    macro_rules! bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                pool.install(|| {
                    b.iter_batched(
                        || lines.clone().take(*n).collect::<$vec<&str>>(),
                        |words| {
                            words
                                .into_par_iter()
                                .map(|it| (it, is_palindrome(&it)))
                                .fold($vec::new, |mut vec, x| {
                                    vec.push(x);
                                    vec
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
        10000, 20000, 40000, 60000, 80000, 100000, 200000, 300000, 370103,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, Vec);
        bench!(PVEC_UNBALANCED, p, PVec);
        bench!(RRBVEC_BALANCED, p, RbVec);
        bench!(RRBVEC_UNBALANCED, p, RrbVec);
    }

    group.finish();
}

fn words_map_1(criterion: &mut Criterion) {
    words_map_seq(criterion);
}

fn words_map_2(criterion: &mut Criterion) {
    words_map_par(criterion, 2);
}

fn words_map_4(criterion: &mut Criterion) {
    words_map_par(criterion, 4);
}

fn words_map_8(criterion: &mut Criterion) {
    words_map_par(criterion, 8);
}

fn words_filter_seq(criterion: &mut Criterion) {
    *criterion = Criterion::default().with_plots().sample_size(50);

    let mut group = criterion.benchmark_group(format!("words_filter_with_thread_num_1"));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let file = fs::read_to_string("benches/words/words.txt").expect("Oops, something went wrong");
    let lines = file.lines();

    macro_rules! bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                b.iter_batched(
                    || lines.clone().take(*n).collect::<$vec<&str>>(),
                    |words| {
                        words
                            .into_iter()
                            .filter(|it| is_palindrome(&it))
                            .collect::<$vec<&str>>()
                    },
                    BatchSize::SmallInput,
                );
            });
        };
    }

    let params = vec![
        10000, 20000, 40000, 60000, 80000, 100000, 200000, 300000, 370103,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, Vec);
        bench!(PVEC_UNBALANCED, p, PVec);
        bench!(RRBVEC_BALANCED, p, RbVec);
        bench!(RRBVEC_UNBALANCED, p, RrbVec);
    }

    group.finish();
}

fn words_filter_par(criterion: &mut Criterion, num_threads: usize) {
    *criterion = Criterion::default().with_plots().sample_size(50);

    let mut group =
        criterion.benchmark_group(format!("words_filter_with_thread_num_{}", num_threads));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let file = fs::read_to_string("benches/words/words.txt").expect("Oops, something went wrong");
    let lines = file.lines();

    macro_rules! bench {
        ($name:ident, $p:ident, $vec:ident) => {
            group.bench_with_input(BenchmarkId::new($name, $p), $p, |b, n| {
                pool.install(|| {
                    b.iter_batched(
                        || lines.clone().take(*n).collect::<$vec<&str>>(),
                        |words| {
                            words
                                .into_par_iter()
                                .filter(|it| is_palindrome(&it))
                                .fold($vec::new, |mut vec, x| {
                                    vec.push(x);
                                    vec
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
        10000, 20000, 40000, 60000, 80000, 100000, 200000, 300000, 370103,
    ];

    for p in params.iter() {
        bench!(STD_VEC, p, Vec);
        bench!(PVEC_UNBALANCED, p, PVec);
        bench!(RRBVEC_BALANCED, p, RbVec);
        bench!(RRBVEC_UNBALANCED, p, RrbVec);
    }

    group.finish();
}

fn words_filter_1(criterion: &mut Criterion) {
    words_filter_seq(criterion);
}

fn words_filter_2(criterion: &mut Criterion) {
    words_filter_par(criterion, 2);
}

fn words_filter_4(criterion: &mut Criterion) {
    words_filter_par(criterion, 4);
}

fn words_filter_8(criterion: &mut Criterion) {
    words_filter_par(criterion, 8);
}

criterion_group!(
    benches,
    words_map_1,
    words_map_2,
    words_map_4,
    words_map_8,
    words_filter_1,
    words_filter_2,
    words_filter_4,
    words_filter_8,
);
