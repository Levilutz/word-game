use std::collections::{HashMap, HashSet};

use crate::{
    hint::{CharHint, WordHint},
    word::Word,
    word_search::Query,
};

/// Check whether a clue is possible for a given word.
///
/// The case this looks for is Elsewhere hints after Nowhere hints for a given char.
/// The default wordle hint generation will always fill the first characters possible
/// with Elsewhere hints.
pub fn clue_possible<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    guess: Word<WORD_SIZE, ALPHABET_SIZE>,
    word_hint: WordHint<WORD_SIZE>,
) -> bool {
    let mut nowhere_chars: HashSet<u8> = HashSet::new();
    for ind in 0..WORD_SIZE {
        let guess_char = guess.0[ind];
        let char_hint = word_hint.0[ind];

        match char_hint {
            CharHint::Nowhere => {
                nowhere_chars.insert(guess_char);
            }
            CharHint::Elsewhere => {
                if nowhere_chars.contains(&guess_char) {
                    return false;
                }
            }
            CharHint::Correct => {}
        }
    }
    true
}

pub fn clue_to_query<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    guess: Word<WORD_SIZE, ALPHABET_SIZE>,
    word_hint: WordHint<WORD_SIZE>,
) -> Query {
    let mut sub_queries = vec![];

    let mut incorrect_chars: HashSet<u8> = HashSet::new();
    let mut num_per_char_by_hint: HashMap<(u8, CharHint), usize> = HashMap::new();

    for ind in 0..WORD_SIZE {
        let guess_char = guess.0[ind];
        let char_hint = word_hint.0[ind];

        *num_per_char_by_hint
            .entry((guess_char, char_hint))
            .or_insert(0) += 1;

        match char_hint {
            CharHint::Correct => sub_queries.push(Query::Match {
                ind,
                chr: guess_char,
            }),
            CharHint::Elsewhere => {
                incorrect_chars.insert(guess_char);
                sub_queries.push(Query::Not(Box::new(Query::Match {
                    ind,
                    chr: guess_char,
                })))
            }
            CharHint::Nowhere => {
                incorrect_chars.insert(guess_char);
                sub_queries.push(Query::Not(Box::new(Query::Match {
                    ind: ind,
                    chr: guess_char,
                })))
            }
        }
    }

    // Add additional facts derivable from elsewhere hints
    for incorrect_char in incorrect_chars {
        // Get how many of each hint affected this char
        let num_correct = num_per_char_by_hint
            .get(&(incorrect_char, CharHint::Correct))
            .cloned()
            .unwrap_or(0);

        let num_elsewhere = num_per_char_by_hint
            .get(&(incorrect_char, CharHint::Elsewhere))
            .cloned()
            .unwrap_or(0);

        let num_nowhere = num_per_char_by_hint
            .get(&(incorrect_char, CharHint::Nowhere))
            .cloned()
            .unwrap_or(0);

        if num_nowhere > 0 {
            // If some showed as Nowhere, we know exactly how many of this char are present
            sub_queries.push(Query::CountExact {
                count: num_correct + num_elsewhere,
                chr: incorrect_char,
            });
        } else if num_elsewhere > 0 {
            // In this case we have a lower bound on the number of this char that are present
            sub_queries.push(Query::CountAtLeast {
                count: num_correct + num_elsewhere,
                chr: incorrect_char,
            });
        }
    }

    Query::And(sub_queries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_has_all_facts() {
        // Guess is board, answer is bread
        let guess: Word<5, 26> = Word::from_str("board");
        let word_hint = WordHint::from("√X~~√");
        let query = clue_to_query(guess, word_hint);
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
