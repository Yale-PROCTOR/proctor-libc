# proctor-libc

`proctor-libc` provides safe, Rust-idiomatic equivalents of functions from the C
standard library. Each port aims to preserve the corresponding libc function's
behavior for every input whose behavior libc defines, including its return value.

C types are represented consistently: `char` as `i8`, `int` as `i32`, `long` as
`i64`, `float` as `f32`, `double` as `f64`, `long double` as `f128::f128`, and
`size_t` as `usize`. Null-terminated strings are represented by `i8` slices
containing a null byte where the libc interface requires one.

Function documentation is intentionally concise because behavior follows the
corresponding libc function; it focuses on Rust-specific return values and error
handling where needed.
