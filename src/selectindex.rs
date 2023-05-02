use crate::bit::WORDSIZE;
use crate::raw_bit_vector::BV;
pub const SELECT_LARGE_BLOCKSIZE: usize = 1 << 10;
const SELECT_DENSE_THRESHOLD: usize = SELECT_LARGE_BLOCKSIZE * SELECT_LARGE_BLOCKSIZE;
pub const SELECT_SMALL_BLOCKSIZE: usize = WORDSIZE;

const SELECT_BRANCH: usize = 6;

pub struct SelectIndex {
    data: Box<[(SelectBox, usize)]>,
}
impl SelectIndex {
    pub fn new(mut bv: BV) -> Self {
        let mut ret = Vec::new();
        let n = bv.len();
        let mut sum = 0;
        let mut start = 0;
        for i in 0..n {
            sum += bv.access(i);
            if sum == SELECT_LARGE_BLOCKSIZE + 1 {
                let x = bv.split_off(i);
                if x.len() < SELECT_DENSE_THRESHOLD {
                    ret.push((SelectBox::Dense(DenseSelectBox::new(x)), start));
                } else {
                    ret.push((SelectBox::Sparse(SparseSelectBox::new(x)), start));
                }
                start = i;
                sum = 1;
            }
        }
        Self {
            data: ret.into_boxed_slice(),
        }
    }
    pub fn data(&self, i: usize) -> &(SelectBox, usize) {
        &self.data[i]
    }
}

pub enum SelectBox {
    Sparse(SparseSelectBox),
    Dense(DenseSelectBox),
}

pub struct SparseSelectBox {
    index: Box<[usize]>,
}

impl SparseSelectBox {
    pub fn new(bv: BV) -> Self {
        let mut ret = Vec::new();
        for i in 0..bv.len() {
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

pub struct DenseSelectBox {
    tree: Box<[u16]>,
    height: usize,
}

impl DenseSelectBox {
    pub fn new(bv: BV) -> Self {
        let blocks = (bv.len() + SELECT_SMALL_BLOCKSIZE - 1) / SELECT_SMALL_BLOCKSIZE;
        let (height, _leaves) = {
            let mut height = 0;
            let mut leaf = 1;
            while leaf < blocks {
                leaf *= SELECT_BRANCH;
                height += 1;
            }
            (height, leaf)
        };
        let mut tree = vec![0; (SELECT_BRANCH.pow(height) - 1) / (SELECT_BRANCH - 1)];
        let inner = (SELECT_BRANCH.pow(height - 1) - 1) / (SELECT_BRANCH - 1);
        for i in 0..blocks {
            tree[inner + i] = bv.word(i).count_ones() as u16;
        }
        for i in (0..inner).rev() {
            for j in 0..SELECT_BRANCH {
                tree[i] += tree[i * SELECT_BRANCH + j];
            }
        }
        Self{tree: tree.into_boxed_slice(), height: height as usize}
   }
    pub fn select(&self, now: usize, i: usize) -> (usize, usize) {
        if now * SELECT_BRANCH + 1 >= self.tree.len() {
            // 今みているのが葉
            let inner = (SELECT_BRANCH.pow(self.height as u32) - 1) / (SELECT_BRANCH - 1);
            return (now - inner, i);
        }
        let mut sum = 0;
        for j in 1..=SELECT_BRANCH {
            let child = now * SELECT_BRANCH + j;
            let ns = sum + self.tree[child] as usize;
            if ns > i {
                return self.select(child, i - sum);
            }
            sum = ns;
        }
        return (0, 0);
    }
}
