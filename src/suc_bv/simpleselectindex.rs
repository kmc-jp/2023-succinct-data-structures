use super::raw_bit_vector::BV;
use super::{SelectIndex, RankIndex};

pub const SELECT_LARGE_BLOCKSIZE: usize = 1 << 12;
const SELECT_DENSE_THRESHOLD: usize = 1 << 24;

pub enum SelectBox {
    Sparse(SparseSelectBox),
    Dense,
}

pub struct SimpleSelectIndex {
    data0: Box<[(SelectBox, usize)]>,
    data1: Box<[(SelectBox, usize)]>,
}
impl<R: RankIndex> SelectIndex<R> for SimpleSelectIndex {
    fn new(bv: &BV) -> Self {
        let n = bv.len();
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
        Self {data0: data0.into_boxed_slice(), data1: data1.into_boxed_slice()}
    }
    fn select1(&self, bv: &BV, rank: &R, i: usize) -> Option<usize> {
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match &self.data1[block_idx] {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.data1[block_idx + 1];
                while end - start > 1 {
                    let m = (end + start) / 2;
                    let pops = rank.rank1(bv, m);
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
    fn select0(&self, bv: &BV, rank: &R, i: usize) -> Option<usize> {
        let block_idx = i / SELECT_LARGE_BLOCKSIZE;
        let ret = match &self.data0[block_idx] {
            (SelectBox::Sparse(a), _) => a.access(i % SELECT_LARGE_BLOCKSIZE),
            (SelectBox::Dense, mut start) => {
                let (_, mut end) = self.data0[block_idx + 1];
                while end - start > 1 {
                    let m = (end + start) / 2;
                    let pops = rank.rank0(bv, m);
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
