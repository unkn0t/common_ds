use super::SegmentTree;

pub struct Iter<'a, T, M> 
where
    T: Copy,
    M: Fn(T, T) -> T,
{
    tree: &'a SegmentTree<T, M>,
    vertex: usize,
}

impl<'a, T, M> Iter<'a, T, M> 
where
    T: Copy,
    M: Fn(T, T) -> T,
{
    const START_VERTEX: usize = SegmentTree::<T, M>::START_VERTEX;

    pub(super) fn new(tree: &'a SegmentTree<T, M>) -> Self {
        Self {
            tree,
            vertex: Self::START_VERTEX,
        }
    }
    
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.vertex >= self.tree.len()
    }

    #[inline]
    pub fn index(&self) -> Option<usize> {
        self.vertex.checked_sub(self.tree.len())
    }

    pub fn parent(&mut self) -> Option<&T> {
        match self.vertex {
            Self::START_VERTEX => None, 
            _ => { 
                self.vertex = super::parent(self.vertex);
                self.tree.data.get(self.vertex)
            }
        }
    }
    
    pub fn left(&mut self) -> Option<&T> {
        if self.is_leaf() {
            None
        } else { 
            self.vertex = left_child(self.vertex);
            self.tree.data.get(self.vertex)
        }
    }
    
    pub fn right(&mut self) -> Option<&T> {
        if self.is_leaf() {
            None
        } else { 
            self.vertex = right_child(self.vertex);
            self.tree.data.get(self.vertex)
        }
    }
}

#[inline]
const fn left_child(vertex: usize) -> usize {
    vertex << 1
}

#[inline]
const fn right_child(vertex: usize) -> usize {
    (vertex << 1) | 1
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Add;

    /// https://leetcode.com/problems/queue-reconstruction-by-height/
    #[test]
    fn queue_reconstruction() {
        let mut query = vec![vec![7,0], vec![4,4], vec![7,1], vec![5,0], vec![6,1], vec![5,2]];
        query.sort();
        
        let len = query.len().next_power_of_two();
        let mut ans = vec![vec![0, 0]; len];

        let values = vec![1; len];
        let mut segtree = SegmentTree::build(&values, <u32 as Add>::add, 0);    
        
        let mut last_height = 0;
        let mut last_height_count = 0;
        for value in query {
            let mut before = value[1] + 1;
            let mut iter = segtree.iter();
            if last_height == value[0] {
                before -= last_height_count;
                last_height_count += 1;
            } else {
                last_height = value[0];
                last_height_count = 1;
            }

            while !iter.is_leaf() {
                let left = *iter.left().unwrap();
                if left < before {
                    before -= left;
                    iter.parent().unwrap();
                    iter.right().unwrap();
                }
            }

            let position = iter.index().unwrap();
            segtree.assign_single(position, 0);
            ans[position] = vec![last_height, value[1]];
        }  

        assert_eq!(&ans[..6], &[vec![5,0],vec![7,0],vec![5,2],vec![6,1],vec![4,4],vec![7,1]]);
    }
}
