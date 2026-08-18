//! Integer formatting adapters for C `printf` semantics that Rust formatting
//! does not express directly.
//!
//! Use [`signed`] for the `d` and `i` conversions and [`unsigned`] for the
//! `u`, `o`, `x`, and `X` conversions. The wrappers preserve the input's Rust
//! primitive type; the value passed to a wrapper should therefore be the value
//! after the C length conversion. For example, an `i8` represents the value
//! formatted by `%hhd`, while an `i32` represents the value formatted by `%d`.
//!
//! The wrappers are unnecessary when native Rust formatting already has the
//! same result. They are intended for C integer precision, the space-sign flag,
//! and C alternate octal or hexadecimal form. Using a wrapper in an ordinary
//! case is nevertheless equivalent:
//!
//! ```
//! use proctor_libc::printf::{signed, unsigned};
//!
//! // printf("%d %i", 42, -7);
//! assert_eq!(format!("{} {}", signed(42_i32), signed(-7_i32)), "42 -7");
//! // Native `format!("{} {}", 42_i32, -7_i32)` is already exact.
//!
//! // printf("%u %o %x %X", 42u, 42u, 42u, 42u);
//! assert_eq!(
//!     format!(
//!         "{} {:o} {:x} {:X}",
//!         unsigned(42_u32),
//!         unsigned(42_u32),
//!         unsigned(42_u32),
//!         unsigned(42_u32),
//!     ),
//!     "42 52 2a 2A",
//! );
//! // The corresponding native formatting is already exact too.
//! ```
//!
//! Rust width, left alignment, sign, and zero padding map to the C `printf`
//! flags `width`, `-`, `+`, and `0`. These cases also work natively when no
//! wrapper-only behavior is combined with them:
//!
//! ```
//! use proctor_libc::printf::{signed, unsigned};
//!
//! // printf("%8d", 42);     printf("%-8d", 42);
//! assert_eq!(format!("{:8}", signed(42_i32)), "      42");
//! assert_eq!(format!("{:<8}", signed(42_i32)), "42      ");
//! // printf("%+d", 42);     printf("%08d", -42);
//! assert_eq!(format!("{:+}", signed(42_i32)), "+42");
//! assert_eq!(format!("{:08}", signed(-42_i32)), "-0000042");
//! // Native i32 formatting produces the same four results.
//!
//! // printf("%-8u", 42u);   printf("%08X", 42u);
//! assert_eq!(format!("{:<8}", unsigned(42_u32)), "42      ");
//! assert_eq!(format!("{:08X}", unsigned(42_u32)), "0000002A");
//! // Native u32 formatting is sufficient for these cases as well.
//!
//! // The C `-` flag overrides `0`: printf("%-08d", 42);
//! assert_eq!(format!("{:<08}", signed(42_i32)), "42      ");
//! ```
//!
//! A Rust integer precision on these wrappers has C's minimum-digit meaning.
//! It also disables `0` padding, and zero formatted with precision zero emits
//! no digits:
//!
//! ```
//! use proctor_libc::printf::{signed, unsigned};
//!
//! // printf("%.5d", 42);    printf("%08.5d", 42);
//! assert_eq!(format!("{:.5}", signed(42_i32)), "00042");
//! assert_eq!(format!("{:08.5}", signed(42_i32)), "   00042");
//! // Both C `-` and precision override `0`: printf("%-08.5u", 42u);
//! assert_eq!(format!("{:<08.5}", unsigned(42_u32)), "00042   ");
//! // printf("%.0u", 0u);
//! assert_eq!(format!("{:.0}", unsigned(0_u32)), "");
//! // Native Rust integer formatting does not provide C integer precision, so
//! // the wrappers are required for these cases.
//! ```
//!
//! Call [`Signed::space_sign`] for C's space flag. Rust's `+` flag takes
//! precedence, as it does for `printf`:
//!
//! ```
//! use proctor_libc::printf::signed;
//!
//! // printf("% d", 42);     printf("%+ d", 42);
//! assert_eq!(format!("{}", signed(42_i32).space_sign()), " 42");
//! assert_eq!(format!("{:+}", signed(42_i32).space_sign()), "+42");
//! // Native Rust has no space-sign flag, so this wrapper is required.
//! ```
//!
//! The `#` flag uses C's octal and hexadecimal forms, including their special
//! handling of zero:
//!
//! ```
//! use proctor_libc::printf::unsigned;
//!
//! // printf("%#o %#x %#X", 42u, 42u, 42u);
//! assert_eq!(
//!     format!(
//!         "{:#o} {:#x} {:#X}",
//!         unsigned(42_u32),
//!         unsigned(42_u32),
//!         unsigned(42_u32),
//!     ),
//!     "052 0x2a 0X2A",
//! );
//! // printf("%#.0o %#x", 0u, 0u);
//! assert_eq!(
//!     format!("{:#.0o} {:#x}", unsigned(0_u32), unsigned(0_u32)),
//!     "0 0",
//! );
//! // `0` pads after the prefix, but precision disables `0`:
//! // printf("%#08x %#08.4x", 42u, 42u);
//! assert_eq!(
//!     format!("{:#08x} {:#08.4x}", unsigned(42_u32), unsigned(42_u32)),
//!     "0x00002a   0x002a",
//! );
//! // Rust's native alternate octal prefix and zero-valued hexadecimal form
//! // differ from C, so the wrapper is required whenever `#` is used.
//! ```

