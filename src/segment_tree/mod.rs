/// ---------------------------------------------------
/// Inspired by https://codeforces.com/blog/entry/18051
/// ---------------------------------------------------

mod iter;

pub use iter::Iter;

use std::ops::{RangeBounds, Bound};

/// We can not ensure this requirments with Rust
/// (1) merge(a, neutral) = a
/// (2) merge(merge(a, b), c) = merge(a, merge(b, c))
pub struct SegmentTree<T, M> 
where
    T: Copy,
    M: Fn(T, T) -> T,
{
    data: Vec<T>,
    merge_fn: M,
    neutral: T,
}


/// We can not ensure this requirments with Rust
/// (1) lazy(a, neutral) = a
/// (2) lazy(merge(a, b), c) = merge(lazy(a, c), lazy(b, c))
pub struct LazySegmentTree<T, M, L> 
where
    T: Copy,
    M: Fn(T, T) -> T,
    L: Fn(T, T) -> T,
{
    tree: SegmentTree<T, M>,
    delayed: Vec<T>,
    lazy_fn: L,
    neutral: T,
}


/// We can not ensure this requirments with Rust
/// (1) segment(a, k) = merge(merge(..., a), a) {k times}
pub struct AssignmentSegmentTree<T, M, S> 
where
    T: Copy,
    M: Fn(T, T) -> T,
    S: Fn(T, usize) -> T,
{
    tree: SegmentTree<T, M>,
    delayed: Vec<Option<T>>,
    segment_fn: S,
}

impl<T, M> SegmentTree<T, M> 
where
    T: Copy,
    M: Fn(T, T) -> T,
{
    const START_VERTEX: usize = 1;

    pub fn new(merge_fn: M, neutral: T) -> Self {
        Self {
            data: Vec::new(),
            merge_fn,
            neutral,
        }
    }

    pub fn build(values: &[T], merge: M, neutral: T) -> Self {
        let mut res = Self::new(merge, neutral);
        res.init_with(values);
        res
    }

    pub fn init_with(&mut self, values: &[T]) {
        let len = values.len();

        self.data.clear();
        self.data.resize(len, self.neutral);
        self.data.extend_from_slice(values);
    
        for vertex in (1..len).rev() {
            let (left, right) = children(vertex);
            self.data[vertex] = self.merge(self.data[left], self.data[right]); 
        }
    }
    
    pub fn assign_single(&mut self, position: usize, value: T) {
        let mut vertex = self.vertex_from_position(position);
        self.data[vertex] = value;

        while vertex > Self::START_VERTEX {
            vertex = parent(vertex);
            let (left, right) = children(vertex);
            self.data[vertex] = self.merge(self.data[left], self.data[right]); 
        }
    }
    
    pub fn query_range<R: RangeBounds<usize>>(&self, range: R) -> T {
        let (left, right) = self.range_into_segment(range);
        self.query(left, right)
    }

    pub fn query(&self, left: usize, right: usize) -> T {
        let mut left_res = self.neutral;
        let mut right_res = self.neutral;
        let mut left_vertex = self.vertex_from_position(left);
        let mut right_vertex = self.vertex_from_position(right + 1);

        while left_vertex < right_vertex {
            if (left_vertex & 1) == 1 {
                left_res = self.merge(left_res, self.data[left_vertex]);
                left_vertex += 1;
            }
            
            if (right_vertex & 1) == 1 {
                right_vertex -= 1;
                right_res = self.merge(self.data[right_vertex], right_res);
            }

            left_vertex = parent(left_vertex);
            right_vertex = parent(right_vertex);
        }

        self.merge(left_res, right_res)
    }
    
    // FIXME: we want to use iter to solve problems 
    // like find kth zero, but now we can only use
    // it when self.len() is power of 2
    pub fn iter(&self) -> Iter<'_, T, M> {
        assert!(self.len().is_power_of_two());
        Iter::new(&self)
    }

    pub fn with_assignment<S: Fn(T, usize) -> T>(self, segment_fn: S) -> AssignmentSegmentTree<T, M, S> {
        AssignmentSegmentTree::new(self, segment_fn)
    }
 
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() >> 1
    }
    
    #[inline]
    fn merge(&self, left: T, right: T) -> T {
        (self.merge_fn)(left, right)
    }
    
    #[inline]
    fn vertex_from_position(&self, position: usize) -> usize {
        position + self.len()
    }

    fn range_into_segment<R: RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        let left = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(before_start) => *before_start + 1,
            _ => 0,
        };

        let right = match range.end_bound() {
            Bound::Included(end) => *end,
            Bound::Excluded(after_end) => *after_end - 1,
            _ => self.len() - 1,
        };

        (left, right)
    }
}

