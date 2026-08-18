//! Safe, Rust-idiomatic equivalents of C standard library functions.
//!
//! Each function preserves the corresponding libc function's behavior for inputs
//! whose behavior libc defines, with priority given to preserving its return value.
//! C types map to fixed Rust types: `char` to `i8`, `int` to `i32`, `long` to `i64`,
//! `float` to `f32`, `double` to `f64`, `long double` to [`struct@f128::f128`], and
//! `size_t` to `usize`. Null-terminated strings are represented by `i8` slices
//! containing a null byte where required by the libc interface.
//!
//! Function documentation is intentionally concise because behavior follows libc;
//! it notes Rust-specific return values and error handling where needed.

pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod strings;

pub use stdio::printf;
pub use stdio::{
    fgetc, fgets, fputc, fputs, fread, fseek, ftell, fwrite, getchar, putchar, puts, rewind,
};
#[cfg(target_os = "linux")]
pub use stdio::{remove, rename};
pub use stdlib::{atof, atoi, atol, strtod, strtof, strtol, strtold, strtoul};
pub use string::{
    memchr, memchr_mut, memcmp, strcat, strchr, strchr_mut, strcmp, strcpy, strcspn, strdup,
    strlen, strncat, strncmp, strncpy, strndup, strrchr, strrchr_mut, strspn, strstr, strstr_mut,
};
pub use strings::{strcasecmp, strncasecmp};
