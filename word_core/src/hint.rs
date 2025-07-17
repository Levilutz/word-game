use std::{cmp::min, collections::HashMap};

use crate::word::Word;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Hint {
    /// The submitted character is correct
    Correct,

    /// The submitted character is present in the word, but not here
    Elsewhere,

    /// The submitted character is not present in the word
    Nowhere,
}

/// Given a guess and an answer, compute the set of hints.
pub fn guess_to_hints<const WORD_SIZE: usize>(
    answer: Word<WORD_SIZE>,
    guess: Word<WORD_SIZE>,
) -> [Hint; WORD_SIZE] {
    // Initialize with Nowhere hints
    let mut hints = [Hint::Nowhere; WORD_SIZE];

    // For every character in the answer that the guess missed, how many were missed
    let mut missed_answer_char_counts: HashMap<u8, usize> = HashMap::new();

    // For every character in the guess that was missed, which inds contain it
    let mut incorrect_guess_char_inds: HashMap<u8, Vec<usize>> = HashMap::new();

    for ind in 0..WORD_SIZE {
        let answer_char = answer.0[ind];
        let guess_char = guess.0[ind];

        if answer_char == guess_char {
            hints[ind] = Hint::Correct
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
                hints[*guess_ind] = Hint::Elsewhere
            }
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use Hint::{Correct, Elsewhere, Nowhere};

    fn assert_hints<const WORD_SIZE: usize>(answer: &str, guess: &str, hints: [Hint; WORD_SIZE]) {
        assert_eq!(
            guess_to_hints(Word::from_str(answer), Word::from_str(guess)),
            hints
        );
    }

    #[test]
    fn test_no_matches() {
        assert_hints("aaaaa", "bbbbb", [Nowhere; 5]);
    }

    #[test]
    fn test_alternating_correct() {
        assert_hints(
            "ababa",
            "acaca",
            [Correct, Nowhere, Correct, Nowhere, Correct],
        );
    }

    #[test]
    fn test_elsewhere_simple() {
        assert_hints(
            "aabaa",
            "cbccc",
            [Nowhere, Elsewhere, Nowhere, Nowhere, Nowhere],
        );
    }

    #[test]
    fn test_elsewhere_and_correct() {
        assert_hints(
            "ababa",
            "ccbbc",
            [Nowhere, Nowhere, Elsewhere, Correct, Nowhere],
        );
    }

    #[test]
    fn test_multiple_elsewhere_and_correct() {
        assert_hints(
            "aabbb",
            "bbbcc",
            [Elsewhere, Elsewhere, Correct, Nowhere, Nowhere],
        );
    }

    #[test]
    fn test_many_elsewhere_and_correct() {
        assert_hints(
            "aabab",
            "bbbcc",
            [Elsewhere, Nowhere, Correct, Nowhere, Nowhere],
        );
    }
}