impl<T, M> SegmentTree<T, M> 
where
    T: Copy + Eq,
    M: Fn(T, T) -> T,
{
    pub fn with_lazy<L: Fn(T, T) -> T>(self, lazy_fn: L, neutral: T) -> LazySegmentTree<T, M, L> {
        LazySegmentTree::new(self, lazy_fn, neutral)
    }
}

impl<T, M, L> LazySegmentTree<T, M, L> 
where
    T: Copy + Eq,
    M: Fn(T, T) -> T,
    L: Fn(T, T) -> T,
{
    const START_VERTEX: usize = SegmentTree::<T, M>::START_VERTEX;

    pub fn new(tree: SegmentTree<T, M>, lazy_fn: L, neutral: T) -> Self {
        let delayed = vec![neutral; tree.len()];

        Self {
            tree,
            delayed,
            lazy_fn,
            neutral,
        }
    }
   
    pub fn modify_single(&mut self, position: usize, value: T) {
        let vertex = self.tree.vertex_from_position(position);
        self.apply(vertex, value);
        self.build(vertex);
    }
 
    pub fn modify_range<R: RangeBounds<usize>>(&mut self, range: R, value: T) {
        let (left, right) = self.tree.range_into_segment(range);
        self.modify(left, right, value)
    }

    pub fn modify(&mut self, left: usize, right: usize, value: T) {
        let mut left_vertex = self.tree.vertex_from_position(left);
        let mut right_vertex = self.tree.vertex_from_position(right + 1);
        
        let left_subtree = left_vertex;
        let right_subtree = right_vertex - 1;
        
        while left_vertex < right_vertex {
            if (left_vertex & 1) == 1 {
                self.apply(left_vertex, value);
                left_vertex += 1;
            }
            
            if (right_vertex & 1) == 1 {
                right_vertex -= 1;
                self.apply(right_vertex, value);
            }

            left_vertex = parent(left_vertex);
            right_vertex = parent(right_vertex);
        }

        self.build(left_subtree);
        self.build(right_subtree);
    }
   
    pub fn query_range<R: RangeBounds<usize>>(&mut self, range: R) -> T {
        let (left, right) = self.tree.range_into_segment(range);
        self.query(left, right)
    }

    pub fn query(&mut self, left: usize, right: usize) -> T {
        let left_vertex = self.tree.vertex_from_position(left);
        let right_vertex = self.tree.vertex_from_position(right + 1);
        
        self.push(left_vertex);
        self.push(right_vertex - 1);
   
        self.tree.query(left, right)
    }   
    
    pub fn init_with(&mut self, values: &[T]) {
        self.tree.init_with(values);        
        self.delayed.clear();
        self.delayed.resize(values.len(), self.neutral);
    }

    fn build(&mut self, vertex: usize) {
        let mut vertex = vertex;

        while vertex > Self::START_VERTEX {
            vertex = parent(vertex);
            let (left, right) = children(vertex);
            self.tree.data[vertex] = self.lazy(
                self.tree.merge(self.tree.data[left], self.tree.data[right]),
                self.delayed[vertex]
            );
        }
    }

    fn push(&mut self, vertex: usize) {
        for bit in (1..=self.height()).rev() {
            let ancestor = vertex >> bit;

            if self.delayed[ancestor] != self.neutral {
                let (left, right) = children(ancestor);
                self.apply(left, self.delayed[ancestor]); 
                self.apply(right, self.delayed[ancestor]); 
                self.delayed[ancestor] = self.neutral;
            }
        }
    }

    fn apply(&mut self, vertex: usize, value: T) {
        self.tree.data[vertex] = self.lazy(self.tree.data[vertex], value);
        
        if self.is_not_leaf(vertex) {
            self.delayed[vertex] = self.lazy(self.delayed[vertex], value); 
        }
    }

    #[inline]
    fn height(&self) -> usize {
        let len = self.tree.len();
        (usize::BITS - len.leading_zeros()) as usize
    }

    #[inline]
    fn is_not_leaf(&self, vertex: usize) -> bool {
        vertex < self.tree.len()
    }

    #[inline]
    fn lazy(&self, left: T, right: T) -> T {
        (self.lazy_fn)(left, right)
    }
}

