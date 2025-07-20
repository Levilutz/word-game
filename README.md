# word-game

Guess the word!

## Parity testing of word filtering

```sh
cargo run --example test_parity --release word_lists/very-common.txt
```

## Time generation of simplest decision tree

```sh
cargo build --example calc_decision_tree --release && time ./target/release/examples/calc_decision_tree word_lists/test.txt
```
