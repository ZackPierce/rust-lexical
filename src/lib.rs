//! Fast lexical conversion routines.
//!
//! TODO(document)

// FEATURES

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
#![cfg_attr(feature = "alloc", feature(alloc))]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(test, feature = "alloc", not(feature = "std")))]
extern crate wee_alloc;

#[cfg(feature = "f128")]
extern crate f128;

#[cfg(test)]
#[macro_use]
extern crate assert_float_eq;

#[cfg(all(feature = "alloc", not(feature = "std")))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Facade around the core features for name mangling.
pub(crate) mod sealed {
    #[cfg(not(feature = "std"))]
    pub use core::*;

    #[cfg(feature = "std")]
    pub use std::*;
}

// Hide the implementation details.
#[macro_use]
mod util;

mod float;

// Publicly export the low-level APIs.
// Macros used in atoi are required for atof, so export those.
#[macro_use]
pub mod atoi;

pub mod atof;
pub mod ftoa;
pub mod itoa;

#[doc(hidden)]
pub mod table;

#[doc(hidden)]
pub mod traits;

// HIGH LEVEL

use sealed::convert::AsRef;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::string::String;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::vec::Vec;

use traits::{Aton, Ntoa};

/// High-level conversion of a number to a decimal-encoded string.
///
/// * `n`       - Number to convert to string.
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// assert_eq!(lexical::to_string(5), "5");
/// assert_eq!(lexical::to_string(0.0), "0.0");
/// # }
/// ```
#[inline(always)]
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn to_string<N: Ntoa>(n: N) -> String {
    to_string_digits(n, 10)
}

/// High-level conversion of a number to string with a custom radix.
///
/// * `n`       - Number to convert to string.
/// * `base`    - Number of unique digits for the number (radix).
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// assert_eq!(lexical::to_string_digits(5, 10), "5");
/// assert_eq!(lexical::to_string_digits(0.0, 10), "0.0");
/// # }
/// ```
#[inline(always)]
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn to_string_digits<N: Ntoa>(n: N, base: u8) -> String {
    n.serialize_to_string(base)
}

/// High-level conversion of decimal-encoded bytes to a number.
///
/// This function **always** returns a number, parsing until invalid
/// digits are found. For an error-checking version of this function,
/// use [`try_parse`].
///
/// * `bytes`   - Byte slice to convert to number.
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// // String overloads
/// assert_eq!(lexical::parse::<i32, _>("5"), 5);
/// assert_eq!(lexical::parse::<i32, _>("1a"), 1);
/// assert_eq!(lexical::parse::<f32, _>("0"), 0.0);
///
/// // Bytes overloads
/// assert_eq!(lexical::parse::<i32, _>(b"5"), 5);
/// assert_eq!(lexical::parse::<i32, _>(b"1a"), 1);
/// assert_eq!(lexical::parse::<f32, _>(b"0"), 0.0);
/// # }
/// ```
///
/// [`try_parse`]: fn.try_parse.html
#[inline(always)]
pub fn parse<N: Aton, Bytes: AsRef<[u8]>>(bytes: Bytes) -> N {
    parse_digits::<N, Bytes>(bytes, 10)
}

/// High-level conversion of bytes to a number with a custom radix.
///
/// This function **always** returns a number, parsing until invalid
/// digits are found. For an error-checking version of this function,
/// use [`try_parse_digits`].
///
/// * `bytes`   - Byte slice to convert to number.
/// * `base`    - Number of unique digits for the number (radix).
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// // String overloads
/// assert_eq!(lexical::parse_digits::<i32, _>(b"5", 10), 5);
/// assert_eq!(lexical::parse_digits::<i32, _>(b"1a", 10), 1);
/// assert_eq!(lexical::parse_digits::<f32, _>(b"0", 10), 0.0);
///
/// // Bytes overloads
/// assert_eq!(lexical::parse_digits::<i32, _>(b"5", 10), 5);
/// assert_eq!(lexical::parse_digits::<i32, _>(b"1a", 10), 1);
/// assert_eq!(lexical::parse_digits::<f32, _>(b"0", 10), 0.0);
/// # }
/// ```
///
/// [`try_parse_digits`]: fn.try_parse_digits.html
#[inline(always)]
pub fn parse_digits<N: Aton, Bytes: AsRef<[u8]>>(bytes: Bytes, base: u8) -> N {
    N::deserialize_from_bytes(bytes.as_ref(), base)
}

/// High-level conversion of decimal-encoded bytes to a number.
///
/// This function only returns a value if the entire string is
/// successfully parsed. For an unchecked version of this function,
/// use [`parse`].
///
/// * `bytes`   - Byte slice to convert to number.
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// // String overloads
/// assert_eq!(lexical::try_parse::<i32, _>("5"), Ok(5));
/// assert_eq!(lexical::try_parse::<i32, _>("1a"), Err(1));
/// assert_eq!(lexical::try_parse::<f32, _>("0"), Ok(0.0));
///
/// // Bytes overloads
/// assert_eq!(lexical::try_parse::<i32, _>(b"5"), Ok(5));
/// assert_eq!(lexical::try_parse::<i32, _>(b"1a"), Err(1));
/// assert_eq!(lexical::try_parse::<f32, _>(b"0"), Ok(0.0));
/// # }
/// ```
///
/// [`parse`]: fn.parse.html
#[inline(always)]
pub fn try_parse<N: Aton, Bytes: AsRef<[u8]>>(bytes: Bytes)
    -> Result<N, usize>
{
    try_parse_digits::<N, Bytes>(bytes, 10)
}

/// High-level conversion of bytes to a number with a custom radix.
///
/// This function only returns a value if the entire string is
/// successfully parsed. For an unchecked version of this function,
/// use [`parse_digits`].
///
/// * `bytes`   - Byte slice to convert to number.
/// * `base`    - Number of unique digits for the number (radix).
///
/// # Examples
///
/// ```rust
/// # extern crate lexical;
/// # pub fn main() {
/// // String overloads
/// assert_eq!(lexical::try_parse_digits::<i32, _>(b"5", 10), Ok(5));
/// assert_eq!(lexical::try_parse_digits::<i32, _>(b"1a", 10), Err(1));
/// assert_eq!(lexical::try_parse_digits::<f32, _>(b"0", 10), Ok(0.0));
///
/// // Bytes overloads
/// assert_eq!(lexical::try_parse_digits::<i32, _>(b"5", 10), Ok(5));
/// assert_eq!(lexical::try_parse_digits::<i32, _>(b"1a", 10), Err(1));
/// assert_eq!(lexical::try_parse_digits::<f32, _>(b"0", 10), Ok(0.0));
/// # }
/// ```
///
/// [`parse_digits`]: fn.parse_digits.html
#[inline(always)]
pub fn try_parse_digits<N: Aton, Bytes: AsRef<[u8]>>(bytes: Bytes, base: u8)
    -> Result<N, usize>
{
    N::try_deserialize_from_bytes(bytes.as_ref(), base)
}