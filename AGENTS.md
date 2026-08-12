* You are an expert in C and Rust.
* The goal of this project is to implement a Rust library that provides safe
  functions equivalent to those in libc.
* Each function must preserve the exact non-UB behavior of the intended libc
  function with a Rust-idiomatic signature.
* For input whose behavior is undefined by the libc specification, the Rust
  function may behave arbitrarily. Docstrings must not describe the UB condition
  or behavior, and tests must not exercise such input.
* When a libc function reads a null-terminated C string and assumes a null byte
  within its bound, a Rust port taking an `i8` slice must assume that the slice
  contains `0`. A slice without `0` is UB and is subject to the preceding rule.
* Prioritize preserving the C return when choosing a signature.
* Map C types to Rust as follows: `char` to `i8`, `int` to `i32`, `long` to
  `i64`, `float` to `f32`, `double` to `f64`, `long double` to `f128::f128`,
  and `size_t` to `usize`.
* Each function should have good performance. For multi-byte operations, process
  as much data as possible at once instead of repeatedly processing one byte.
* After editing the code, run `cargo fmt` and `cargo clippy --workspace
  --all-targets` and resolve clippy warnings.
* Place functions corresponding to \<stdio.h\> functions under
  `src/stdio/mod.rs`, and place relevant test cases under `src/stdio/tests.rs`.
* Place functions corresponding to \<stdlib.h\> functions under
  `src/stdlib/mod.rs`, and place relevant test cases under `src/stdlib/tests.rs`.
* Place functions corresponding to \<string.h\> functions under
  `src/string/mod.rs`, and place relevant test cases under `src/string/tests.rs`.
* Place functions corresponding to \<strings.h\> functions under
  `src/strings/mod.rs`, and place relevant test cases under `src/strings/tests.rs`.
* Each test case should not exhibit any actual file-system update.
* Keep function docstrings self-contained and concise: use a one-line summary
  plus Rust-specific return and error details, without restating libc behavior
  or referring to another function's docstring. If the return value is trivial
  (the same as libc, with no error handling), the one-line summary is enough.

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
