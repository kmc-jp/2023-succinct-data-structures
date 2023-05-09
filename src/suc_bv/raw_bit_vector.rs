use crate::bit::{WORDSIZE, word_access, shrd};

#[derive(Clone)]
pub struct BV {
    data: Vec<usize>,
    length: usize,
}

impl BV {
    pub fn new(length: usize) -> Self{
        Self{data: vec![0; (length + WORDSIZE - 1) / WORDSIZE], length}
    }
    pub fn access(&self, idx: usize) -> usize {
        if idx >= self.length {
            return 0;
        }
        word_access(self.data[idx / WORDSIZE], idx % WORDSIZE)
    }
    pub fn set(&mut self, idx: usize, bit: bool) {
        if bit {
            self.data[idx / WORDSIZE] |= 1 << idx % WORDSIZE;
        } else {
            self.data[idx / WORDSIZE] &= !(1 << idx % WORDSIZE);
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn word(&self, word_idx: usize) -> usize {
        self.data[word_idx]
    }
    pub fn shiftr(&mut self, shift: usize) {
        // 下位の方向に移動
        for i in 0..(self.data.len() - 1) {
            self.data[i] = shrd(self.data[i], self.data[i + 1], shift);
        }
        self.length -= shift;
        if (self.length + WORDSIZE - 1) / WORDSIZE < self.data.len() {
            self.data.pop();
        }
    }
    pub fn split_off(&mut self, idx: usize) -> Self {
        let word_idx = idx / WORDSIZE;
        let mut v1 = self.data.split_off(idx);
        let a = self.data[0];
        let inword = idx % WORDSIZE;
        if inword != 0 {
            v1.push(a & ((1 << inword) - 1));
        }
        self.data[0] &= !((1 << inword) - 1);
        self.length -= word_idx * WORDSIZE;
        self.shiftr(inword);
        Self{data: v1, length: idx}
    }
}
