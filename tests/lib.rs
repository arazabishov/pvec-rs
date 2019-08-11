extern crate pvec;

use pvec::RrbVec;

#[cfg(not(feature = "small_branch"))]
const BRANCH_FACTOR: usize = 32;

#[cfg(feature = "small_branch")]
const BRANCH_FACTOR: usize = 4;

#[test]
fn len_matches_actual_size() {
    const N: usize = 5000;

    let mut vec = RrbVec::new();

    for i in 0..N {
        vec.push(i);
    }

    assert_eq!(vec.len(), N);

    for i in 0..N {
        assert_eq!(*vec.get(i).unwrap(), i);
    }
}

#[test]
fn len_matches_len_cloned() {
    const N: usize = 5000;

    let mut vec = RrbVec::new();

    for i in 0..N {
        vec.push(i);
    }

    let vec_0 = vec.clone();
    assert_eq!(vec.len(), N);
    assert_eq!(vec_0.len(), N);

    for i in 0..N {
        vec.push(i);
    }

    assert_eq!(vec.len(), 2 * N);
    assert_eq!(vec_0.len(), N);

    for i in 0..N {
        assert_eq!(*vec.get(i).unwrap(), i);
        assert_eq!(*vec_0.get(i).unwrap(), i);
    }

    for i in 0..N {
        assert_eq!(*vec.get(i + N).unwrap(), i);
    }
}

#[test]
fn mutate_in_place_must_not_mutate_cloned_rrbvec() {
    const N: usize = 32 * 4;

    let mut vec = RrbVec::new();

    for i in 0..N {
        vec.push(i);
    }

    let vec_0 = vec.clone();
    assert_eq!(vec.len(), N);
    assert_eq!(vec_0.len(), N);

    for i in 0..(N / 2) {
        *vec.get_mut(i).unwrap() += 1;
    }

    assert_eq!(vec.len(), N);
    assert_eq!(vec_0.len(), N);

    for i in 0..(N / 2) {
        assert_eq!(*vec.get(i).unwrap(), i + 1);
        assert_eq!(*vec_0.get(i).unwrap(), i);
    }

    // the second half ought to be untouched
    for i in N / 2..N {
        assert_eq!(*vec.get(i).unwrap(), i);
        assert_eq!(*vec_0.get(i).unwrap(), i);
        assert_eq!(
            vec.get(i).unwrap() as *const usize,
            vec_0.get(i).unwrap() as *const usize
        );
    }
}

#[test]
fn pop_must_not_mutate_cloned_rrbvec() {
    const N: usize = 32 * 4;

    let mut vec = RrbVec::new();

    for i in 0..N {
        vec.push(i);
    }

    let vec_0 = vec.clone();
    assert_eq!(vec.len(), N);
    assert_eq!(vec_0.len(), N);

    for _ in 0..(N / 2) {
        vec.pop();
    }

    assert_eq!(vec.len(), N / 2);
    assert_eq!(vec_0.len(), N);

    for i in 0..(N / 2) {
        assert_eq!(*vec.get(i).unwrap(), i);
        assert_eq!(*vec_0.get(i).unwrap(), i);
    }

    for i in N / 2..N {
        assert_eq!(*vec_0.get(i).unwrap(), i);
    }
}

#[test]
fn push_pop_must_return_expected_values() {
    const N: usize = 32 * 4;

    let mut vec = RrbVec::new();

    for i in 0..N {
        vec.push(i)
    }

    assert_eq!(vec.len(), N);

    for i in (0..N).rev() {
        assert_eq!(vec.pop().unwrap(), i);
    }

    for i in 0..N {
        vec.push(i)
    }

    assert_eq!(vec.len(), N);

    for i in (0..N).rev() {
        assert_eq!(vec.pop().unwrap(), i);
    }

    assert_eq!(vec.len(), 0);
}

#[test]
fn append_must_maintain_vectors_in_correct_state_after_clone() {
    let mut vec_l = RrbVec::new();
    let mut vec_c = RrbVec::new();
    let mut vec_r = RrbVec::new();

    let mut branch_value = 0;

    for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR * BRANCH_FACTOR {
        vec_l.push(branch_value);
        branch_value += 1;
    }

    for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
        vec_c.push(branch_value);
        branch_value += 1;
    }

    for _ in 0..BRANCH_FACTOR * BRANCH_FACTOR {
        vec_r.push(branch_value);
        branch_value += 1;
    }

    let vec_l_clone = vec_l.clone();
    let vec_c_clone = vec_c.clone();
    let vec_r_clone = vec_r.clone();

    vec_l.append(&mut vec_c);
    vec_l.append(&mut vec_r);

    assert_eq!(
        vec_l.len(),
        vec_l_clone.len() + vec_c_clone.len() + vec_r_clone.len()
    );

    let mut branch_test_value = 0;

    for i in 0..vec_l_clone.len() {
        assert_eq!(*vec_l_clone.get(i).unwrap(), branch_test_value);
        branch_test_value += 1;
    }

    for i in 0..vec_c_clone.len() {
        assert_eq!(*vec_c_clone.get(i).unwrap(), branch_test_value);
        branch_test_value += 1;
    }

    for i in 0..vec_r_clone.len() {
        assert_eq!(*vec_r_clone.get(i).unwrap(), branch_test_value);
        branch_test_value += 1;
    }
}

