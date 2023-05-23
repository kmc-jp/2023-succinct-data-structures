use super::BVIndex;
use super::raw_bit_vector::BV;
use crate::bit::*;

pub const SELECT_LARGE_BLOCKSIZE: usize = 1 << 12;
const SELECT_DENSE_THRESHOLD: usize = 1 << 24;
pub const RANK_LARGE_BLOCKSIZE: usize = 1 << 15;
pub const RANK_SMALL_BLOCKSIZE: usize = 256;

#[derive(Debug, Clone)]
pub enum SelectBox {
    Sparse(SparseSelectBox),
    Dense,
}

#[derive(Debug, Clone)]
pub struct SuccinctBVIndex {
    large: Box<[u64]>, // 2^16 bitごと
    small: Box<[u16]>,
    data0: Box<[(SelectBox, usize)]>,
    data1: Box<[(SelectBox, usize)]>,
}

impl BVIndex for SuccinctBVIndex {
    fn new(bv: &BV) -> Self {
        let n = bv.len();
        let (large, small) = {
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
                sum += bv.raw_word(i).count_ones() as u64;
                blocksum += bv.raw_word(i).count_ones() as u16;
            }
            large.push(sum);
            small.push(0);
            let large = large.into_boxed_slice();
            let small = small.into_boxed_slice();
            (large, small)
        };
        let data0 = {
            let mut data0 = Vec::new();
            let mut sum = 0;
            let mut start = 0;
            for i in 0..n {
                sum += (bv.access(i) == 0) as usize;
                if sum == SELECT_LARGE_BLOCKSIZE + 1 {
                    if (i - start) < SELECT_DENSE_THRESHOLD {
                        data0.push((SelectBox::Dense, start));
                    } else {
                        data0.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start, true)), start));
                    }
                    start = i;
                    sum = 1;
                }
            }
            if start != n - 1 {
                if bv.len() < SELECT_DENSE_THRESHOLD {
                    data0.push((SelectBox::Dense, start));
                } else {
                    data0.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start, true)), start));
                }
            }
            data0.push((SelectBox::Dense, n));
            data0
        };
        let data1 = {
            let mut data1 = Vec::new();
            let mut sum = 0;
            let mut start = 0;
            for i in 0..n {
                sum += bv.access(i);
                if sum == SELECT_LARGE_BLOCKSIZE + 1 {
                    if (i - start) < SELECT_DENSE_THRESHOLD {
                        data1.push((SelectBox::Dense, start));
                    } else {
                        data1.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start, false)), start));
                    }
                    start = i;
                    sum = 1;
                }
            }
            if start != n - 1 {
                if bv.len() < SELECT_DENSE_THRESHOLD {
                    data1.push((SelectBox::Dense, start));
                } else {
                    data1.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start, false)), start));
                }
            }
            data1.push((SelectBox::Dense, n));
            data1
        };
        Self {large, small, data0: data0.into_boxed_slice(), data1: data1.into_boxed_slice()}
    }
    fn rank1(&self, bv: &BV, i: usize) -> usize {
        let large_idx = i / RANK_LARGE_BLOCKSIZE;
        let small_idx = i / RANK_SMALL_BLOCKSIZE;
        let word_idx = i / WORDSIZE;
        let mut sum = self.large[large_idx] as usize + self.small[small_idx] as usize;
        for j in small_idx * RANK_SMALL_BLOCKSIZE / WORDSIZE..word_idx {
            sum += bv.raw_word(j).count_ones() as usize;
        }
        sum += word_rank1(bv.raw_word(word_idx), i % WORDSIZE);
        sum as usize
    }
    fn select1(&self, bv: &BV, i: usize) -> usize {
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match &self.data1[block_idx] {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.data1[block_idx + 1];
                while end - start > WORDSIZE {
                    let m = (end + start) / 2;
                    let pops = self.rank1(bv, m);
                    if pops > i {
                        end = m
                    } else {
                        start = m
                    }
                }
                let rem = i - self.rank1(bv, start);
                start + word_select1(bv.word_from_idx(start), rem)
            },
        };
        ret
    }
    fn select0(&self, bv: &BV, i: usize) -> usize {
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match &self.data0[block_idx] {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.data0[block_idx + 1];
                while end - start > WORDSIZE {
                    let m = (end + start) / 2;
                    let pops = self.rank0(bv, m);
                    if pops > i {
                        end = m
                    } else {
                        start = m
                    }
                }
                let rem = i - self.rank0(bv, start);
                start + word_select0(bv.word_from_idx(start), rem)
            },
        };
        ret
    }
}

#[derive(Debug, Clone)]
pub struct SparseSelectBox {
    index: Box<[usize]>,
}

impl SparseSelectBox {
    pub fn new(bv: &BV, start: usize, reverse: bool) -> Self {
        let mut ret = Vec::new();
        let n = bv.len();
        for i in start..start + n {
            if bv.access(i) == !reverse as usize {
                ret.push(i);
            }
        }
        Self{index: ret.into_boxed_slice()}
    }
    pub fn access(&self, i: usize) -> usize {
        self.index[i]
    }
}
