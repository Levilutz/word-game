pub mod column;
pub mod word;
pub mod word_search;

use std::{
    cmp::min,
    collections::{HashMap, HashSet},
};

use crate::{word::Word, word_search::Query};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Hint {
    /// The submitted character is correct
    Correct,

    /// The submitted character is present in the word, but not here
    Elsewhere,

    /// The submitted character is not present in the word
    Nowhere,
}

pub fn clue_to_query<const WORD_SIZE: usize>(
    guess: Word<WORD_SIZE>,
    hints: [Hint; WORD_SIZE],
) -> Query {
    let mut sub_queries = vec![];

    let mut elsewhere_chars: HashSet<u8> = HashSet::new();
    let mut num_per_char_by_hint: HashMap<(u8, Hint), usize> = HashMap::new();

    for ind in 0..WORD_SIZE {
        let guess_chr = guess.0[ind];
        let hint = hints[ind];

        *num_per_char_by_hint.entry((guess_chr, hint)).or_insert(0) += 1;

        match hint {
            Hint::Correct => sub_queries.push(Query::Match {
                ind,
                chr: guess_chr,
            }),
            Hint::Elsewhere => {
                elsewhere_chars.insert(guess_chr);
                sub_queries.push(Query::Not(Box::new(Query::Match {
                    ind,
                    chr: guess_chr,
                })))
            }
            Hint::Nowhere => sub_queries.push(Query::CountExact {
                count: 0,
                chr: guess_chr,
            }),
        }
    }

    // Add additional facts derivable from elsewhere hints
    for elsewhere_char in elsewhere_chars {
        // Get how many of each hint affected this char
        let num_correct = num_per_char_by_hint
            .get(&(elsewhere_char, Hint::Correct))
            .cloned()
            .unwrap_or(0);

        let num_elsewhere = num_per_char_by_hint
            .get(&(elsewhere_char, Hint::Elsewhere))
            .cloned()
            .unwrap_or(0);

        let num_nowhere = num_per_char_by_hint
            .get(&(elsewhere_char, Hint::Nowhere))
            .cloned()
            .unwrap_or(0);

        if num_nowhere > 0 {
            // If some showed as Nowhere, we know exactly how many of this char are present
            sub_queries.push(Query::CountExact {
                count: num_correct + num_elsewhere,
                chr: elsewhere_char,
            });
        } else {
            // In this case we have a lower bound on the number of this char that are present
            sub_queries.push(Query::CountAtLeast {
                count: num_correct + num_elsewhere,
                chr: elsewhere_char,
            });
        }
    }

    Query::And(sub_queries)
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

    #[test]
    fn test_query_has_all_facts() {
        // Guess is board, answer is bread
        let guess: Word<5> = Word::from_str("board");
        let hints = [Correct, Nowhere, Elsewhere, Elsewhere, Correct];
        let query = clue_to_query(guess, hints);
        let Query::And(sub_queries) = query else {
            panic!("non-And returned");
        };

        // Ensure all expected facts exist in the sub-queries
        // println!("{:#?}", sub_queries);
        assert!(sub_queries.contains(&Query::Match { ind: 0, chr: 1 }));
        assert!(sub_queries.contains(&Query::Match { ind: 4, chr: 3 }));
        assert!(sub_queries.contains(&Query::CountExact { count: 0, chr: 14 }));
        assert!(sub_queries.contains(&Query::Not(Box::new(Query::Match { ind: 2, chr: 0 }))));
        assert!(sub_queries.contains(&Query::Not(Box::new(Query::Match { ind: 3, chr: 17 }))));
        assert!(sub_queries.contains(&Query::CountAtLeast { count: 1, chr: 0 }));
        assert!(sub_queries.contains(&Query::CountAtLeast { count: 1, chr: 17 }));
    }
}
