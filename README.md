# word-game

Guess the word!

This is currently an attempted implementation of a perfect ["word game"](https://news.ycombinator.com/item?id=39618193) solver. The current iteration of the solver appears capable of generating a perfect decision tree, but is far too imperformant to do so within a reasonable amount of time.

Once we can generate decision trees quickly, this repo might be extended to include utilities / a published web interface.

[PRs](https://github.com/Levilutz/word-game/pulls?q=is%3Apr) contain details on experimentation towards improving search performance.

## Parity testing of word filtering via optimized search

```sh
cargo run --example test_parity --release word_lists/very-common.txt
```

## Test time it takes to generate a decision tree for the simplest scenario

```sh
cargo build --example calc_decision_tree --release && time ./target/release/examples/calc_decision_tree word_lists/test.txt
```