use std::fmt::{self, Alignment, Formatter};

const SPACE_PADDING: &str = concat!(
    "                                ",
    "                                ",
);
const ZERO_PADDING: &str = concat!(
    "00000000000000000000000000000000",
    "00000000000000000000000000000000",
);

mod private {
    pub trait SealedSigned: Copy {
        fn to_i128(self) -> i128;
    }

    pub trait SealedUnsigned: Copy {
        fn to_u128(self) -> u128;
    }
}

/// A primitive signed integer accepted by [`signed`].
///
/// This sealed trait is implemented for `i8`, `i16`, `i32`, `i64`, and
/// `isize`.
pub trait SignedValue: private::SealedSigned {}

/// A primitive unsigned integer accepted by [`unsigned`].
///
/// This sealed trait is implemented for `u8`, `u16`, `u32`, `u64`, and
/// `usize`.
pub trait UnsignedValue: private::SealedUnsigned {}

macro_rules! impl_signed_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::SealedSigned for $ty {
                fn to_i128(self) -> i128 {
                    self as i128
                }
            }

            impl SignedValue for $ty {}
        )+
    };
}

macro_rules! impl_unsigned_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::SealedUnsigned for $ty {
                fn to_u128(self) -> u128 {
                    self as u128
                }
            }

            impl UnsignedValue for $ty {}
        )+
    };
}

impl_signed_value!(i8, i16, i32, i64, isize);
impl_unsigned_value!(u8, u16, u32, u64, usize);

/// Wraps a signed primitive for C `printf`-compatible `d` or `i` formatting.
///
/// The wrapper is required for integer precision or [`Signed::space_sign`].
/// Native Rust formatting is already exact for ordinary `d`/`i`, width, `-`,
/// `+`, and `0` cases. See the [module documentation](self) for paired Rust and
/// C examples of every supported conversion and flag.
pub fn signed<T: SignedValue>(value: T) -> Signed<T> {
    Signed {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` signed-integer formatting.
#[derive(Clone, Copy)]
pub struct Signed<T: SignedValue> {
    value: T,
    space_sign: bool,
}

impl<T: SignedValue> Signed<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{}", signed(7_i8).space_sign())` is equivalent to
    /// `printf("% hhd", 7)`, while formatting the same wrapper with `{:+}` is
    /// equivalent to `printf("%+ hhd", 7)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: SignedValue> fmt::Display for Signed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = self.value.to_i128();
        write_integer(
            f,
            value.unsigned_abs(),
            value.is_negative(),
            self.space_sign,
            Base::Decimal,
        )
    }
}

/// Wraps an unsigned primitive for C `printf`-compatible `u`, `o`, `x`, or `X`
/// formatting.
///
/// The wrapper is required for integer precision or the `#` flag. Native Rust
/// formatting is already exact for ordinary conversions, width, `-`, and `0`
/// cases. See the [module documentation](self) for paired Rust and C examples
/// of every supported conversion and flag.
pub fn unsigned<T: UnsignedValue>(value: T) -> Unsigned<T> {
    Unsigned { value }
}

