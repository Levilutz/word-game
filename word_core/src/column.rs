use std::ops;

/// A simple column of booleans packed into a u64 for performant binary ops.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column {
    len: usize,
    col: Vec<u64>,
}

impl Column {
    /// Generate a column with `len` true values
    pub fn from_true(len: usize) -> Self {
        let num_chunks = (len + 63) / 64;
        Self {
            len,
            col: vec![u64::MAX; num_chunks],
        }
    }

    /// Generate a column with `len` false values
    pub fn from_false(len: usize) -> Self {
        let num_chunks = (len + 63) / 64;
        Self {
            len,
            col: vec![0; num_chunks],
        }
    }

    /// Generate a set of 1-hot columns from a row of ints.
    pub fn one_hot_values(values: &[u64], max_val: u64) -> Vec<Self> {
        let mut cols = vec![Self::from_false(values.len()); max_val as usize];
        for (i, val) in values.iter().enumerate() {
            cols[*val as usize].set(i, true);
        }
        cols
    }

    /// Generate a column from a list of bools
    pub fn from_bools(bools: &[bool]) -> Self {
        let num_chunks = (bools.len() + 63) / 64; // Divide & round-up
        let mut col = Vec::with_capacity(num_chunks);

        for chunk in bools.chunks(64) {
            let mut value = 0;
            for (i, bit) in chunk.iter().enumerate() {
                value |= (*bit as u64) << i;
            }
            col.push(value);
        }

        Self {
            len: bools.len(),
            col,
        }
    }

    /// Reconstruct a vec of bools from a column
    pub fn to_bools(&self) -> Vec<bool> {
        let mut out = Vec::with_capacity(self.len);

        for (chunk_ind, value) in self.col.iter().enumerate() {
            for bit_ind in 0..64 {
                if chunk_ind * 64 + bit_ind >= self.len {
                    break;
                }
                out.push((*value & (1 << bit_ind)) != 0);
            }
        }

        out
    }

    /// Get the number of items stored in this col
    pub fn len(&self) -> usize {
        self.len
    }

    /// Count how many true values exist in this col
    pub fn count_true(&self) -> u64 {
        let (full_chunks, partial_chunk) = self.by_chunk_fill();
        let mut out = full_chunks
            .iter()
            .map(|chunk| chunk.count_ones() as u64)
            .sum();
        if let Some(partial_chunk) = partial_chunk {
            out += (first_n_bits(self.len as u64 % 64) & partial_chunk).count_ones() as u64
        }
        out
    }

    /// Count how many false values exist in this col
    pub fn count_false(&self) -> u64 {
        self.len as u64 - self.count_true()
    }

    /// Get the indices in the column that have true.
    pub fn true_inds(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.count_true() as usize);
        for (chunk_ind, value) in self.col.iter().enumerate() {
            if *value == 0 {
                continue;
            }
            for bit_ind in 0..64 {
                let global_ind = chunk_ind * 64 + bit_ind;
                if global_ind >= self.len {
                    break;
                }
                if (*value & (1 << bit_ind)) != 0 {
                    out.push(global_ind);
                }
            }
        }
        out
    }

    /// Get the value at a particular ind
    pub fn get(&self, ind: usize) -> bool {
        if ind >= self.len {
            panic!("Cannot access col ind {} with len {}", ind, self.len)
        }
        return (self.col[ind / 64] & (1 << (ind % 64))) != 0;
    }

    /// Set the value at a particular ind
    pub fn set(&mut self, ind: usize, val: bool) {
        if ind >= self.len {
            panic!("Cannot set col ind {} with len {}", ind, self.len)
        }
        if val {
            self.col[ind / 64] |= 1 << (ind % 64);
        } else {
            self.col[ind / 64] &= !(1 << (ind % 64));
        }
    }

    /// Get a new column with only the entries with indices in the given list.
    ///
    /// ```rs
    /// let bools = vec![true, false, true, false, true, false];
    /// let col = Column::from_bools(&bools);
    /// assert_eq!(col.filter(&[2, 3, 4]), vec![true, false, true]);
    /// ```
    pub fn filter(&self, inds: &[usize]) -> Self {
        let mut out = Self::from_false(inds.len());
        inds.iter()
            .enumerate()
            .filter(|(_new_ind, old_ind)| self.get(**old_ind))
            .for_each(|(new_ind, _old_ind)| out.set(new_ind, true));
        out
    }

    /// Return all of the full chunks and optionally a non-full end chunk
    fn by_chunk_fill(&self) -> (&[u64], Option<u64>) {
        if self.len % 64 == 0 {
            (&self.col, None)
        } else {
            (
                &self.col[0..self.col.len() - 1],
                Some(self.col[self.col.len() - 1]),
            )
        }
    }
}

impl ops::BitAndAssign for Column {
    /// Bitwise and the rhs into this value. Will panic if different length.
    fn bitand_assign(&mut self, rhs: Self) {
        if self.len != rhs.len {
            panic!("Cannot &= columns of length {} != {}", self.len, rhs.len);
        }
        self.col
            .iter_mut()
            .zip(rhs.col.iter())
            .for_each(|(item, &rhs_item)| *item &= rhs_item);
    }
}

