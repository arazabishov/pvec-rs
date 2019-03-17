#![cfg_attr(test, feature(test))]

#[macro_use]
extern crate criterion;
extern crate dogged;
extern crate im;
extern crate pvec;
extern crate rand;
extern crate test as test_crate;

use criterion::{Criterion, ParameterizedBenchmark};
use dogged::DVec;
use im::Vector;
use rand::{Rng, SeedableRng, XorShiftRng};

use pvec::pvec::PVec;

fn push(criterion: &mut Criterion) {
    criterion.bench(
        "push",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                bencher.iter(|| {
                    let mut vec = Vec::new();

                    for i in 0..*n {
                        vec.push(i);
                    }
                })
            },
            vec![1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("dvec", |bencher, n| {
            bencher.iter(|| {
                let mut vec = DVec::new();

                for i in 0..*n {
                    vec.push(i);
                }
            });
        })
        .with_function("im", |bencher, n| {
            bencher.iter(|| {
                let mut vec = Vector::new();

                for i in 0..*n {
                    vec.push_back(i);
                }
            });
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter(|| {
                let mut vec = PVec::new();

                for i in 0..*n {
                    vec.push(i);
                }
            })
        }),
    );
}

fn push_clone(criterion: &mut Criterion) {
    criterion.bench(
        "push_clone",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                bencher.iter(|| {
                    let mut vec = Vec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..*n {
                        vec.push(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                })
            },
            vec![1000, 5000, 10000, 20000],
        )
        .with_function("dvec", |bencher, n| {
            bencher.iter(|| {
                let mut vec = DVec::new();
                let mut vec_one = vec.clone();

                for i in 0..*n {
                    vec.push(i);
                    vec_one = vec.clone();
                }

                drop(vec_one);
            });
        })
        .with_function("im", |bencher, n| {
            bencher.iter(|| {
                let mut vec = Vector::new();
                let mut vec_one = vec.clone();

                for i in 0..*n {
                    vec.push_back(i);
                    vec_one = vec.clone();
                }

                drop(vec_one);
            });
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter(|| {
                let mut vec = PVec::new();
                let mut vec_one = vec.clone();

                for i in 0..*n {
                    vec.push(i);
                    vec_one = vec.clone();
                }

                drop(vec_one);
            })
        }),
    );
}

fn index_sequentially(criterion: &mut Criterion) {
    criterion.bench(
        "index_sequentially",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                let mut vec = Vec::new();

                for i in 0..*n {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    for i in 0..*n {
                        assert_eq!(vec[i], i * 2);
                    }
                })
            },
            vec![1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("dvec", |bencher, n| {
            let mut vec = DVec::new();

            for i in 0..*n {
                vec.push(i * 2);
            }

            bencher.iter(|| {
                for i in 0..*n {
                    assert_eq!(vec[i], i * 2);
                }
            });
        })
        .with_function("im", |bencher, n| {
            let mut vec = Vector::new();

            for i in 0..*n {
                vec.push_back(i * 2);
            }

            bencher.iter(|| {
                for i in 0..*n {
                    assert_eq!(vec[i], i * 2);
                }
            });
        })
        .with_function("pvec", |bencher, n| {
            let mut vec = PVec::new();

            for i in 0..*n {
                vec.push(i * 2);
            }

            bencher.iter(|| {
                for i in 0..*n {
                    assert_eq!(vec[i], i * 2);
                }
            });
        }),
    );
}

fn index_randomly(criterion: &mut Criterion) {
    criterion.bench(
        "index_randomly",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                let mut vec = Vec::new();
                for i in 0..*n {
                    vec.push(i * 2);
                }

                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                bencher.iter(|| {
                    for _ in 0..*n {
                        let j = (rng.next_u32() as usize) % *n;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            },
            vec![1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("dvec", |bencher, n| {
            let mut vec = DVec::new();
            for i in 0..*n {
                vec.push(i * 2);
            }

            let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
            bencher.iter(|| {
                for _ in 0..*n {
                    let j = (rng.next_u32() as usize) % *n;
                    assert_eq!(*vec.get(j).unwrap(), j * 2);
                }
            });
        })
        .with_function("im", |bencher, n| {
            let mut vec = Vector::new();
            for i in 0..*n {
                vec.push_back(i * 2);
            }

            let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
            bencher.iter(|| {
                for _ in 0..*n {
                    let j = (rng.next_u32() as usize) % *n;
                    assert_eq!(*vec.get(j).unwrap(), j * 2);
                }
            });
        })
        .with_function("pvec", |bencher, n| {
            let mut vec = PVec::new();
            for i in 0..*n {
                vec.push(i * 2);
            }

            let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
            bencher.iter(|| {
                for _ in 0..*n {
                    let j = (rng.next_u32() as usize) % *n;
                    assert_eq!(*vec.get(j).unwrap(), j * 2);
                }
            });
        }),
    );
}

fn append(criterion: &mut Criterion) {
    criterion.bench(
        "append",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                let mut vec_one = Vec::new();

                for i in 0..*n {
                    vec_one.push(i);
                }

                bencher.iter(|| {
                    let mut vec_two = Vec::new();

                    for _ in 0..16 {
                        vec_two.append(&mut vec_one.clone());
                    }

                    drop(vec_two)
                });
            },
            vec![1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("im", |bencher, n| {
            let mut vec_one = Vector::new();

            for i in 0..*n {
                vec_one.push_back(i);
            }

            bencher.iter(|| {
                let mut vec_two = Vector::new();

                for _ in 0..16 {
                    vec_two.append(vec_one.clone());
                }

                drop(vec_two)
            });
        })
        .with_function("pvec", |bencher, n| {
            let mut vec_one = PVec::new();

            for i in 0..*n {
                vec_one.push(i);
            }

            bencher.iter(|| {
                let mut vec_two = PVec::new();

                for _ in 0..16 {
                    vec_two.append(&mut vec_one.clone());
                }

                drop(vec_two)
            });
        }),
    );
}

fn append_push(criterion: &mut Criterion) {
    criterion.bench(
        "append_push",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                let mut vec_one = Vec::new();

                for i in 0..*n {
                    vec_one.push(i);
                }

                bencher.iter(|| {
                    let mut vec_two = Vec::new();

                    for i in 0..1024 {
                        if i % 2 == 0 {
                            vec_two.push(i);
                        } else {
                            vec_two.append(&mut vec_one.clone());
                        }
                    }

                    drop(vec_two)
                });
            },
            vec![1000, 5000, 10000, 50000],
        )
        .with_function("im", |bencher, n| {
            let mut vec_one = Vector::new();

            for i in 0..*n {
                vec_one.push_back(i);
            }

            bencher.iter(|| {
                let mut vec_two = Vector::new();

                for i in 0..1024 {
                    if i % 2 == 0 {
                        vec_two.push_back(i);
                    } else {
                        vec_two.append(vec_one.clone());
                    }
                }

                drop(vec_two)
            });
        })
        .with_function("pvec", |bencher, n| {
            let mut vec_one = PVec::new();

            for i in 0..*n {
                vec_one.push(i);
            }

            bencher.iter(|| {
                let mut vec_two = PVec::new();

                for i in 0..1024 {
                    if i % 2 == 0 {
                        vec_two.push(i);
                    } else {
                        vec_two.append(&mut vec_one.clone());
                    }
                }

                drop(vec_two)
            });
        }),
    );
}

criterion_group!(
    benches,
    push,
    push_clone,
    index_sequentially,
    index_randomly,
    append,
    append_push
);
criterion_main!(benches);