/// A type-preserving adapter for C `printf` unsigned-integer formatting.
#[derive(Clone, Copy)]
pub struct Unsigned<T: UnsignedValue> {
    value: T,
}

impl<T: UnsignedValue> fmt::Display for Unsigned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_integer(f, self.value.to_u128(), false, false, Base::Decimal)
    }
}

impl<T: UnsignedValue> fmt::Octal for Unsigned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_integer(f, self.value.to_u128(), false, false, Base::Octal)
    }
}

impl<T: UnsignedValue> fmt::LowerHex for Unsigned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_integer(f, self.value.to_u128(), false, false, Base::LowerHex)
    }
}

impl<T: UnsignedValue> fmt::UpperHex for Unsigned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_integer(f, self.value.to_u128(), false, false, Base::UpperHex)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Base {
    Decimal,
    Octal,
    LowerHex,
    UpperHex,
}

impl Base {
    fn radix(self) -> u128 {
        match self {
            Self::Decimal => 10,
            Self::Octal => 8,
            Self::LowerHex | Self::UpperHex => 16,
        }
    }

    fn digit(self, value: u128) -> u8 {
        match value {
            0..=9 => b'0' + value as u8,
            10..=15 if self == Self::UpperHex => b'A' + (value - 10) as u8,
            10..=15 => b'a' + (value - 10) as u8,
            _ => unreachable!(),
        }
    }
}

fn write_integer(
    f: &mut Formatter<'_>,
    magnitude: u128,
    negative: bool,
    space_sign: bool,
    base: Base,
) -> fmt::Result {
    let mut digit_buffer = [0_u8; 128];
    let mut digit_start = digit_buffer.len();
    let precision = f.precision();

    if magnitude != 0 || precision != Some(0) {
        let radix = base.radix();
        let mut remaining = magnitude;
        loop {
            digit_start -= 1;
            digit_buffer[digit_start] = base.digit(remaining % radix);
            remaining /= radix;
            if remaining == 0 {
                break;
            }
        }
    }

    let digits = &digit_buffer[digit_start..];
    let mut precision_zeros = precision.unwrap_or_default().saturating_sub(digits.len());
    let alternate = f.alternate();

    if alternate && base == Base::Octal && precision_zeros == 0 && digits.first() != Some(&b'0') {
        precision_zeros = 1;
    }

    let prefix = if alternate && magnitude != 0 {
        match base {
            Base::LowerHex => "0x",
            Base::UpperHex => "0X",
            Base::Decimal | Base::Octal => "",
        }
    } else {
        ""
    };
    let sign = if negative {
        "-"
    } else if f.sign_plus() {
        "+"
    } else if space_sign {
        " "
    } else {
        ""
    };

    let content_width = sign.len() + prefix.len() + precision_zeros + digits.len();
    let padding = f.width().unwrap_or_default().saturating_sub(content_width);
    let alignment = f.align().unwrap_or(Alignment::Right);
    let zero_padding =
        f.sign_aware_zero_pad() && precision.is_none() && alignment == Alignment::Right;

    let (left_spaces, right_spaces) = if zero_padding {
        (0, 0)
    } else {
        match alignment {
            Alignment::Left => (0, padding),
            Alignment::Right => (padding, 0),
            Alignment::Center => (padding / 2, padding - padding / 2),
        }
    };

    write_repeated(f, SPACE_PADDING, left_spaces)?;
    f.write_str(sign)?;
    f.write_str(prefix)?;
    if zero_padding {
        write_repeated(f, ZERO_PADDING, padding)?;
    }
    write_repeated(f, ZERO_PADDING, precision_zeros)?;
    // SAFETY: `digits` contains only ASCII bytes produced by `Base::digit`.
    f.write_str(unsafe { std::str::from_utf8_unchecked(digits) })?;
    write_repeated(f, SPACE_PADDING, right_spaces)
}

fn write_repeated(f: &mut Formatter<'_>, chunk: &str, mut count: usize) -> fmt::Result {
    while count >= chunk.len() {
        f.write_str(chunk)?;
        count -= chunk.len();
    }
    if count != 0 {
        f.write_str(&chunk[..count])?;
    }
    Ok(())
}
