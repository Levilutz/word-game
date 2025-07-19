use crate::{hint::WordHint, word::Word};

pub fn dumb_search_words<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    words: &[Word<WORD_SIZE, ALPHABET_SIZE>],
    guess: Word<WORD_SIZE, ALPHABET_SIZE>,
    word_hint: WordHint<WORD_SIZE>,
) -> Vec<Word<WORD_SIZE, ALPHABET_SIZE>> {
    words
        .iter()
        .filter_map(|word| {
            if WordHint::from_guess_and_answer(&guess, word) == word_hint {
                Some(*word)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_words() {
        let words = vec![
            "badly", "basic", "basis", "beach", "begin", "being", "below", "bench", "bible",
            "birth", "black", "blade", "blame", "blind", "block", "blood", "board", "brain",
            "brand", "bread", "break", "brick", "brief", "bring", "broad", "brown", "brush",
            "build", "bunch", "buyer",
        ];
        let results = dumb_search_words(
            &words
                .iter()
                .map(|word| Word::from_str(word))
                .collect::<Vec<Word<5, 26>>>(),
            Word::from_str("board"),
            WordHint::from("√X~~√"),
        );
        assert_eq!(results, vec![Word::from_str("bread")])
    }
}
