use super::Class;
use super::Status;
use super::Rounding;

use decnumber::*;
use context::*;
use libc::{c_char, int32_t, uint8_t, uint32_t};
#[cfg(feature = "ord_subset")]
use ord_subset;
#[cfg(feature = "rustc-serialize")]
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
#[cfg(feature = "serde")]
use serde;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Sum;
use std::mem::uninitialized;
use std::num::FpCategory;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign,
               Neg, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl,
               ShlAssign, Shr, ShrAssign};
use std::str::FromStr;
use std::string::ToString;
use std::str::from_utf8_unchecked;

thread_local!(static CTX: RefCell<Context> = RefCell::new(d64::default_context()));

impl Default for d64 {
    fn default() -> Self {
        d64::zero()
    }
}

#[cfg(feature = "ord_subset")]
impl ord_subset::OrdSubset for d64 {
    fn is_outside_order(&self) -> bool {
        self.is_nan()
    }
}

#[cfg(feature = "ord_subset")]
impl Into<ord_subset::OrdVar<d64>> for d64 {
    fn into(self) -> ord_subset::OrdVar<d64> {
        ord_subset::OrdVar::new(self)
    }
}

#[cfg(feature = "rustc-serialize")]
impl Decodable for d64 {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let s = try!(d.read_str());
        Ok(Self::from_str(&s).expect("unreachable"))
    }
}

#[cfg(feature = "rustc-serialize")]
impl Encodable for d64 {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_str(&format!("{}", self))
    }
}

impl Hash for d64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for d64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::ser::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for d64 {
    fn deserialize<D>(deserializer: D) -> Result<d64, D::Error>
        where D: serde::de::Deserializer<'de>
    {
        deserializer.deserialize_str(d64Visitor)
    }
}

#[cfg(feature = "serde")]
#[allow(non_camel_case_types)]
struct d64Visitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for d64Visitor {
    type Value = d64;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a d64 value")
    }

    fn visit_str<E>(self, s: &str) -> Result<d64, E>
        where E: serde::de::Error
    {
        use serde::de::Unexpected;
        d64::from_str(s).map_err(|_| E::invalid_value(Unexpected::Str(s), &self))
    }
}

/// Converts an i32 to d64. The result is exact and no error is possible.
impl From<i32> for d64 {
    fn from(val: i32) -> d64 {
        unsafe {
            let mut res: d64 = uninitialized();
            *decDoubleFromInt32(&mut res, val)
        }
    }
}

/// Converts an u32 to d64. The result is exact and no error is possible.
impl From<u32> for d64 {
    fn from(val: u32) -> d64 {
        unsafe {
            let mut res: d64 = uninitialized();
            *decDoubleFromUInt32(&mut res, val)
        }
    }
}

impl AsRef<d64> for d64 {
    fn as_ref(&self) -> &d64 {
        &self
    }
}

/// Converts a string to d64. The length of the coefficient and the size of the exponent are
/// checked by this routine, so rounding will be applied if necessary, and this may set status
/// flags (`UNDERFLOW`, `OVERFLOW`) will be reported, or rounding applied, as necessary. There is
/// no limit to the coefficient length for finite inputs; NaN payloads must be integers with no
/// more than 33 digits. Exponents may have up to nine significant digits. The syntax of the string
/// is fully checked; if it is not valid, the result will be a quiet NaN and an error flag will be
/// set.
impl FromStr for d64 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        let cstr = match CString::new(s) {
            Err(..) => CString::new("qNaN").unwrap(),
            Ok(cstr) => cstr,
        };
        d64::with_context(|ctx| {
            let mut res: d64;
            unsafe {
                res = uninitialized();
                decDoubleFromString(&mut res, cstr.as_ptr(), ctx);
            }
            Ok(res)
        })
    }
}

/// Converts an u64 to d64. The result may be inexact/rounded.
impl From<u64> for d64 {
    fn from(val: u64) -> d64 {
        d64::from_str(&val.to_string()).unwrap()
    }
}

/// Converts an i64 to d64. The result is maybe inexact.
impl From<i64> for d64 {
    fn from(val: i64) -> d64 {
        if val < 0 {
            -d64::from(!(val as u64) + 1)
        } else {
            d64::from(val as u64)
        }
    }
}             

/// Converts this d64 to an i32. It uses Rounding::HalfEven.
impl Into<i32> for d64 {
    fn into(self) -> i32 {
        d64::with_context(|ctx| unsafe { decDoubleToInt32(&self, ctx, ctx.rounding) })
    }
}

