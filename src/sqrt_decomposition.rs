use std::ops::Add;

pub struct SqrtDecomposition<T> {
    data: Vec<T>,
    blocks: Vec<T>,
    block_len: usize,
}

impl<T> SqrtDecomposition<T>
where
    T: Copy + Default + Add<Output = T>
{
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            blocks: Vec::new(),
            block_len: 0,
        }
    }
    
    pub fn build(values: &[T]) -> Self {
        let mut res = Self::new();
        res.init_with(values);
        res
    }

    pub fn init_with(&mut self, values: &[T]) {
        self.data.clear();
        self.blocks.clear();

        self.data.extend_from_slice(values);
        self.block_len = 1 + (values.len() as f32).sqrt() as usize;
        
        for block in self.data.chunks(self.block_len) {
            let sum = block.iter().copied().fold(T::default(), <T as Add<T>>::add);
            self.blocks.push(sum); 
        }
    }

    pub fn sum(&self, left: usize, right: usize) -> T {
        let left_block = left / self.block_len;
        let right_block = right / self.block_len;

        if left_block == right_block {
            self.data[left..=right].iter()
                .copied()
                .fold(T::default(), <T as Add>::add)
        } else {
            let mut res = self.blocks[left_block + 1..right_block].iter()
                .copied()
                .fold(T::default(), <T as Add>::add);
            
            let left_end = (left_block + 1) * self.block_len;
            let right_start = right_block * self.block_len;

            res = res + self.data[left..left_end].iter()
                .copied()
                .fold(T::default(), <T as Add>::add);
                
            res = res + self.data[right_start..=right].iter()
                .copied()
                .fold(T::default(), <T as Add>::add);

            res
        }
    }
}
