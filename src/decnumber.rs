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
