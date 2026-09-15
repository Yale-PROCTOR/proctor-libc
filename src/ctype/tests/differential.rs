//! Differential tests against libc in its startup C locale.

use super::{
    isalnum, isalpha, isblank, iscntrl, isdigit, isgraph, islower, isprint, ispunct, isspace,
    isupper, isxdigit, tolower, toupper,
};

type ClassificationFunction = fn(i32) -> i32;
type LibcFunction = unsafe extern "C" fn(libc::c_int) -> libc::c_int;

fn defined_inputs() -> impl Iterator<Item = i32> {
    std::iter::once(-1).chain(0..=i32::from(u8::MAX))
}

fn check_classification(name: &str, function: ClassificationFunction, libc_function: LibcFunction) {
    for c in defined_inputs() {
        // SAFETY: `c` is either EOF or representable as `unsigned char`.
        let libc_result = unsafe { libc_function(c) };
        let result = function(c);
        assert!(
            matches!(result, 0 | 1),
            "{name} returned {result} for {c:#04x}"
        );
        assert_eq!(
            result != 0,
            libc_result != 0,
            "{name} differs for {c:#04x}: libc returned {libc_result}"
        );
    }
}

#[test]
fn classification_functions_match_libc() {
    for (name, function, libc_function) in [
        (
            "isalnum",
            isalnum as ClassificationFunction,
            libc::isalnum as LibcFunction,
        ),
        ("isalpha", isalpha, libc::isalpha),
        ("isblank", isblank, libc::isblank),
        ("iscntrl", iscntrl, libc::iscntrl),
        ("isdigit", isdigit, libc::isdigit),
        ("isgraph", isgraph, libc::isgraph),
        ("islower", islower, libc::islower),
        ("isprint", isprint, libc::isprint),
        ("ispunct", ispunct, libc::ispunct),
        ("isspace", isspace, libc::isspace),
        ("isupper", isupper, libc::isupper),
        ("isxdigit", isxdigit, libc::isxdigit),
    ] {
        check_classification(name, function, libc_function);
    }
}

#[test]
fn case_conversion_functions_match_libc() {
    for c in defined_inputs() {
        // SAFETY: `c` is either EOF or representable as `unsigned char`.
        let libc_lower = unsafe { libc::tolower(c) };
        // SAFETY: `c` is either EOF or representable as `unsigned char`.
        let libc_upper = unsafe { libc::toupper(c) };

        assert_eq!(tolower(c), libc_lower, "tolower differs for {c:#04x}");
        assert_eq!(toupper(c), libc_upper, "toupper differs for {c:#04x}");
    }
}
