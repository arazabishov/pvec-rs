#![cfg_attr(test, feature(test))]

#[macro_use]
extern crate criterion;
extern crate dogged;
extern crate im;
extern crate pvec;
extern crate rand;
extern crate test as test_crate;

use criterion::{black_box, BatchSize, Criterion, ParameterizedBenchmark};
use dogged::DVec;
use im::Vector as IVec;
use rand::{Rng, SeedableRng, XorShiftRng};

use pvec::PVec;

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
            vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000],
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
                let mut vec = IVec::new();

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
            vec![100, 500, 1000, 5000, 10000, 20000],
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
                let mut vec = IVec::new();
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

fn pop_clone(criterion: &mut Criterion) {
    criterion.bench(
        "pop_clone",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                let mut vec = Vec::new();

                for i in 0..*n {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    let mut vec_one = vec.clone();
                    let mut vec_two = vec_one.clone();

                    for _ in 0..*n {
                        let _ = vec_one.pop();
                        vec_two = vec_one.clone();
                    }

                    drop(vec_two);
                })
            },
            vec![100, 500, 1000, 5000, 10000, 20000],
        )
        .with_function("im-rs", |bencher, n| {
            let mut vec = IVec::new();

            for i in 0..*n {
                vec.push_back(i * 2);
            }

            bencher.iter(|| {
                let mut vec_one = vec.clone();
                let mut vec_two = vec_one.clone();

                for _ in 0..*n {
                    let _ = vec_one.pop_back();
                    vec_two = vec_one.clone();
                }

                drop(vec_two);
            });
        })
        .with_function("pvec", |bencher, n| {
            let mut vec = PVec::new();

            for i in 0..*n {
                vec.push(i * 2);
            }

            bencher.iter(|| {
                let mut vec_one = vec.clone();
                let mut vec_two = vec_one.clone();

                for _ in 0..*n {
                    let _ = vec_one.pop();
                    vec_two = vec_one.clone();
                }

                drop(vec_two);
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
                bencher.iter_batched(
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
                    },
                    BatchSize::SmallInput,
                )
            },
            vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("dvec", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut vec = DVec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
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
        })
        .with_function("im-rs", |bencher, n| {
            bencher.iter_batched(
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
                },
                BatchSize::SmallInput,
            )
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter_batched(
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
                },
                BatchSize::SmallInput,
            )
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
                        let _ = vec.get(j);
                    }
                });
            },
            vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000],
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
                    let _ = vec.get(j);
                }
            });
        })
        .with_function("im-rs", |bencher, n| {
            let mut vec = IVec::new();
            for i in 0..*n {
                vec.push_back(i * 2);
            }

            let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
            bencher.iter(|| {
                for _ in 0..*n {
                    let j = (rng.next_u32() as usize) % *n;
                    let _ = vec.get(j);
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
                    let _ = vec.get(j);
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
                bencher.iter_batched(
                    || {
                        let mut input = Vec::new();
                        for _ in 0..16 {
                            let mut vec = Vec::new();

                            for i in 0..*n {
                                vec.push(i);
                            }

                            input.push(vec)
                        }
                        input
                    },
                    |mut data| {
                        let mut vec_two = Vec::new();

                        for mut input in data.iter_mut() {
                            vec_two.append(&mut input);
                        }

                        vec_two
                    },
                    BatchSize::SmallInput,
                );
            },
            vec![8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192],
        )
        .with_function("im-rs", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut input = Vec::new();
                    for _ in 0..16 {
                        let mut vec = IVec::new();

                        for i in 0..*n {
                            vec.push_back(i);
                        }

                        input.push(vec)
                    }
                    input
                },
                |data| {
                    let mut vec_two = IVec::new();

                    for mut input in data.into_iter() {
                        vec_two.append(input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            );
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut input = Vec::new();
                    for _ in 0..16 {
                        let mut vec = PVec::new();

                        for i in 0..*n {
                            vec.push(i);
                        }

                        input.push(vec)
                    }
                    input
                },
                |data| {
                    let mut vec_two = PVec::new();

                    for mut input in data.into_iter() {
                        vec_two.append(&mut input);
                    }

                    vec_two
                },
                BatchSize::SmallInput,
            );
        }),
    );
}

fn append_clone(criterion: &mut Criterion) {
    criterion.bench(
        "append_clone",
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
            vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("im-rs", |bencher, n| {
            let mut vec_one = IVec::new();

            for i in 0..*n {
                vec_one.push_back(i);
            }

            bencher.iter(|| {
                let mut vec_two = IVec::new();

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
            vec![100, 500, 1000, 5000, 10000, 50000],
        )
        .with_function("im-rs", |bencher, n| {
            let mut vec_one = IVec::new();

            for i in 0..*n {
                vec_one.push_back(i);
            }

            bencher.iter(|| {
                let mut vec_two = IVec::new();

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

fn iterator(criterion: &mut Criterion) {
    criterion.bench(
        "iterator",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                bencher.iter_batched(
                    || {
                        let mut vec = Vec::new();

                        for i in 0..*n {
                            vec.push(i * 2);
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
            },
            vec![100, 500, 1000, 5000, 10000, 50000, 100000, 200000, 500000],
        )
        .with_function("im-rs", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut vec = IVec::new();

                    for i in 0..*n {
                        vec.push_back(i * 2);
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
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut vec = PVec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
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
        }),
    );
}

fn split_off(criterion: &mut Criterion) {
    criterion.bench(
        "split_off",
        ParameterizedBenchmark::new(
            "std",
            |bencher, n| {
                bencher.iter_batched(
                    || {
                        let mut vec = Vec::new();

                        for i in 0..*n {
                            vec.push(i * 2);
                        }

                        vec
                    },
                    |mut data| {
                        for i in (0..data.len()).rev() {
                            data.split_off(i);
                        }

                        data
                    },
                    BatchSize::SmallInput,
                )
            },
            vec![
                32, 64, 128, 512, 768, 1024, 2048, 4096, 10000, 20000, 40000, 80000, 120000, 500000,
            ],
        )
        .with_function("im-rs", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut vec = IVec::new();

                    for i in 0..*n {
                        vec.push_back(i * 2);
                    }

                    vec
                },
                |mut data| {
                    for i in (0..data.len()).rev() {
                        data.split_off(i);
                    }

                    data
                },
                BatchSize::SmallInput,
            )
        })
        .with_function("pvec", |bencher, n| {
            bencher.iter_batched(
                || {
                    let mut vec = PVec::new();

                    for i in 0..*n {
                        vec.push(i * 2);
                    }

                    vec
                },
                |mut data| {
                    for i in (0..data.len()).rev() {
                        data.split_off(i);
                    }

                    data
                },
                BatchSize::SmallInput,
            )
        }),
    );
}

criterion_group!(
    benches,
    push,
    push_clone,
    pop_clone,
    index_sequentially,
    iterator,
    index_randomly,
    split_off,
    append_clone,
    append_push,
    append
);
criterion_main!(benches);
