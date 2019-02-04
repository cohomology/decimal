use super::Class;
use super::Status; 
use super::Rounding;  

use context::*;
use libc::{c_char, uint8_t, int32_t, uint32_t}; 

#[repr(C)]
#[derive(Clone, Copy)]
pub struct decNumber {
    digits: i32,
    exponent: i32,
    bits: u8,
    // DECPUN = 3 because this is the fastest for conversion between decNumber and
    // decDouble/decQuad
    // DECNUMDIGITS = 34 because we use decQuad maximally
    // 12 = ((DECNUMDIGITS+DECDPUN-1)/DECDPUN)
    lsu: [u16; 12],
} 

#[repr(C)]
#[derive(Clone, Copy)]
/// A 64-bit decimal floating point type.
pub struct d64 {
    pub(crate) bytes: [uint8_t; 8],
} 

#[repr(C)]
#[derive(Clone, Copy)]
/// A 128-bit decimal floating point type.
pub struct d128 {
    bytes: [uint8_t; 16],
} 

extern "C" {
    // Context.
    pub fn decContextDefault(ctx: *mut Context, kind: uint32_t) -> *mut Context;
    // Utilities and conversions, extractors, etc.
    pub fn decDoubleFromInt32(res: *mut d64, src: int32_t) -> *mut d64;
    pub fn decDoubleFromString(res: *mut d64, s: *const c_char, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleFromUInt32(res: *mut d64, src: uint32_t) -> *mut d64;
    pub fn decDoubleToString(src: *const d64, s: *mut c_char) -> *mut c_char;
    pub fn decDoubleToInt32(src: *const d64, ctx: *mut Context, round: Rounding) -> int32_t;
    pub fn decDoubleToUInt32(src: *const d64, ctx: *mut Context, round: Rounding) -> uint32_t;
    pub fn decDoubleToEngString(res: *const d64, s: *mut c_char) -> *mut c_char;
    pub fn decDoubleZero(res: *mut d64) -> *mut d64;
    // Computational.
    pub fn decDoubleAbs(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleAdd(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleAnd(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleDivide(res: *mut d64,
                     a: *const d64,
                     b: *const d64,
                     ctx: *mut Context)
                     -> *mut d64;
    pub fn decDoubleFMA(res: *mut d64,
                  a: *const d64,
                  b: *const d64,
                  c: *const d64,
                  ctx: *mut Context)
                  -> *mut d64;
    pub fn decDoubleInvert(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleLogB(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleMax(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleMin(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleMinus(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleMultiply(res: *mut d64,
                       a: *const d64,
                       b: *const d64,
                       ctx: *mut Context)
                       -> *mut d64;
    pub fn decDoubleNextMinus(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleNextPlus(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleNextToward(res: *mut d64,
                         src: *const d64,
                         other: *const d64,
                         ctx: *mut Context)
                         -> *mut d64;
    pub fn decDoubleOr(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleQuantize(res: *mut d64,
                       a: *const d64,
                       b: *const d64,
                       ctx: *mut Context)
                       -> *mut d64;
    pub fn decDoubleReduce(res: *mut d64, src: *const d64, ctx: *mut Context) -> *mut d64;
    pub fn decDoubleRemainder(res: *mut d64,
                        a: *const d64,
                        b: *const d64,
                        ctx: *mut Context)
                        -> *mut d64;
    pub fn decDoubleRotate(res: *mut d64,
                     a: *const d64,
                     b: *const d64,
                     ctx: *mut Context)
                     -> *mut d64;
    pub fn decDoubleScaleB(res: *mut d64,
                     a: *const d64,
                     b: *const d64,
                     ctx: *mut Context)
                     -> *mut d64;
    pub fn decDoubleShift(res: *mut d64,
                    a: *const d64,
                    b: *const d64,
                    ctx: *mut Context)
                    -> *mut d64;
    pub fn decDoubleSubtract(res: *mut d64,
                       a: *const d64,
                       b: *const d64,
                       ctx: *mut Context)
                       -> *mut d64;
    pub fn decDoubleXor(res: *mut d64, a: *const d64, b: *const d64, ctx: *mut Context) -> *mut d64;
    // Comparisons.
    pub fn decDoubleCompare(res: *mut d64,
                      a: *const d64,
                      b: *const d64,
                      ctx: *mut Context)
                      -> *mut d64;
    pub fn decDoubleCompareTotal(res: *mut d64,
                           a: *const d64,
                           b: *const d64,
                           ctx: *mut Context)
                           -> *mut d64;
    // Copies.
    pub fn decDoubleCanonical(res: *mut d64, src: *const d64) -> *mut d64;
    // Non-computational.
    pub fn decDoubleClass(src: *const d64) -> Class;
    pub fn decDoubleDigits(src: *const d64) -> uint32_t;
    pub fn decDoubleIsCanonical(src: *const d64) -> uint32_t;
    pub fn decDoubleIsFinite(src: *const d64) -> uint32_t;
    pub fn decDoubleIsInteger(src: *const d64) -> uint32_t;
    pub fn decDoubleIsLogical(src: *const d64) -> uint32_t;
    pub fn decDoubleIsInfinite(src: *const d64) -> uint32_t;
    pub fn decDoubleIsNaN(src: *const d64) -> uint32_t;
    pub fn decDoubleIsNegative(src: *const d64) -> uint32_t;
    pub fn decDoubleIsNormal(src: *const d64) -> uint32_t;
    pub fn decDoubleIsPositive(src: *const d64) -> uint32_t;
    pub fn decDoubleIsSignaling(src: *const d64) -> uint32_t;
    pub fn decDoubleIsSigned(src: *const d64) -> uint32_t;
    pub fn decDoubleIsSubnormal(src: *const d64) -> uint32_t;
    pub fn decDoubleIsZero(src: *const d64) -> uint32_t;
    pub fn decimal64FromNumber(res: *mut d64, src: *const decNumber, ctx: *mut Context) -> *mut d64;
    pub fn decimal64ToNumber(src: *const d64, res: *mut decNumber) -> *mut decNumber;
    pub fn decimal128FromNumber(res: *mut d128, src: *const decNumber, ctx: *mut Context) -> *mut d128;
    pub fn decimal128ToNumber(src: *const d128, res: *mut decNumber) -> *mut decNumber; 
    // decQuad
    pub fn decQuadFromBCD(res: *mut d128, exp: i32, bcd: *const u8, sign: i32) -> *mut d128;
    pub fn decQuadFromInt32(res: *mut d128, src: int32_t) -> *mut d128;
    pub fn decQuadFromString(res: *mut d128, s: *const c_char, ctx: *mut Context) -> *mut d128;
    pub fn decQuadFromUInt32(res: *mut d128, src: uint32_t) -> *mut d128;
    pub fn decQuadToString(src: *const d128, s: *mut c_char) -> *mut c_char;
    pub fn decQuadToInt32(src: *const d128, ctx: *mut Context, round: Rounding) -> int32_t;
    pub fn decQuadToUInt32(src: *const d128, ctx: *mut Context, round: Rounding) -> uint32_t;
    pub fn decQuadToEngString(res: *const d128, s: *mut c_char) -> *mut c_char;
    pub fn decQuadZero(res: *mut d128) -> *mut d128;
    // Computational.
    pub fn decQuadAbs(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadAdd(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadAnd(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadDivide(res: *mut d128,
                     a: *const d128,
                     b: *const d128,
                     ctx: *mut Context)
                     -> *mut d128;
    pub fn decQuadFMA(res: *mut d128,
                  a: *const d128,
                  b: *const d128,
                  c: *const d128,
                  ctx: *mut Context)
                  -> *mut d128;
    pub fn decQuadInvert(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadLogB(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadMax(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadMin(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadMinus(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadMultiply(res: *mut d128,
                       a: *const d128,
                       b: *const d128,
                       ctx: *mut Context)
                       -> *mut d128;
    pub fn decQuadNextMinus(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadNextPlus(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadNextToward(res: *mut d128,
                         src: *const d128,
                         other: *const d128,
                         ctx: *mut Context)
                         -> *mut d128;
    pub fn decQuadOr(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadQuantize(res: *mut d128,
                       a: *const d128,
                       b: *const d128,
                       ctx: *mut Context)
                       -> *mut d128;
    pub fn decQuadReduce(res: *mut d128, src: *const d128, ctx: *mut Context) -> *mut d128;
    pub fn decQuadRemainder(res: *mut d128,
                        a: *const d128,
                        b: *const d128,
                        ctx: *mut Context)
                        -> *mut d128;
    pub fn decQuadRotate(res: *mut d128,
                     a: *const d128,
                     b: *const d128,
                     ctx: *mut Context)
                     -> *mut d128;
    pub fn decQuadScaleB(res: *mut d128,
                     a: *const d128,
                     b: *const d128,
                     ctx: *mut Context)
                     -> *mut d128;
    pub fn decQuadShift(res: *mut d128,
                    a: *const d128,
                    b: *const d128,
                    ctx: *mut Context)
                    -> *mut d128;
    pub fn decQuadSubtract(res: *mut d128,
                       a: *const d128,
                       b: *const d128,
                       ctx: *mut Context)
                       -> *mut d128;
    pub fn decQuadXor(res: *mut d128, a: *const d128, b: *const d128, ctx: *mut Context) -> *mut d128;
    // Comparisons.
    pub fn decQuadCompare(res: *mut d128,
                      a: *const d128,
                      b: *const d128,
                      ctx: *mut Context)
                      -> *mut d128;
    pub fn decQuadCompareTotal(res: *mut d128,
                           a: *const d128,
                           b: *const d128,
                           ctx: *mut Context)
                           -> *mut d128;
    // Copies.
    pub fn decQuadCanonical(res: *mut d128, src: *const d128) -> *mut d128;
    // Non-computational.
    pub fn decQuadClass(src: *const d128) -> Class;
    pub fn decQuadDigits(src: *const d128) -> uint32_t;
    pub fn decQuadIsCanonical(src: *const d128) -> uint32_t;
    pub fn decQuadIsFinite(src: *const d128) -> uint32_t;
    pub fn decQuadIsInteger(src: *const d128) -> uint32_t;
    pub fn decQuadIsLogical(src: *const d128) -> uint32_t;
    pub fn decQuadIsInfinite(src: *const d128) -> uint32_t;
    pub fn decQuadIsNaN(src: *const d128) -> uint32_t;
    pub fn decQuadIsNegative(src: *const d128) -> uint32_t;
    pub fn decQuadIsNormal(src: *const d128) -> uint32_t;
    pub fn decQuadIsPositive(src: *const d128) -> uint32_t;
    pub fn decQuadIsSignaling(src: *const d128) -> uint32_t;
    pub fn decQuadIsSigned(src: *const d128) -> uint32_t;
    pub fn decQuadIsSubnormal(src: *const d128) -> uint32_t;
    pub fn decQuadIsZero(src: *const d128) -> uint32_t; 

    // decNumber stuff. 
    pub fn decNumberPower(res: *mut decNumber,
                          lhs: *const decNumber,
                          rhs: *const decNumber,
                          ctx: *mut Context)
                          -> *mut decNumber;
    pub fn decNumberExp(res: *mut decNumber,
                        lhs: *const decNumber,
                        rhs: *const decNumber,
                        ctx: *mut Context)
                        -> *mut decNumber;
    pub fn decNumberLn(res: *mut decNumber,
                       rhs: *const decNumber,
                       ctx: *mut Context)
                       -> *mut decNumber;
    pub fn decNumberLog10(res: *mut decNumber,
                      rhs: *const decNumber,
                      ctx: *mut Context)
                      -> *mut decNumber; 
} 
