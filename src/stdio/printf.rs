//! Formatting adapters for C `printf` semantics that Rust formatting does not
//! express directly.
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
//!
//! Use [`fixed`] for the `f` conversion and [`fixed_upper`] for the `F`
//! conversion. A missing Rust precision has C's default of six digits after
//! the radix point:
//!
//! ```
//! use proctor_libc::printf::{fixed, fixed_upper};
//!
//! // printf("%f %F", 1.25, 1.25);
//! assert_eq!(
//!     format!("{} {}", fixed(1.25_f64), fixed_upper(1.25_f64)),
//!     "1.250000 1.250000",
//! );
//! // printf("%lf %lF", 1.25, 1.25);
//! assert_eq!(
//!     format!("{} {}", fixed(1.25_f64), fixed_upper(1.25_f64)),
//!     "1.250000 1.250000",
//! );
//! ```
//!
//! C promotes a `float` argument to `double`, so an `f32` passed to [`fixed`]
//! or [`fixed_upper`] has the semantics of `%f` or `%F` after that promotion.
//! The `f128::f128` implementation is the project's IEEE binary128 mapping for
//! C `long double` and the `%Lf` and `%LF` spellings:
//!
//! ```
//! use proctor_libc::printf::{fixed, fixed_upper};
//!
//! let float_value = 1.25_f32;
//! // printf("%f %F", (double)float_value, (double)float_value);
//! assert_eq!(
//!     format!("{} {}", fixed(float_value), fixed_upper(float_value)),
//!     "1.250000 1.250000",
//! );
//!
//! let long_double_value = f128::f128::new(1.25_f64);
//! // Intended equivalents: printf("%Lf %LF", long_double_value, long_double_value);
//! assert_eq!(
//!     format!(
//!         "{} {}",
//!         fixed(long_double_value),
//!         fixed_upper(long_double_value),
//!     ),
//!     "1.250000 1.250000",
//! );
//! ```
//!
//! Rust precision, width, left alignment, sign, alternate form, and zero
//! padding map to C precision, `width`, `-`, `+`, `#`, and `0`. [`fixed`] is
//! needed even in simple cases because native Rust omits C's default six
//! fractional digits. For an explicit nonzero precision, however, native Rust
//! formatting is already exact for finite `f32` and `f64` values when no
//! wrapper-only behavior is used:
//!
//! ```
//! use proctor_libc::printf::fixed;
//!
//! // printf("%.2f %.0f", 1.25, 1.5);
//! assert_eq!(format!("{:.2} {:.0}", fixed(1.25_f64), fixed(1.5_f64)), "1.25 2");
//! // Native format!("{:.2} {:.0}", 1.25_f64, 1.5_f64) is exact here.
//!
//! // printf("%8.2f %-8.2f", 1.25, 1.25);
//! assert_eq!(format!("{:8.2} {:<8.2}", fixed(1.25_f64), fixed(1.25_f64)),
//!            "    1.25 1.25    ");
//! // printf("%+08.2f %-08.2f", 1.25, 1.25);
//! assert_eq!(format!("{:+08.2} {:<08.2}", fixed(1.25_f64), fixed(1.25_f64)),
//!            "+0001.25 1.25    ");
//! // These explicit-precision cases also work with native f32/f64 formatting.
//! ```
//!
//! Call [`Fixed::space_sign`] or [`FixedUpper::space_sign`] for C's space flag.
//! Rust's `+` flag takes precedence:
//!
//! ```
//! use proctor_libc::printf::fixed;
//!
//! // printf("% f %+ f", 1.25, 1.25);
//! assert_eq!(
//!     format!("{} {:+}", fixed(1.25_f64).space_sign(), fixed(1.25_f64).space_sign()),
//!     " 1.250000 +1.250000",
//! );
//! // Native Rust has no space-sign flag, so the wrapper is required.
//! ```
//!
//! C's `#` flag retains the radix point at precision zero. This differs from
//! native Rust, and zero padding remains sign-aware:
//!
//! ```
//! use proctor_libc::printf::fixed;
//!
//! // printf("%#.0f %08.2f", 2.0, -1.25);
//! assert_eq!(format!("{:#.0} {:08.2}", fixed(2.0_f64), fixed(-1.25_f64)),
//!            "2. -0001.25");
//! // printf("%#08.0f %-#08.0f", 2.0, 2.0);
//! assert_eq!(format!("{:#08.0} {:<#08.0}", fixed(2.0_f64), fixed(2.0_f64)),
//!            "0000002. 2.      ");
//! ```
//!
//! [`fixed_upper`] is observably different for nonfinite values:
//!
//! ```
//! use proctor_libc::printf::{fixed, fixed_upper};
//!
//! // printf("%f %F", INFINITY, INFINITY);
//! assert_eq!(format!("{} {}", fixed(f64::INFINITY), fixed_upper(f64::INFINITY)),
//!            "inf INF");
//! // Native Rust uses `inf`/`NaN`, which does not cover both C spellings.
//! ```
//!
//! Use [`scientific`] with Rust's `e` or `E` formatting trait for C's `e` or
//! `E` conversion. The wrapper is always required: Rust's native exponent
//! formatting does not guarantee C's `+` on a nonnegative exponent or its
//! minimum of two exponent digits. A missing precision still means six digits
//! after the radix point:
//!
//! ```
//! use proctor_libc::printf::scientific;
//!
//! // printf("%e %E", 1.25, 1.25);
//! assert_eq!(
//!     format!("{:e} {:E}", scientific(1.25_f64), scientific(1.25_f64)),
//!     "1.250000e+00 1.250000E+00",
//! );
//! // printf("%le %lE", 1.25, 1.25);
//! assert_eq!(
//!     format!("{:e} {:E}", scientific(1.25_f64), scientific(1.25_f64)),
//!     "1.250000e+00 1.250000E+00",
//! );
//! ```
//!
//! C promotes `float` to `double`. Binary128 supplies the project's intended
//! `%Le` and `%LE` behavior without assuming that the host `long double` has a
//! compatible ABI:
//!
//! ```
//! use proctor_libc::printf::scientific;
//!
//! let float_value = 1.25_f32;
//! // printf("%e %E", (double)float_value, (double)float_value);
//! assert_eq!(
//!     format!("{:e} {:E}", scientific(float_value), scientific(float_value)),
//!     "1.250000e+00 1.250000E+00",
//! );
//!
//! let long_double_value = f128::f128::new(1.25_f64);
//! // Intended equivalents: printf("%Le %LE", long_double_value, long_double_value);
//! assert_eq!(
//!     format!(
//!         "{:e} {:E}",
//!         scientific(long_double_value),
//!         scientific(long_double_value),
//!     ),
//!     "1.250000e+00 1.250000E+00",
//! );
//! ```
//!
//! Precision, width, left alignment, sign, alternate form, and zero padding
//! map to the corresponding C precision, `width`, `-`, `+`, `#`, and `0`:
//!
//! ```
//! use proctor_libc::printf::scientific;
//!
//! // printf("%.2e %.0E %#.0e", 1.25, 1.5, 2.0);
//! assert_eq!(
//!     format!(
//!         "{:.2e} {:.0E} {:#.0e}",
//!         scientific(1.25_f64),
//!         scientific(1.5_f64),
//!         scientific(2.0_f64),
//!     ),
//!     "1.25e+00 2E+00 2.e+00",
//! );
//! // printf("%12.2e %-12.2e", 1.25, 1.25);
//! assert_eq!(
//!     format!("{:12.2e} {:<12.2e}", scientific(1.25_f64), scientific(1.25_f64)),
//!     "    1.25e+00 1.25e+00    ",
//! );
//! // printf("%+012.2e %-012.2e", 1.25, 1.25);
//! assert_eq!(
//!     format!("{:+012.2e} {:<012.2e}", scientific(1.25_f64), scientific(1.25_f64)),
//!     "+0001.25e+00 1.25e+00    ",
//! );
//! ```
//!
//! Call [`Scientific::space_sign`] for C's space flag. Rust's `+` flag takes
//! precedence:
//!
//! ```
//! use proctor_libc::printf::scientific;
//!
//! // printf("% e %+ e", 1.25, 1.25);
//! assert_eq!(
//!     format!(
//!         "{:e} {:+e}",
//!         scientific(1.25_f64).space_sign(),
//!         scientific(1.25_f64).space_sign(),
//!     ),
//!     " 1.250000e+00 +1.250000e+00",
//! );
//! ```
//!
//! [`Scientific`] intentionally does not implement [`std::fmt::Display`]; the
//! source conversion must select lowercase or uppercase exponent formatting:
//!
//! ```compile_fail
//! use proctor_libc::printf::scientific;
//!
//! let _ = format!("{}", scientific(1.25_f64));
//! ```
//!
//! Use [`general`] for the `g` conversion and [`general_upper`] for `G`.
//! Rust has no corresponding general-format specifier, so the wrapper is
//! always required at the format-specification level. A missing precision
//! means six significant digits, while an explicit precision zero means one:
//!
//! ```
//! use proctor_libc::printf::{general, general_upper};
//!
//! // printf("%g %G", 123.45, 1.0e6);
//! assert_eq!(
//!     format!("{} {}", general(123.45_f64), general_upper(1.0e6_f64)),
//!     "123.45 1E+06",
//! );
//! // printf("%lg %lG", 123.45, 1.0e6);
//! assert_eq!(
//!     format!("{} {}", general(123.45_f64), general_upper(1.0e6_f64)),
//!     "123.45 1E+06",
//! );
//! // printf("%.0g %.3g", 12.5, 12.5);
//! assert_eq!(format!("{:.0} {:.3}", general(12.5_f64), general(12.5_f64)),
//!            "1e+01 12.5");
//! ```
//!
//! The conversion first rounds to the requested number of significant digits.
//! It uses scientific notation when the resulting decimal exponent is less
//! than `-4` or at least the precision, and fixed notation otherwise:
//!
//! ```
//! use proctor_libc::printf::general;
//!
//! // printf("%.4g %.4g", 0.0001, 0.00001);
//! assert_eq!(
//!     format!("{:.4} {:.4}", general(0.0001_f64), general(0.00001_f64)),
//!     "0.0001 1e-05",
//! );
//! // Rounding can change the exponent and therefore the selected style:
//! // printf("%.4g", 9999.6);
//! assert_eq!(format!("{:.4}", general(9999.6_f64)), "1e+04");
//! ```
//!
//! Trailing fractional zeros and an unnecessary radix point are removed by
//! default. C's `#` flag retains them:
//!
//! ```
//! use proctor_libc::printf::{general, general_upper};
//!
//! // printf("%.6g %#.6g", 123.0, 123.0);
//! assert_eq!(format!("{:.6} {:#.6}", general(123.0_f64), general(123.0_f64)),
//!            "123 123.000");
//! // printf("%.6G %#.6G", 1.23e10, 1.23e10);
//! assert_eq!(
//!     format!("{:.6} {:#.6}", general_upper(1.23e10_f64), general_upper(1.23e10_f64)),
//!     "1.23E+10 1.23000E+10",
//! );
//! ```
//!
//! Rust width, left alignment, sign, alternate form, and zero padding map to
//! C `width`, `-`, `+`, `#`, and `0`. Call [`General::space_sign`] or
//! [`GeneralUpper::space_sign`] for C's space flag; `+` takes precedence:
//!
//! ```
//! use proctor_libc::printf::general;
//!
//! // printf("%10.4g %-10.4g", 12.5, 12.5);
//! assert_eq!(format!("{:10.4} {:<10.4}", general(12.5_f64), general(12.5_f64)),
//!            "      12.5 12.5      ");
//! // printf("%+010.4g % 010.4g", 12.5, 12.5);
//! assert_eq!(
//!     format!("{:+010.4} {:010.4}", general(12.5_f64), general(12.5_f64).space_sign()),
//!     "+0000012.5  0000012.5",
//! );
//! // `-` overrides `0`: printf("%-010.4g", 12.5);
//! assert_eq!(format!("{:<010.4}", general(12.5_f64)), "12.5      ");
//! ```
//!
//! C promotes `float` to `double`. Binary128 supplies the project's intended
//! `%Lg` and `%LG` behavior without assuming that the host `long double` has a
//! compatible ABI:
//!
//! ```
//! use proctor_libc::printf::{general, general_upper};
//!
//! let float_value = 1.25_f32;
//! // printf("%g %G", (double)float_value, (double)float_value);
//! assert_eq!(
//!     format!("{} {}", general(float_value), general_upper(float_value)),
//!     "1.25 1.25",
//! );
//!
//! let long_double_value = f128::f128::new(1.25_f64);
//! // Intended equivalents: printf("%Lg %LG", long_double_value, long_double_value);
//! assert_eq!(
//!     format!("{} {}", general(long_double_value), general_upper(long_double_value)),
//!     "1.25 1.25",
//! );
//! ```
//!
//! Use [`hex_float`] with Rust's `x` or `X` formatting trait for C's `a` or
//! `A` conversion. Primitive Rust floats do not implement hexadecimal
//! formatting, so the wrapper is always required. A missing precision emits
//! an exact representation and removes trailing zero hexadecimal digits:
//!
//! ```
//! use proctor_libc::printf::hex_float;
//!
//! // printf("%a %A", 1.5, 1.5);
//! assert_eq!(
//!     format!("{:x} {:X}", hex_float(1.5_f64), hex_float(1.5_f64)),
//!     "0x1.8p+0 0X1.8P+0",
//! );
//! // `%la` and `%lA` have the same `double` argument and result.
//! assert_eq!(
//!     format!("{:x} {:X}", hex_float(1.5_f64), hex_float(1.5_f64)),
//!     "0x1.8p+0 0X1.8P+0",
//! );
//! ```
//!
//! Precision counts hexadecimal digits after the radix point. Precision zero
//! omits the radix unless `#` requests it. Width, left alignment, sign, and
//! zero padding map to C's `width`, `-`, `+`, and `0`; zero padding follows
//! the sign and `0x` or `0X` prefix:
//!
//! ```
//! use proctor_libc::printf::hex_float;
//!
//! // printf("%.3a %.0a %#.0a", 1.5, 1.5, 1.5);
//! assert_eq!(
//!     format!("{:.3x} {:.0x} {:#.0x}",
//!             hex_float(1.5_f64), hex_float(1.5_f64), hex_float(1.5_f64)),
//!     "0x1.800p+0 0x2p+0 0x2.p+0",
//! );
//! // printf("%12.2a %-12.2a %+012.2a", 1.5, 1.5, 1.5);
//! assert_eq!(
//!     format!("{:12.2x} {:<12.2x} {:+012.2x}",
//!             hex_float(1.5_f64), hex_float(1.5_f64), hex_float(1.5_f64)),
//!     "   0x1.80p+0 0x1.80p+0    +0x001.80p+0",
//! );
//! ```
//!
//! Call [`HexFloat::space_sign`] for C's space flag. Rust's `+` flag takes
//! precedence:
//!
//! ```
//! use proctor_libc::printf::hex_float;
//!
//! // printf("% a %+ a", 1.5, 1.5);
//! assert_eq!(
//!     format!("{:x} {:+x}",
//!             hex_float(1.5_f64).space_sign(),
//!             hex_float(1.5_f64).space_sign()),
//!     " 0x1.8p+0 +0x1.8p+0",
//! );
//! ```
//!
//! C promotes `float` to `double` before `%a` formatting. Consequently an
//! `f32` subnormal is formatted as its exact, usually normal, binary64 value:
//!
//! ```
//! use proctor_libc::printf::hex_float;
//!
//! let float_subnormal = f32::from_bits(1);
//! // printf("%a", (double)float_subnormal);
//! assert_eq!(format!("{:x}", hex_float(float_subnormal)), "0x1p-149");
//!
//! let long_double_value = f128::f128::new(1.5_f64);
//! // Intended equivalents: printf("%La %LA", long_double_value, long_double_value);
//! assert_eq!(
//!     format!("{:x} {:X}", hex_float(long_double_value), hex_float(long_double_value)),
//!     "0x1.8p+0 0X1.8P+0",
//! );
//! ```
//!
//! Effective binary64 and binary128 subnormals use the deterministic GNU and
//! libquadmath convention: a leading zero and the type's minimum-normal
//! exponent. Binary128 is the project's intended `%La`/`%LA` mapping and is
//! not passed to a host `long double` conversion when that ABI differs.
//! Floating formatting uses the C locale and round-to-nearest, ties-to-even;
//! it does not track an active process locale or floating-point rounding mode.
//!
//! [`HexFloat`] intentionally does not implement [`std::fmt::Display`]; the
//! source conversion must select lowercase or uppercase hexadecimal formatting:
//!
//! ```compile_fail
//! use proctor_libc::printf::hex_float;
//!
//! let _ = format!("{}", hex_float(1.5_f64));
//! ```
//!
//! Use [`byte_string`] for the `s` conversion when PROCTOR represents the C
//! `char *` as an `i8` slice. It reinterprets each `i8` as the corresponding
//! byte, stops at the first NUL, and uses C's byte-counted width and precision:
//!
//! ```
//! use proctor_libc::printf::byte_string;
//!
//! let text = [b'h' as i8, b'e' as i8, b'l' as i8, b'l' as i8, b'o' as i8, 0];
//! // printf("%s", text);
//! assert_eq!(format!("{}", byte_string(&text)), "hello");
//! // printf("%10s", text);    printf("%-10s", text);
//! assert_eq!(format!("{:10}", byte_string(&text)), "     hello");
//! assert_eq!(format!("{:<10}", byte_string(&text)), "hello     ");
//! // printf("%.3s", text);    printf("%10.3s", text);
//! assert_eq!(format!("{:.3}", byte_string(&text)), "hel");
//! assert_eq!(format!("{:10.3}", byte_string(&text)), "       hel");
//! ```
//!
//! Precision is a maximum byte count and can make a NUL unnecessary when the
//! slice contains at least that many bytes. Bytes following the first NUL or
//! the precision boundary are not inspected:
//!
//! ```
//! use proctor_libc::printf::byte_string;
//!
//! let terminated = [b'o' as i8, b'k' as i8, 0, b'x' as i8];
//! // printf("%s", terminated);    printf("%.0s", terminated);
//! assert_eq!(format!("{}", byte_string(&terminated)), "ok");
//! assert_eq!(format!("{:.0}", byte_string(&terminated)), "");
//!
//! let bounded = [b'a' as i8, b'b' as i8, b'c' as i8];
//! // printf("%.2s", bounded);
//! assert_eq!(format!("{:.2}", byte_string(&bounded)), "ab");
//! ```
//!
//! The selected output bytes must be valid UTF-8. Within that supported
//! PROCTOR domain they are preserved exactly, including multibyte encodings,
//! while width still counts bytes rather than Unicode scalar values:
//!
//! ```
//! use proctor_libc::printf::byte_string;
//!
//! let text = [0xc3_u8 as i8, 0xa9_u8 as i8, b'!' as i8, 0]; // "é!"
//! // printf("%5s", text);     printf("%.2s", text);
//! assert_eq!(format!("{:5}", byte_string(&text)).as_bytes(), b"  \xc3\xa9!");
//! assert_eq!(format!("{:.2}", byte_string(&text)).as_bytes(), b"\xc3\xa9");
//! ```
//!
//! A precision can split a UTF-8 encoding; such selected output, like any
//! other non-UTF-8 byte sequence, is outside this wrapper's supported domain.
//! The implementation remains memory-safe and reports a formatting error.
//! Native `&str` formatting is sufficient for simple ASCII strings when C's
//! byte counts and NUL representation are irrelevant. This wrapper is needed
//! for PROCTOR's `&[i8]` mapping, first-NUL selection, and byte-counted fields.
//!
//! Floating formatting uses the C locale's `.` radix character and
//! round-to-nearest, ties-to-even. It does not track an active process locale
//! or floating-point rounding mode. Binary128 formatting is tested directly
//! and by invariants, but is not differentially checked against host
//! long-double conversions on systems where C `long double` has a different
//! representation.
//!
//! Only `f32`, `f64`, and `f128::f128` are accepted:
//!
//! ```compile_fail
//! let _ = proctor_libc::printf::fixed(1_i32);
//! ```

