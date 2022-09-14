

/// Easily performs a `return Err(format!("..."))` for maximum laziness
macro_rules! error {
    ($($args:tt)*) => {{
        return Err(format!($($args)*).into());
    }}
}
pub(crate) use error;

/// Calculates the number of bits a rust type requires
pub const fn bit_size<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

/// Calculates the minimum number of bits required to represent a number n
pub fn min_bits(n: u32) -> u32 {
    bit_size::<u32>() as u32 - n.leading_zeros() - 1
}

pub fn is_pow2(n: u32) -> bool {
    n.count_ones() == 1
}

