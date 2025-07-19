use crate::column::Column;
use crate::word::Word;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Query {
    /// Filter for words that contain an instance of `chr` at the specified `ind`
    Match { ind: usize, chr: u8 },

    /// Filter for words that contain exactly `count` instances of `chr`
    CountExact { count: usize, chr: u8 },

    /// Filter for words that contain at least `count` instances of `chr`
    CountAtLeast { count: usize, chr: u8 },

    /// Filter for words that do not satisfy the child query
    Not(Box<Query>),

    /// Filter for words that satisfy all of the child queries
    And(Vec<Query>),

    /// Filter for words that satisfy any of the child queries
    Or(Vec<Query>),
}

pub struct SearchableWords<const WORD_SIZE: usize, const ALPHABET_SIZE: u8> {
    words: Vec<Word<WORD_SIZE, ALPHABET_SIZE>>,
    columns: Vec<Column>,
}

impl<const WORD_SIZE: usize, const ALPHABET_SIZE: u8> SearchableWords<WORD_SIZE, ALPHABET_SIZE> {
    /// Given a set of words and an alphabet size, build a search table of word data.
    pub fn build(words: Vec<Word<WORD_SIZE, ALPHABET_SIZE>>) -> Self {
        let num_cols = (ALPHABET_SIZE as usize) * WORD_SIZE * 3;
        let mut columns = Vec::with_capacity(num_cols);

        // Push precomputed columns for each char
        for chr in 0..ALPHABET_SIZE {
            // Push exact match columns
            let match_cols = (0..WORD_SIZE).map(|ind| {
                Column::from_bools(
                    &words
                        .iter()
                        .map(|word| word.0[ind] == chr)
                        .collect::<Vec<bool>>(),
                )
            });
            columns.extend(match_cols);

            // Compute counts for this character once
            let counts: Vec<u64> = words
                .iter()
                .map(|word| word.count_chr(chr) as u64)
                .collect();

            // Push count-exact columns
            let exact_count_cols = Column::one_hot_values(&counts, WORD_SIZE as u64 + 1);
            columns.extend(exact_count_cols.into_iter());

            // Push count-at-least columns
            for threshold_count in 1..WORD_SIZE {
                let word_count_at_least_threshold: Vec<bool> = counts
                    .iter()
                    .map(|word_count| *word_count >= threshold_count as u64)
                    .collect();
                let count_at_least_col = Column::from_bools(&word_count_at_least_threshold);
                columns.push(count_at_least_col);
            }
        }

        Self { words, columns }
    }

    /// Evaluate the query and produce an output column that represents a mask over rows.
    pub fn eval_query(&self, query: Query) -> Column {
        match query {
            Query::Match { ind, chr } => {
                let chr_block_start = (WORD_SIZE * 3) * chr as usize;
                let chr_block_match_cols_start = chr_block_start + 0;
                let target_col = chr_block_match_cols_start + ind;
                self.columns[target_col].clone()
            }
            Query::CountExact { count, chr } => {
                let chr_block_start = (WORD_SIZE * 3) * chr as usize;
                let chr_block_count_exact_cols_start = chr_block_start + WORD_SIZE;
                let target_col = chr_block_count_exact_cols_start + count;
                self.columns[target_col].clone()
            }
            Query::CountAtLeast { count, chr } => {
                if count == 0 {
                    Column::from_true(self.words.len())
                } else if count == WORD_SIZE {
                    self.eval_query(Query::CountExact { count, chr })
                } else {
                    let chr_block_start = (WORD_SIZE * 3) * chr as usize;
                    let chr_block_count_at_least_cols_start = chr_block_start + WORD_SIZE * 2 + 1;
                    let target_col = chr_block_count_at_least_cols_start + count - 1;
                    self.columns[target_col].clone()
                }
            }
            Query::Not(query) => !self.eval_query(*query),
            Query::And(queries) => {
                queries
                    .into_iter()
                    .fold(Column::from_true(self.words.len()), |mut acc, query| {
                        acc &= self.eval_query(query);
                        acc
                    })
            }
            Query::Or(queries) => {
                queries
                    .into_iter()
                    .fold(Column::from_false(self.words.len()), |mut acc, query| {
                        acc |= self.eval_query(query);
                        acc
                    })
            }
        }
    }

    /// Given a mask over rows, extract the words filtered by that mask.
    pub fn filter_words(&self, mask: &Column) -> Vec<Word<WORD_SIZE, ALPHABET_SIZE>> {
        mask.true_inds()
            .into_iter()
            .map(|ind| self.words[ind])
            .collect()
    }

    /// Given a mask over rows, extract a new table filtered by that mask.
    pub fn filter(&self, mask: &Column) -> Self {
        let inds = mask.true_inds();
        Self {
            words: inds.iter().map(|ind| self.words[*ind]).collect(),
            columns: self.columns.iter().map(|col| col.filter(&inds)).collect(),
        }
    }

