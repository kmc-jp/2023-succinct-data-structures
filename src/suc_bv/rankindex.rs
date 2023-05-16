use crate::bit::{WORDSIZE, word_rank1};
use super::raw_bit_vector::BV;
use super::RankIndex;
pub const RANK_LARGE_BLOCKSIZE: usize = 1 << 15;
pub const RANK_SMALL_BLOCKSIZE: usize = 256;

#[derive(Debug)]
pub struct SuccinctRankIndex {
    large: Box<[u64]>, // 2^16 bitごと
    small: Box<[u16]>,
}

impl RankIndex for SuccinctRankIndex {
    fn new(bv: &BV) -> Self {
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
        // DONE:最後のブロックの処理をしていません
        large.push(sum);
        small.push(0);
        let large = large.into_boxed_slice();
        let small = small.into_boxed_slice();
        Self {large, small}
    }
    fn rank1(&self, bv: &BV, i: usize) -> usize {
        let large_idx = i / RANK_LARGE_BLOCKSIZE;
        let small_idx = i / RANK_SMALL_BLOCKSIZE;
        let word_idx = i / WORDSIZE;
        let mut sum = self.large[large_idx] as usize + self.small[small_idx] as usize;
        for j in small_idx * RANK_SMALL_BLOCKSIZE / WORDSIZE..word_idx {
            sum += bv.word(j).count_ones() as usize;
        }
        sum += word_rank1(bv.word(word_idx), i % WORDSIZE);
        sum as usize
    }
}
