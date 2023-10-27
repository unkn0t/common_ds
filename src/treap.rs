use rand::rngs::SmallRng;
use rand::{SeedableRng, RngCore};

#[derive(Clone, Debug)]
pub struct Treap<K: Ord, R = SmallRng> {
    root: Option<Box<Node<K>>>,
    rng: R, 
}

#[derive(Clone, Debug)]
pub struct ImplicitTreap<T, R = SmallRng> {
    root: Option<Box<ImplicitNode<T>>>,
    rng: R, 
}

#[derive(Clone, Debug)]
struct ImplicitNode<T> {
    value: T,
    size: usize,
    priority: u32,
    left: Option<Box<ImplicitNode<T>>>,
    right: Option<Box<ImplicitNode<T>>>,

}

#[derive(Clone, Debug)]
struct Node<K: Ord> {
    key: K,
    priority: u32,
    left: Option<Box<Node<K>>>,
    right: Option<Box<Node<K>>>,
}

//TODO: Check if keys in left are less than keys in right
pub fn merge<K: Ord, R: SeedableRng + RngCore>(mut left: Treap<K, R>, mut right: Treap<K, R>) 
-> Treap<K, R> {
    let root = merge_nodes(left.root.take(), right.root.take());
    Treap::from_root(root)
}

impl<K: Ord> Treap<K, SmallRng> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K: Ord, R: SeedableRng> Default for Treap<K, R> {
    fn default() -> Self {
        let rng = R::from_entropy();
        Self {
            root: None,
            rng,
        }
    } 
}

impl<K: Ord, R: SeedableRng + RngCore> Treap<K, R> {
    pub fn from_seed(seed: R::Seed) -> Self {
        let rng = R::from_seed(seed);
        Self {
            root: None,
            rng,
        }
    }
    
    /// Returns treap with keys greater or equal than key
    /// Left with keys less than key
    pub fn split(&mut self, key: &K) -> Treap<K, R> {
        let (less, greater) = split_node(self.root.take(), key);
        self.root = less;
        Treap::from_root(greater)
    }

    //TODO: Better approach 
    pub fn insert(&mut self, key: K) { 
        let (less, mut greater) = split_node(self.root.take(), &key);
        let new_node = Node::new(key, self.rng.next_u32());
        greater = merge_nodes(new_node.as_root(), greater);
        self.root = merge_nodes(less, greater); 
    }
    
    pub fn contains(&self, key: &K) -> bool {
        let mut node = self.root.as_ref();
        while let Some(nd) = node {
            if nd.key == *key {
                return true;
            }
            
            if nd.key < *key {
                node = nd.right.as_ref(); 
            } else {
                node = nd.left.as_ref();
            }
        }

        false
    }
    
    /// left inclusive
    /// right exclusive
    pub fn remove_range(&mut self, left: &K, right: &K) {
        assert!(left < right);
        let mut greater_left = self.split(left); 
        let mut greater_right = greater_left.split(right);
        self.root = merge_nodes(self.root.take(), greater_right.root.take()); 
    }

    pub fn remove(&mut self, _key: &K) {
        unimplemented!()
    }

    fn from_root(root: Option<Box<Node<K>>>) -> Self {
        let mut this = Self::default();
        this.root = root;
        this
    }
}

pub fn merge_implicit<T, R: SeedableRng + RngCore>(mut left: ImplicitTreap<T, R>, 
    mut right: ImplicitTreap<T, R>) -> ImplicitTreap<T, R> 
{
    let root = merge_implicit_nodes(left.root.take(), right.root.take());
    ImplicitTreap::from_root(root)
}

impl<T, R: SeedableRng> Default for ImplicitTreap<T, R> {
    fn default() -> Self {
        let rng = R::from_entropy();

        Self {
            root: None,
            rng,
        }
    }
}

impl<T> ImplicitTreap<T, SmallRng> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, R: SeedableRng + RngCore> ImplicitTreap<T, R> {
    pub fn from_seed(seed: R::Seed) -> Self {
        let rng = R::from_seed(seed);
        Self {
            root: None,
            rng,
        }
    }
   
    pub fn get(&self, index: usize) -> Option<&T> {
        let mut node = self.root.as_ref();
        let mut index = index;

        while let Some(nd) = node {
            let left_size = node_size(&nd.left);
            
            if left_size == index {
                return Some(&nd.value);
            }
                
            if left_size < index {
                node = nd.right.as_ref();
                index -= left_size + 1;
            } else {
                node = nd.left.as_ref();
            }
        }
        
        None
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut node = self.root.as_mut();
        let mut index = index;

        while let Some(nd) = node {
            let left_size = node_size(&nd.left);
            
            if left_size == index {
                return Some(&mut nd.value);
            }
                
            if left_size < index {
                node = nd.right.as_mut();
                index -= left_size + 1;
            } else {
                node = nd.left.as_mut();
            }
        }
        
        None
    }

    pub fn split(&mut self, index: usize) -> ImplicitTreap<T, R> {
        let (less, greater) = split_implicit_node(self.root.take(), index);
        self.root = less;
        ImplicitTreap::from_root(greater)
    }

