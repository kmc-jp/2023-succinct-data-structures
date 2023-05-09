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
        if word_idx < self.data.len() { self.data[word_idx] } else { 0 }
    }
    pub fn not(&mut self) {
        for i in 0..self.data.len() {
            self.data[i] = !self.data[i];
        }
    }
    fn shrink(&mut self) {
        let t = (self.length + WORDSIZE - 1) / WORDSIZE;
        while t < self.data.len() {
            self.data.pop();
        }
    }
    fn shiftr(&mut self, shift: usize) {
        // 下位の方向に移動
        // shift < 2 * WORDSIZE
        let n = self.data.len();
        for i in 0..(n - 1) {
            self.data[i] = shrd(self.data[i], self.data[i + 1], shift);
        }
        self.data[n - 1] = shrd(self.data[n - 1], 0, shift);
        self.length -= shift;
        self.shrink();
    }
    pub fn split_off(&mut self, idx: usize) -> Self {
        let word_idx = idx / WORDSIZE;
        let mut v1 = self.data.split_off(word_idx);
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

#[cfg(test)]
mod test {
    use rand::Rng;
    use super::*;
    const LENGTH: usize = (1 << 20);
    #[test]
    fn test_split_off() {
        let mut rng = rand::thread_rng();
        let mut raw = vec![false; LENGTH];
        let mut vec = BV::new(LENGTH);
        for i in 0..LENGTH  {
            raw[i] = rng.gen();
            vec.set(i, raw[i]);
        }
        let v1 = vec.split_off(LENGTH / 3);
        assert_eq!(v1.len(), LENGTH / 3);
        assert_eq!(vec.len(), LENGTH - LENGTH / 3);
        assert_eq!(v1.len() + vec.len(), LENGTH);
    }
}
