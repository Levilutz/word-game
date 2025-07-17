use std::collections::{HashMap, HashSet};

use crate::{hint::Hint, word::Word, word_search::Query};

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

#[cfg(test)]
mod tests {
    use super::*;
    use Hint::{Correct, Elsewhere, Nowhere};

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
