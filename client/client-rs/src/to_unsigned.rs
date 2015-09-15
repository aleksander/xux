use std::{i8, u8, i16, u16, i32, u32, i64, u64, isize, usize};

pub trait ToUnsigned<A> {
    fn to_unsigned (&self) -> A;
}

impl ToUnsigned<u8> for i8 {
    fn to_unsigned (&self) -> u8 {
        *self as u8 ^ (u8::MAX / 2 + 1)
    }
}

impl ToUnsigned<u16> for i16 {
    fn to_unsigned (&self) -> u16 {
        *self as u16 ^ (u16::MAX / 2 + 1)
    }
}

impl ToUnsigned<u32> for i32 {
    fn to_unsigned (&self) -> u32 {
        *self as u32 ^ (u32::MAX / 2 + 1)
    }
}

impl ToUnsigned<u64> for i64 {
    fn to_unsigned (&self) -> u64 {
        *self as u64 ^ (u64::MAX / 2 + 1)
    }
}

impl ToUnsigned<usize> for isize {
    fn to_unsigned (&self) -> usize {
        *self as usize ^ (usize::MAX / 2 + 1)
    }
}

/* TODO transmute this to tests
fn main () {
    for i in (i8::MIN as isize)..(i8::MAX as isize + 1) {
        let a = i as i8;
        let r1 = (a as isize + 128) as u8;
        let r2 = if a < 0 { (a + 127 + 1) as u8 } else { a as u8 + 128 };
        let r3 = if a < 0 { a as u8 & 127 } else { a as u8 | 128 };
        //let r4 = a as u8 ^ 128;
        let r4: u8 = a.to_unsigned();
        println!("{:5} {:08b} > {:4} {:4} {:4} {:4} {:08b}", a, a, r1, r2, r3, r4, r1);
        assert!(r1 == r2 && r2 == r3 && r3 == r4);
    }

    println!("{} > {}", i8::MIN, i8::MIN.to_unsigned());
    println!("{} > {}", i8::MAX, i8::MAX.to_unsigned());
    println!("");
    println!("{} > {}", i16::MIN, i16::MIN.to_unsigned());
    println!("{} > {}", i16::MAX, i16::MAX.to_unsigned());
    println!("");
    println!("{} > {}", i32::MIN, i32::MIN.to_unsigned());
    println!("{} > {}", i32::MAX, i32::MAX.to_unsigned());
    println!("");
    println!("{} > {}", i64::MIN, i64::MIN.to_unsigned());
    println!("{} > {}", i64::MAX, i64::MAX.to_unsigned());
    println!("");
    println!("{} > {}", isize::MIN, isize::MIN.to_unsigned());
    println!("{} > {}", isize::MAX, isize::MAX.to_unsigned());
}
*/
