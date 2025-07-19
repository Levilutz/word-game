use std::{cmp::min, collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize, Serializer, de::Visitor};

use crate::word::Word;

/// A hint for a single character.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum CharHint {
    /// The submitted character is correct
    Correct,

    /// The submitted character is present in the word, but not here
    Elsewhere,

    /// The submitted character is not present in the word
    Nowhere,
}

impl Display for CharHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharHint::Correct => write!(f, "√"),
            CharHint::Elsewhere => write!(f, "~"),
            CharHint::Nowhere => write!(f, "X"),
        }
    }
}

impl From<char> for CharHint {
    fn from(value: char) -> Self {
        match value {
            '√' => Self::Correct,
            '~' => Self::Elsewhere,
            'X' | 'x' => Self::Nowhere,
            _ => panic!("Invalid char for CharHint: {}", value),
        }
    }
}

/// A hint for a whole word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WordHint<const WORD_SIZE: usize>(pub [CharHint; WORD_SIZE]);

impl<const WORD_SIZE: usize> WordHint<WORD_SIZE> {
    /// Determine what hints should be shown for a given guess and a given answer
    pub fn from_guess_and_answer(guess: &Word<WORD_SIZE>, answer: &Word<WORD_SIZE>) -> Self {
        // Initialize with Nowhere hints
        let mut char_hints = [CharHint::Nowhere; WORD_SIZE];

        // For every character in the answer that the guess missed, how many were missed
        let mut missed_answer_char_counts: HashMap<u8, usize> = HashMap::new();

        // For every character in the guess that was missed, which inds contain it
        let mut incorrect_guess_char_inds: HashMap<u8, Vec<usize>> = HashMap::new();

        for ind in 0..WORD_SIZE {
            let answer_char = answer.0[ind];
            let guess_char = guess.0[ind];

            if answer_char == guess_char {
                char_hints[ind] = CharHint::Correct
            } else {
                *missed_answer_char_counts.entry(answer_char).or_insert(0) += 1;
                incorrect_guess_char_inds
                    .entry(guess_char)
                    .or_default()
                    .push(ind);
            }
        }

        // For every missed answer character that was in the guess, set the first N to Elsewhere
        for (answer_char, num_missed) in missed_answer_char_counts.into_iter() {
            if let Some(guess_inds) = incorrect_guess_char_inds.get(&answer_char) {
                let num_missed_to_show = min(num_missed, guess_inds.len());
                for guess_ind in &guess_inds[0..num_missed_to_show] {
                    char_hints[*guess_ind] = CharHint::Elsewhere
                }
            }
        }

        Self(char_hints)
    }

    /// Get all possible hints for this word size
    pub fn all_possible() -> Vec<Self> {
        (0..3usize.pow(WORD_SIZE as u32))
            .map(|ind| {
                let mut ind = ind;
                let mut char_hints = [CharHint::Correct; WORD_SIZE];
                for digit in 0..WORD_SIZE {
                    char_hints[digit] = match ind % 3 {
                        0 => CharHint::Correct,
                        1 => CharHint::Elsewhere,
                        _ => CharHint::Nowhere,
                    };
                    ind /= 3;
                }
                Self(char_hints)
            })
            .collect()
    }

    /// Color a guess word based on this hint
    pub fn color_guess(&self, guess: &Word<WORD_SIZE>) -> String {
        let mut out: Vec<String> = vec![];
        for ind in 0..WORD_SIZE {
            match self.0[ind] {
                CharHint::Correct => out.push("\x1b[42m".to_string()),
                CharHint::Elsewhere => out.push("\x1b[43m".to_string()),
                CharHint::Nowhere => out.push("\x1b[41m".to_string()),
            }
            out.push(format!("{}", (b'A' + guess.0[ind]) as char));
            out.push("\x1b[0m".to_string())
        }
        out.join("")
    }

    /// Is this hint all correct
    pub fn all_correct(&self) -> bool {
        return self.0 == [CharHint::Correct; WORD_SIZE];
    }
}

impl<const WORD_SIZE: usize> Display for WordHint<WORD_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for hint in self.0 {
            hint.fmt(f)?
        }
        Ok(())
    }
}

impl<const WORD_SIZE: usize> From<&str> for WordHint<WORD_SIZE> {
    fn from(value: &str) -> Self {
        let mut char_hints = [CharHint::Correct; WORD_SIZE];
        for (ind, char_value) in value.chars().enumerate() {
            char_hints[ind] = CharHint::from(char_value)
        }
        Self(char_hints)
    }
}

