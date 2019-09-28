extern crate criterion;

#[cfg(feature = "arc")]
extern crate im;

#[cfg(not(feature = "arc"))]
extern crate im_rc;
extern crate num;
extern crate pvec;
extern crate rand;
extern crate rand_xorshift;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
extern crate rayon;

use criterion::*;

mod sequential;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod life;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod pythagoras;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod factorial;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod collect;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
criterion_main!(
    life::benches,
    pythagoras::benches,
    factorial::benches,
    sequential::benches,
    collect::benches
);

#[cfg(not(all(feature = "arc", feature = "rayon-iter")))]
criterion_main!(sequential::benches);
