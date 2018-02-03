const MASK: usize = 0b11111;

fn shift(key: usize, shift: usize) -> usize {
    return (key >> shift) & MASK;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_must_return_correct_index() {
        // 0d141 - 0b10001101
        assert_eq!(shift(141, 0), 0b01101);
        assert_eq!(shift(141, 5), 0b00100);
        assert_eq!(shift(141, 10), 0b0000);
        assert_eq!(shift(141, 15), 0b0000);
        assert_eq!(shift(141, 20), 0b0000);
        assert_eq!(shift(141, 25), 0b0000);
        assert_eq!(shift(141, 30), 0b0000);
        assert_eq!(shift(141, 25), 0b0000);
    }
}