/// Converts this d64 to an u32. It uses Rounding::HalfEven.
impl Into<u32> for d64 {
    fn into(self) -> u32 {
        d64::with_context(|ctx| unsafe { decDoubleToUInt32(&self, ctx, ctx.rounding) })
    }
}

/// Formats a d64. Finite numbers will be converted to a string with exponential notation if the
/// exponent is positive or if the magnitude of x is less than 1 and would require more than five
/// zeros between the decimal point and the first significant digit. Note that strings which are
/// not simply numbers (one of Infinity, –Infinity, NaN, or sNaN) are possible. A NaN string may
/// have a leading – sign and/or following payload digits. No digits follow the NaN string if the
/// payload is 0.
impl fmt::Display for d64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0; 43];
        unsafe {
            decDoubleToString(self, buf.as_mut().as_mut_ptr());
            let cstr = CStr::from_ptr(buf.as_ptr());
            fmt.pad(from_utf8_unchecked(cstr.to_bytes()))
        }
    }
}

/// Same as `fmt::Display`.
impl fmt::Debug for d64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

/// Formats a d64 with engineering notation. This is the same as fmt::Display except that if
/// exponential notation is used the exponent will be a multiple of 3.
impl fmt::LowerExp for d64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = [0; 43];
        unsafe {
            decDoubleToEngString(self, buf.as_mut().as_mut_ptr());
            let cstr = CStr::from_ptr(buf.as_ptr());
            fmt.pad(from_utf8_unchecked(cstr.to_bytes()))
        }
    }
}

/// Formats a d64 to hexadecimal binary representation.
impl fmt::LowerHex for d64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for b in self.bytes.iter().rev() {
            try!(write!(fmt, "{:02x}", b));
        }
        Ok(())
    }
}

impl PartialEq<d64> for d64 {
    fn eq(&self, other: &d64) -> bool {
        self.compare(other).is_zero()
    }
}

impl PartialOrd<d64> for d64 {
    fn partial_cmp(&self, other: &d64) -> Option<::std::cmp::Ordering> {
        use std::cmp::Ordering;
        match self.compare(other) {
            v if v.is_nan() => None,
            v if v.is_zero() => Some(Ordering::Equal),
            v if v.is_positive() => Some(Ordering::Greater),
            v if v.is_negative() => Some(Ordering::Less),
            _ => unreachable!(),
        }
    }
}

