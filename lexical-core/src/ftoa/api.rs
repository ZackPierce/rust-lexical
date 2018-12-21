//! Low-level API generator.
//!
//! Uses either the internal "Grisu2", or the external "Grisu3" or "Ryu"
//! algorithms provided by `https://github.com/dtolnay`.

use util::*;

#[cfg(feature = "radix")]
use super::radix::{double_radix, float_radix};

// Select the back-end
cfg_if! {
if #[cfg(feature = "grisu3")] {
    use super::grisu3::{double_decimal, float_decimal};
} else if #[cfg(feature = "ryu")] {
    use super::ryu::{double_decimal, float_decimal};
} else {
    use super::grisu2::{double_decimal, float_decimal};
}}  //cfg_if

// TRAITS

/// Trait to define serialization of a float to string.
pub(crate) trait FloatToString: Float {
    /// Export float to decimal string with optimized algorithm.
    fn decimal<'a>(self, bytes: &'a mut [u8]) -> &'a mut [u8];

    /// Export float to radix string with slow algorithm.
    #[cfg(feature = "radix")]
    fn radix<'a>(self, radix: u32, bytes: &'a mut [u8]) -> &'a mut [u8];
}

impl FloatToString for f32 {
    #[inline(always)]
    fn decimal<'a>(self, bytes: &'a mut [u8]) -> &'a mut [u8] {
        float_decimal(self, bytes)
    }

    #[cfg(feature = "radix")]
    #[inline(always)]
    fn radix<'a>(self, radix: u32, bytes: &'a mut [u8]) -> &'a mut [u8] {
        float_radix(self, radix, bytes)
    }
}

impl FloatToString for f64 {
    #[inline(always)]
    fn decimal<'a>(self, bytes: &'a mut [u8]) -> &'a mut [u8] {
        double_decimal(self, bytes)
    }

    #[cfg(feature = "radix")]
    #[inline(always)]
    fn radix<'a>(self, radix: u32, bytes: &'a mut [u8]) -> &'a mut [u8] {
        double_radix(self, radix, bytes)
    }
}

// FTOA

/// Forward the correct arguments the ideal encoder.
#[inline]
fn forward<'a, F: FloatToString>(value: F, radix: u32, bytes: &'a mut [u8])
    -> &'a mut [u8]
{
    debug_assert_radix!(radix);

    #[cfg(not(feature = "radix"))] {
        value.decimal(bytes)
    }

    #[cfg(feature = "radix")] {
        match radix {
            10 => value.decimal(bytes),
            _  => value.radix(radix, bytes),
        }
    }
}

/// Convert float-to-string and handle special (positive) floats.
#[inline]
fn filter_special<'a, F: FloatToString>(value: F, radix: u32, bytes: &'a mut [u8])
    -> &'a mut [u8]
{
    // Logic errors, disable in release builds.
    debug_assert!(value.is_sign_positive(), "Value cannot be negative.");
    debug_assert_radix!(radix);

    // We already check for 0 in `filter_sign` if value.is_zero().
    #[cfg(not(feature = "trim_floats"))] {
        if value.is_zero() {
            return copy_to_dst(bytes, "0.0");
        }
    }

    // Unsafe to allow read-access to global mutables.
    unsafe {
        if value.is_nan() {
            copy_to_dst(bytes, NAN_STRING)
        } else if value.is_special() {
            // Must be positive infinity, we've already handled sign
            copy_to_dst(bytes, INF_STRING)
        } else {
            forward(value, radix, bytes)
        }
    }
}

/// Handle +/- values.
#[inline]
fn filter_sign<'a, F: FloatToString>(mut value: F, radix: u32, bytes: &'a mut [u8])
    -> &'a mut [u8]
{
    debug_assert_radix!(radix);

    // Export "-0.0" and "0.0" as "0" with trimmed floats.
    #[cfg(feature = "trim_floats")] {
        if value.is_zero() {
            bytes[0] = b'0';
            return &mut bytes[1..];
        }
    }

    // If the sign bit is set, invert it and just set the first
    // value to "-".
    if value.is_sign_negative() {
        bytes[0] = b'-';
        value = -value;
        filter_special(value, radix, &mut bytes[1..])
    } else {
        filter_special(value, radix, bytes)
    }
}

