mod selectindex;
mod raw_bit_vector;
pub mod suc_index;

use raw_bit_vector::BV;
pub use suc_index::SuccinctBVIndex;

pub trait BitVector {
    fn access(&self, i: usize) -> usize;
    fn rank1(&self, i: usize) -> usize;
    fn rank0(&self, i: usize) -> usize;
    fn select1(&self, i: usize) -> Option<usize>;
    fn select0(&self, i: usize) -> Option<usize>;
}

pub(crate) trait SelectIndex<R: RankIndex> {
    fn new(bv: &BV) -> Self;
    fn select0(&self, bv: &BV, rank: &R, i: usize) -> Option<usize>;
    fn select1(&self, bv: &BV, rank: &R, i: usize) -> Option<usize>;
}

pub(crate) trait RankIndex {
    fn new(bv: &BV) -> Self;
    fn rank1(&self, bv: &BV, idx: usize) -> usize;
    fn rank0(&self, bv: &BV, idx: usize) -> usize {
        idx - self.rank1(bv, idx)
    }
}

pub trait BVIndex {
    fn new(bv: &BV) -> Self;
    fn rank1(&self, bv: &BV, idx: usize) -> usize;
    fn rank0(&self, bv: &BV, idx: usize) -> usize {
        idx - self.rank1(bv, idx)
    }
    fn select0(&self, bv: &BV, i: usize) -> usize {
        let mut start = 0;
        let mut end = bv.len();
        while end - start > 1 {
            let m = (end + start) / 2;
            let pops = self.rank0(bv, m);
            if pops > i {
                end = m
            } else {
                start = m
            }
        }
        start
    }
    fn select1(&self, bv: &BV, i: usize) -> usize{
        let mut start = 0;
        let mut end = bv.len();
        while end - start > 1 {
            let m = (end + start) / 2;
            let pops = self.rank1(bv, m);
            if pops > i {
                end = m
            } else {
                start = m
            }
        }
        start
    }
}

pub struct SucBV<I: BVIndex> {
    raw_data: BV,
    index: I,
}

impl<I: BVIndex> BitVector for SucBV<I> {
    fn access(&self, i: usize) -> usize {
        self.raw_data.access(i)
    }
    fn rank1(&self, i: usize) -> usize {
        self.index.rank1(&self.raw_data, i)
    }
    fn rank0(&self, i: usize) -> usize {
        self.index.rank0(&self.raw_data, i)
    }
    fn select1(&self, i: usize) -> Option<usize> {
        // select1(i) = max {j | rank1(j) <= i}
        if i >= self.rank1(self.raw_data.len()) {
            // error
            return None;
        }
        Some(self.index.select1(&self.raw_data, i))
    }
    fn select0(&self, i: usize) -> Option<usize> {
        if i >= self.rank0(self.raw_data.len()) {
            // error
            return None;
        }
        Some(self.index.select0(&self.raw_data, i))
    }
}

pub struct SucBVBuilder {
    data: BV
}

impl SucBVBuilder {
    pub fn new(length: usize) -> SucBVBuilder {
        Self {
            data: BV::new(length),
        }
    }
    pub fn set(&mut self, idx: usize, bit: bool) {
        self.data.set(idx, bit);
    }
    pub fn build(self) -> SucBV<SuccinctBVIndex> {
        let index = SuccinctBVIndex::new(&self.data);
        SucBV {
            raw_data: self.data, index
        }
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use super::*;
    const LENGTH: usize = (1 << 20) + 1000;
    #[test]
    fn test_access() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut builder = SucBVBuilder::new(LENGTH);
        for i in 0..LENGTH  {
            raw[i] = rng.gen();
            builder.set(i, raw[i]);
        }
        let vec = builder.build();
        for i in 0..LENGTH {
            assert_eq!(raw[i], vec.access(i) == 1)
        }
    }
    #[test]
    fn test_rank() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut sum = vec![0; LENGTH + 1];
        let mut builder = SucBVBuilder::new(LENGTH);
        for i in 0..LENGTH  {
            raw[i] = rng.gen();
            sum[i + 1] = sum[i] + raw[i] as usize;
            builder.set(i, raw[i]);
        }
        let vec = builder.build();
        // dbg!(&vec.rank);
        for i in 0..=LENGTH {
            assert_eq!(sum[i], vec.rank1(i), " at {} th loop", i)
        }
    }
    #[test]
    fn test_select1_random() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut indices = Vec::new();
        let mut builder = SucBVBuilder::new(LENGTH);
        let mut popcnt = 0;
        for i in 0..LENGTH {
            raw[i] = rng.gen();
            if raw[i] {
                indices.push(i);
                popcnt += 1;
            }
            builder.set(i, raw[i]);
        }
        let vec = builder.build();
        for i in 0..popcnt {
            if indices[i] != vec.select1(i).unwrap() {
                assert_eq!(indices[i], vec.select1(i).unwrap());
            }
        }
    }
    #[test]
    fn test_select0() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut indices = Vec::new();
        let mut builder = SucBVBuilder::new(LENGTH);
        let mut zerocnt = 0;
        for i in 0..LENGTH  {
            raw[i] = rng.gen();
            if !raw[i] {
                indices.push(i);
                zerocnt += 1;
            }
            builder.set(i, raw[i]);
        }
        let vec = builder.build();
        for i in 0..zerocnt {
            assert_eq!(indices[i], vec.select0(i).unwrap())
        }
    }
}
