

/// Easily performs a `return Err(format!("..."))` for maximum laziness
macro_rules! error {
    ($($args:tt)*) => {{
        return Err(format!($($args)*).into());
    }}
}
pub(crate) use error;

/// Helper functions for bit operations
pub mod bit_ops {
    /// Calculates the number of bits a rust type requires
    pub const fn bit_size<T>() -> usize {
        std::mem::size_of::<T>() * 8
    }

    /// Calculates the minimum number of bits required to represent a number n
    pub fn min_bits(n: u32) -> u32 {
        bit_size::<u32>() as u32 - n.leading_zeros() - 1
    }

    /// Checks if a number is a power of 2
    pub fn is_pow2(n: u32) -> bool {
        n.count_ones() == 1
    }

    pub trait SplitBits
    where Self: Sized,
    {
        fn split_bits(&self, n: usize) -> (Self, Self);
    }

    impl SplitBits for u32 {
        fn split_bits(&self, n: usize) -> (Self, Self) {
            let mask = (1 << n) - 1;
            let right = self & mask;
            let left = ( self &! mask) >> n;
            (left, right)
        }
    }

    /// Splits a u32 into two u32s at the bit index
    pub fn split_bits(x: u32, n: usize) -> (u32, u32) {
        let mask = (1 << n) - 1;
        let right = x & mask;
        let left = ( x &! mask) >> n;
        (left, right)
    }

    #[cfg(test)]
    mod test {
        use super::split_bits;

        #[test]
        fn does_it_even_work() {
            let (x, y) = split_bits(119, 3);
            assert_eq!((x,y), (14,7));
        }

        #[test]
        fn try_harder() {
            let (x, y) = split_bits(2273197461, 13);
            assert_eq!((x,y), (277489,7573))
        }
    }
}