use crate::bit::WORDSIZE;
pub const RANK_LARGE_BLOCKSIZE: usize = 1 << 16;
pub const RANK_SMALL_BLOCKSIZE: usize = 256;

pub struct RankIndex {
    large: Box<[u64]>, // 2^16 bitごと
    small: Box<[u16]>,
}

impl RankIndex {
    pub fn new(data: &[usize]) -> Self {
        let n = data.len();
        let nl = WORDSIZE * n / RANK_LARGE_BLOCKSIZE;
        let ns = WORDSIZE * n / RANK_SMALL_BLOCKSIZE;
        let mut large = vec![0 as u64; nl + 1];
        let mut small = vec![0 as u16; ns + 1];
        let mut sum = 0;
        let mut blocksum = 0;
        for i in 0..n {
            if WORDSIZE * i % RANK_LARGE_BLOCKSIZE == 0 {
                large[WORDSIZE * i / RANK_LARGE_BLOCKSIZE] = sum;
                blocksum = 0;
            }
            if WORDSIZE * i % RANK_SMALL_BLOCKSIZE == 0 {
                small[WORDSIZE * i / RANK_SMALL_BLOCKSIZE] = blocksum;
            }
            sum += data[i].count_ones() as u64;
            blocksum += data[i].count_ones() as u16;
        }
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