use num_bigint::BigUint;
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

    pub trait SealedFixed: Copy {
        const FRACTION_BITS: u32;
        const EXPONENT_BITS: u32;
        const EXPONENT_BIAS: i32;
        const HEX_FRACTION_BITS: u32;
        const HEX_MIN_NORMAL_EXPONENT: i32;

        fn bits(self) -> u128;
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

/// A primitive floating-point value accepted by [`fixed`], [`fixed_upper`],
/// [`scientific`], [`general`], [`general_upper`], and [`hex_float`].
///
/// This sealed trait is implemented for `f32`, `f64`, and
/// [`struct@f128::f128`].
pub trait FixedValue: private::SealedFixed {}

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

impl private::SealedFixed for f32 {
    const FRACTION_BITS: u32 = 23;
    const EXPONENT_BITS: u32 = 8;
    const EXPONENT_BIAS: i32 = 127;
    const HEX_FRACTION_BITS: u32 = 52;
    const HEX_MIN_NORMAL_EXPONENT: i32 = -1022;

    fn bits(self) -> u128 {
        self.to_bits() as u128
    }
}

impl FixedValue for f32 {}

impl private::SealedFixed for f64 {
    const FRACTION_BITS: u32 = 52;
    const EXPONENT_BITS: u32 = 11;
    const EXPONENT_BIAS: i32 = 1023;
    const HEX_FRACTION_BITS: u32 = 52;
    const HEX_MIN_NORMAL_EXPONENT: i32 = -1022;

    fn bits(self) -> u128 {
        self.to_bits() as u128
    }
}

impl FixedValue for f64 {}

impl private::SealedFixed for f128::f128 {
    const FRACTION_BITS: u32 = 112;
    const EXPONENT_BITS: u32 = 15;
    const EXPONENT_BIAS: i32 = 16383;
    const HEX_FRACTION_BITS: u32 = 112;
    const HEX_MIN_NORMAL_EXPONENT: i32 = -16382;

    fn bits(self) -> u128 {
        u128::from_ne_bytes(self.into_inner())
    }
}

impl FixedValue for f128::f128 {}

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

/// Wraps a floating-point primitive for C `printf`-compatible `f` formatting.
///
/// The wrapper supplies C's default precision of six, exact binary128
/// formatting, lowercase nonfinite spelling, alternate form at precision zero,
/// and [`Fixed::space_sign`]. Native Rust formatting is already exact for
/// finite `f32` and `f64` values when an explicit nonzero precision is present
/// and none of those wrapper-only behaviors is needed. See the [module
/// documentation](self) for paired Rust and C examples of every supported type,
/// spelling, and flag.
pub fn fixed<T: FixedValue>(value: T) -> Fixed<T> {
    Fixed {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` lowercase fixed-point formatting.
#[derive(Clone, Copy)]
pub struct Fixed<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> Fixed<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{}", fixed(1.25_f32).space_sign())` is equivalent to
    /// `printf("% f", (double)1.25f)`, while formatting the same wrapper with
    /// `{:+}` is equivalent to `printf("%+ f", (double)1.25f)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::Display for Fixed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_fixed(
            f,
            FloatParts::from_value(self.value),
            self.space_sign,
            false,
        )
    }
}

