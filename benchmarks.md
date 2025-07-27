# Current benchmarks for best performance on each word list / max depth combo

"word list" & "max depth" are configured, outputs are "est cost" and "time".
For an even playing field, should run with `max_cost` configured to best est cost + .01.
"time" is just decision tree search time, not including precomputation time.
Lowest depth that achieves best est cost is bold for each word list.

| word list                   | max depth | est cost   | time (s)     | commit                                                                                           |
| --------------------------- | --------- | ---------- | ------------ | ------------------------------------------------------------------------------------------------ |
| <u>50-test</u>              |
| 50-test                     | 4         | 3.06       | 0.017s       | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| **50-test**                 | **5**     | **3.04**   | **0.011s**   | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| 50-test                     | 6         | 3.04       | 0.013s       | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| <u>250-some-very-common</u> |
| **250-some-very-common**    | **4**     | **2.6840** | **0.011s**   | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| 250-some-very-common        | 5         | 2.6840     | 0.011s       | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| 250-some-very-common        | 6         | 2.6840     | 0.011s       | [ab607e3](https://github.com/Levilutz/word-game/commit/ab607e343d3684cba787b1e44b1312bae3869a66) |
| <u>483-very-common</u>      |
| **483-very-common**         | **4**     | **2.8944** | **3.028s**   | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| 483-very-common             | 5         | 2.8944     | 3.116s       | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| 483-very-common             | 6         | 2.8944     | 3.085s       | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| <u>695-some-common</u>      |
| **695-some-common**         | **4**     | **3.0302** | **31.234ss** | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| 695-some-common             | 5         | 3.0302     | 25.381s      | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| 695-some-common             | 6         | 3.0302     | 25.559s      | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| <u>1000-some-common</u>     |
| 1000-some-common            | 4         | 3.1490     | 1119.701s    | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| **1000-some-common**        | **5**     | **3.1440** | **183.888s** | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
| 1000-some-common            | 6         | 3.1440     | 183.323s     | [b7ba3d1](https://github.com/Levilutz/word-game/commit/b7ba3d1cafaaa377acfd612a8bb2227ec7956df2) |
