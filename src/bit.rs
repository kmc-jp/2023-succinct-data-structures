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
    let (mask, _) = ((1 as usize) << i).overflowing_sub(1);
    let a = x & mask;
    unsafe {_popcnt64(a as i64) as usize}
}

#[cfg(not(target_feature = "sse4.2"))]
pub fn word_rank1(x: usize, i: usize) -> usize {
    let (mask, _) = ((1 as usize) << i).overflowing_sub(1);
    let a = x & mask;
    a.count_ones() as usize
}

#[inline]
#[cfg(all(target_feature = "bmi2", target_feature = "bmi1"))]
pub fn word_select1(x: usize, i: usize) -> usize {
    use std::arch::x86_64::{_pdep_u64, _tzcnt_u64};
    unsafe { _tzcnt_u64(_pdep_u64(1 << i, x as u64)) as usize }
}

#[cfg(not(any(target_feature = "bmi2", target_feature = "bmi1")))]
pub fn word_select1(x: usize, i: usize) -> usize {
    const M1: u64 = 0x5555555555555555;
    const M2: u64 = 0x3333333333333333;
    const M4: u64 = 0x0f0f0f0f0f0f0f0f;
    const M8: u64 = 0x00ff00ff00ff00ff;
    let c1 = bit;
    let c2 = c1 - ((c1 >> 1) & M1);
    let c4 = ((c2 >> 2) & M2) + (c2 & M2);
    let c8 = ((c4 >> 4) + c4) & M4;
    let c16 = ((c8 >> 8) + c8) & M8;
    let c32 = (c16 >> 16) + c16;
    let mut i = idx as u64;
    let mut r = 0;
    let mut t = c32 & 0x3f;
    if i >= t {
        r += 32;
        i -= t;
    }
    t = (c16 >> r) & 0x1f;
    if i >= t {
        r += 16;
        i -= t;
    }
    t = (c8 >> r) & 0x0f;
    if i >= t {
        r += 8;
        i -= t;
    }
    t = (c4 >> r) & 0x07;
    if i >= t {
        r += 4;
        i -= t;
    }
    t = (c2 >> r) & 0x03;
    if i >= t {
        r += 2;
        i -= t;
    }
    t = (c1 >> r) & 0x01;
    if i >= t {
        r += 1;
    }
    r
}

#[inline]
#[cfg(all(target_feature = "bmi2", target_feature = "bmi1"))]
pub fn word_select0(x: usize, i: usize) -> usize {
    use std::arch::x86_64::{_pdep_u64, _tzcnt_u64};
    unsafe { _tzcnt_u64(_pdep_u64(1 << i, !x as u64)) as usize }
}

#[inline]
pub fn shrd(x: usize, y: usize, i: usize) -> usize {
    // xとyを連結した(xの方が上位)ビット列(すなわちx*2^64 + y)をiだけ右にシフトする
    let x = x.checked_shl((WORDSIZE - i) as u32).unwrap_or(0);
    x | (y >> i)
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
                assert_eq!(word_rank1(r, j), (r & ((1 << j) - 1)).count_ones() as usize);
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
    #[test]
    fn test_shrd() {
        let a: usize = 0x0f0f_8f8f_013f_1034;
        let b: usize = 0x013f_1034_1340_180a;
        assert_eq!(shrd(a, b, 0), b);
        assert_eq!(shrd(a, b, 4), 0x4013f10341340180);
    }
}
