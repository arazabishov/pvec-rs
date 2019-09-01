#![cfg_attr(test, feature(test))]

extern crate criterion;
extern crate dogged;
extern crate im;
extern crate num;
extern crate pvec;
extern crate rand;
extern crate rand_xorshift;
extern crate test as test_crate;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

use criterion::*;

mod serial;

#[cfg(all(test, feature = "arc", feature = "rayon-iter"))]
mod life;

#[cfg(all(test, feature = "arc", feature = "rayon-iter"))]
mod pythagoras;

#[cfg(all(test, feature = "arc", feature = "rayon-iter"))]
mod factorial;

#[cfg(all(test, feature = "arc", feature = "rayon-iter"))]
mod collect;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
criterion_main!(
    life::benches,
    pythagoras::benches,
    factorial::benches,
    serial::benches,
    collect::benches
);

#[cfg(not(all(feature = "arc", feature = "rayon-iter")))]
criterion_main!(serial::benches);
