#![cfg_attr(test, feature(test))]

extern crate persistent;
extern crate test as test_crate;

use persistent::pvec::PVec;

macro_rules! push {
    ($mod_name: ident, $N: expr) => {
        mod $mod_name {
            use PVec;
            use test_crate;
            const N: usize = $N;

            #[bench]
            fn dogged(b: &mut test_crate::Bencher) {
                b.iter(|| {
                    let mut vec = PVec::new();
                    for i in 0 .. N {
                        vec.push(i);
                    }
                });
            }

            #[bench]
            fn standard(b: &mut test_crate::Bencher) {
                b.iter(|| {
                    let mut vec = Vec::new();
                    for i in 0 .. N {
                        vec.push(i);
                    }
                });
            }
        }
    }
}

push!(push_5000, 5000);
push!(push_50000, 50000);
push!(push_500000, 500000);