impl ops::BitOrAssign for Column {
    /// Bitwise or the rhs into this value. Will panic if different length.
    fn bitor_assign(&mut self, rhs: Self) {
        if self.len != rhs.len {
            panic!("Cannot |= columns of length {} != {}", self.len, rhs.len);
        }
        self.col
            .iter_mut()
            .zip(rhs.col.iter())
            .for_each(|(item, &rhs_item)| *item |= rhs_item);
    }
}

impl ops::Not for Column {
    type Output = Self;

    /// Bitwise negate the value.
    fn not(self) -> Self::Output {
        Self {
            len: self.len,
            col: self.col.iter().map(|item| !item).collect(),
        }
    }
}

/// Generate a u64 with the first n bits set to 1
fn first_n_bits(n: u64) -> u64 {
    if n >= 64 { u64::MAX } else { (1 << n) - 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_bin_str(raw: &str) -> Vec<bool> {
        raw.bytes().map(|byte| byte == b'1').collect()
    }

    #[test]
    fn test_pack_unpack_empty() {
        let bools = vec![];
        let col = Column::from_bools(&bools);
        assert_eq!(col.col.len(), 0);
        assert_eq!(bools, col.to_bools());
    }

    #[test]
    fn test_pack_unpack_single() {
        let bools = vec![true];
        let col = Column::from_bools(&bools);
        assert_eq!(col.col.len(), 1);
        assert_eq!(bools, col.to_bools());
    }

    #[test]
    fn test_pack_unpack_one_full_chunk() {
        let bools: Vec<bool> = (0..64).map(|i| i % 2 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.col.len(), 1);
        assert_eq!(bools, col.to_bools());
    }

    #[test]
    fn test_pack_unpack_many_chunks() {
        let bools: Vec<bool> = (0..192).map(|i| i % 3 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.col.len(), 3);
        assert_eq!(bools, col.to_bools());
    }

    #[test]
    fn test_pack_unpack_many_chunks_and_partial() {
        let bools: Vec<bool> = (0..223).map(|i| i % 5 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.col.len(), 4);
        assert_eq!(bools, col.to_bools());
    }

    #[test]
    fn test_from_true() {
        let col = Column::from_true(223);
        assert_eq!(col.len(), 223);
        assert_eq!(col.to_bools(), vec![true; 223]);
    }

    #[test]
    fn test_from_false() {
        let col = Column::from_false(223);
        assert_eq!(col.len(), 223);
        assert_eq!(col.to_bools(), vec![false; 223]);
    }

    #[test]
    fn test_generate_one_hot() {
        let cols = Column::one_hot_values(&[0, 1, 2, 1, 2, 1], 3);
        assert_eq!(
            cols,
            vec![
                Column::from_bools(&[true, false, false, false, false, false]),
                Column::from_bools(&[false, true, false, true, false, true]),
                Column::from_bools(&[false, false, true, false, true, false]),
            ]
        )
    }

    #[test]
    fn test_count_true_false() {
        let bools: Vec<bool> = (0..223).map(|i| i % 5 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.count_true(), 45);
        assert_eq!(col.count_false(), 223 - 45);
    }

    #[test]
    fn test_count_true_false_when_ones_in_junk() {
        let col = Column::from_true(223);
        assert_eq!(col.count_true(), 223);
        assert_eq!(col.count_false(), 0);
    }

    #[test]
    fn test_get_true_inds() {
        let bools: Vec<bool> = (0..223).map(|i| i % 5 == 0).collect();
        let col = Column::from_bools(&bools);
        let expected = (0..223).filter(|val| val % 5 == 0).collect::<Vec<usize>>();
        assert_eq!(col.true_inds(), expected);
    }

    #[test]
    fn test_set_get_initial_false() {
        let mut col = Column::from_false(223);
        for ind in 0..223 {
            col.set(ind, ind % 5 == 0);
        }
        for ind in 0..223 {
            assert_eq!(col.get(ind), ind % 5 == 0);
        }
    }

    #[test]
    fn test_set_get_initial_true() {
        let mut col = Column::from_true(223);
        for ind in 0..223 {
            col.set(ind, ind % 5 == 0);
        }
        for ind in 0..223 {
            assert_eq!(col.get(ind), ind % 5 == 0);
        }
    }

    #[test]
    fn test_filter() {
        let col = Column::from_bools(&parse_bin_str(
            "01001010001100100101110001010000000000111101101001100011001101110100001011110111100010001011110",
        ));
        assert_eq!(col.len(), 95);
        assert_eq!(col.count_true(), 44);

        let mask = Column::from_bools(&parse_bin_str(
            "10001110101011100110111010000110110000110010111100001011101001001011100111100000001000001001101",
        ));
        assert_eq!(mask.len(), 95);
        assert_eq!(mask.count_true(), 45);

        let mask_inds = mask.true_inds();
        assert_eq!(mask_inds.len(), 45);
        for ind in 0..mask.len() {
            assert_eq!(mask_inds.contains(&ind), mask.get(ind));
        }

        let expected = Column::from_bools(&parse_bin_str(
            "010101001101100000011010100110110000011101110",
        ));
        assert_eq!(expected.len(), 45);
        assert_eq!(expected.count_true(), 21);

        assert_eq!(col.filter(&mask.true_inds()), expected)
    }
}
