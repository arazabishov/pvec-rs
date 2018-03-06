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