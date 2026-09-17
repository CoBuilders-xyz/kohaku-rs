# kohaku-tornadocash-circuit

Tornadocash zk circuits for kohaku.

Includes tornadocash's circuit artifacts instead of downloading them seperately and wraps [`websnark-rs`](https://crates.io/crates/websnark-rs) with a simple API for proof generation. Tornadocash's artifacts are ~10mb compressed, so this crate will substantially increase the size of your binary.
