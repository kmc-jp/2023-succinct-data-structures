use crate::bit::WORDSIZE;
use super::raw_bit_vector::BV;
pub const RANK_LARGE_BLOCKSIZE: usize = 1 << 16;
pub const RANK_SMALL_BLOCKSIZE: usize = 256;

pub struct RankIndex {
    large: Box<[u64]>, // 2^16 bitごと
    small: Box<[u16]>,
}

impl RankIndex {
    pub fn new(bv: &BV) -> Self {
        let n = bv.len();
        let words = (bv.len() + WORDSIZE - 1) / WORDSIZE;
        let nl = (n + RANK_LARGE_BLOCKSIZE - 1) / RANK_LARGE_BLOCKSIZE;
        let ns = (n + RANK_SMALL_BLOCKSIZE - 1) / RANK_SMALL_BLOCKSIZE;
        let mut large = vec![0 as u64; nl];
        let mut small = vec![0 as u16; ns];
        let mut sum = 0;
        let mut blocksum = 0;
        for i in 0..words {
            if WORDSIZE * i % RANK_LARGE_BLOCKSIZE == 0 {
                large[WORDSIZE * i / RANK_LARGE_BLOCKSIZE] = sum;
                blocksum = 0;
            }
            if WORDSIZE * i % RANK_SMALL_BLOCKSIZE == 0 {
                small[WORDSIZE * i / RANK_SMALL_BLOCKSIZE] = blocksum;
            }
            sum += bv.word(i).count_ones() as u64;
            blocksum += bv.word(i).count_ones() as u16;
        }
        // TODO:最後のブロックの処理をしていません
        let large = large.into_boxed_slice();
        let small = small.into_boxed_slice();
        Self {large, small}
    }
    pub fn large(&self, i:usize) -> u64 {
        self.large[i]
    }
    pub fn small(&self, i:usize) -> u16 {
        self.small[i]
    }
}