/// Iteratively filter simple cases then invoke serializer.
#[inline(always)]
fn ftoa<F: FloatToString>(value: F, radix: u32, bytes: &mut [u8])
    -> usize
{
    let first = bytes.as_ptr();
    let slc = filter_sign(value, radix, bytes);
    distance(first, slc.as_ptr())
}

/// Trim a trailing ".0" from a float.
#[inline(always)]
fn trim<'a>(bytes: &'a mut [u8])
    -> &'a mut[u8]
{
    // Trim a trailing ".0" from a float.
    #[cfg(feature = "trim_floats")] {
        if ends_with_slice(bytes, b".0") {
            slice_from_span_mut(bytes.as_mut_ptr(), bytes.len() - 2)
        } else {
            bytes
        }
    }

    #[cfg(not(feature = "trim_floats"))] {
        bytes
    }
}

// UNSAFE API

/// Expand the generic ftoa function for specified types.
macro_rules! wrap {
    ($name:ident, $t:ty) => (
        /// Serialize float and return bytes written to.
        #[inline]
        fn $name<'a>(value: $t, base: u8, bytes: &'a mut [u8])
            -> &'a mut [u8]
        {
            // Check buffer has sufficient capacity.
            let len = ftoa(value, base.into(), bytes);
            trim(&mut bytes[..len])
        }
    )
}

wrap!(f32toa_impl, f32);
wrap!(f64toa_impl, f64);

// LOW-LEVEL API
// -------------

// RANGE API (FFI)
generate_to_range_api!(f32toa_range, f32, f32toa_impl, MAX_F32_SIZE);
generate_to_range_api!(f64toa_range, f64, f64toa_impl, MAX_F64_SIZE);

// SLICE API
generate_to_slice_api!(f32toa_slice, f32, f32toa_impl, MAX_F32_SIZE);
generate_to_slice_api!(f64toa_slice, f64, f64toa_impl, MAX_F64_SIZE);

// TESTS
// -----

#[cfg(test)]
mod tests {
    use util::test::*;
    use super::*;
    use atof::*;

    // Test data for roundtrips.
    const F32_DATA : [f32; 31] = [0., 0.1, 1., 1.1, 12., 12.1, 123., 123.1, 1234., 1234.1, 12345., 12345.1, 123456., 123456.1, 1234567., 1234567.1, 12345678., 12345678.1, 123456789., 123456789.1, 123456789.12, 123456789.123, 123456789.1234, 123456789.12345, 1.2345678912345e8, 1.2345e+8, 1.2345e+11, 1.2345e+38, 1.2345e-8, 1.2345e-11, 1.2345e-38];
    const F64_DATA: [f64; 33] = [0., 0.1, 1., 1.1, 12., 12.1, 123., 123.1, 1234., 1234.1, 12345., 12345.1, 123456., 123456.1, 1234567., 1234567.1, 12345678., 12345678.1, 123456789., 123456789.1, 123456789.12, 123456789.123, 123456789.1234, 123456789.12345, 1.2345678912345e8, 1.2345e+8, 1.2345e+11, 1.2345e+38, 1.2345e+308, 1.2345e-8, 1.2345e-11, 1.2345e-38, 1.2345e-299];

    #[cfg(feature = "radix")]
    #[test]
    fn f32toa_base2_test() {
        let mut buffer = new_buffer();
        // positive
        #[cfg(feature = "trim_floats")] {
            assert_eq!(as_slice(b"0"), f32toa_slice(0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"0"), f32toa_slice(-0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"1"), f32toa_slice(1.0, 2, &mut buffer));
            assert_eq!(as_slice(b"10"), f32toa_slice(2.0, 2, &mut buffer));
        }

