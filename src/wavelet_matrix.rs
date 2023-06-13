use crate::bit::word_access;
use crate::suc_bv::{BitVector, SucBV, SucBVBuilder, SuccinctBVIndex};

type BV = SucBV<SuccinctBVIndex>;

pub struct WaveletMatrix {
    data: Box<[BV]>,
    len: usize,
}

impl WaveletMatrix {
    pub fn new(init: Vec<u64>) -> Self {
        let len = init.len();
        let bitlen = init.iter().max().unwrap_or(&1).next_power_of_two().trailing_zeros();
        let mut data = Vec::new();
        let mut before = init;
        let mut after = Vec::with_capacity(len);
        for l in (0..bitlen).rev() {
            let mut builder = SucBVBuilder::new(len);
            let mut zero = Vec::new();
            let mut one = Vec::new();
            for i in 0..len {
                builder.set(i, word_access(before[i] as usize, l as usize) == 1);
                match word_access(before[i] as usize, l as usize) {
                    0 => zero.push(before[i]),
                    1 => one.push(before[i]),
                    _ => unreachable!(),
                }
            }
            let bv = builder.build();
            data.push(bv);
            after.splice(.., zero.into_iter().chain(one.into_iter()));
            std::mem::swap(&mut before, &mut after);
        }
        Self {
            data: data.into_boxed_slice(), len
        }
    }

    pub fn access(&self, mut idx: usize) -> u64 {
        let mut ret = 0;
        for (l, ref bv) in (0..self.data.len()).rev().zip(self.data.iter()) {
            match bv.access(idx) {
                0 => idx = bv.rank0(idx),
                1 => {
                    ret |= 1 << l;
                    idx = bv.rank0(self.len) + bv.rank1(idx)
                }
                _ => unreachable!(),
            }
        }
        ret
    }

    pub fn rank(&self, value: u64, idx: usize) -> u64 {
        let (mut l, mut r) = (0, idx);
        for (i, ref bv) in (0..self.data.len()).rev().zip(self.data.iter()) {
            match word_access(value as usize, i) {
                0 => {
                    l = bv.rank0(l);
                    r = bv.rank0(r);
                },
                1 => {
                    l = bv.rank0(self.len) + bv.rank1(l);
                    r = bv.rank0(self.len) + bv.rank1(r);
                },
                _ => unreachable!(),
            }
        }
        (r - l) as u64
    }

    pub fn select(&self, value: u64, k: usize) -> usize {
        let mut idx = 0;
        for (i, ref bv) in (0..self.data.len()).rev().zip(self.data.iter()) {
            match word_access(value as usize, i) {
                0 => {
                    idx = bv.rank0(idx);
                },
                1 => {
                    idx = bv.rank0(self.len) + bv.rank1(idx);
                },
                _ => unreachable!(),
            }
        }
        idx += k;
        for bv in self.data.iter().rev() {
            let r = bv.rank0(self.len);
            if idx < r {
                idx = bv.select0(idx).unwrap();
            } else {
                idx = bv.select1(idx - r).unwrap();
            }
        }
        idx
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    const LEN: usize = 1 << 5;
    const QUERY: usize = 1 << 10;
    const BIT:u64 = 5;
    fn generate_random_wm() -> (WaveletMatrix, Vec<u64>){
        let mut rng = rand::thread_rng();
        let init = (0..LEN).map(|_| rng.gen_range(0, 1 << BIT)).collect::<Vec<_>>();
        let wm = WaveletMatrix::new(init.clone());
        assert_eq!(wm.data.len(), BIT as usize);
        (wm, init)
    }
    fn eprint_wm(wm: &WaveletMatrix) {
        for bv in wm.data.iter() {
            let mut s = String::new();
            for i in 0..wm.len {
                match bv.access(i) {
                    0 => s.push('0'),
                    1 => s.push('1'),
                    _ => unreachable!(),
                }
            }
            eprintln!("{}", s);
        }
    }
    #[test]
    fn test_access() {
        let (wm, init) = generate_random_wm();
        dbg!(&init);
        eprint_wm(&wm);
        for i in 0..LEN {
            assert_eq!(wm.access(i), init[i]);
        }
    }
    #[test]
    fn test_rank() {
        let (wm, init) = generate_random_wm();
        let mut rng = rand::thread_rng();
        for _ in 0..QUERY {
            let val = rng.gen_range(0, 1 << BIT);
            for i in 0..LEN {
                assert_eq!(
                    wm.rank(val, i),
                    init[..i].iter().filter(|&&x| x == val).count() as u64
                );
            }
        }
    }
    #[test]
    fn test_select() {
        let (wm, init) = generate_random_wm();
        for i in 0..LEN {
            let val = init[i];
            let count = init[0..i].iter().filter(|&&x| x == val).count();
            assert_eq!(wm.select(val, count), i);
        }
    }
}
