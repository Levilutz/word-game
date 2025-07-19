use std::fmt::Display;

use serde::{Deserialize, Serialize, Serializer, de::Visitor};

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

impl<const WORD_SIZE: usize> Serialize for Word<WORD_SIZE, 26> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

struct WordVisitor<const WORD_SIZE: usize>;

impl<'de, const WORD_SIZE: usize> Visitor<'de> for WordVisitor<WORD_SIZE> {
    type Value = Word<WORD_SIZE, 26>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a word")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Word::from_str(v))
    }
}

impl<'de, const WORD_SIZE: usize> Deserialize<'de> for Word<WORD_SIZE, 26> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(WordVisitor::<WORD_SIZE>)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&Word::<5, 26>::from_str("abcdz")).unwrap(),
            "\"ABCDZ\""
        );
    }

    #[test]
    fn test_deserialize() {
        let result: Word<5, 26> = serde_json::from_str("\"zdcba\"").unwrap();
        assert_eq!(result, Word::<5, 26>::from_str("zdcba"),);
    }

    #[test]
    fn test_serde() {
        let original = Word::<5, 26>::from_str("azbyc");
        let json = serde_json::to_string(&original).unwrap();
        let reconstructed = serde_json::from_str(&json).unwrap();
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_serde_as_map_key() {
        let original: HashMap<Word<5, 26>, u64> = HashMap::from([
            (Word::from_str("abcde"), 5),
            (Word::from_str("fghij"), 3),
            (Word::from_str("vwxyz"), 1),
        ]);
        let json = serde_json::to_string(&original).unwrap();
        let reconstructed = serde_json::from_str(&json).unwrap();
        assert_eq!(original, reconstructed);
    }
}