    /// Get a reference to the words contained in this data structure.
    pub fn words(&self) -> &[Word<WORD_SIZE, ALPHABET_SIZE>] {
        &self.words
    }

    /// Get the number of possible answers in this set.
    pub fn len(&self) -> usize {
        self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_subtract(a: &[&'static str], b: &[&'static str]) -> Vec<&'static str> {
        a.iter().copied().filter(|item| !b.contains(item)).collect()
    }

    fn words_from_strs<const WORD_SIZE: usize>(words: &[&str]) -> Vec<Word<WORD_SIZE, 26>> {
        words.iter().map(|word| Word::from_str(word)).collect()
    }

    fn assert_query_result<const WORD_SIZE: usize>(
        words: &[&str],
        query: Query,
        expected: &[&str],
    ) {
        let words: SearchableWords<WORD_SIZE, 26> = SearchableWords::build(words_from_strs(words));
        // println!("{:#?}", words.columns.iter().map(|col| col.to_bools()).collect::<Vec<Vec<bool>>>());
        let mask = words.eval_query(query);
        let result = words.filter_words(&mask);
        assert_eq!(
            result,
            words_from_strs(expected),
            "{:?} != {:?}",
            result
                .iter()
                .map(|word| format!("{}", word))
                .collect::<Vec<String>>(),
            expected
                .iter()
                .map(|word| word.to_ascii_uppercase())
                .collect::<Vec<String>>(),
        );
    }

    fn assert_query_result_and_inverse<const WORD_SIZE: usize>(
        words: &[&'static str],
        query: Query,
        expected: &[&'static str],
    ) {
        assert_query_result::<WORD_SIZE>(words, query.clone(), expected);
        assert_query_result::<WORD_SIZE>(
            words,
            Query::Not(Box::new(query)),
            &set_subtract(words, expected),
        );
    }

    #[test]
    fn test_query_match() {
        assert_query_result_and_inverse::<3>(
            &["foo", "bar", "baz"],
            Query::Match { ind: 1, chr: 0 },
            &["bar", "baz"],
        );
    }

    #[test]
    fn test_query_count_exact() {
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountExact { count: 0, chr: 0 },
            &["bbc", "cbc"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountExact { count: 1, chr: 0 },
            &["abc", "bca"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountExact { count: 2, chr: 0 },
            &["baa", "aac"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountExact { count: 3, chr: 0 },
            &["aaa"],
        );
    }

    #[test]
    fn test_query_count_at_least() {
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountAtLeast { count: 0, chr: 0 },
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountAtLeast { count: 1, chr: 0 },
            &["abc", "bca", "baa", "aac", "aaa"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountAtLeast { count: 2, chr: 0 },
            &["baa", "aac", "aaa"],
        );
        assert_query_result_and_inverse::<3>(
            &["bbc", "cbc", "abc", "bca", "baa", "aac", "aaa"],
            Query::CountAtLeast { count: 3, chr: 0 },
            &["aaa"],
        );
    }

    #[test]
    fn test_query_and_group() {
        assert_query_result_and_inverse::<3>(
            &["foo", "bar", "baz", "biz", "buz"],
            Query::And(vec![
                Query::Match { ind: 1, chr: 0 },
                Query::CountAtLeast { count: 1, chr: 25 },
            ]),
            &["baz"],
        );
    }

    #[test]
    fn test_query_or_group() {
        assert_query_result_and_inverse::<3>(
            &["foo", "bar", "baz", "biz", "buz"],
            Query::Or(vec![
                Query::Match { ind: 1, chr: 0 },
                Query::CountAtLeast { count: 1, chr: 25 },
            ]),
            &["bar", "baz", "biz", "buz"],
        );
    }

    #[test]
    fn test_query_realistic() {
        // Realistic query for when the answer is 'bread' and the guess was 'board'
        assert_query_result_and_inverse::<5>(
            &[
                "badly", "basic", "basis", "beach", "begin", "being", "below", "bench", "bible",
                "birth", "black", "blade", "blame", "blind", "block", "blood", "board", "brain",
                "brand", "bread", "break", "brick", "brief", "bring", "broad", "brown", "brush",
                "build", "bunch", "buyer",
            ],
            Query::And(vec![
                // The B was correct
                Query::Match { ind: 0, chr: 1 },
                // The D was correct
                Query::Match { ind: 4, chr: 3 },
                // There is no O
                Query::Not(Box::new(Query::CountAtLeast { count: 1, chr: 14 })),
                // The A was elsewhere
                Query::And(vec![
                    Query::CountAtLeast { count: 1, chr: 0 },
                    Query::Not(Box::new(Query::Match { ind: 2, chr: 0 })),
                ]),
                // The R was elsewhere (alternate representation)
                Query::Or(vec![
                    Query::Match { ind: 1, chr: 17 },
                    Query::Match { ind: 2, chr: 17 },
                ]),
            ]),
            &["bread"],
        );
    }
}