    //TODO: Better approach 
    pub fn insert_before(&mut self, index: usize, value: T) { 
        let (less, mut greater) = split_implicit_node(self.root.take(), index);
        let new_node = ImplicitNode::new(value, self.rng.next_u32());
        greater = merge_implicit_nodes(new_node.as_root(), greater);
        self.root = merge_implicit_nodes(less, greater); 
    }
    
    /// left inclusive
    /// right exclusive
    pub fn remove_range(&mut self, left: usize, right: usize) {
        assert!(left < right);
        let mut greater_left = self.split(left);
        let mut greater_right = greater_left.split(right);
        self.root = merge_implicit_nodes(self.root.take(), greater_right.root.take()); 
    }

    //TODO: better approach
    pub fn remove(&mut self, index: usize) {
        self.remove_range(index, index + 1)
    }

    fn from_root(root: Option<Box<ImplicitNode<T>>>) -> Self {
        let mut this = Self::default();
        this.root = root;
        this
    }
}

fn merge_nodes<K: Ord>(left: Option<Box<Node<K>>>, right: Option<Box<Node<K>>>) 
-> Option<Box<Node<K>>> {
    if left.is_none() {
        return right;
    }

    if right.is_none() {
        return left;
    }
    
    let mut left = left.unwrap();
    let mut right = right.unwrap();

    if left.priority > right.priority {
        left.right = merge_nodes(left.right, Some(right));
        Some(left)
    } else {
        right.left = merge_nodes(Some(left), right.left);
        Some(right)
    }
}

fn split_node<K: Ord>(node: Option<Box<Node<K>>>, key: &K) 
-> (Option<Box<Node<K>>>, Option<Box<Node<K>>>) {
    match node {
        None => (None, None),
        Some(mut node) => {
            if node.key < *key {
                let (l, r) = split_node(node.right, key);
                node.right = l;
                (Some(node), r)
            } else {
                let (l, r) = split_node(node.left, key);
                node.left = r;
                (l, Some(node))
            }
        }
    }
}

fn node_size<T>(node: &Option<Box<ImplicitNode<T>>>) -> usize {
    match node {
        None => 0,
        Some(node) => node.size,
    }
}

fn merge_implicit_nodes<T>(left: Option<Box<ImplicitNode<T>>>, right: Option<Box<ImplicitNode<T>>>) 
-> Option<Box<ImplicitNode<T>>> {
    if left.is_none() {
        return right;
    }

    if right.is_none() {
        return left;
    }
    
    let mut left = left.unwrap();
    let mut right = right.unwrap();

    if left.priority > right.priority {
        left.right = merge_implicit_nodes(left.right, Some(right));
        left.update_size();
        Some(left)
    } else {
        right.left = merge_implicit_nodes(Some(left), right.left);
        right.update_size();
        Some(right)
    }
}

fn split_implicit_node<T>(node: Option<Box<ImplicitNode<T>>>, index: usize) 
-> (Option<Box<ImplicitNode<T>>>, Option<Box<ImplicitNode<T>>>) {
    match node {
        None => (None, None),
        Some(mut node) => {
            let left_size = node_size(&node.left);
            if left_size < index {
                let (l, r) = split_implicit_node(node.right, index - left_size - 1);
                node.right = l;
                node.update_size();
                (Some(node), r)
            } else {
                let (l, r) = split_implicit_node(node.left, index);
                node.left = r;
                node.update_size();
                (l, Some(node))
            }
        }
    }
}

impl<T> ImplicitNode<T> {
    fn new(value: T, priority: u32) -> Self {
        Self {
            value,
            size: 1,
            priority,
            left: None,
            right: None,
        }
    }
    
    fn update_size(&mut self) {
        self.size = node_size(&self.left) + node_size(&self.right) + 1;
    }

    fn as_root(self) -> Option<Box<Self>> {
        Some(Box::new(self))
    }
}

impl<K: Ord> Node<K> {
    fn new(key: K, priority: u32) -> Self {
        Self {
            key,
            priority,
            left: None,
            right: None,
        }
    }

    fn as_root(self) -> Option<Box<Self>> {
        Some(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treap_works() {
        let mut treap = Treap::new();
            
        treap.insert(10);
        assert!(treap.contains(&10));
        assert!(!treap.contains(&5));
        treap.insert(5);
        assert!(treap.contains(&10));
        assert!(treap.contains(&5));
        treap.remove_range(&5, &6);
        assert!(treap.contains(&10));
        assert!(!treap.contains(&5));
    }

    #[test]
    fn implicit_treap_works() {
        let mut treap = ImplicitTreap::new();
           
        treap.insert_before(0, 5); // 5
        treap.insert_before(1, 3); // 5 3
        treap.insert_before(1, 4); // 5 4 3
        assert_eq!(treap.get(0), Some(&5));
        assert_eq!(treap.get_mut(1), Some(&mut 4));
        treap.insert_before(0, 2); // 2 5 4 3
        assert_eq!(treap.get(0), Some(&2));
        assert_eq!(treap.get_mut(1), Some(&mut 5));
        treap.insert_before(4, 1); // 2 5 4 3 1
        assert_eq!(treap.get(4), Some(&1));
    }
}