fn interleaving_different_operations_must_maintain_correct_internal_state(vec_size: usize) {
    let mut vec = RrbVec::new();
    let mut vec_item = 0;

    for i in 0..128 {
        if i % 2 == 0 {
            let mut vec_temp = RrbVec::new();

            for _ in 0..vec_size {
                vec_temp.push(vec_item);
                vec_item += 1;
            }

            assert_eq!(vec_temp.len(), vec_size);

            vec.append(&mut vec_temp);

            assert_eq!(vec_temp.len(), 0);
        } else {
            for _ in 0..(vec_size + vec_size) {
                vec.push(vec_item);
                vec_item += 1;
            }
        }

        assert_eq!(vec.len(), vec_item);

        for i in 0..vec.len() {
            assert_eq!(*vec.get(i).unwrap(), i);
            assert_eq!(*vec.get_mut(i).unwrap(), i);
        }

        let mut vec_one_clone = vec.clone();
        for i in (0..vec_item).rev() {
            assert_eq!(vec_one_clone.pop().unwrap(), i);
        }

        assert_eq!(vec_one_clone.len(), 0);
    }

    assert_eq!(vec.len(), vec_item);

    let mut vec_clone = vec.clone();
    for i in (0..vec_item).rev() {
        assert_eq!(vec_clone.pop().unwrap(), i);

        for j in 0..vec_clone.len() {
            assert_eq!(*vec_clone.get(j).unwrap(), j);
            assert_eq!(*vec_clone.get_mut(j).unwrap(), j);
        }
    }
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_4() {
    interleaving_different_operations_must_maintain_correct_internal_state(4);
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_5() {
    interleaving_different_operations_must_maintain_correct_internal_state(5);
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_16() {
    interleaving_different_operations_must_maintain_correct_internal_state(16);
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_17() {
    interleaving_different_operations_must_maintain_correct_internal_state(17);
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_32() {
    interleaving_different_operations_must_maintain_correct_internal_state(32);
}

#[test]
fn interleaving_different_operations_must_maintain_correct_internal_state_for_var_sizes_33() {
    interleaving_different_operations_must_maintain_correct_internal_state(33);
}

#[test]
fn interleaving_push_and_append_operations_must_maintain_correct_internal_state_for_var_sizes_32() {
    let mut vec_one = RrbVec::new();

    for i in 0..32 {
        vec_one.push(i);
    }

    let mut vec_two = RrbVec::new();

    for i in 0..1024 {
        if i % 2 == 0 {
            vec_two.push(i);
        } else {
            vec_two.append(&mut vec_one.clone());
        }

        for k in 0..vec_two.len() {
            vec_two.get(k).unwrap();
        }
    }
}

#[test]
fn zero_sized_values() {
    let mut v = RrbVec::new();
    assert_eq!(v.len(), 0);

    v.push(());
    assert_eq!(v.len(), 1);

    v.push(());
    assert_eq!(v.len(), 2);
    assert_eq!(v.pop(), Some(()));
    assert_eq!(v.pop(), Some(()));
    assert_eq!(v.pop(), None);

    assert_eq!(v.len(), 0);

    v.push(());
    assert_eq!(v.len(), 1);

    v.push(());
    assert_eq!(v.len(), 2);

    for i in 0..v.len() {
        v.get(i);
    }
    assert_eq!(v.len(), 2);

    v.push(());
    assert_eq!(v.len(), 3);

    v.push(());
    assert_eq!(v.len(), 4);

    for i in 0..v.len() {
        v.get_mut(i);
    }
    assert_eq!(v.len(), 4);
}

#[test]
fn split_off() {
    let mut vec_one = vec![1, 2, 3, 4, 5, 6];
    let vec_two = vec_one.split_off(4);

    assert_eq!(vec_one, [1, 2, 3, 4]);
    assert_eq!(vec_two, [5, 6]);
}

#[test]
fn interleaving_append_split_off_operations() {
    let mut vec = RrbVec::new();
    let mut value = 0;

    for size in 1..(BRANCH_FACTOR * 8 + BRANCH_FACTOR) {
        let mut another_vec = RrbVec::new();
        for _ in 0..size {
            another_vec.push(value);
            value += 1;
        }

        vec.append(&mut another_vec);

        let mid = vec.len() / 2;
        let mut right = vec.split_off(mid);

        vec.append(&mut right);
        value = vec.len();
    }

    for i in 0..value {
        assert_eq!(vec.get(i).cloned(), Some(i));
    }
}
