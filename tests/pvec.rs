extern crate persistent;

use persistent::pvec::PVec;

#[test]
fn len_matches_actual_size() {
    const N: usize = 5000;

    let mut pvec = PVec::new();

    for i in 0..N {
        pvec.push(i);
    }

    assert_eq!(pvec.len(), N);

    for i in 0..N {
        assert_eq!(*pvec.get(i).unwrap(), i);
    }
}

#[test]
fn len_matches_len_cloned() {
    const N: usize = 5000;

    let mut pvec = PVec::new();

    for i in 0..N {
        pvec.push(i);
    }

    let pvec_0 = pvec.clone();
    assert_eq!(pvec.len(), N);
    assert_eq!(pvec_0.len(), N);

    for i in 0..N {
        pvec.push(i);
    }

    assert_eq!(pvec.len(), 2 * N);
    assert_eq!(pvec_0.len(), N);

    for i in 0..N {
        assert_eq!(*pvec.get(i).unwrap(), i);
        assert_eq!(*pvec_0.get(i).unwrap(), i);
    }

    for i in 0..N {
        assert_eq!(*pvec.get(i + N).unwrap(), i);
    }
}

#[test]
fn mutate_in_place_must_not_mutate_cloned_pvec() {
    const N: usize = 32 * 4;

    let mut pvec = PVec::new();

    for i in 0..N {
        pvec.push(i);
    }

    let pvec_0 = pvec.clone();
    assert_eq!(pvec.len(), N);
    assert_eq!(pvec_0.len(), N);

    for i in 0..(N / 2) {
        *pvec.get_mut(i).unwrap() += 1;
    }

    assert_eq!(pvec.len(), N);
    assert_eq!(pvec_0.len(), N);

    for i in 0..(N / 2) {
        assert_eq!(*pvec.get(i).unwrap(), i + 1);
        assert_eq!(*pvec_0.get(i).unwrap(), i);
    }

    // the second half ought to be untouched
    for i in N / 2..N {
        assert_eq!(*pvec.get(i).unwrap(), i);
        assert_eq!(*pvec_0.get(i).unwrap(), i);
        assert_eq!(
            pvec.get(i).unwrap() as *const usize,
            pvec_0.get(i).unwrap() as *const usize
        );
    }
}

#[test]
fn pop_must_not_mutate_cloned_pvec() {
    const N: usize = 32 * 4;

    let mut pvec = PVec::new();

    for i in 0..N {
        pvec.push(i);
    }

    let pvec_0 = pvec.clone();
    assert_eq!(pvec.len(), N);
    assert_eq!(pvec_0.len(), N);

    for _ in 0..(N / 2) {
        pvec.pop();
    }

    assert_eq!(pvec.len(), N / 2);
    assert_eq!(pvec_0.len(), N);

    for i in 0..(N / 2) {
        assert_eq!(*pvec.get(i).unwrap(), i);
        assert_eq!(*pvec_0.get(i).unwrap(), i);
    }

    // the second half ought to be untouched
    for i in N / 2..N {
        assert_eq!(*pvec_0.get(i).unwrap(), i);
    }
}

#[test]
fn push_pop_must_return_expected_values() {
    const N: usize = 32 * 4;

    let mut pvec = PVec::new();

    for i in 0..N {
        pvec.push(i)
    }

    assert_eq!(pvec.len(), N);

    for i in (0..N).rev() {
        assert_eq!(pvec.pop().unwrap(), i);
    }

    for i in 0..N {
        pvec.push(i)
    }

    assert_eq!(pvec.len(), N);

    for i in (0..N).rev() {
        assert_eq!(pvec.pop().unwrap(), i);
    }

    assert_eq!(pvec.len(), 0);
}
