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
const IM_RS_VECTOR_RELAXED: &str = "im-rs-vector-relaxed";

const PVEC_RRBVEC_BALANCED: &str = "pvec-rrbvec-balanced";
const PVEC_RRBVEC_RELAXED: &str = "pvec-rrbvec-relaxed";
const PVEC_STD: &str = "pvec-std";

const RRBVEC: &str = "rrbvec";
const RBVEC: &str = "rbvec";

#[cfg(all(feature = "arc", feature = "rayon-iter"))]
criterion_main!(
    sequential::benches,
    vecaddition::benches,
    words::benches,
    fold::benches,
);

#[cfg(not(all(feature = "arc", feature = "rayon-iter")))]
criterion_main!(sequential::benches);
