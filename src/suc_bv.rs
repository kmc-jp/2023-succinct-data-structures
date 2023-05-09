use crate::bit::*;

mod rankindex;
mod selectindex;
mod raw_bit_vector;
mod simpleselectindex;

use rankindex::*;
use raw_bit_vector::BV;
use simpleselectindex::*;

pub trait BitVector {
    fn access(&self, i: usize) -> usize;
    fn rank1(&self, i: usize) -> usize;
    fn rank0(&self, i: usize) -> usize {
        i - self.rank1(i)
    }
    fn select1(&self, i: usize) -> Option<usize>;
    fn select0(&self, i: usize) -> Option<usize>;
}

pub struct SucBV {
    raw_data: BV,
    rank: RankIndex,
    select1: SimpleSelectIndex,
    select0: SimpleSelectIndex,
}

impl BitVector for SucBV {
    fn access(&self, i: usize) -> usize {
        self.raw_data.access(i)
    }
    fn rank1(&self, i: usize) -> usize {
        let large_idx = i / RANK_LARGE_BLOCKSIZE;
        let small_idx = i / RANK_SMALL_BLOCKSIZE;
        let word_idx = i / WORDSIZE;
        let mut sum = self.rank.large(large_idx) as usize + self.rank.small(small_idx) as usize;
        for j in small_idx * RANK_SMALL_BLOCKSIZE / WORDSIZE..word_idx {
            sum += self.raw_data.word(j).count_ones() as usize;
        }
        sum += word_rank1(self.raw_data.word(word_idx), i % WORDSIZE);
        sum as usize
    }
    fn rank0(&self, i: usize) -> usize {
        i - self.rank1(i)
    }
    fn select1(&self, i: usize) -> Option<usize> {
        // let large_idx = i / SELECT_LARGE_BLOCKSIZE;
        // match self.select1.data(large_idx) {
        //     (Sparse(a), start) => start + a.access(i % SELECT_LARGE_BLOCKSIZE),
        //     (Dense(a), start) => {
        //         let (block_idx, rest) = a.select(0, i % SELECT_LARGE_BLOCKSIZE);
        //         let start = start + block_idx * SELECT_SMALL_BLOCKSIZE;
        //         let word_idx = start / WORDSIZE;
        //         let word = shrd(self.raw_data.word(word_idx + 1), self.raw_data.word(word_idx), start % WORDSIZE);
        //         word_select1(word, rest)
        //     },
        // }
        // select1(i) = max {j | rank1(j) <= i}
        if i >= self.rank1(self.raw_data.len()) {
            // error
            return None;
        }
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match self.select1.data(block_idx) {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.select1.data(block_idx + 1);
                while end - start > 1 {
                    let m = (end + start) / 2;
                    let pops = self.rank1(m);
                    if pops > i {
                        end = m
                    } else {
                        start = m
                    }
                }
                start
            },
        };
        Some(ret)
    }
    fn select0(&self, i: usize) -> Option<usize> {
        if i >= self.rank0(self.raw_data.len()) {
            // error
            return None;
        }
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match self.select0.data(block_idx) {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.select0.data(block_idx + 1);
                while end - start > 1 {
                    let m = (end + start) / 2;
                    let pops = self.rank0(m);
                    if pops > i {
                        end = m
                    } else {
                        start = m
                    }
                }
                start
            },
        };
        Some(ret)
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
    pub fn build(mut self) -> SucBV {
        let rank = RankIndex::new(&self.data);
        let select1 = SimpleSelectIndex::new(&self.data);
        self.data.not();
        let select0 = SimpleSelectIndex::new(&self.data);
        self.data.not();
        SucBV {
            raw_data: self.data, rank, select1, select0
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
    fn test_select1() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut indices = Vec::new();
        let mut builder = SucBVBuilder::new(LENGTH);
        let mut popcnt = 0;
        for i in 0..LENGTH  {
            raw[i] = rng.gen();
            if raw[i] {
                indices.push(i);
                popcnt += 1;
            }
            builder.set(i, raw[i]);
        }
        let vec = builder.build();
        for i in 0..popcnt {
            assert_eq!(indices[i], vec.select1(i).unwrap())
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
