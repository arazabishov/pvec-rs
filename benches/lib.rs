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
mod vecaddition;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod words;

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
mod fold;

const STD_VEC: &str = "std-vec";
const IM_RS_VECTOR_BALANCED: &str = "im-rs-vector-balanced";
const IM_RS_VECTOR_UNBALANCED: &str = "im-rs-vector-unbalanced";
const RRBVEC_BALANCED: &str = "rrbvec-balanced";
const RRBVEC_UNBALANCED: &str = "rrbvec-unbalanced";
const PVEC_BALANCED: &str = "pvec-balanced";
const PVEC_UNBALANCED: &str = "pvec-unbalanced";

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
criterion_main!(
    sequential::benches,
    vecaddition::benches,
    words::benches,
    fold::benches,
);

#[cfg(not(all(feature = "arc", feature = "rayon-iter")))]
criterion_main!(sequential::benches);
