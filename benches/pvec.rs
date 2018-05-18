#![cfg_attr(test, feature(test))]

extern crate dogged;
extern crate persistent;
extern crate rand;
extern crate test as test_crate;

use dogged::DVec;
use persistent::pvec::PVec;
use rand::{Rng, SeedableRng, XorShiftRng};

fn push_vec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = Vec::new();

        for i in 0..n {
            vec.push(i);
        }
    });
}

fn push_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = PVec::new();

        for i in 0..n {
            vec.push(i);
        }
    });
}

fn push_dvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = DVec::new();

        for i in 0..n {
            vec.push(i);
        }
    });
}

#[bench]
fn push_vec_5000(bencher: &mut test_crate::Bencher) {
    push_vec(bencher, 5000);
}

#[bench]
fn push_pvec_5000(bencher: &mut test_crate::Bencher) {
    push_pvec(bencher, 5000);
}

#[bench]
fn push_dvec_5000(bencher: &mut test_crate::Bencher) {
    push_dvec(bencher, 5000);
}

#[bench]
fn push_vec_50000(bencher: &mut test_crate::Bencher) {
    push_vec(bencher, 50000);
}

#[bench]
fn push_pvec_50000(bencher: &mut test_crate::Bencher) {
    push_pvec(bencher, 50000);
}

#[bench]
fn push_dvec_50000(bencher: &mut test_crate::Bencher) {
    push_dvec(bencher, 50000);
}

#[bench]
fn push_vec_500000(bencher: &mut test_crate::Bencher) {
    push_vec(bencher, 500000);
}

#[bench]
fn push_pvec_500000(bencher: &mut test_crate::Bencher) {
    push_pvec(bencher, 500000);
}

#[bench]
fn push_dvec_500000(bencher: &mut test_crate::Bencher) {
    push_dvec(bencher, 500000);
}


fn push_clone_vec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = Vec::new();
        let mut vec_one = vec.clone();

        for i in 0..n {
            vec.push(i);
            vec_one = vec.clone();
        }

        drop(vec_one);
    });
}

fn push_clone_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = PVec::new();
        let mut vec_one = vec.clone();

        for i in 0..n {
            vec.push(i);
            vec_one = vec.clone();
        }

        drop(vec_one);
    });
}

fn push_clone_dvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = DVec::new();
        let mut vec_one = vec.clone();

        for i in 0..n {
            vec.push(i);
            vec_one = vec.clone();
        }

        drop(vec_one);
    });
}

#[bench]
fn push_clone_vec_5000(bencher: &mut test_crate::Bencher) {
    push_clone_vec(bencher, 5000);
}

#[bench]
fn push_clone_pvec_5000(bencher: &mut test_crate::Bencher) {
    push_clone_pvec(bencher, 5000);
}

#[bench]
fn push_clone_dvec_5000(bencher: &mut test_crate::Bencher) {
    push_clone_dvec(bencher, 5000);
}

fn pop_vec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i * 2);
        }

        for i in (0..n).rev() {
            assert_eq!(vec.pop().unwrap(), i * 2);
        }
    });
}

fn pop_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = PVec::new();
        for i in 0..n {
            vec.push(i * 2);
        }

        for i in (0..n).rev() {
            assert_eq!(vec.pop().unwrap(), i * 2);
        }
    });
}

#[bench]
fn pop_vec_5000(bencher: &mut test_crate::Bencher) {
    pop_vec(bencher, 5000);
}

#[bench]
fn pop_pvec_5000(bencher: &mut test_crate::Bencher) {
    pop_pvec(bencher, 5000);
}

#[bench]
fn pop_vec_50000(bencher: &mut test_crate::Bencher) {
    pop_vec(bencher, 50000);
}

#[bench]
fn pop_pvec_50000(bencher: &mut test_crate::Bencher) {
    pop_pvec(bencher, 50000);
}

#[bench]
fn pop_vec_500000(bencher: &mut test_crate::Bencher) {
    pop_vec(bencher, 500000);
}

#[bench]
fn pop_pvec_500000(bencher: &mut test_crate::Bencher) {
    pop_pvec(bencher, 500000);
}

fn pop_clone_vec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = Vec::new();
        let mut vec_one = vec.clone();

        for i in 0..n {
            vec.push(i * 2);
        }

        for _ in 0..n {
            vec.pop();
            vec_one = vec.clone();
        }

        drop(vec_one);
    });
}

fn pop_clone_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    bencher.iter(|| {
        let mut vec = PVec::new();
        let mut vec_one = vec.clone();

        for i in 0..n {
            vec.push(i * 2);
        }

        for _ in 0..n {
            vec.pop();
            vec_one = vec.clone();
        }

        drop(vec_one);
    });
}