/// Wraps a floating-point primitive for C `printf`-compatible `F` formatting.
///
/// This has the same fixed-point semantics as [`fixed`] and uses uppercase
/// `INF` and `NAN`. See the [module documentation](self) for paired Rust and C
/// examples of `%F`, `%lF`, `%LF`, and every supported flag.
pub fn fixed_upper<T: FixedValue>(value: T) -> FixedUpper<T> {
    FixedUpper {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` uppercase fixed-point formatting.
#[derive(Clone, Copy)]
pub struct FixedUpper<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> FixedUpper<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{}", fixed_upper(1.25_f64).space_sign())` is equivalent to
    /// `printf("% F", 1.25)`, while formatting the same wrapper with `{:+}` is
    /// equivalent to `printf("%+ F", 1.25)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::Display for FixedUpper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_fixed(f, FloatParts::from_value(self.value), self.space_sign, true)
    }
}

/// Wraps a floating-point primitive for C `printf`-compatible `e` or `E`
/// formatting.
///
/// Format the returned wrapper with Rust's `e` or `E` trait to select the
/// conversion case. It intentionally does not implement [`fmt::Display`]. See
/// the [module documentation](self) for paired Rust and C examples of every
/// supported type, spelling, and flag.
pub fn scientific<T: FixedValue>(value: T) -> Scientific<T> {
    Scientific {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` scientific formatting.
#[derive(Clone, Copy)]
pub struct Scientific<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> Scientific<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{:e}", scientific(1.25_f32).space_sign())` is equivalent to
    /// `printf("% e", (double)1.25f)`, while formatting the same wrapper with
    /// `{:+e}` is equivalent to `printf("%+ e", (double)1.25f)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::LowerExp for Scientific<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_scientific(
            f,
            FloatParts::from_value(self.value),
            self.space_sign,
            false,
        )
    }
}

