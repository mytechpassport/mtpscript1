# BUGFIX LOG

- [ ] `mtpscript-core/src/ir/lower.rs:194` – `params` is never read in the lambda lowering match arm, which triggered a compiler warning and may hide unfinished logic for parameter handling (seen during `cargo test -p mtpscript-core`).
- [ ] `mtpscript-core/src/parser/mod.rs:630` – `check_next` is defined but unused, so either it should be integrated into the parser (to support lookahead scenarios) or removed to avoid dead code warnings from the compiler.
- [ ] `mtpscript-core/src/types/builtins.rs:128` – The `ctx` binding inside `test_option_result_acceptance_criteria` is never used, weakening that test and emitting an unused-variable warning; either add assertions or drop the local to clear the warning.
- [ ] `mtpscript-core/src/parser/mod.rs:58` – Unary `!` is parsed into `BinOp::Or` instead of a dedicated not operation, so `!expr` doesn’t behave correctly and violates the spec’s boolean semantics.
- [ ] `mtpscript-core/src/effects/async_effect.rs:101` – The fallback `format!("{:?}", expr)` when encoding complex expressions for `promiseHash` produces non-deterministic bytes, breaking the Async await hashing guarantees required by §7-a.
