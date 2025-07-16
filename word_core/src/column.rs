/// A simple column of booleans packed into a u64 for performant binary ops.
pub struct Column(Vec<u64>);

impl Column {
    pub fn from_bools(bools: &[bool]) -> Self {
        let num_chunks = (bools.len() + 63) / 64; // Divide & round-up
        let mut out = Vec::with_capacity(num_chunks);

        for chunk in bools.chunks(64) {
            let mut value = 0;
            for (i, bit) in chunk.iter().enumerate() {
                value |= (*bit as u64) << i;
            }
            out.push(value);
        }

        Self(out)
    }

    pub fn to_bools(&self, num_bools: usize) -> Vec<bool> {
        let mut out = Vec::with_capacity(num_bools);

        for (chunk_ind, value) in self.0.iter().enumerate() {
            for bit_ind in 0..64 {
                if chunk_ind * 64 + bit_ind >= num_bools {
                    break;
                }
                out.push(*value & (1 << bit_ind) != 0);
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_empty() {
        let bools = vec![];
        let col = Column::from_bools(&bools);
        assert_eq!(col.0.len(), 0);
        assert_eq!(bools, col.to_bools(bools.len()));
    }

    #[test]
    fn test_pack_unpack_single() {
        let bools = vec![true];
        let col = Column::from_bools(&bools);
        assert_eq!(col.0.len(), 1);
        assert_eq!(bools, col.to_bools(bools.len()));
    }

    #[test]
    fn test_pack_unpack_one_full_chunk() {
        let bools: Vec<bool> = (0..64).map(|i| i % 2 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.0.len(), 1);
        assert_eq!(bools, col.to_bools(bools.len()));
    }

    #[test]
    fn test_pack_unpack_many_chunks() {
        let bools: Vec<bool> = (0..192).map(|i| i % 3 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.0.len(), 3);
        assert_eq!(bools, col.to_bools(bools.len()));
    }

    #[test]
    fn test_pack_unpack_many_chunks_and_partial() {
        let bools: Vec<bool> = (0..223).map(|i| i % 5 == 0).collect();
        let col = Column::from_bools(&bools);
        assert_eq!(col.0.len(), 4);
        assert_eq!(bools, col.to_bools(bools.len()));
    }
}