impl<T: FixedValue> fmt::UpperExp for Scientific<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_scientific(f, FloatParts::from_value(self.value), self.space_sign, true)
    }
}

/// Wraps a floating-point primitive for C `printf`-compatible `g` formatting.
///
/// Rust has no native general floating-point conversion, so this wrapper is
/// required for every `%g`, `%lg`, or intended binary128 `%Lg` conversion. See
/// the [module documentation](self) for paired Rust and C examples of every
/// supported type, precision, and flag.
pub fn general<T: FixedValue>(value: T) -> General<T> {
    General {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` lowercase general formatting.
#[derive(Clone, Copy)]
pub struct General<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> General<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{}", general(1.25_f32).space_sign())` is equivalent to
    /// `printf("% g", (double)1.25f)`, while formatting the same wrapper with
    /// `{:+}` is equivalent to `printf("%+ g", (double)1.25f)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::Display for General<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_general(
            f,
            FloatParts::from_value(self.value),
            self.space_sign,
            false,
        )
    }
}

/// Wraps a floating-point primitive for C `printf`-compatible `G` formatting.
///
/// This has the same general-format semantics as [`general`] and uses `E`,
/// `INF`, and `NAN`. See the [module documentation](self) for paired Rust and C
/// examples of `%G`, `%lG`, `%LG`, and every supported flag.
pub fn general_upper<T: FixedValue>(value: T) -> GeneralUpper<T> {
    GeneralUpper {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` uppercase general formatting.
#[derive(Clone, Copy)]
pub struct GeneralUpper<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> GeneralUpper<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{}", general_upper(1.25_f64).space_sign())` is equivalent to
    /// `printf("% G", 1.25)`, while formatting the same wrapper with `{:+}` is
    /// equivalent to `printf("%+ G", 1.25)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::Display for GeneralUpper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_general(f, FloatParts::from_value(self.value), self.space_sign, true)
    }
}

