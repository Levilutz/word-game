#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Hint {
    /// The submitted character is correct
    Correct,

    /// The submitted character is present in the word, but not here
    Elsewhere,

    /// The submitted character is not present in the word
    Nowhere,
}

/// Given a guess and an answer represented as u8 vecs, compute the set of hints.
///
/// This fn will panic if the answer is shorter than the guess.
pub fn guess_to_hints_unchecked(answer: &[u8], guess: &[u8]) -> Vec<Hint> {
    // Start by marking all the green tiles
    let hints: Vec<Hint> = guess
        .iter()
        .enumerate()
        .map(|(i, guess_char)| {
            if answer[i] == *guess_char {
                Hint::Correct
            } else {
                Hint::Nowhere
            }
        })
        .collect();
    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use Hint::{Correct, Nowhere};

    #[test]
    fn test_no_matches() {
        let result = guess_to_hints_unchecked(&[0; 5], &[1; 5]);
        assert_eq!(result, vec![Nowhere; 5]);
    }

    #[test]
    fn test_alternating_correct() {
        let result = guess_to_hints_unchecked(&[0, 1, 0, 1, 0], &[0, 2, 0, 2, 0]);
        assert_eq!(result, vec![Correct, Nowhere, Correct, Nowhere, Correct]);
    }
}