macro_rules! ffi_unary_op {
    ($(#[$attr:meta])* impl $op:ident, $method:ident, $ffi:ident for $t:ident) => {
        $(#[$attr])*
        impl $op for $t {
            type Output = $t;

            fn $method(mut self) -> $t {
                $t::with_context(|ctx| {
                    unsafe { *$ffi(&mut self, &self, ctx)}
                })
            }
        }

        impl<'a> $op for &'a $t {
            type Output = $t;

            fn $method(self) -> $t {
                $t::with_context(|ctx| {
                    unsafe { let mut res: $t = uninitialized(); *$ffi(&mut res, self, ctx)}
                })
            }
        }
    }
}

macro_rules! ffi_binary_op {
    ($(#[$attr:meta])* impl $op:ident, $method:ident, $ffi:ident for $t:ident) => {
        $(#[$attr])*
        impl $op<$t> for $t {
            type Output = $t;

            fn $method(mut self, other: $t) -> $t {
                $t::with_context(|ctx| {
                    unsafe { *$ffi(&mut self, &self, &other, ctx)}
                })
            }
        }

        impl<'a> $op<$t> for &'a $t {
            type Output = $t;

            fn $method(self, mut other: $t) -> $t {
                $t::with_context(|ctx| {
                    unsafe { *$ffi(&mut other, self, &other, ctx) }
                })
            }
        }

        impl<'a> $op<&'a$t> for $t {
            type Output = $t;

            fn $method(mut self, other: &'a $t) -> $t {
                $t::with_context(|ctx| {
                    unsafe { *$ffi(&mut self, &self, other, ctx) }
                })
            }
        }

        impl<'a, 'b> $op<&'a $t> for &'b $t {
            type Output = $t;

            fn $method(self, other: &'a $t) -> $t {
                $t::with_context(|ctx| {
                    unsafe { let mut res: $t = uninitialized(); *$ffi(&mut res, self, other, ctx) }
                })
            }
        }
    }
}

macro_rules! ffi_unary_assign_op {
    ($(#[$attr:meta])* impl $op:ident, $method:ident, $ffi:ident for $t:ident) => {
        $(#[$attr])*
        impl $op<$t> for $t {
            fn $method(&mut self, other: $t) {
                $t::with_context(|ctx| {
                    unsafe { $ffi(self, self, &other, ctx); }
                })
            }
        }
    }
}

ffi_binary_op!(impl Add, add, decDoubleAdd for d64);
ffi_binary_op!(impl Sub, sub, decDoubleSubtract for d64);
ffi_binary_op!(impl Mul, mul, decDoubleMultiply for d64);
ffi_binary_op!(impl Div, div, decDoubleDivide for d64);
ffi_binary_op!(
/// The operands must be zero or positive, an integer (finite with zero exponent) and comprise
/// only zeros and/or ones; if not, INVALID_OPERATION is set.
    impl BitAnd, bitand, decDoubleAnd for d64);
ffi_binary_op!(
/// The operands must be zero or positive, an integer (finite with zero exponent) and comprise
/// only zeros and/or ones; if not, INVALID_OPERATION is set.
    impl BitOr, bitor, decDoubleOr for d64);
ffi_binary_op!(
/// The operands must be zero or positive, an integer (finite with zero exponent) and comprise
/// only zeros and/or ones; if not, INVALID_OPERATION is set.
    impl BitXor, bitxor, decDoubleXor for d64);
ffi_binary_op!(impl Rem, rem, decDoubleRemainder for d64);

ffi_unary_assign_op!(impl AddAssign, add_assign, decDoubleAdd for d64);
ffi_unary_assign_op!(impl SubAssign, sub_assign, decDoubleSubtract for d64);
ffi_unary_assign_op!(impl MulAssign, mul_assign, decDoubleMultiply for d64);
ffi_unary_assign_op!(impl DivAssign, div_assign, decDoubleDivide for d64);
ffi_unary_assign_op!(impl BitAndAssign, bitand_assign, decDoubleAnd for d64);
ffi_unary_assign_op!(impl BitOrAssign, bitor_assign, decDoubleOr for d64);
ffi_unary_assign_op!(impl BitXorAssign, bitxor_assign, decDoubleXor for d64);
ffi_unary_assign_op!(impl RemAssign, rem_assign, decDoubleRemainder for d64);

ffi_unary_op!(impl Neg, neg, decDoubleMinus for d64);
ffi_unary_op!(
/// The operand must be zero or positive, an integer (finite with zero exponent) and comprise
/// only zeros and/or ones; if not, INVALID_OPERATION is set.
    impl Not, not, decDoubleInvert for d64);

/// The result is `self` with the digits of the coefficient shifted to the left without adjusting
/// the exponent or the sign of `self`. Any digits ‘shifted in’ from the right will be 0. `amount`
/// is the count of positions to shift and must be a in the range –34 through +34. NaNs are
/// propagated as usual. If `self` is infinite the result is Infinity of the same sign. No status
/// is set unless `amount` is invalid or `self` is an sNaN.
impl Shl<usize> for d64 {
    type Output = d64;

    fn shl(mut self, amount: usize) -> d64 {
        let shift = d64::from(amount as u32);
        d64::with_context(|ctx| unsafe { *decDoubleShift(&mut self, &self, &shift, ctx) })
    }
}

impl<'a> Shl<usize> for &'a d64 {
    type Output = d64;

    fn shl(self, amount: usize) -> d64 {
        let shift = d64::from(amount as u32);
        d64::with_context(|ctx| {
            unsafe {
                let mut res: d64 = uninitialized();
                *decDoubleShift(&mut res, self, &shift, ctx)
            }
        })
    }
}

impl ShlAssign<usize> for d64 {
    fn shl_assign(&mut self, amount: usize) {
        let shift = d64::from(amount as u32);
        d64::with_context(|ctx| {
            unsafe {
                decDoubleShift(self, self, &shift, ctx);
            }
        })
    }
}

/// The result is `self` with the digits of the coefficient shifted to the right without adjusting
/// the exponent or the sign of `self`. Any digits ‘shifted in’ from the left will be 0. `amount`
/// is the count of positions to shift and must be a in the range –34 through +34. NaNs are
/// propagated as usual. If `self` is infinite the result is Infinity of the same sign. No status
/// is set unless `amount` is invalid or `self` is an sNaN.
impl Shr<usize> for d64 {
    type Output = d64;

    fn shr(mut self, amount: usize) -> d64 {
        let shift = -d64::from(amount as u32);
        d64::with_context(|ctx| unsafe { *decDoubleShift(&mut self, &self, &shift, ctx) })
    }
}

impl<'a> Shr<usize> for &'a d64 {
    type Output = d64;

    fn shr(self, amount: usize) -> d64 {
        let shift = -d64::from(amount as u32);
        d64::with_context(|ctx| {
            unsafe {
                let mut res: d64 = uninitialized();
                *decDoubleShift(&mut res, self, &shift, ctx)
            }
        })
    }
}

impl ShrAssign<usize> for d64 {
    fn shr_assign(&mut self, amount: usize) {
        let shift = -d64::from(amount as u32);
        d64::with_context(|ctx| {
            unsafe {
                decDoubleShift(self, self, &shift, ctx);
            }
        })
    }
}

impl<T> Sum<T> for d64 where T: Borrow<d64> {
    fn sum<I: IntoIterator<Item = T>>(iter: I) -> d64 {
        iter.into_iter()
            .fold(d64::zero(), |acc, val|
                acc + val.borrow())
    }
}

impl d64 {
    fn default_context() -> Context {
        unsafe {
            let mut res: Context = uninitialized();
            *decContextDefault(&mut res, 64)
        }
    }

    fn with_context<F, R>(f: F) -> R
        where F: FnOnce(&mut Context) -> R
    {
        CTX.with(|ctx| f(&mut ctx.borrow_mut()))
    }

    /// Creates a d64 from raw bytes. Endianess is host dependent.
    pub unsafe fn from_raw_bytes(bytes: [u8; 8]) -> d64 {
        d64 { bytes: bytes }
    }

    /// Returns raw bytes for this d64. Endianess is host dependent.
    pub fn to_raw_bytes(&self) -> [u8; 8] {
        self.bytes
    }

    /// Returns the thread local status.
    pub fn get_status() -> Status {
        d64::with_context(|ctx| Status::from_bits_truncate(ctx.status))
    }

    /// Sets the thread local status.
    pub fn set_status(status: Status) {
        d64::with_context(|ctx| ctx.status = status.bits());
    }

    /// Reads the hex binary representation from a string. This is the reverse of formatting with
    /// {:x}.
    pub fn from_hex(s: &str) -> d64 {
        if s.len() != 32 {
            Self::from_str("qNaN").unwrap()
        } else {
            unsafe {
                let mut res: d64 = uninitialized();
                for (i, octet) in s.as_bytes().chunks(2).rev().enumerate() {
                    res.bytes[i] = match u8::from_str_radix(from_utf8_unchecked(octet), 8) {
                        Ok(val) => val,
                        Err(..) => return Self::from_str("qNaN").unwrap(),
                    };
                }
                res
            }
        }
    }

    // Utilities and conversions, extractors, etc.

    /// Returns the d64 representing +0.
    pub fn zero() -> d64 {
        unsafe {
            let mut res = uninitialized();
            *decDoubleZero(&mut res)
        }
    }

    /// Returns the d64 representing +Infinity.
    pub fn infinity() -> d64 {
        d64!(Infinity)
    }

    /// Returns the d64 representing -Infinity.
    pub fn neg_infinity() -> d64 {
        d64!(-Infinity)
    }

    // Computational.

    /// Returns the absolute value of `self`.
    pub fn abs(mut self) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleAbs(&mut self, &self, ctx) })
    }

    /// Calculates the fused multiply-add `self` × `a` + `b` and returns the result. The multiply
    /// is carried out first and is exact, so this operation has only the one, final, rounding.
    pub fn mul_add<O: AsRef<d64>>(mut self, a: O, b: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            *decDoubleFMA(&mut self, &self, a.as_ref(), b.as_ref(), ctx)
        })
    }

    /// Returns the adjusted exponent of `self`, according to IEEE 754 rules. That is, the exponent
    /// returned is calculated as if the decimal point followed the first significant digit (so,
    /// for example, if `self` were 123 then the result would be 2). If `self` is infinite, the
    /// result is +Infinity. If `self` is a zero, the result is –Infinity, and the
    /// `DIVISION_BY_ZERO` flag is set. If `self` is less than zero, the absolute value of `self`
    /// is used. If `self` is 1, the result is 0. NaNs are handled (propagated) as for arithmetic
    /// operations.
    pub fn logb(mut self) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleLogB(&mut self, &self, ctx) })
    }

    /// If both `self` and `other` are numeric (not NaNs) this returns the larger of the two
    /// (compared using total ordering, to give a well-defined result). If either (but not both of)
    /// is a quiet NaN then the other argument is the result; otherwise NaNs are handled as for
    /// arithmetic operations.
    pub fn max<O: AsRef<d64>>(mut self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleMax(&mut self, &self, other.as_ref(), ctx) })
    }

    /// If both `self` and `other`  are numeric (not NaNs) this returns the smaller of the two
    /// (compared using total ordering, to give a well-defined result). If either (but not both of)
    /// is a quiet NaN then the other argument is the result; otherwise NaNs are handled as for
    /// arithmetic operations.
    pub fn min<O: AsRef<d64>>(mut self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleMin(&mut self, &self, other.as_ref(), ctx) })
    }

    /// Returns the ‘next’ d64 to `self` in the direction of +Infinity according to IEEE 754 rules
    /// for nextUp. The only status possible is `INVALID_OPERATION` (from an sNaN).
    pub fn next(mut self) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleNextPlus(&mut self, &self, ctx) })
    }

    /// Returns the ‘next’ d64 to `self` in the direction of –Infinity according to IEEE 754 rules
    /// for nextDown. The only status possible is `INVALID_OPERATION` (from an sNaN).
    pub fn previous(mut self) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleNextMinus(&mut self, &self, ctx) })
    }

    /// The number is set to the result of raising `self` to the power of `exp`. Results will be
    /// exact when `exp` has an integral value and the result does not need to be rounded, and also
    /// will be exact in certain special cases, such as when `self` is a zero (see the arithmetic
    /// specification for details). Inexact results will always be full precision, and will almost
    /// always be correctly rounded, but may be up to 1 _ulp_ (unit in last place) in error in rare
    /// cases. This is a mathematical function; the 10<sup>6</sup> restrictions on precision and
    /// range apply as described above, except that the normal range of values is allowed if `exp`
    /// has an integral value in the range –1999999997 through +999999999.
    pub fn pow<O: AsRef<d64>>(mut self, exp: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut num_self: decNumber = uninitialized();
            let mut num_exp: decNumber = uninitialized();
            decimal64ToNumber(&self, &mut num_self);
            decimal64ToNumber(exp.as_ref(), &mut num_exp);
            decNumberPower(&mut num_self, &num_self, &num_exp, ctx);
            *decimal64FromNumber(&mut self, &num_self, ctx)
        })
    }

    /// The number is set to _e_ raised to the power of `exp`. Finite results will always be full
    /// precision and inexact, except when `exp` is a zero or –Infinity (giving 1 or 0
    /// respectively). Inexact results will almost always be correctly rounded, but may be up to 1
    /// ulp (unit in last place) in error in rare cases. This is a mathematical function; the
    /// 10<sup>6</sup> restrictions on precision and range apply as described above.
    pub fn exp<O: AsRef<d64>>(mut self, exp: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut num_self: decNumber = uninitialized();
            let mut num_exp: decNumber = uninitialized();
            decimal64ToNumber(&self, &mut num_self);
            decimal64ToNumber(exp.as_ref(), &mut num_exp);
            decNumberExp(&mut num_self, &num_self, &num_exp, ctx);
            *decimal64FromNumber(&mut self, &num_self, ctx)
        })
    }

    /// The number is set to the natural logarithm (logarithm in base e) of `self`. `self` must be
    /// positive or a zero. Finite results will always be full precision and inexact, except when
    /// `self` is equal to 1, which gives an exact result of 0. Inexact results will almost always
    /// be correctly rounded, but may be up to 1 ulp (unit in last place) in error in rare cases.
    /// This is a mathematical function; the 10<sup>6</sup> restrictions on precision and range
    /// apply as described above.
    pub fn ln(mut self) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut num_self: decNumber = uninitialized();
            decimal64ToNumber(&self, &mut num_self);
            decNumberLn(&mut num_self, &num_self, ctx);
            *decimal64FromNumber(&mut self, &num_self, ctx)
        })
    }

    /// The number is set to the logarithm in base ten of `self`. `self` must be positive or a
    /// zero. Finite results will always be full precision and inexact, except when `self` is equal
    /// to an integral power of ten, in which case the result is the exact integer. Inexact results
    /// will almost always be correctly rounded, but may be up to 1 ulp (unit in last place) in
    /// error in rare cases. This is a mathematical function; the 10<sup>6</sup> restrictions on
    /// precision and range apply as described above.
    pub fn log10(mut self) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut num_self: decNumber = uninitialized();
            decimal64ToNumber(&self, &mut num_self);
            decNumberLog10(&mut num_self, &num_self, ctx);
            *decimal64FromNumber(&mut self, &num_self, ctx)
        })
    }

    /// Returns the ‘next’ d64 to `self` in the direction of `other` according to proposed IEEE
    /// 754  rules for nextAfter.  If `self` == `other` the result is `self`. If either operand is
    /// a NaN the result is as for arithmetic operations. Otherwise (the operands are numeric and
    /// different) the result of adding (or subtracting) an infinitesimal positive amount to `self`
    /// and rounding towards +Infinity (or –Infinity) is returned, depending on whether `other` is
    /// larger  (or smaller) than `self`. The addition will set flags, except that if the result is
    /// normal  (finite, non-zero, and not subnormal) no flags are set.
    pub fn towards<O: AsRef<d64>>(mut self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            *decDoubleNextToward(&mut self, &self, other.as_ref(), ctx)
        })
    }

    /// Returns `self` set to have the same quantum as `other`, if possible (that is, numerically
    /// the same value but rounded or padded if necessary to have the same exponent as `other`, for
    /// example to round a monetary quantity to cents).
    pub fn quantize<O: AsRef<d64>>(mut self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleQuantize(&mut self, &self, other.as_ref(), ctx) })
    }

    /// Returns a copy of `self` with its coefficient reduced to its shortest possible form without
    /// changing the value of the result. This removes all possible trailing zeros from the
    /// coefficient (some may remain when the number is very close to the most positive or most
    /// negative number). Infinities and NaNs are unchanged and no status is set unless `self` is
    /// an sNaN. If `self` is a zero the result exponent is 0.
    pub fn reduce(mut self) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleReduce(&mut self, &self, ctx) })
    }

    /// The result is a copy of `self` with the digits of the coefficient rotated to the left (if
    /// `amount` is positive) or to the right (if `amount` is negative) without adjusting the
    /// exponent or the sign of `self`. `amount` is the count of positions to rotate and must be a
    /// finite integer (with exponent=0) in the range -34 through +34. NaNs are propagated as
    /// usual. If `self` is infinite the result is Infinity of the same sign. No status is set
    /// unless `amount` is invalid or an operand is an sNaN.
    pub fn rotate<O: AsRef<d64>>(mut self, amount: O) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleRotate(&mut self, &self, amount.as_ref(), ctx) })
    }

    /// This calculates `self` × 10<sup>`other`</sup> and returns the result. `other` must be an
    /// integer (finite with exponent=0) in the range ±2 × (34 + 6144), typically resulting from
    /// `logb`. Underflow and overflow might occur. NaNs propagate as usual.
    pub fn scaleb<O: AsRef<d64>>(mut self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe { *decDoubleScaleB(&mut self, &self, other.as_ref(), ctx) })
    }

    // Comparisons.

    /// Compares `self` and `other` numerically and returns the result. The result may be –1, 0, 1,
    /// or NaN (unordered); –1 indicates that `self` is less than `other`, 0 indicates that they
    /// are numerically equal, and 1 indicates that `self` is greater than `other`. NaN is returned
    /// only if `self` or `other` is a NaN.
    pub fn compare<O: AsRef<d64>>(&self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut res: d64 = uninitialized();
            *decDoubleCompare(&mut res, self, other.as_ref(), ctx)
        })
    }

    /// Compares `self` and `other` using the IEEE 754 total ordering (which takes into account the
    /// exponent) and returns the result. No status is set (a signaling NaN is ordered between
    /// Infinity and NaN). The result will be –1, 0, or 1.
    pub fn compare_total<O: AsRef<d64>>(&self, other: O) -> d64 {
        d64::with_context(|ctx| unsafe {
            let mut res: d64 = uninitialized();
            *decDoubleCompareTotal(&mut res, self, other.as_ref(), ctx)
        })
    }

    // Copies.

    /// Returns `self` ensuring that the encoding is canonical.
    pub fn canonical(mut self) -> d64 {
        unsafe { *decDoubleCanonical(&mut self, &self) }
    }

    // Non-computational.

    /// Returns the class of `self`.
    pub fn class(&self) -> Class {
        unsafe { decDoubleClass(self) }
    }

    /// Same as `class()` but returns `std::num::FpCategory`.
    pub fn classify(&self) -> FpCategory {
        use std::num::FpCategory::*;
        use super::Class::*;

        match self.class() {
            Qnan | Snan => Nan,
            PosInf | NegInf => Infinite,
            PosZero | NegZero => Zero,
            PosNormal | NegNormal => Normal,
            PosSubnormal | NegSubnormal => Subnormal,
        }
    }

    /// Returns the number of significant digits in `self`. If `self` is a zero or is infinite, 1
    /// is returned. If `self` is a NaN then the number of digits in the payload is returned.
    pub fn digits(&self) -> u32 {
        unsafe { decDoubleDigits(self) }
    }

    /// Returns `true` if the encoding of `self` is canonical, or `false` otherwise.
    pub fn is_canonical(&self) -> bool {
        unsafe { decDoubleIsCanonical(self) != 0 }
    }

    /// Returns `true` if `self` is neither infinite nor a NaN, or `false` otherwise.
    pub fn is_finite(&self) -> bool {
        unsafe { decDoubleIsFinite(self) != 0 }
    }

    /// Returns `true` if `self` is finite and its exponent is zero, or `false` otherwise.
    pub fn is_integer(&self) -> bool {
        unsafe { decDoubleIsInteger(self) != 0 }
    }

    /// Returns `true` if `self`  is a valid argument for logical operations (that is, `self` is
    /// zero or positive, an integer (finite with a zero exponent) and comprises only zeros and/or
    /// ones), or `false` otherwise.
    pub fn is_logical(&self) -> bool {
        unsafe { decDoubleIsLogical(self) != 0 }
    }

    /// Returns `true` if the encoding of `self` is an infinity, or `false` otherwise.
    pub fn is_infinite(&self) -> bool {
        unsafe { decDoubleIsInfinite(self) != 0 }
    }

    /// Returns `true` if `self` is a NaN (quiet or signaling), or `false` otherwise.
    pub fn is_nan(&self) -> bool {
        unsafe { decDoubleIsNaN(self) != 0 }
    }

    /// Returns `true` if `self` is less than zero and not a NaN, or `false` otherwise.
    pub fn is_negative(&self) -> bool {
        unsafe { decDoubleIsNegative(self) != 0 }
    }

    /// Returns `true` if `self` is a normal number (that is, is finite, non-zero, and not
    /// subnormal), or `false` otherwise.
    pub fn is_normal(&self) -> bool {
        unsafe { decDoubleIsNormal(self) != 0 }
    }

    /// Returns `true` if `self` is greater than zero and not a NaN, or `false` otherwise.
    pub fn is_positive(&self) -> bool {
        unsafe { decDoubleIsPositive(self) != 0 }
    }

    /// Returns `true` if `self` is a signaling NaN, or `false` otherwise.
    pub fn is_signaling(&self) -> bool {
        unsafe { decDoubleIsSignaling(self) != 0 }
    }

    /// Returns `true` if `self` has a minus sign, or `false` otherwise. Note that zeros and NaNs
    /// may have a minus sign.
    pub fn is_signed(&self) -> bool {
        unsafe { decDoubleIsSigned(self) != 0 }
    }

    /// Returns `true` if `self` is subnormal (that is, finite, non-zero, and with magnitude less
    /// than 10<sup>-6143</sup>), or `false` otherwise.
    pub fn is_subnormal(&self) -> bool {
        unsafe { decDoubleIsSubnormal(self) != 0 }
    }

    /// Returns `true` if `self` is zero, or `false` otherwise.
    pub fn is_zero(&self) -> bool {
        unsafe { decDoubleIsZero(self) != 0 }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "ord_subset", feature = "rustc-serialize"))]
    use super::*;
    #[cfg(any(feature = "ord_subset", feature = "serde"))]
    use std::collections::BTreeMap;

    #[cfg(feature = "ord_subset")]
    use ord_subset;

    #[cfg(feature = "rustc-serialize")]
    use rustc_serialize::json;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    #[test]
    fn default() {
        assert_eq!(d64::zero(), d64::default());
        assert_eq!(d64::zero(), Default::default());
    }

    #[test]
    fn special() {
        assert!(d64::infinity().is_infinite());
        assert!(!d64::infinity().is_negative());

        assert!(d64::neg_infinity().is_infinite());
        assert!(d64::neg_infinity().is_negative());

        assert_eq!(d64::infinity() + d64!(1), d64::infinity());
    }

    #[cfg(feature = "ord_subset")]
    #[test]
    #[should_panic]
    fn test_ord_subset_nan() {
        ord_subset::OrdVar::new(d64!(NaN));
    }

    #[cfg(feature = "ord_subset")]
    #[test]
    #[should_panic]
    fn test_ord_subset_qnan() {
        ord_subset::OrdVar::new(d64!(qNaN));
    }

    #[cfg(feature = "ord_subset")]
    #[test]
    fn test_ord_subset_zero() {
        assert_eq!(*ord_subset::OrdVar::new(d64::zero()), d64::zero());
    }

    #[cfg(feature = "ord_subset")]
    #[test]
    fn test_into_for_btreemap() {
        let mut m = BTreeMap::<ord_subset::OrdVar<d64>, i64>::new();
        m.insert(d64!(1.1).into(), 1);
        assert_eq!(m[&d64!(1.1).into()], 1);
    }

    #[cfg(feature = "rustc-serialize")]
    #[test]
    fn test_rustc_serialize() {
        #[derive(RustcDecodable, RustcEncodable, PartialEq, Debug)]
        struct Test {
            price: d64,
        };
        let a = Test { price: d64!(12.3456) };
        assert_eq!(json::encode(&a).unwrap(), "{\"price\":\"12.3456\"}");
        let b = json::decode("{\"price\":\"12.3456\"}").unwrap();
        assert_eq!(a, b);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let mut a = BTreeMap::new();
        a.insert("price".to_string(), d64!(432.232));
        a.insert("amt".to_string(), d64!(9.9));
        assert_eq!(&to_string(&a).unwrap(),
            "{\"amt\":\"9.9\",\"price\":\"432.232\"}");
        let b = from_str("{\"price\":\"432.232\",\"amt\":\"9.9\"}").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn unary_op() {
        assert_eq!(d64!(-1.1), -d64!(1.1));
        assert_eq!(d64!(-1.1), -&d64!(1.1));
    }

    #[test]
    fn binary_op() {
        assert_eq!(d64!(3.33), d64!(1.11) + d64!(2.22));
        assert_eq!(d64!(3.33), &d64!(1.11) + d64!(2.22));
        assert_eq!(d64!(3.33), d64!(1.11) + &d64!(2.22));
        assert_eq!(d64!(3.33), &d64!(1.11) + &d64!(2.22));
        assert_eq!(d64!(5) << 2, d64!(500));
        assert_eq!(d64!(500) >> 1, d64!(50));
    }

    #[test]
    fn assign_op() {
        let mut x = d64!(1);
        x += d64!(2);
        assert_eq!(x, d64!(3));
        x *= d64!(3);
        assert_eq!(x, d64!(9));
        x -= d64!(1);
        assert_eq!(x, d64!(8));
        x /= d64!(16);
        assert_eq!(x, d64!(0.5));
        x <<= 2;
        assert_eq!(x, d64!(50));
        x >>= 1;
        assert_eq!(x, d64!(5));
    }

    #[test]
    fn as_ref_operand() {
        assert_eq!(d64!(1.1), d64!(1.1).min(d64!(2.2)));
        assert_eq!(d64!(1.1), d64!(1.1).min(&d64!(2.2)));
    }

    #[test]
    fn from_i64() {
        assert_eq!(d64::from_str(&::std::i64::MAX.to_string()).unwrap(),
                   d64::from(::std::i64::MAX));
        assert_eq!(d64::from(0i32), d64::from(0i64));
        assert_eq!(d64::from_str(&(::std::i64::MIN).to_string()).unwrap(),
                   d64::from(::std::i64::MIN));
    }

    #[test]
    fn from_u64() {
        assert_eq!(d64::from_str(&::std::u64::MAX.to_string()).unwrap(),
                   d64::from(::std::u64::MAX));
        assert_eq!(d64::from(0i32), d64::from(0u64));
        assert_eq!(d64::from_str(&(::std::u64::MIN).to_string()).unwrap(),
                   d64::from(::std::u64::MIN));
    }

    #[test]
    fn test_sum() {
        let decimals = vec![d64!(1), d64!(2), d64!(3), d64!(4)];

        assert_eq!(d64!(10), decimals.iter().sum());

        assert_eq!(d64!(10), decimals.into_iter().sum());
    }
}