/// Wraps a floating-point primitive for C `printf`-compatible `a` or `A`
/// formatting.
///
/// Format the returned wrapper with Rust's `x` or `X` trait to select the
/// conversion case. Primitive Rust floats have no native hexadecimal
/// formatter, and this adapter intentionally does not implement
/// [`fmt::Display`]. See the [module documentation](self) for paired Rust and C
/// examples of every supported type, precision, and flag.
pub fn hex_float<T: FixedValue>(value: T) -> HexFloat<T> {
    HexFloat {
        value,
        space_sign: false,
    }
}

/// A type-preserving adapter for C `printf` hexadecimal floating formatting.
#[derive(Clone, Copy)]
pub struct HexFloat<T: FixedValue> {
    value: T,
    space_sign: bool,
}

impl<T: FixedValue> HexFloat<T> {
    /// Requests C's space-sign flag for a nonnegative value.
    ///
    /// A Rust `+` formatting flag overrides the space. For example,
    /// `format!("{:x}", hex_float(1.5_f32).space_sign())` is equivalent to
    /// `printf("% a", (double)1.5f)`, while formatting the same wrapper with
    /// `{:+x}` is equivalent to `printf("%+ a", (double)1.5f)`.
    pub fn space_sign(mut self) -> Self {
        self.space_sign = true;
        self
    }
}

impl<T: FixedValue> fmt::LowerHex for HexFloat<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_hex_float::<T>(
            f,
            FloatParts::from_value(self.value),
            self.space_sign,
            false,
        )
    }
}

impl<T: FixedValue> fmt::UpperHex for HexFloat<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_hex_float::<T>(f, FloatParts::from_value(self.value), self.space_sign, true)
    }
}

/// Wraps a PROCTOR `char *` mapping for C `printf`-compatible `s` formatting.
///
/// The selected bytes must be valid UTF-8. Precision and width count bytes,
/// and missing precision selects bytes preceding the first NUL. See the
/// [module documentation](self) for paired Rust and C examples of `%s`, width,
/// left alignment, precision, NUL termination, and multibyte UTF-8.
pub fn byte_string(value: &[i8]) -> ByteString<'_> {
    ByteString { value }
}

/// A borrowed adapter for C `printf` string formatting in the valid-UTF-8
/// output domain.
#[derive(Clone, Copy)]
pub struct ByteString<'a> {
    value: &'a [i8],
}

