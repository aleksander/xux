use std::{u8, u16, u32, u64, usize};

pub trait ShiftToUnsigned<A> {
    fn shift_to_unsigned(self) -> A;
}

impl ShiftToUnsigned<u8> for i8 {
    fn shift_to_unsigned(self) -> u8 {
        self as u8 ^ (u8::MAX / 2 + 1)
    }
}

impl ShiftToUnsigned<u16> for i16 {
    fn shift_to_unsigned(self) -> u16 {
        self as u16 ^ (u16::MAX / 2 + 1)
    }
}

impl ShiftToUnsigned<u32> for i32 {
    fn shift_to_unsigned(self) -> u32 {
        self as u32 ^ (u32::MAX / 2 + 1)
    }
}

impl ShiftToUnsigned<u64> for i64 {
    fn shift_to_unsigned(self) -> u64 {
        self as u64 ^ (u64::MAX / 2 + 1)
    }
}

impl ShiftToUnsigned<usize> for isize {
    fn shift_to_unsigned(self) -> usize {
        self as usize ^ (usize::MAX / 2 + 1)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::ops::Add;
    use std::num::One;

    trait MinMax {
        fn min() -> Self;
        fn max() -> Self;
    }

    impl MinMax for i8 {
        fn min() -> i8 {
            use std::i8;
            i8::MIN
        }
        fn max() -> i8 {
            use std::i8;
            i8::MAX
        }
    }

    impl MinMax for u8 {
        fn min() -> u8 {
            use std::u8;
            u8::MIN
        }
        fn max() -> u8 {
            use std::u8;
            u8::MAX
        }
    }

    impl MinMax for i16 {
        fn min() -> i16 {
            use std::i16;
            i16::MIN
        }
        fn max() -> i16 {
            use std::i16;
            i16::MAX
        }
    }

    impl MinMax for u16 {
        fn min() -> u16 {
            use std::u16;
            u16::MIN
        }
        fn max() -> u16 {
            use std::u16;
            u16::MAX
        }
    }

    impl MinMax for i32 {
        fn min() -> i32 {
            use std::i32;
            i32::MIN
        }
        fn max() -> i32 {
            use std::i32;
            i32::MAX
        }
    }

    impl MinMax for u32 {
        fn min() -> u32 {
            use std::u32;
            u32::MIN
        }
        fn max() -> u32 {
            use std::u32;
            u32::MAX
        }
    }

    impl MinMax for i64 {
        fn min() -> i64 {
            use std::i64;
            i64::MIN
        }
        fn max() -> i64 {
            use std::i64;
            i64::MAX
        }
    }

    impl MinMax for u64 {
        fn min() -> u64 {
            use std::u64;
            u64::MIN
        }
        fn max() -> u64 {
            use std::u64;
            u64::MAX
        }
    }

    impl MinMax for isize {
        fn min() -> isize {
            use std::isize;
            isize::MIN
        }
        fn max() -> isize {
            use std::isize;
            isize::MAX
        }
    }

    impl MinMax for usize {
        fn min() -> usize {
            use std::usize;
            usize::MIN
        }
        fn max() -> usize {
            use std::usize;
            usize::MAX
        }
    }

    fn shift_test<A: MinMax + ShiftToUnsigned<B> + PartialEq + Add<Output = A> + One + Copy, B: MinMax + PartialEq + Add<Output = B> + One>() {
        let mut a = A::min();
        let mut b = B::min();
        loop {
            let c = a.shift_to_unsigned();
            if b != c {
                panic!();
            }
            if a == A::max() {
                break;
            }
            a = a + A::one();
            b = b + B::one();
        }
    }

    #[test]
    fn shift_test_i8() {
        shift_test::<i8, u8>();
    }

    #[test]
    fn shift_test_i16() {
        shift_test::<i16, u16>();
    }

    #[test]
    // TODO #[cfg(profile = "release")]
    fn shift_test_i32() {
        shift_test::<i32, u32>();
    }

    #[test]
    // TODO #[cfg(profile = "release")]
    fn shift_test_i64() {
        shift_test::<i64, u64>();
    }

    #[test]
    // TODO #[cfg(profile = "release")]
    fn shift_test_isize() {
        shift_test::<isize, usize>();
    }
}
