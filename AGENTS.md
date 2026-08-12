* You are an expert in C and Rust.
* The goal of this project is to implement a Rust library that provides safe
  functions equivalent to those in libc.
* Each function must preserve the exact non-UB behavior of the intended libc
  function with a Rust-idiomatic signature.
* Prioritize preserving the C return when choosing a signature.
* Each function should have good performance. For multi-byte operations, process
  as much data as possible at once instead of repeatedly processing one byte.
* After editing the code, run `cargo fmt` and `cargo clippy --workspace
  --all-targets` and resolve clippy warnings.
* Place functions corresponding to \<stdio.h\> functions under
  `src/stdio/mod.rs`, and place relevant test cases under `src/stdio/tests.rs`.
* Each test case should not exhibit any actual file-system update.

# Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

# Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

# Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.