impl fmt::Display for ByteString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes: &[u8] = bytemuck::cast_slice(self.value);
        let selected = match f.precision() {
            Some(0) => &bytes[..0],
            Some(precision) => {
                let within_precision = &bytes[..bytes.len().min(precision)];
                if let Some(nul) = within_precision.iter().position(|&byte| byte == 0) {
                    &within_precision[..nul]
                } else if bytes.len() >= precision {
                    &bytes[..precision]
                } else {
                    return Err(fmt::Error);
                }
            }
            None => {
                let nul = bytes.iter().position(|&byte| byte == 0).ok_or(fmt::Error)?;
                &bytes[..nul]
            }
        };
        let text = std::str::from_utf8(selected).map_err(|_| fmt::Error)?;

        let padding = f.width().unwrap_or_default().saturating_sub(selected.len());
        let (left_spaces, right_spaces) = match f.align().unwrap_or(Alignment::Right) {
            Alignment::Left => (0, padding),
            Alignment::Right => (padding, 0),
            Alignment::Center => (padding / 2, padding - padding / 2),
        };
        write_repeated(f, SPACE_PADDING, left_spaces)?;
        f.write_str(text)?;
        write_repeated(f, SPACE_PADDING, right_spaces)
    }
}

#[derive(Clone, Copy)]
enum FloatParts {
    Finite {
        negative: bool,
        significand: u128,
        binary_exponent: i32,
    },
    Infinite {
        negative: bool,
    },
    Nan {
        negative: bool,
    },
}

impl FloatParts {
    fn from_value<T: FixedValue>(value: T) -> Self {
        Self::from_ieee_bits(
            value.bits(),
            T::FRACTION_BITS,
            T::EXPONENT_BITS,
            T::EXPONENT_BIAS,
        )
    }

    fn from_ieee_bits(
        bits: u128,
        fraction_bits: u32,
        exponent_bits: u32,
        exponent_bias: i32,
    ) -> Self {
        let negative = bits >> (fraction_bits + exponent_bits) != 0;
        let exponent_mask = (1_u128 << exponent_bits) - 1;
        let exponent_field = (bits >> fraction_bits) & exponent_mask;
        let fraction_mask = (1_u128 << fraction_bits) - 1;
        let fraction = bits & fraction_mask;

        if exponent_field == exponent_mask {
            return if fraction == 0 {
                Self::Infinite { negative }
            } else {
                Self::Nan { negative }
            };
        }

        let (significand, unbiased_exponent) = if exponent_field == 0 {
            (fraction, 1 - exponent_bias)
        } else {
            (
                (1_u128 << fraction_bits) | fraction,
                exponent_field as i32 - exponent_bias,
            )
        };
        Self::Finite {
            negative,
            significand,
            binary_exponent: unbiased_exponent - fraction_bits as i32,
        }
    }
}

fn write_fixed(
    f: &mut Formatter<'_>,
    parts: FloatParts,
    space_sign: bool,
    uppercase: bool,
) -> fmt::Result {
    let (negative, body, finite) = match parts {
        FloatParts::Finite {
            negative,
            significand,
            binary_exponent,
        } => (
            negative,
            fixed_body(
                significand,
                binary_exponent,
                f.precision().unwrap_or(6),
                f.alternate(),
            ),
            true,
        ),
        FloatParts::Infinite { negative } => (
            negative,
            if uppercase { "INF" } else { "inf" }.to_owned(),
            false,
        ),
        FloatParts::Nan { negative } => (
            negative,
            if uppercase { "NAN" } else { "nan" }.to_owned(),
            false,
        ),
    };

    write_float_field(f, negative, &body, finite, space_sign)
}

fn write_scientific(
    f: &mut Formatter<'_>,
    parts: FloatParts,
    space_sign: bool,
    uppercase: bool,
) -> fmt::Result {
    let (negative, body, finite) = match parts {
        FloatParts::Finite {
            negative,
            significand,
            binary_exponent,
        } => (
            negative,
            scientific_body(
                significand,
                binary_exponent,
                f.precision().unwrap_or(6),
                f.alternate(),
                uppercase,
            ),
            true,
        ),
        FloatParts::Infinite { negative } => (
            negative,
            if uppercase { "INF" } else { "inf" }.to_owned(),
            false,
        ),
        FloatParts::Nan { negative } => (
            negative,
            if uppercase { "NAN" } else { "nan" }.to_owned(),
            false,
        ),
    };

    write_float_field(f, negative, &body, finite, space_sign)
}

fn write_general(
    f: &mut Formatter<'_>,
    parts: FloatParts,
    space_sign: bool,
    uppercase: bool,
) -> fmt::Result {
    let (negative, body, finite) = match parts {
        FloatParts::Finite {
            negative,
            significand,
            binary_exponent,
        } => (
            negative,
            general_body(
                significand,
                binary_exponent,
                f.precision().unwrap_or(6).max(1),
                f.alternate(),
                uppercase,
            ),
            true,
        ),
        FloatParts::Infinite { negative } => (
            negative,
            if uppercase { "INF" } else { "inf" }.to_owned(),
            false,
        ),
        FloatParts::Nan { negative } => (
            negative,
            if uppercase { "NAN" } else { "nan" }.to_owned(),
            false,
        ),
    };

    write_float_field(f, negative, &body, finite, space_sign)
}

fn write_hex_float<T: FixedValue>(
    f: &mut Formatter<'_>,
    parts: FloatParts,
    space_sign: bool,
    uppercase: bool,
) -> fmt::Result {
    match parts {
        FloatParts::Finite {
            negative,
            significand,
            binary_exponent,
        } => {
            let body = hex_float_body(
                significand,
                binary_exponent,
                T::HEX_FRACTION_BITS,
                T::HEX_MIN_NORMAL_EXPONENT,
                f.precision(),
                f.alternate(),
                uppercase,
            );
            write_hex_float_field(f, negative, &body, space_sign, uppercase)
        }
        FloatParts::Infinite { negative } => write_float_field(
            f,
            negative,
            if uppercase { "INF" } else { "inf" },
            false,
            space_sign,
        ),
        FloatParts::Nan { negative } => write_float_field(
            f,
            negative,
            if uppercase { "NAN" } else { "nan" },
            false,
            space_sign,
        ),
    }
}