        #[cfg(not(feature = "trim_floats"))] {
            assert_eq!(as_slice(b"0.0"), f32toa_slice(0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"-0.0"), f32toa_slice(-0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"1.0"), f32toa_slice(1.0, 2, &mut buffer));
            assert_eq!(as_slice(b"10.0"), f32toa_slice(2.0, 2, &mut buffer));
        }

        assert_eq!(as_slice(b"1.1"), f32toa_slice(1.5, 2, &mut buffer));
        assert_eq!(as_slice(b"1.01"), f32toa_slice(1.25, 2, &mut buffer));
        assert_eq!(b"1.001111000000110010", &f32toa_slice(1.2345678901234567890e0, 2, &mut buffer)[..20]);
        assert_eq!(b"1100.010110000111111", &f32toa_slice(1.2345678901234567890e1, 2, &mut buffer)[..20]);
        assert_eq!(b"1111011.011101001111", &f32toa_slice(1.2345678901234567890e2, 2, &mut buffer)[..20]);
        assert_eq!(b"10011010010.10010001", &f32toa_slice(1.2345678901234567890e3, 2, &mut buffer)[..20]);

        // negative
        assert_eq!(b"-1.001111000000110010", &f32toa_slice(-1.2345678901234567890e0, 2, &mut buffer)[..21]);
        assert_eq!(b"-1100.010110000111111", &f32toa_slice(-1.2345678901234567890e1, 2, &mut buffer)[..21]);
        assert_eq!(b"-1111011.011101001111", &f32toa_slice(-1.2345678901234567890e2, 2, &mut buffer)[..21]);
        assert_eq!(b"-10011010010.10010001", &f32toa_slice(-1.2345678901234567890e3, 2, &mut buffer)[..21]);

        // special
        assert_eq!(as_slice(b"NaN"), f32toa_slice(f32::NAN, 2, &mut buffer));
        assert_eq!(as_slice(b"inf"), f32toa_slice(f32::INFINITY, 2, &mut buffer));

        // bugfixes
        assert_eq!(as_slice(b"1.1010100000101011110001e-11011"), f32toa_slice(0.000000012345, 2, &mut buffer));
    }

    #[test]
    fn f32toa_base10_test() {
        let mut buffer = new_buffer();
        // positive
        #[cfg(feature = "trim_floats")] {
            assert_eq!(as_slice(b"0"), f32toa_slice(0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"0"), f32toa_slice(-0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"1"), f32toa_slice(1.0, 10, &mut buffer));
            assert_eq!(as_slice(b"10"), f32toa_slice(10.0, 10, &mut buffer));
        }

        #[cfg(not(feature = "trim_floats"))] {
            assert_eq!(as_slice(b"0.0"), f32toa_slice(0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"-0.0"), f32toa_slice(-0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"1.0"), f32toa_slice(1.0, 10, &mut buffer));
            assert_eq!(as_slice(b"10.0"), f32toa_slice(10.0, 10, &mut buffer));
        }

        assert_eq!(as_slice(b"1.234567"), &f32toa_slice(1.2345678901234567890e0, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"12.34567"), &f32toa_slice(1.2345678901234567890e1, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"123.4567"), &f32toa_slice(1.2345678901234567890e2, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"1234.567"), &f32toa_slice(1.2345678901234567890e3, 10, &mut buffer)[..8]);

        // negative
        assert_eq!(as_slice(b"-1.234567"), &f32toa_slice(-1.2345678901234567890e0, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-12.34567"), &f32toa_slice(-1.2345678901234567890e1, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-123.4567"), &f32toa_slice(-1.2345678901234567890e2, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-1234.567"), &f32toa_slice(-1.2345678901234567890e3, 10, &mut buffer)[..9]);

        // special
        assert_eq!(as_slice(b"NaN"), f32toa_slice(f32::NAN, 10, &mut buffer));
        assert_eq!(as_slice(b"inf"), f32toa_slice(f32::INFINITY, 10, &mut buffer));
    }

    #[test]
    fn f32toa_base10_roundtrip_test() {
        let mut buffer = new_buffer();
        for f in F32_DATA.iter() {
            let s = f32toa_slice(*f, 10, &mut buffer);
            assert_relative_eq!(atof32_slice(10, s), *f, epsilon=1e-6, max_relative=1e-6);
        }
    }

    #[cfg(feature = "radix")]
    #[test]
    fn f32toa_basen_roundtrip_test() {
        let mut buffer = new_buffer();
        for f in F32_DATA.iter() {
            for radix in 2..37 {
                // The lower accuracy is due to slight rounding errors of
                // ftoa for the Grisu method with non-10 bases.
                let s = f32toa_slice(*f, radix, &mut buffer);
                assert_relative_eq!(atof32_slice(radix, s), *f, max_relative=2e-5);
            }
        }
    }

    #[cfg(feature = "radix")]
    #[test]
    fn f64toa_base2_test() {
        let mut buffer = new_buffer();
        // positive
        #[cfg(feature = "trim_floats")] {
            assert_eq!(as_slice(b"0"), f64toa_slice(0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"0"), f64toa_slice(-0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"1"), f64toa_slice(1.0, 2, &mut buffer));
            assert_eq!(as_slice(b"10"), f64toa_slice(2.0, 2, &mut buffer));
        }

        #[cfg(not(feature = "trim_floats"))] {
            assert_eq!(as_slice(b"0.0"), f64toa_slice(0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"-0.0"), f64toa_slice(-0.0, 2, &mut buffer));
            assert_eq!(as_slice(b"1.0"), f64toa_slice(1.0, 2, &mut buffer));
            assert_eq!(as_slice(b"10.0"), f64toa_slice(2.0, 2, &mut buffer));
        }

        assert_eq!(as_slice(b"1.00111100000011001010010000101000110001"), &f64toa_slice(1.2345678901234567890e0, 2, &mut buffer)[..40]);
        assert_eq!(as_slice(b"1100.01011000011111100110100110010111101"), &f64toa_slice(1.2345678901234567890e1, 2, &mut buffer)[..40]);
        assert_eq!(as_slice(b"1111011.01110100111100000001111111101101"), &f64toa_slice(1.2345678901234567890e2, 2, &mut buffer)[..40]);
        assert_eq!(as_slice(b"10011010010.1001000101100001001111110100"), &f64toa_slice(1.2345678901234567890e3, 2, &mut buffer)[..40]);

        // negative
        assert_eq!(as_slice(b"-1.00111100000011001010010000101000110001"), &f64toa_slice(-1.2345678901234567890e0, 2, &mut buffer)[..41]);
        assert_eq!(as_slice(b"-1100.01011000011111100110100110010111101"), &f64toa_slice(-1.2345678901234567890e1, 2, &mut buffer)[..41]);
        assert_eq!(as_slice(b"-1111011.01110100111100000001111111101101"), &f64toa_slice(-1.2345678901234567890e2, 2, &mut buffer)[..41]);
        assert_eq!(as_slice(b"-10011010010.1001000101100001001111110100"), &f64toa_slice(-1.2345678901234567890e3, 2, &mut buffer)[..41]);

        // special
        assert_eq!(as_slice(b"NaN"), f64toa_slice(f64::NAN, 2, &mut buffer));
        assert_eq!(as_slice(b"inf"), f64toa_slice(f64::INFINITY, 2, &mut buffer));
    }

    #[test]
    fn f64toa_base10_test() {
        let mut buffer = new_buffer();
        // positive
        #[cfg(feature = "trim_floats")] {
            assert_eq!(as_slice(b"0"), f64toa_slice(0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"0"), f64toa_slice(-0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"1"), f64toa_slice(1.0, 10, &mut buffer));
            assert_eq!(as_slice(b"10"), f64toa_slice(10.0, 10, &mut buffer));
        }

        #[cfg(not(feature = "trim_floats"))] {
            assert_eq!(as_slice(b"0.0"), f64toa_slice(0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"-0.0"), f64toa_slice(-0.0, 10, &mut buffer));
            assert_eq!(as_slice(b"1.0"), f64toa_slice(1.0, 10, &mut buffer));
            assert_eq!(as_slice(b"10.0"), f64toa_slice(10.0, 10, &mut buffer));
        }

        assert_eq!(as_slice(b"1.234567"), &f64toa_slice(1.2345678901234567890e0, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"12.34567"), &f64toa_slice(1.2345678901234567890e1, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"123.4567"), &f64toa_slice(1.2345678901234567890e2, 10, &mut buffer)[..8]);
        assert_eq!(as_slice(b"1234.567"), &f64toa_slice(1.2345678901234567890e3, 10, &mut buffer)[..8]);

        // negative
        assert_eq!(as_slice(b"-1.234567"), &f64toa_slice(-1.2345678901234567890e0, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-12.34567"), &f64toa_slice(-1.2345678901234567890e1, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-123.4567"), &f64toa_slice(-1.2345678901234567890e2, 10, &mut buffer)[..9]);
        assert_eq!(as_slice(b"-1234.567"), &f64toa_slice(-1.2345678901234567890e3, 10, &mut buffer)[..9]);

        // special
        assert_eq!(b"NaN".to_vec(), f64toa_slice(f64::NAN, 10, &mut buffer));
        assert_eq!(b"inf".to_vec(), f64toa_slice(f64::INFINITY, 10, &mut buffer));
    }

    #[test]
    fn f64toa_base10_roundtrip_test() {
        let mut buffer = new_buffer();
        for f in F64_DATA.iter() {
            let s = f64toa_slice(*f, 10, &mut buffer);
            assert_relative_eq!(atof64_slice(10, s), *f, epsilon=1e-12, max_relative=1e-12);
        }
    }

    #[cfg(feature = "radix")]
    #[test]
    fn f64toa_basen_roundtrip_test() {
        let mut buffer = new_buffer();
        for f in F64_DATA.iter() {
            for radix in 2..37 {
                // The lower accuracy is due to slight rounding errors of
                // ftoa for the Grisu method with non-10 bases.
                let s = f64toa_slice(*f, radix, &mut buffer);
                assert_relative_eq!(atof64_slice(radix, s), *f, max_relative=3e-5);
            }
        }
    }

    #[cfg(feature = "correct")]
    quickcheck! {
        fn f32_quickcheck(f: f32) -> bool {
            let mut buffer = new_buffer();
            f == atof32_slice(10, f32toa_slice(f, 10, &mut buffer))
        }

        fn f64_quickcheck(f: f64) -> bool {
            let mut buffer = new_buffer();
            f == atof64_slice(10, f64toa_slice(f, 10, &mut buffer))
        }
    }

    #[cfg(feature = "correct")]
    proptest! {
        #[test]
        fn f332_proptest(i in f32::MIN..f32::MAX) {
            let mut buffer = new_buffer();
            i == atof32_slice(10, f32toa_slice(i, 10, &mut buffer))
        }

        #[test]
        fn f64_proptest(i in f64::MIN..f64::MAX) {
            let mut buffer = new_buffer();
            i == atof64_slice(10, f64toa_slice(i, 10, &mut buffer))
        }
    }

    #[test]
    #[should_panic]
    fn f32toa_buffer_test() {
        let mut buffer = [b'0'; MAX_F32_SIZE-1];
        f64toa_slice(1.2345, 10, &mut buffer);
    }

    #[test]
    #[should_panic]
    fn f64toa_buffer_test() {
        let mut buffer = [b'0'; MAX_F64_SIZE-1];
        f64toa_slice(1.2345, 10, &mut buffer);
    }
}
