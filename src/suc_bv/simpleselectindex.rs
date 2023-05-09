use super::raw_bit_vector::BV;

pub const SELECT_LARGE_BLOCKSIZE: usize = 1 << 12;
const SELECT_DENSE_THRESHOLD: usize = 1 << 24;

pub enum SelectBox {
    Sparse(SparseSelectBox),
    Dense,
}

pub struct SimpleSelectIndex {
    data: Box<[(SelectBox, usize)]>,
}
impl SimpleSelectIndex {
    pub fn new(bv: &BV) -> Self {
        let mut ret = Vec::new();
        let n = bv.len();
        let mut sum = 0;
        let mut start = 0;
        for i in 0..n {
            sum += bv.access(i);
            if sum == SELECT_LARGE_BLOCKSIZE + 1 {
                if (i - start) < SELECT_DENSE_THRESHOLD {
                    ret.push((SelectBox::Dense, start));
                } else {
                    ret.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start)), start));
                }
                start = i;
                sum = 1;
            }
        }
        if start != n - 1 {
            if bv.len() < SELECT_DENSE_THRESHOLD {
                ret.push((SelectBox::Dense, start));
            } else {
                ret.push((SelectBox::Sparse(SparseSelectBox::new(&bv, start)), start));
            }
        }
        ret.push((SelectBox::Dense, n));
        Self {
            data: ret.into_boxed_slice(),
        }
    }
    pub fn data(&self, i: usize) -> &(SelectBox, usize) {
        &self.data[i]
    }
}

pub struct SparseSelectBox {
    index: Box<[usize]>,
}

impl SparseSelectBox {
    pub fn new(bv: &BV, start: usize) -> Self {
        let mut ret = Vec::new();
        let n = bv.len();
        for i in start..start + n {
            if bv.access(i) == 1 {
                ret.push(i);
            }
        }
        Self{index: ret.into_boxed_slice()}
    }
    pub fn access(&self, i: usize) -> usize {
        self.index[i]
    }
}
