use crate::rankindex::{RANK_LARGE_BLOCKSIZE, RANK_SMALL_BLOCKSIZE, RankIndex};
use crate::selectindex::*;
use crate::selectindex::SelectBox::*;
use crate::bit::*;
use crate::raw_bit_vector::BV;

pub struct SucBV {
    raw_data: BV,
    rank: RankIndex,
    select1: SelectIndex,
    // select0: SelectIndex,
}

impl SucBV {
    pub fn access(&self, i: usize) -> usize {
        self.raw_data.access(i)
    }
    pub fn rank1(&self, i: usize) -> usize {
        let large_idx = i / RANK_LARGE_BLOCKSIZE;
        let small_idx = i / RANK_SMALL_BLOCKSIZE;
        let word_idx = i / WORDSIZE;
        let mut sum = self.rank.large(large_idx) as usize + self.rank.small(small_idx) as usize;
        for j in small_idx * RANK_SMALL_BLOCKSIZE..word_idx {
            sum += self.raw_data.word(j).count_ones() as usize;
        }
        sum += word_rank1(self.raw_data.word(word_idx), i % WORDSIZE);
        sum as usize
    }
    pub fn rank0(&self, i: usize) -> usize {
        i - self.rank1(i)
    }
    pub fn select1(&self, i: usize) -> usize {
        let large_idx = i / SELECT_LARGE_BLOCKSIZE;
        match self.select1.data(large_idx) {
            (Sparse(a), start) => start + a.access(i % SELECT_LARGE_BLOCKSIZE),
            (Dense(a), start) => {
                let (block_idx, rest) = a.select(0, i % SELECT_LARGE_BLOCKSIZE);
                let start = start + block_idx * SELECT_SMALL_BLOCKSIZE;
                let word_idx = start / WORDSIZE;
                let word = shrd(self.raw_data.word(word_idx + 1), self.raw_data.word(word_idx), start % WORDSIZE);
                word_select1(word, rest)
            },
        }
    }
    pub fn select0(&self, _i: usize) -> usize {
        todo!()
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
    pub fn build(self) -> SucBV {
        let rank = RankIndex::new(&self.data);
        let select1 = SelectIndex::new(self.data.clone());
        SucBV {
            raw_data: self.data, rank, select1, //select0: select0
        }
    }
}