#[bench]
fn pop_clone_vec_5000(bencher: &mut test_crate::Bencher) {
    pop_clone_vec(bencher, 5000);
}

#[bench]
fn pop_clone_pvec_5000(bencher: &mut test_crate::Bencher) {
    pop_clone_pvec(bencher, 5000);
}

fn index_sequentially_vec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = Vec::new();

    for i in 0..n {
        vec.push(i * 2);
    }

    bencher.iter(|| {
        for i in 0..n {
            assert_eq!(vec[i], i * 2);
        }
    });
}

fn index_sequentially_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = PVec::new();

    for i in 0..n {
        vec.push(i * 2);
    }

    bencher.iter(|| {
        for i in 0..n {
            assert_eq!(vec[i], i * 2);
        }
    });
}

fn index_sequentially_dvec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = DVec::new();

    for i in 0..n {
        vec.push(i * 2);
    }

    bencher.iter(|| {
        for i in 0..n {
            assert_eq!(vec[i], i * 2);
        }
    });
}

#[bench]
fn index_sequentially_vec_5000(bencher: &mut test_crate::Bencher) {
    index_sequentially_vec(bencher, 5000);
}

#[bench]
fn index_sequentially_pvec_5000(bencher: &mut test_crate::Bencher) {
    index_sequentially_pvec(bencher, 5000);
}

#[bench]
fn index_sequentially_dvec_5000(bencher: &mut test_crate::Bencher) {
    index_sequentially_dvec(bencher, 5000);
}

#[bench]
fn index_sequentially_vec_50000(bencher: &mut test_crate::Bencher) {
    index_sequentially_vec(bencher, 50000);
}

#[bench]
fn index_sequentially_pvec_50000(bencher: &mut test_crate::Bencher) {
    index_sequentially_pvec(bencher, 50000);
}

#[bench]
fn index_sequentially_dvec_50000(bencher: &mut test_crate::Bencher) {
    index_sequentially_dvec(bencher, 50000);
}

#[bench]
fn index_sequentially_vec_500000(bencher: &mut test_crate::Bencher) {
    index_sequentially_vec(bencher, 500000);
}

#[bench]
fn index_sequentially_pvec_500000(bencher: &mut test_crate::Bencher) {
    index_sequentially_pvec(bencher, 500000);
}

#[bench]
fn index_sequentially_dvec_500000(bencher: &mut test_crate::Bencher) {
    index_sequentially_dvec(bencher, 500000);
}

fn index_randomly_vec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i * 2);
    }

    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    bencher.iter(|| {
        for _ in 0..n {
            let j = (rng.next_u32() as usize) % n;
            assert_eq!(*vec.get(j).unwrap(), j * 2);
        }
    });
}

fn index_randomly_pvec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = PVec::new();
    for i in 0..n {
        vec.push(i * 2);
    }

    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    bencher.iter(|| {
        for _ in 0..n {
            let j = (rng.next_u32() as usize) % n;
            assert_eq!(*vec.get(j).unwrap(), j * 2);
        }
    });
}

fn index_randomly_dvec(bencher: &mut test_crate::Bencher, n: usize) {
    let mut vec = DVec::new();
    for i in 0..n {
        vec.push(i * 2);
    }

    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    bencher.iter(|| {
        for _ in 0..n {
            let j = (rng.next_u32() as usize) % n;
            assert_eq!(*vec.get(j).unwrap(), j * 2);
        }
    });
}

#[bench]
fn index_randomly_vec_5000(bencher: &mut test_crate::Bencher) {
    index_randomly_vec(bencher, 5000);
}

#[bench]
fn index_randomly_pvec_5000(bencher: &mut test_crate::Bencher) {
    index_randomly_pvec(bencher, 5000);
}

#[bench]
fn index_randomly_dvec_5000(bencher: &mut test_crate::Bencher) {
    index_randomly_dvec(bencher, 5000);
}

#[bench]
fn index_randomly_vec_50000(bencher: &mut test_crate::Bencher) {
    index_randomly_vec(bencher, 50000);
}

#[bench]
fn index_randomly_pvec_50000(bencher: &mut test_crate::Bencher) {
    index_randomly_pvec(bencher, 50000);
}

#[bench]
fn index_randomly_dvec_50000(bencher: &mut test_crate::Bencher) {
    index_randomly_dvec(bencher, 50000);
}

#[bench]
fn index_randomly_vec_500000(bencher: &mut test_crate::Bencher) {
    index_randomly_vec(bencher, 500000);
}

#[bench]
fn index_randomly_pvec_500000(bencher: &mut test_crate::Bencher) {
    index_randomly_pvec(bencher, 500000);
}

#[bench]
fn index_randomly_dvec_500000(bencher: &mut test_crate::Bencher) {
    index_randomly_dvec(bencher, 500000);
}

