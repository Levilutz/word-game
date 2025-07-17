use crate::{
    hint::{Hint, guess_to_hints},
    word::Word,
};

pub fn dumb_search_words<const WORD_SIZE: usize>(
    words: &[Word<WORD_SIZE>],
    guess: Word<WORD_SIZE>,
    hints: [Hint; WORD_SIZE],
) -> Vec<Word<WORD_SIZE>> {
    words
        .iter()
        .filter_map(|word| {
            if guess_to_hints(*word, guess) == hints {
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
    use Hint::{Correct, Elsewhere, Nowhere};

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
                .collect::<Vec<Word<5>>>(),
            Word::from_str("board"),
            [Correct, Nowhere, Elsewhere, Elsewhere, Correct],
        );
        assert_eq!(results, vec![Word::from_str("bread")])
    }
}
