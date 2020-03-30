extern crate pvec;

use pvec::core::RrbVec;

// TODO: calculate the baseline of the process
// memory consumption: 843776 bytes.
fn main() {
    append(200000);

    // let mut bag = Vec::new();
    // for _ in 0..1024 {
    //     let mut cln = vec.clone();

    //     for i in (0..100000).step_by(128) {
    //         *cln.get_mut(i).unwrap() += 1;
    //     }

    //     bag.push(cln);
    // }

    // drop(bag);
}

// Building a different vectors from
// scratch to demonstrate its memory
// footprint.
fn push(n: usize) {
    let mut vec = RrbVec::new();

    for i in 0..n {
        vec.push(i);
    }

    drop(vec);
}

// This benchmark demonstrates how structural
// sharing can save memory.
fn update_clone(n: usize) {
    let mut vec = RrbVec::new();

    for i in 0..n {
        vec.push(i);
    }

    let mut bag = Vec::with_capacity(n);

    for i in 0..n {
        let mut cln = vec.clone();
        *cln.get_mut(i).unwrap() += 1;

        bag.push(cln);
    }

    drop(bag);
}

fn append(n: usize) {
    let mut vec = Vec::new();    
    let mut i = 1;

    while i < n && (vec.len() + i) <= n {
        let mut vec_t = Vec::new();

        for j in 0..i {
            vec_t.push(j);
        }

        // println!("vec_len={}", vec.len());
        vec.append(&mut vec_t);
        i *= 2;
    }

    let mut vec_t = Vec::new();    
    for j in vec.len()..n {
        vec_t.push(j);
    }

    vec.append(&mut vec_t);
    // println!("vec_len={}", vec.len());

    drop(vec);
}

// The test runner should be written in Rust
// it has to start a new process for every bench
// record the result, output it in CSV format
// run the benchmark for different sizes
