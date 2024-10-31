use num_bigint::BigUint;
use num_bigint::{BigInt, Sign};
use std::iter::repeat;

pub fn to_bigint(bits: &[bool]) -> BigInt {
    if bits.last() != Some(&true) {
        let bits = bits
            .iter()
            .map(|x| if *x { 1 } else { 0 })
            .collect::<Vec<_>>();
        BigInt::from_radix_le(Sign::Plus, &bits, 2).unwrap()
    } else {
        let bits = bits
            .iter()
            .map(|x| if *x { 0 } else { 1 })
            .collect::<Vec<_>>();
        -(BigInt::from_radix_le(Sign::Plus, &bits, 2).unwrap() + 1_i32)
    }
}

pub fn from_bigint(bi: &BigInt, len: usize) -> Vec<bool> {
    if bi < &BigInt::ZERO {
        let bi = -bi - 1_i32;
        let bits = from_bigint(&bi, len);
        bits.iter().map(|x| !x).collect::<Vec<_>>()
    } else {
        (0..len as u64).map(|pos| bi.bit(pos)).collect()
    }
}

pub fn to_biguint(bits: &[bool]) -> BigUint {
    let bits = bits
        .iter()
        .map(|x| if *x { 1 } else { 0 })
        .collect::<Vec<_>>();
    BigUint::from_radix_le(&bits, 2).unwrap()
}

pub fn from_biguint(bi: &BigUint, len: usize) -> Vec<bool> {
    (0..len as u64).map(|pos| bi.bit(pos)).collect()
}

pub(crate) fn add_one(a: &[bool]) -> Vec<bool> {
    a.iter()
        .scan(true, |carry, b| {
            let sum = b ^ *carry;
            *carry &= b;
            Some(sum)
        })
        .collect()
}

pub(crate) fn full_add(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter()
        .zip(b.iter())
        .scan(false, |carry, (a, b)| {
            let sum = a ^ b ^ *carry;
            let new_carry = (a & b) | (a & *carry) | (b & *carry);
            *carry = new_carry;
            Some(sum)
        })
        .collect()
}

pub(crate) fn bit_not(a: &[bool]) -> Vec<bool> {
    a.iter().map(|b| !b).collect()
}

pub(crate) fn bit_neg(a: &[bool]) -> Vec<bool> {
    add_one(&bit_not(a))
}

pub(crate) fn full_sub(a: &[bool], b: &[bool]) -> Vec<bool> {
    full_add(a, &bit_neg(b))
}

pub(crate) fn bits_xor(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a ^ b).collect()
}

pub(crate) fn bits_and(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a & b).collect()
}

pub(crate) fn bits_or(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a | b).collect()
}

pub(crate) fn bits_shl(a: &[bool], b: i64) -> Vec<bool> {
    repeat(false)
        .take(b as usize)
        .chain(a.iter().copied())
        .take(a.len())
        .collect()
}

pub(crate) fn bits_shr(a: &[bool], b: i64) -> Vec<bool> {
    a.iter()
        .copied()
        .skip(b as usize)
        .chain(repeat(false).take(b as usize))
        .take(a.len())
        .collect()
}

pub(crate) fn bits_shr_signed(a: &[bool], b: i64) -> Vec<bool> {
    let sign = a.last().copied().unwrap_or(false);
    a.iter()
        .copied()
        .skip(b as usize)
        .chain(repeat(sign))
        .take(a.len())
        .collect()
}

pub fn move_nbits_to_msb(a: &[bool], n: usize) -> Vec<bool> {
    let (left, right) = a.split_at(n);
    [right, left].concat()
}

#[macro_export]
macro_rules! const_max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr), +) => (
        if $x > const_max!($($z), +) {
            $x
        } else {
            const_max!($($z), +)
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_test() {
        let a = vec![true, false, true];
        let b = [a.as_slice()].concat();
        assert_eq!(b, vec![true, false, true]);
    }

    #[test]
    fn test_const_max_macro() {
        assert_eq!(const_max!(1, 2, 3, 4, 5), 5);
        assert_eq!(const_max!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10), 10);
    }

    #[test]
    fn test_move_nbits_to_msb() {
        let a: Vec<bool> = (0..200).map(|_| rand::random()).collect();
        for n in 0..a.len() {
            let b = move_nbits_to_msb(&a, n);
            let c = a.iter().skip(n).chain(a.iter().take(n));
            assert!(c.eq(b.iter()));
        }
    }

    #[test]
    fn test_bigint_conversion() {
        let bits = vec![true, false, true, false]; // 5
        let bi = to_bigint(&bits);
        assert_eq!(bi, BigInt::from(5));
        let bits_regen = from_bigint(&bi, 4);
        assert_eq!(bits_regen, bits);
        let bits = vec![true, true, false, true]; // -5
        let bi = to_bigint(&bits);
        assert_eq!(bi, BigInt::from(-5));
        let bits_regen = from_bigint(&bi, 4);
        assert_eq!(bits_regen, bits);
    }

    #[test]
    fn test_bigint_extend_behavior() {
        let bits = vec![true, false, true, false]; // 5
        let bi = to_bigint(&bits);
        let bits_regen = from_bigint(&bi, 8);
        assert_eq!(
            bits_regen,
            vec![true, false, true, false, false, false, false, false]
        );
    }
}
