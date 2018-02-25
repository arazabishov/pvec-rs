use std::sync::Arc;

const MASK: usize = 0b11111;
const NODE_CHILDREN_SIZE: usize = 32;
const BITS_PER_LEVEL: usize = 5;

fn shift(key: usize, shift: usize) -> usize {
    return (key >> shift) & MASK;
}

enum Node<T> {
    Branch {
        children: [Option<Arc<Node<T>>>; NODE_CHILDREN_SIZE]
    },
    Leaf {
        value: Option<T>
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_must_return_correct_index() {
        // 0d141 - 0b10001101
        assert_eq!(shift(141, 0), 0b01101);
        assert_eq!(shift(141, 5), 0b00100);
        assert_eq!(shift(141, 10), 0b00000);
        assert_eq!(shift(141, 15), 0b00000);
        assert_eq!(shift(141, 20), 0b00000);
        assert_eq!(shift(141, 25), 0b00000);
        assert_eq!(shift(141, 30), 0b00000);
        assert_eq!(shift(141, 25), 0b00000);
    }
}