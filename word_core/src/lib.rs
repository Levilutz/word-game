use std::{cmp::min, collections::HashMap};

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
    let mut hints: Vec<Hint> = guess
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

    // Count how many of each char are present in the incorrect tiles of the answer
    let missed_counts: HashMap<u8, u8> = answer
        .iter()
        .zip(hints.iter())
        .filter_map(|(answer_char, hint)| match hint {
            Hint::Correct => None,
            _ => Some(*answer_char),
        })
        .fold(HashMap::new(), |mut map, char| {
            *map.entry(char).or_insert(0) += 1;
            map
        });

    // Precompute indicies of each incorrect char in guess
    let guess_char_inds: HashMap<u8, Vec<usize>> = guess
        .iter()
        .zip(hints.iter())
        .enumerate()
        .filter_map(|(i, (answer_char, hint))| match hint {
            Hint::Correct => None,
            _ => Some((i, *answer_char)),
        })
        .fold(HashMap::new(), |mut map, (ind, char)| {
            map.entry(char).or_default().push(ind);
            map
        });

    // For every missed answer char that was in the guess, set the first N to Elsewhere
    for (answer_char, num_missed) in missed_counts.into_iter() {
        if let Some(inds) = guess_char_inds.get(&answer_char) {
            let num_elsewhere = min(num_missed as usize, inds.len());
            for ind in &inds[0..num_elsewhere] {
                hints[*ind] = Hint::Elsewhere
            }
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use Hint::{Correct, Elsewhere, Nowhere};

    #[test]
    fn test_no_matches() {
        assert_eq!(guess_to_hints_unchecked(&[0; 5], &[1; 5]), vec![Nowhere; 5]);
    }

    #[test]
    fn test_alternating_correct() {
        assert_eq!(
            guess_to_hints_unchecked(&[0, 1, 0, 1, 0], &[0, 2, 0, 2, 0]),
            vec![Correct, Nowhere, Correct, Nowhere, Correct]
        );
    }

    #[test]
    fn test_elsewhere_simple() {
        assert_eq!(
            guess_to_hints_unchecked(&[0, 0, 1, 0, 0], &[2, 1, 2, 2, 2]),
            vec![Nowhere, Elsewhere, Nowhere, Nowhere, Nowhere]
        );
    }

    #[test]
    fn test_elsewhere_and_correct() {
        assert_eq!(
            guess_to_hints_unchecked(&[0, 1, 0, 1, 0], &[2, 2, 1, 1, 2]),
            vec![Nowhere, Nowhere, Elsewhere, Correct, Nowhere]
        );
    }

    #[test]
    fn test_multiple_elsewhere_and_correct() {
        assert_eq!(
            guess_to_hints_unchecked(&[0, 0, 1, 1, 1], &[1, 1, 1, 2, 2]),
            vec![Elsewhere, Elsewhere, Correct, Nowhere, Nowhere]
        );
    }

    #[test]
    fn test_many_elsewhere_and_correct() {
        assert_eq!(
            guess_to_hints_unchecked(&[0, 0, 1, 0, 1], &[1, 1, 1, 2, 2]),
            vec![Elsewhere, Nowhere, Correct, Nowhere, Nowhere]
        );
    }
}