impl<T, M, S> AssignmentSegmentTree<T, M, S> 
where
    T: Copy,
    M: Fn(T, T) -> T,
    S: Fn(T, usize) -> T,
{
    const START_VERTEX: usize = SegmentTree::<T, M>::START_VERTEX;

    pub fn new(tree: SegmentTree<T, M>, segment_fn: S) -> Self {
        let delayed = vec![None; tree.len()];

        Self {
            tree,
            delayed,
            segment_fn,
        }
    }
   
    pub fn assign_single(&mut self, position: usize, value: T) {
        self.push(position, position + 1);

        let vertex = self.tree.vertex_from_position(position);
        self.apply(vertex, value, 1);
        self.build(position, position + 1);
    }
 
    pub fn assign_range<R: RangeBounds<usize>>(&mut self, range: R, value: T) {
        let (left, right) = self.tree.range_into_segment(range);
        self.assign(left, right, value)
    }

    pub fn assign(&mut self, left: usize, right: usize, value: T) {
        self.push(left, left + 1);
        self.push(right, right + 1);
        
        let mut left_vertex = self.tree.vertex_from_position(left);
        let mut right_vertex = self.tree.vertex_from_position(right + 1);
        let mut seg_len = 1;

        while left_vertex < right_vertex {
            if (left_vertex & 1) == 1 {
                self.apply(left_vertex, value, seg_len);
                left_vertex += 1;
            }
            
            if (right_vertex & 1) == 1 {
                right_vertex -= 1;
                self.apply(right_vertex, value, seg_len);
            }

            left_vertex = parent(left_vertex);
            right_vertex = parent(right_vertex);
            seg_len <<= 1;
        }

        self.build(left, left + 1);
        self.build(right, right + 1);
    }
   
    pub fn query_range<R: RangeBounds<usize>>(&mut self, range: R) -> T {
        let (left, right) = self.tree.range_into_segment(range);
        self.query(left, right)
    }

    pub fn query(&mut self, left: usize, right: usize) -> T {
        self.push(left, left + 1);
        self.push(right, right + 1);
   
        self.tree.query(left, right)
    }   
    
    pub fn init_with(&mut self, values: &[T]) {
        self.tree.init_with(values);        
        self.delayed.clear();
        self.delayed.resize(values.len(), None);
    }

    fn build(&mut self, left: usize, right: usize) {
        let mut seg_len = 2;
        
        let mut left_vertex = self.tree.vertex_from_position(left);
        let mut right_vertex = self.tree.vertex_from_position(right - 1);

        while left_vertex > Self::START_VERTEX {
            left_vertex = parent(left_vertex);
            right_vertex = parent(right_vertex);
            
            for vertex in (left_vertex..=right_vertex).rev() {
                self.recalculate(vertex, seg_len);
            }

            seg_len <<= 1;
        }
    }

    fn recalculate(&mut self, vertex: usize, seg_len: usize) {
        match self.delayed[vertex] {
            Some(delayed) => self.tree.data[vertex] = self.segment(delayed, seg_len),
            None => {
                let (left, right) = children(vertex);
                self.tree.data[vertex] = self.tree.merge(self.tree.data[left], self.tree.data[right]);
            } 
        }
    }

    fn push(&mut self, left: usize, right: usize) {
        let mut height = self.height();
        let mut seg_len = 1 << (height - 1);

        let left_vertex = self.tree.vertex_from_position(left);
        let right_vertex = self.tree.vertex_from_position(right - 1);

        while height > 0 {
            for vertex in (left_vertex >> height)..=(right_vertex >> height) {
                if let Some(delayed) = self.delayed[vertex] {
                    let (left_child, right_child) = children(vertex);
                    self.apply(left_child, delayed, seg_len); 
                    self.apply(right_child, delayed, seg_len); 
                    self.delayed[vertex] = None;
                }
            }

            seg_len >>= 1;
            height -= 1;
        }
    }

    fn apply(&mut self, vertex: usize, value: T, seg_len: usize) {
        self.tree.data[vertex] = self.segment(value, seg_len);
        
        if self.is_not_leaf(vertex) {
            self.delayed[vertex] = Some(value); 
        }
    }

    #[inline]
    fn height(&self) -> usize {
        let len = self.tree.len();
        (usize::BITS - len.leading_zeros()) as usize
    }

    #[inline]
    fn is_not_leaf(&self, vertex: usize) -> bool {
        vertex < self.tree.len()
    }

    #[inline]
    fn segment(&self, value: T, len: usize) -> T {
        (self.segment_fn)(value, len)
    }
}