impl<const WORD_SIZE: usize> Serialize for WordHint<WORD_SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

struct WordHintVisitor<const WORD_SIZE: usize>;

impl<'de, const WORD_SIZE: usize> Visitor<'de> for WordHintVisitor<WORD_SIZE> {
    type Value = WordHint<WORD_SIZE>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a set of character hints")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(WordHint::from(v))
    }
}

impl<'de, const WORD_SIZE: usize> Deserialize<'de> for WordHint<WORD_SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(WordHintVisitor::<WORD_SIZE>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_word_hint<const WORD_SIZE: usize>(answer: &str, guess: &str, word_hint: &str) {
        let word_hint: WordHint<WORD_SIZE> = WordHint::from(word_hint);
        assert_eq!(
            WordHint::from_guess_and_answer(&Word::from_str(guess), &Word::from_str(answer)),
            word_hint,
        )
    }

    #[test]
    fn test_no_matches() {
        assert_word_hint::<5>("aaaaa", "bbbbb", "xxxxx");
    }

    #[test]
    fn test_alternating_correct() {
        assert_word_hint::<5>("ababa", "acaca", "√X√X√");
    }

    #[test]
    fn test_elsewhere_simple() {
        assert_word_hint::<5>("aabaa", "cbccc", "X~XXX");
    }

    #[test]
    fn test_elsewhere_and_correct() {
        assert_word_hint::<5>("ababa", "ccbbc", "XX~√X");
    }

    #[test]
    fn test_multiple_elsewhere_and_correct() {
        assert_word_hint::<5>("aabbb", "bbbcc", "~~√XX");
    }

    #[test]
    fn test_many_elsewhere_and_correct() {
        assert_word_hint::<5>("aabab", "bbbcc", "~X√XX");
    }

    #[test]
    fn test_all_hints_1() {
        assert_eq!(
            WordHint::<1>::all_possible(),
            vec![
                WordHint::from("√"),
                WordHint::from("~"),
                WordHint::from("X")
            ]
        );
    }

    #[test]
    fn test_all_hints_2() {
        assert_eq!(
            WordHint::<2>::all_possible(),
            vec![
                WordHint::from("√√"),
                WordHint::from("~√"),
                WordHint::from("X√"),
                WordHint::from("√~"),
                WordHint::from("~~"),
                WordHint::from("X~"),
                WordHint::from("√X"),
                WordHint::from("~X"),
                WordHint::from("XX"),
            ]
        )
    }

    #[test]
    fn test_all_hints_3() {
        assert_eq!(
            WordHint::<3>::all_possible(),
            vec![
                WordHint::from("√√√"),
                WordHint::from("~√√"),
                WordHint::from("X√√"),
                WordHint::from("√~√"),
                WordHint::from("~~√"),
                WordHint::from("X~√"),
                WordHint::from("√X√"),
                WordHint::from("~X√"),
                WordHint::from("XX√"),
                WordHint::from("√√~"),
                WordHint::from("~√~"),
                WordHint::from("X√~"),
                WordHint::from("√~~"),
                WordHint::from("~~~"),
                WordHint::from("X~~"),
                WordHint::from("√X~"),
                WordHint::from("~X~"),
                WordHint::from("XX~"),
                WordHint::from("√√X"),
                WordHint::from("~√X"),
                WordHint::from("X√X"),
                WordHint::from("√~X"),
                WordHint::from("~~X"),
                WordHint::from("X~X"),
                WordHint::from("√XX"),
                WordHint::from("~XX"),
                WordHint::from("XXX"),
            ]
        )
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&WordHint::<5>::from("√~X√~")).unwrap(),
            "\"√~X√~\""
        );
    }

    #[test]
    fn test_deserialize() {
        let result: WordHint<5> = serde_json::from_str("\"√~X√~\"").unwrap();
        assert_eq!(result, WordHint::<5>::from("√~X√~"),);
    }

    #[test]
    fn test_serde() {
        let original = WordHint::<5>::from("X~X√~");
        let json = serde_json::to_string(&original).unwrap();
        let reconstructed = serde_json::from_str(&json).unwrap();
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_serde_as_map_key() {
        let original: HashMap<WordHint<5>, u64> = HashMap::from([
            (WordHint::from("X~X√~"), 5),
            (WordHint::from("~√~√X"), 3),
            (WordHint::from("√√√√√"), 1),
        ]);
        let json = serde_json::to_string(&original).unwrap();
        let reconstructed = serde_json::from_str(&json).unwrap();
        assert_eq!(original, reconstructed);
    }
}
