#![cfg_attr(test, feature(test))]

extern crate dogged;
extern crate im;
extern crate pvec;
extern crate rand;
extern crate test as test_crate;

use dogged::DVec;
use im::Vector;
use rand::{Rng, SeedableRng, XorShiftRng};

use pvec::pvec::PVec;

macro_rules! push {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = Vec::new();

                    for i in 0..N {
                        vec.push(i);
                    }
                });
            }

            #[bench]
            fn dogged(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = DVec::new();

                    for i in 0..N {
                        vec.push(i);
                    }
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = PVec::new();

                    for i in 0..N {
                        vec.push(i);
                    }
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = Vector::new();

                    for i in 0..N {
                        vec.push_back(i);
                    }
                });
            }
        }
    };
}

macro_rules! push_clone {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = Vec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..N {
                        vec.push(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                });
            }

            #[bench]
            fn dogged(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = DVec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..N {
                        vec.push(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = PVec::new();
                    let mut vec_one = vec.clone();

                    for i in 0..N {
                        vec.push(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                bencher.iter(|| {
                    let mut vec = Vector::new();
                    let mut vec_one = vec.clone();

                    for i in 0..N {
                        vec.push_back(i);
                        vec_one = vec.clone();
                    }

                    drop(vec_one);
                });
            }
        }
    };
}

macro_rules! pop {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec = Vec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    let mut vector = vec.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vector.pop().unwrap(), i * 2);
                    }
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec = PVec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    let mut vector = vec.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vector.pop().unwrap(), i * 2);
                    }
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec = Vector::new();

                for i in 0..N {
                    vec.push_back(i * 2);
                }

                bencher.iter(|| {
                    let mut vector = vec.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vector.pop_back().unwrap(), i * 2);
                    }
                });
            }
        }
    };
}

macro_rules! pop_clone {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec = Vec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    let mut vec_one = vec.clone();
                    let mut vec_two = vec_one.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vec_one.pop().unwrap(), i * 2);
                        vec_two = vec_one.clone();
                    }

                    drop(vec_two);
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec = PVec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    let mut vec_one = vec.clone();
                    let mut vec_two = vec_one.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vec_one.pop().unwrap(), i * 2);
                        vec_two = vec_one.clone();
                    }

                    drop(vec_two);
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec = Vector::new();

                for i in 0..N {
                    vec.push_back(i * 2);
                }

                bencher.iter(|| {
                    let mut vec_one = vec.clone();
                    let mut vec_two = vec_one.clone();

                    for i in (0..N).rev() {
                        assert_eq!(vec_one.pop_back().unwrap(), i * 2);
                        vec_two = vec_one.clone();
                    }

                    drop(vec_two);
                });
            }
        }
    };
}

macro_rules! index_sequentially {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec = Vec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    for i in 0..N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }

            #[bench]
            fn dogged(bencher: &mut test_crate::Bencher) {
                let mut vec = DVec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    for i in 0..N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec = PVec::new();

                for i in 0..N {
                    vec.push(i * 2);
                }

                bencher.iter(|| {
                    for i in 0..N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec = Vector::new();

                for i in 0..N {
                    vec.push_back(i * 2);
                }

                bencher.iter(|| {
                    for i in 0..N {
                        assert_eq!(vec[i], i * 2);
                    }
                });
            }
        }
    };
}

macro_rules! index_randomly {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec = Vec::new();
                for i in 0..N {
                    vec.push(i * 2);
                }

                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                bencher.iter(|| {
                    for _ in 0..N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }

            #[bench]
            fn dogged(bencher: &mut test_crate::Bencher) {
                let mut vec = DVec::new();
                for i in 0..N {
                    vec.push(i * 2);
                }

                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                bencher.iter(|| {
                    for _ in 0..N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec = PVec::new();
                for i in 0..N {
                    vec.push(i * 2);
                }

                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                bencher.iter(|| {
                    for _ in 0..N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec = Vector::new();
                for i in 0..N {
                    vec.push_back(i * 2);
                }

                let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
                bencher.iter(|| {
                    for _ in 0..N {
                        let j = (rng.next_u32() as usize) % N;
                        assert_eq!(*vec.get(j).unwrap(), j * 2);
                    }
                });
            }
        }
    };
}

macro_rules! append {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec_one = Vec::new();

                for i in 0..N {
                    vec_one.push(i);
                }

                bencher.iter(|| {
                    let mut vec_two = Vec::new();

                    for _ in 0..16 {
                        vec_two.append(&mut vec_one.clone());
                    }

                    drop(vec_two)
                });
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec_one = PVec::new();

                for i in 0..N {
                    vec_one.push(i);
                }

                bencher.iter(|| {
                    let mut vec_two = PVec::new();

                    for _ in 0..16 {
                        vec_two.append(&mut vec_one.clone());
                    }

                    drop(vec_two)
                });
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec_one = Vector::new();

                for i in 0..N {
                    vec_one.push_back(i);
                }

                bencher.iter(|| {
                    let mut vec_two = Vector::new();

                    for _ in 0..16 {
                        vec_two.append(vec_one.clone());
                    }

                    drop(vec_two)
                });
            }
        }
    };
}

macro_rules! append_push {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use super::*;

            const N: usize = $N;

            #[bench]
            fn standard(bencher: &mut test_crate::Bencher) {
                let mut vec_one = Vec::new();

                for i in 0..N {
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
            }

            #[bench]
            fn pvec(bencher: &mut test_crate::Bencher) {
                let mut vec_one = PVec::new();

                for i in 0..N {
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
            }

            #[bench]
            fn im(bencher: &mut test_crate::Bencher) {
                let mut vec_one = Vector::new();

                for i in 0..N {
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
            }
        }
    };
}

push!(push_5000, 5000);
push!(push_50000, 50000);
push!(push_500000, 500000);
push_clone!(push_clone_5000, 5000);

pop!(pop_5000, 5000);
pop!(pop_50000, 50000);
pop!(pop_500000, 500000);
pop_clone!(pop_clone_5000, 5000);

index_sequentially!(index_sequentially_5000, 5000);
index_sequentially!(index_sequentially_50000, 50000);
index_sequentially!(index_sequentially_500000, 500000);

index_randomly!(index_randomly_5000, 5000);
index_randomly!(index_randomly_50000, 50000);
index_randomly!(index_randomly_500000, 500000);

append!(append_500000, 500000);
append_push!(append_push_50000, 50000);
