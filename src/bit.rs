pub const WORDSIZE: usize = 64;
const LOG_WORDSIZE: usize = 6;


#[inline]
pub fn word_access(x: usize, i: usize) -> usize {
    (x >> i) & 1 
}

#[inline]
#[cfg(target_feature = "sse4.2")]
pub fn word_rank1(x: usize, i: usize) -> usize {
    use std::arch::x86_64::_popcnt64;
    unsafe {_popcnt64(x as i64 & ((1 << i) - 1)) as usize}
}

#[inline]
#[cfg(target_feature = "bmi2")]
pub fn word_select1(x: usize, i: usize) -> usize {
    use std::arch::x86_64::_pdep_u64;
    unsafe { _pdep_u64(1 << i, x as u64).trailing_zeros() as usize }
}

#[cfg(target_feature = "bmi2")]
pub fn word_select0(x: usize, i: usize) -> usize {
    use std::arch::x86_64::_pdep_u64;
    unsafe { _pdep_u64(1 << i, !x as u64).trailing_zeros() as usize }
}

#[inline]
pub fn shrd(x: usize, y: usize, i: usize) -> usize {
    // xとyを連結した(xの方が上位)ビット列をiだけシフトする
    x << (WORDSIZE - i) | y >> i
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    #[test]
    fn test_word_access() {
        let a: usize = 0b01110101_11101011_11100011;
        assert_eq!(word_access(a, 0), 1);
        assert_eq!(word_access(a, 1), 1);
        assert_eq!(word_access(a, 2), 0);
        assert_eq!(word_access(a, 3), 0);
        assert_eq!(word_access(a, 4), 0);
        assert_eq!(word_access(a, 5), 1);
        assert_eq!(word_access(a, 6), 1);
        assert_eq!(word_access(a, 7), 1);
    }
    #[test]
    fn test_word_rank1() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let r: usize = rng.gen();
            for j in 0..64 {
                assert_eq!(word_rank1(r, j), (r & ((1 << j) - 1)).count_ones() as u64);
            }
        }
    }
    #[test]
    fn test_word_select1() {
        let a: usize = 0b01110101_11101011_11100011;
        assert_eq!(word_select1(a, 0), 0);
        assert_eq!(word_select1(a, 1), 1);
        assert_eq!(word_select1(a, 2), 5);
        assert_eq!(word_select1(a, 3), 6);
        assert_eq!(word_select1(a, 4), 7);
        assert_eq!(word_select1(a, 5), 8);
        assert_eq!(word_select1(a, 6), 9);
        assert_eq!(word_select1(a, 7), 11);
    }
}
