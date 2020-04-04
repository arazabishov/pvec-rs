extern crate im_rc;
extern crate pvec;

use pvec::core::{RbVec, RrbVec};
use pvec::PVec;
use std::env;

use im_rc::Vector as IVec;

const STD_VEC: &str = "std-vec";

const IM_RS_VECTOR_BALANCED: &str = "im-rs-vector-balanced";
const IM_RS_VECTOR_RELAXED: &str = "im-rs-vector-relaxed";

const PVEC_RRBVEC_BALANCED: &str = "pvec-rrbvec-balanced";
const PVEC_RRBVEC_RELAXED: &str = "pvec-rrbvec-relaxed";
const PVEC_STD: &str = "pvec-std";

const RRBVEC: &str = "rrbvec";
const RBVEC: &str = "rbvec";

// Building a different vectors from
// scratch to demonstrate its memory
// footprint.
fn push_benches(vec: &str, n: usize) {
    macro_rules! push {
        ($init:expr, $push:ident) => {
            |n| {
                let mut vec = $init();

                for i in 0..n {
                    vec.$push(i);
                }

                drop(vec);
            }
        };
    }

    let push_ivec = push!(|| IVec::new(), push_back);
    let push_std_vec = push!(|| Vec::new(), push);
    let push_rbvec = push!(|| RbVec::new(), push);
    let push_pvec_std = push!(|| PVec::new(), push);
    let push_pvec_rrbvec = push!(|| PVec::new().clone(), push);

    match vec {
        IM_RS_VECTOR_BALANCED => push_ivec(n),
        PVEC_RRBVEC_BALANCED => push_pvec_rrbvec(n),
        PVEC_STD => push_pvec_std(n),
        STD_VEC => push_std_vec(n),
        RBVEC => push_rbvec(n),
        &_ => {}
    }
}

// This benchmark demonstrates how structural
// sharing can save memory.
fn update_benches(vec: &str, n: usize) {
    macro_rules! update_clone {
        ($init:expr, $push:ident) => {
            |n| {
                let mut vec = $init();
                for i in 0..n {
                    vec.$push(i);
                }

                let mut bag = Vec::with_capacity(n);
                for i in 0..n {
                    let mut cln = vec.clone();
                    *cln.get_mut(i).unwrap() += 1;

                    bag.push(cln);
                }

                drop(bag);
            }
        };
    }

    let update_clone_ivec = update_clone!(|| IVec::new(), push_back);
    let update_clone_pvec_rrbvec = update_clone!(|| PVec::new().clone(), push);
    let update_clone_pvec_std = update_clone!(|| PVec::new(), push);
    let update_clone_std_vec = update_clone!(|| Vec::new(), push);
    let update_clone_rrbvec = update_clone!(|| RbVec::new(), push);

    match vec {
        IM_RS_VECTOR_BALANCED => update_clone_ivec(n),
        PVEC_RRBVEC_BALANCED => update_clone_pvec_rrbvec(n),
        PVEC_STD => update_clone_pvec_std(n),
        STD_VEC => update_clone_std_vec(n),
        RBVEC => update_clone_rrbvec(n),
        &_ => panic!("Unsupported vec type={}", vec),
    }
}

mod append {
    use super::*;

    pub fn vec(vec: &mut Vec<usize>, mut data: Vec<usize>) {
        vec.append(&mut data);
    }

    pub fn ivec(vec: &mut IVec<usize>, data: IVec<usize>) {
        vec.append(data);
    }

    pub fn rbvec(vec: &mut RbVec<usize>, mut data: RbVec<usize>) {
        vec.append(&mut data);
    }

    pub fn rrbvec(vec: &mut RrbVec<usize>, mut data: RrbVec<usize>) {
        vec.append(&mut data);
    }

    pub fn pvec(vec: &mut PVec<usize>, mut data: PVec<usize>) {
        vec.append(&mut data);
    }
}

// Evaluates the memory overhead induced by
// appending vectors. This should reveal the
// overhead of relaxed nodes if any.
fn append_benches(vec: &str, n: usize) {
    macro_rules! append {
        ($init:expr, $push:ident, $append:path) => {
            |n| {
                let mut vec = $init();
                let mut i = 1;

                while i < n && (vec.len() + i) <= n {
                    let mut vec_t = $init();

                    for j in 0..i {
                        vec_t.$push(j);
                    }

                    $append(&mut vec, vec_t);
                    i *= 2;
                }

                let mut vec_t = $init();
                for j in vec.len()..n {
                    vec_t.$push(j);
                }

                $append(&mut vec, vec_t);

                drop(vec);
            }
        };
    }

    let append_std_vec = append!(|| Vec::new(), push, append::vec);
    let append_pvec_rrbvec_relaxed = append!(|| PVec::new().clone(), push, append::pvec);
    let append_pvec_std = append!(|| PVec::new(), push, append::pvec);
    let append_rrbvec = append!(|| RrbVec::new(), push, append::rrbvec);
    let append_rbvec = append!(|| RbVec::new(), push, append::rbvec);
    let append_ivec_relaxed = append!(|| IVec::new(), push_back, append::ivec);

    match vec {
        IM_RS_VECTOR_RELAXED => append_ivec_relaxed(n),
        PVEC_RRBVEC_RELAXED => append_pvec_rrbvec_relaxed(n),
        PVEC_STD => append_pvec_std(n),
        STD_VEC => append_std_vec(n),
        RRBVEC => append_rrbvec(n),
        RBVEC => append_rbvec(n),
        &_ => {}
    }
}

fn get_arguments() -> Result<(String, String, usize), &'static str> {
    let args = env::args();

    if args.len() < 4 {
        Err("Not enough arguments provided.")
    } else {
        // The first argument points to the executable
        // path. Hence, we ignore it.
        let mut args_iter = env::args().skip(1);

        // Here we make a strong assumption about present
        // arguments. They're always expected to be provided
        // as this program is only expected to be used from
        // another program - benchmark runner.
        let bench = args_iter.next().unwrap();
        let vec = args_iter.next().unwrap();
        let n = args_iter
            .next()
            .map(|it| it.parse::<usize>())
            .unwrap()
            .expect("n has to be an unsigned integer");

        Ok((bench, vec, n))
    }
}

fn main() {
    match get_arguments() {
        Ok((bench_arg, vec_arg, n)) => {
            let bench = bench_arg.as_str();
            let vec = vec_arg.as_str();

            match bench {
                "push" => push_benches(vec, n),
                "append" => append_benches(vec, n),
                "update_clone" => update_benches(vec, n),
                &_ => {}
            };
        }
        Err(e) => println!("{}", e),
    }
}