fn hex_float_body(
    significand: u128,
    binary_exponent: i32,
    fraction_bits: u32,
    minimum_normal_exponent: i32,
    precision: Option<usize>,
    alternate: bool,
    uppercase: bool,
) -> String {
    debug_assert_eq!(fraction_bits % 4, 0);

    let (mantissa, displayed_exponent) = if significand == 0 {
        (0, 0)
    } else {
        let highest_bit = (u128::BITS - 1 - significand.leading_zeros()) as i32;
        let highest_exponent = highest_bit + binary_exponent;
        if highest_exponent >= minimum_normal_exponent {
            let shift = fraction_bits as i32 - highest_bit;
            debug_assert!(shift >= 0);
            (significand << shift as usize, highest_exponent)
        } else {
            let shift = binary_exponent + fraction_bits as i32 - minimum_normal_exponent;
            debug_assert!(shift >= 0);
            (significand << shift as usize, minimum_normal_exponent)
        }
    };

    let exact_fraction_digits = fraction_bits as usize / 4;
    let (digits, fractional_digits) = match precision {
        Some(requested) if requested < exact_fraction_digits => {
            let discarded_bits = (exact_fraction_digits - requested) * 4;
            let mut rounded = mantissa >> discarded_bits;
            let discarded_mask = (1_u128 << discarded_bits) - 1;
            let discarded = mantissa & discarded_mask;
            let halfway = 1_u128 << (discarded_bits - 1);
            if discarded > halfway || (discarded == halfway && rounded & 1 != 0) {
                rounded += 1;
            }
            (
                padded_hex_digits(rounded, requested + 1, uppercase),
                requested,
            )
        }
        Some(requested) => {
            let mut digits = padded_hex_digits(mantissa, exact_fraction_digits + 1, uppercase);
            digits.extend(std::iter::repeat_n('0', requested - exact_fraction_digits));
            (digits, requested)
        }
        None => {
            let mut digits = padded_hex_digits(mantissa, exact_fraction_digits + 1, uppercase);
            while digits.len() > 1 && digits.ends_with('0') {
                digits.pop();
            }
            let fractional_digits = digits.len() - 1;
            (digits, fractional_digits)
        }
    };

    let mut body = String::with_capacity(digits.len() + 10);
    body.push_str(&digits[..1]);
    if fractional_digits != 0 || alternate {
        body.push('.');
    }
    body.push_str(&digits[1..]);
    body.push(if uppercase { 'P' } else { 'p' });
    if displayed_exponent < 0 {
        body.push('-');
    } else {
        body.push('+');
    }
    body.push_str(&displayed_exponent.unsigned_abs().to_string());
    body
}

fn padded_hex_digits(value: u128, width: usize, uppercase: bool) -> String {
    let digits = if uppercase {
        format!("{value:X}")
    } else {
        format!("{value:x}")
    };
    if digits.len() >= width {
        return digits;
    }

    let mut padded = String::with_capacity(width);
    padded.extend(std::iter::repeat_n('0', width - digits.len()));
    padded.push_str(&digits);
    padded
}

fn write_hex_float_field(
    f: &mut Formatter<'_>,
    negative: bool,
    body: &str,
    space_sign: bool,
    uppercase: bool,
) -> fmt::Result {
    let sign = if negative {
        "-"
    } else if f.sign_plus() {
        "+"
    } else if space_sign {
        " "
    } else {
        ""
    };
    let prefix = if uppercase { "0X" } else { "0x" };
    let content_width = sign.len() + prefix.len() + body.len();
    let padding = f.width().unwrap_or_default().saturating_sub(content_width);
    let alignment = f.align().unwrap_or(Alignment::Right);
    let zero_padding = f.sign_aware_zero_pad() && alignment == Alignment::Right;
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
    f.write_str(body)?;
    write_repeated(f, SPACE_PADDING, right_spaces)
}

fn write_float_field(
    f: &mut Formatter<'_>,
    negative: bool,
    body: &str,
    finite: bool,
    space_sign: bool,
) -> fmt::Result {
    let sign = if negative {
        "-"
    } else if f.sign_plus() {
        "+"
    } else if space_sign {
        " "
    } else {
        ""
    };
    let content_width = sign.len() + body.len();
    let padding = f.width().unwrap_or_default().saturating_sub(content_width);
    let alignment = f.align().unwrap_or(Alignment::Right);
    let zero_padding = finite && f.sign_aware_zero_pad() && alignment == Alignment::Right;
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
    if zero_padding {
        write_repeated(f, ZERO_PADDING, padding)?;
    }
    f.write_str(body)?;
    write_repeated(f, SPACE_PADDING, right_spaces)
}

fn scientific_body(
    significand: u128,
    binary_exponent: i32,
    precision: usize,
    alternate: bool,
    uppercase: bool,
) -> String {
    let (digits, decimal_exponent) =
        rounded_significant_digits(significand, binary_exponent, precision + 1);
    scientific_body_from_digits(&digits, decimal_exponent, alternate, uppercase)
}

fn rounded_significant_digits(
    significand: u128,
    binary_exponent: i32,
    significant_digits: usize,
) -> (String, i32) {
    debug_assert_ne!(significant_digits, 0);
    let mut decimal_exponent = if significand == 0 {
        0
    } else {
        decimal_exponent(significand, binary_exponent)
    };

    let rounded = if decimal_exponent <= 0 {
        let decimal_shift = significant_digits - 1 + (-decimal_exponent) as usize;
        scale_and_round(significand, binary_exponent, decimal_shift)
    } else if significant_digits > decimal_exponent as usize {
        scale_and_round(
            significand,
            binary_exponent,
            significant_digits - 1 - decimal_exponent as usize,
        )
    } else {
        scale_down_and_round(
            significand,
            binary_exponent,
            decimal_exponent as usize + 1 - significant_digits,
        )
    };

    let expected_digits = significant_digits;
    let mut digits = rounded.to_str_radix(10);
    match digits.len().cmp(&expected_digits) {
        std::cmp::Ordering::Greater => {
            decimal_exponent += 1;
            digits.pop();
        }
        std::cmp::Ordering::Less => {
            digits.insert_str(0, &"0".repeat(expected_digits - digits.len()));
        }
        std::cmp::Ordering::Equal => {}
    }

    (digits, decimal_exponent)
}

