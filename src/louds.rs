use crate::suc_bv::{BitVector, SucBV, SucBVBuilder, SuccinctBVIndex};
use std::collections::VecDeque;

pub struct Louds<V: BitVector> {
    bv: V,
}
impl<V: BitVector> Louds<V> {
    pub fn bfs_rank(&self, x: usize) -> usize {
        self.bv.rank1(x)
    }
    pub fn bfs_select(&self, i: usize) -> usize {
        self.bv.select1(i).unwrap()
    }
    pub fn parent_rank(&self, x: usize) -> usize {
        self.bv.rank0(x - 1)
    }
    pub fn first_child_select(&self, i: usize) -> usize {
        self.bv.select0(i).unwrap() + 1
    }
    pub fn isleaf(&self, x: usize) -> bool {
        if self.bv.access(self.first_child_select(self.bfs_rank(x))) == 0 {
            true
        } else{
            false
        }
    }
    pub fn parent(&self, x: usize) -> usize {
        self.bfs_select(self.parent_rank(x))
    }
    pub fn firstchild(&self, x: usize) -> Option<usize> {
        let y = self.first_child_select(self.bfs_rank(x));
        match self.bv.access(y) {
            0 => None,
            _ => Some(y),
        }
    }
    pub fn lastchild(&self, x: usize) -> Option<usize> {
        let y = self.bv.select0(self.bfs_rank(x) + 1).unwrap() - 1;
        match self.bv.access(y) {
            0 => None,
            _ => Some(y),
        }
    }
    pub fn sibling(&self, x: usize) -> Option<usize> {
        match self.bv.access(x + 1) {
            0 => None,
            _ => Some(x + 1),
        }
    }
    pub fn degree(&self, x: usize) -> usize {
        match self.isleaf(x) {
            true => 0,
            false => self.lastchild(x).unwrap() - self.firstchild(x).unwrap() + 1,
        }
    }
    pub fn child(&self, x: usize, i: usize) -> Option<usize> {
        match i > self.degree(x) {
            true => None,
            false => Some(self.firstchild(x).unwrap() + i),
        }
    }
    pub fn childrank(&self, x: usize) -> usize {
        x - self.firstchild(self.parent(x)).unwrap()
    }
}

pub struct LoudsBuilder {
    tree: Vec<Vec<usize>>,
    vertex: usize,
}
// 0番目の頂点を根とする
impl LoudsBuilder {
    pub fn new(n: usize) -> Self {
        let tree = vec![Vec::new(); n];
        Self {
            tree, vertex: n
        }
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.tree[u].push(v);
    }
    pub fn build(self) -> (Louds<SucBV<SuccinctBVIndex>>, Vec<usize>) {
        let mut builder = SucBVBuilder::new(2 * self.vertex + 1);
        builder.set(0, true);
        builder.set(1, false);
        let mut queue = VecDeque::new();
        let mut bfs_order = Vec::new();
        let mut bv_idx = 2;
        queue.push_back(0);
        bfs_order.push(0);
        while !queue.is_empty() {
            let p = queue.pop_front().unwrap();
            for &c in &self.tree[p] {
                queue.push_back(c);
                bfs_order.push(c);
                builder.set(bv_idx, true);
                bv_idx += 1;
            }
            builder.set(bv_idx, false);
            bv_idx += 1;
        }
        let ret = Louds {bv: builder.build()};
        (ret, bfs_order)
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn louds_test() {
        let mut builder = SucBVBuilder::new(21);
        let a = "101101110110001010000";
        for (i, bit) in a.bytes().enumerate() {
            let b = bit - 48;
            builder.set(i, b == 1);
        }
        let bv = builder.build();
        let louds = Louds{bv};
        assert_eq!(louds.bfs_rank(0), 0);
        assert_eq!(louds.bfs_rank(2), 1);
        assert_eq!(louds.bfs_rank(3), 2);
        assert_eq!(louds.bfs_select(0), 0);
        assert_eq!(louds.bfs_select(2), 3);
    }
    #[test]
    fn louds_builder_test() {
        let mut builder = LoudsBuilder::new(10);
        builder.add_edge(0, 1);
        builder.add_edge(0, 2);
        builder.add_edge(1, 3);
        builder.add_edge(1, 4);
        builder.add_edge(1, 5);
        builder.add_edge(2, 6);
        builder.add_edge(2, 7);
        builder.add_edge(3, 8);
        builder.add_edge(6, 9);
        let (louds, bfs_order) = builder.build();
        assert_eq!(louds.bfs_rank(0), 0);
        assert_eq!(louds.bfs_rank(2), 1);
        assert_eq!(louds.bfs_rank(3), 2);
        assert_eq!(louds.bfs_select(0), 0);
        assert_eq!(louds.bfs_select(2), 3);
        dbg!(bfs_order);
    }
}