#[inline]
const fn parent(vertex: usize) -> usize {
    vertex >> 1
}

#[inline]
const fn children(vertex: usize) -> (usize, usize) {
    let left = vertex << 1;
    (left, left | 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Add;

    #[test]
    fn segment_tree() {
        let mut values = [1, 3, 2, 5, 4];
        let mut segtree = SegmentTree::build(&values, i32::min, i32::MAX);

        segtree.assign_single(2, 4);
        values[2] = 4;

        for l in 0..values.len() {
            for r in l..values.len() {
                assert_eq!(segtree.query(l, r), *values[l..=r].iter().min().unwrap(), "l: {l}, r: {r}");
            }
        }
    }

    #[test]
    fn lazy_segment_tree() {
        let mut values = [1, 3, 2, 5, 4];
        let mut segtree = SegmentTree::build(&values, i32::max, i32::MIN)
            .with_lazy(<i32 as Add>::add, 0);

        segtree.modify(0, 2, 2);
        
        for i in 0..=2 {
            values[i] += 2;
        }

        for l in 0..values.len() {
            for r in l..values.len() {
                assert_eq!(segtree.query(l, r), *values[l..=r].iter().max().unwrap(), "l: {l}, r: {r}");
            }
        }
    }
    
    #[test]
    fn assignment_segment_tree() {
        let mut values = [1, 3, 2, 5, 4];
        let mut segtree = SegmentTree::build(&values, i32::min, i32::MAX)
            .with_assignment(|x, _k| x);

        segtree.assign(0, 2, 2);
        
        for i in 0..=2 {
            values[i] = 2;
        }

        for l in 0..values.len() {
            for r in l..values.len() {
                assert_eq!(segtree.query(l, r), *values[l..=r].iter().min().unwrap(), "l: {l}, r: {r}");
            }
        }
    }

    #[test]
    fn range_query() {
        let values = [1, 3, 2, 5, 4];
        let segtree = SegmentTree::build(&values, <i32 as Add>::add, 0);

        assert_eq!(segtree.query_range(..), values[..].iter().fold(0, <i32 as Add<&i32>>::add));
        assert_eq!(segtree.query_range(1..), values[1..].iter().fold(0, <i32 as Add<&i32>>::add));
        assert_eq!(segtree.query_range(..=3), values[..=3].iter().fold(0, <i32 as Add<&i32>>::add));
        assert_eq!(segtree.query_range(2..4), values[2..4].iter().fold(0, <i32 as Add<&i32>>::add));
    }
}