fn scientific_body_from_digits(
    digits: &str,
    decimal_exponent: i32,
    alternate: bool,
    uppercase: bool,
) -> String {
    let precision = digits.len() - 1;
    let mut body = String::with_capacity(digits.len() + 8);
    body.push_str(&digits[..1]);
    if precision != 0 || alternate {
        body.push('.');
    }
    body.push_str(&digits[1..]);
    body.push(if uppercase { 'E' } else { 'e' });
    if decimal_exponent < 0 {
        body.push('-');
    } else {
        body.push('+');
    }
    let exponent_digits = decimal_exponent.unsigned_abs().to_string();
    if exponent_digits.len() < 2 {
        body.push('0');
    }
    body.push_str(&exponent_digits);
    body
}

fn general_body(
    significand: u128,
    binary_exponent: i32,
    precision: usize,
    alternate: bool,
    uppercase: bool,
) -> String {
    let (digits, decimal_exponent) =
        rounded_significant_digits(significand, binary_exponent, precision);
    let use_scientific =
        decimal_exponent < -4 || (decimal_exponent >= 0 && decimal_exponent as usize >= precision);

    if use_scientific {
        let mut body = scientific_body_from_digits(&digits, decimal_exponent, alternate, uppercase);
        if !alternate {
            trim_scientific_fraction(&mut body);
        }
        body
    } else {
        let fractional_digits = (precision as i32 - (decimal_exponent + 1)) as usize;
        let mut body = fixed_body(significand, binary_exponent, fractional_digits, alternate);
        if !alternate {
            trim_fixed_fraction(&mut body);
        }
        body
    }
}

fn trim_fixed_fraction(body: &mut String) {
    if let Some(point) = body.find('.') {
        while body.as_bytes().last() == Some(&b'0') {
            body.pop();
        }
        if body.len() == point + 1 {
            body.pop();
        }
    }
}

fn trim_scientific_fraction(body: &mut String) {
    let exponent = body
        .find(['e', 'E'])
        .expect("scientific body always has an exponent marker");
    let suffix = body.split_off(exponent);
    trim_fixed_fraction(body);
    body.push_str(&suffix);
}

fn decimal_exponent(significand: u128, binary_exponent: i32) -> i32 {
    debug_assert_ne!(significand, 0);
    let highest_binary_exponent =
        (u128::BITS - 1 - significand.leading_zeros()) as i32 + binary_exponent;
    let mut exponent =
        (f64::from(highest_binary_exponent) * std::f64::consts::LOG10_2).floor() as i32;

    while compare_with_power_of_ten(significand, binary_exponent, exponent).is_lt() {
        exponent -= 1;
    }
    while !compare_with_power_of_ten(significand, binary_exponent, exponent + 1).is_lt() {
        exponent += 1;
    }
    exponent
}

fn compare_with_power_of_ten(
    significand: u128,
    binary_exponent: i32,
    decimal_exponent: i32,
) -> std::cmp::Ordering {
    let mut left = BigUint::from(significand);
    let mut right = BigUint::from(1_u8);

    if binary_exponent >= 0 {
        left <<= binary_exponent as usize;
    } else {
        right <<= (-binary_exponent) as usize;
    }

    if decimal_exponent >= 0 {
        right *= pow_ten(decimal_exponent as usize);
    } else {
        left *= pow_ten((-decimal_exponent) as usize);
    }
    left.cmp(&right)
}

fn scale_down_and_round(significand: u128, binary_exponent: i32, decimal_places: usize) -> BigUint {
    if significand == 0 {
        return BigUint::from(0_u8);
    }

    let mut numerator = BigUint::from(significand);
    let mut denominator = pow_five(decimal_places);
    let binary_shift = i64::from(binary_exponent) - decimal_places as i64;
    if binary_shift >= 0 {
        numerator <<= binary_shift as usize;
    } else {
        denominator <<= (-binary_shift) as usize;
    }

    let quotient = &numerator / &denominator;
    let remainder = numerator - &quotient * &denominator;
    let twice_remainder = remainder << 1;
    if twice_remainder > denominator || (twice_remainder == denominator && quotient.bit(0)) {
        quotient + 1_u8
    } else {
        quotient
    }
}

fn fixed_body(
    significand: u128,
    binary_exponent: i32,
    precision: usize,
    alternate: bool,
) -> String {
    let scaled = scale_and_round(significand, binary_exponent, precision);
    let digits = scaled.to_str_radix(10);

    if precision == 0 {
        if alternate {
            return digits + ".";
        }
        return digits;
    }

    if digits.len() > precision {
        let point = digits.len() - precision;
        let mut result = String::with_capacity(digits.len() + 1);
        result.push_str(&digits[..point]);
        result.push('.');
        result.push_str(&digits[point..]);
        result
    } else {
        let leading_zeros = precision - digits.len();
        let mut result = String::with_capacity(precision + 2);
        result.push_str("0.");
        result.extend(std::iter::repeat_n('0', leading_zeros));
        result.push_str(&digits);
        result
    }
}

fn scale_and_round(significand: u128, binary_exponent: i32, precision: usize) -> BigUint {
    if significand == 0 {
        return BigUint::from(0_u8);
    }

    let mut numerator = BigUint::from(significand) * pow_five(precision);
    if binary_exponent >= 0 {
        return numerator << (precision + binary_exponent as usize);
    }

    let denominator_shift = (-binary_exponent) as usize;
    if precision >= denominator_shift {
        return numerator << (precision - denominator_shift);
    }

    let shift = denominator_shift - precision;
    let quotient = &numerator >> shift;
    numerator -= &quotient << shift;
    let halfway = BigUint::from(1_u8) << (shift - 1);
    if numerator > halfway || (numerator == halfway && quotient.bit(0)) {
        quotient + 1_u8
    } else {
        quotient
    }
}

fn pow_five(mut exponent: usize) -> BigUint {
    let mut result = BigUint::from(1_u8);
    let mut base = BigUint::from(5_u8);
    while exponent != 0 {
        if exponent & 1 != 0 {
            result *= &base;
        }
        exponent >>= 1;
        if exponent != 0 {
            base = &base * &base;
        }
    }
    result
}

fn pow_ten(exponent: usize) -> BigUint {
    pow_five(exponent) << exponent
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
