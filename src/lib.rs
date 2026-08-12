pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod strings;

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
