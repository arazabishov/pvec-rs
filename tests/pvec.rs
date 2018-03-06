extern crate persistent;

use persistent::pvec::PVec;

#[test]
fn new_must_return_correctly_initialized_pvec_instance() {
    let mut vec = PVec::new();

    for i in 0..64 {
        vec.push(i);
    }

    for i in 0..64 {
        assert_eq!(vec[i], i);
    }

    assert_eq!(vec.len(), 64);
}