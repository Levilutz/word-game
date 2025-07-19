use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Word<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(pub [u8; WORD_SIZE]);

impl<const WORD_SIZE: usize, const ALPHABET_SIZE: u8> Word<WORD_SIZE, ALPHABET_SIZE> {
    /// Convert from the given raw string. Panics if invalid.
    pub fn from_str(raw: &str) -> Self {
        assert_eq!(raw.bytes().len(), WORD_SIZE);
        let mut out = [0; WORD_SIZE];
        for (ind, byte) in raw.bytes().enumerate() {
            let value = byte.to_ascii_uppercase() - 65;
            assert!(value < 26);
            out[ind] = value;
        }
        Self(out)
    }

    /// Count how many of the given char are in the word.
    pub fn count_chr(&self, chr: u8) -> usize {
        self.0.iter().filter(|self_chr| **self_chr == chr).count()
    }
}

impl<const WORD_SIZE: usize> Display for Word<WORD_SIZE, 26> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chr in self.0 {
            write!(f, "{}", (b'A' + chr) as char)?;
        }
        Ok(())
    }
}
