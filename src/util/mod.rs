//! Utilites contains useful functions that are unrelated to any of the main game modules.

pub mod game_rng;

extern crate num;
use self::num::Num;

/// Modulus function.
/// In Rust the `%` operator is the remainder, not modulus.
pub fn modulus<T: Num + PartialOrd + Copy>(a: T, b: T) -> T {
    ((a % b) + b) % b
}

pub fn generate_gray_code(n: usize) -> Vec<u32> {
    (0..n as u32).map(|x| x ^ (x >> 1)).collect::<Vec<u32>>()
}

// /// Helper function to convert from a binary number to *reflected binary* gray code.
// fn binary_to_gray(x: u32) -> u32 {
//     x ^ (x >> 1)
// }

/// A more efficient version for Gray codes 32 bits or fewer
/// through the use of SWAR (SIMD within a register) techniques.
/// It implements a parallel prefix XOR function.  The assignment
/// statements can be in any order.
///
/// This function can be adapted for longer Gray codes by adding steps.
/// A 4-bit variant changes a binary number (abcd)2 to (abcd)2 ^ (00ab)2,
/// then to (abcd)2 ^ (00ab)2 ^ (0abc)2 ^ (000a)2.
/// Taken from Wikipedia.
pub fn gray_to_binary32(mut x: u32) -> u32 {
    x = x ^ (x >> 16);
    x = x ^ (x >> 8);
    x = x ^ (x >> 4);
    x = x ^ (x >> 2);
    x = x ^ (x >> 1);
    x
}

// use std::cmp;
// /// Mutably borrow two *separate* elements from the given slice.
// /// Panics when the indices are equal or out of bounds.
// pub fn mut_two<T>(items: &mut [T], first_index: usize, second_index: usize) -> (&mut T, &mut T) {
//     assert!(first_index != second_index);
//     let split_at_index = cmp::max(first_index, second_index);
//     let (first_slice, second_slice) = items.split_at_mut(split_at_index);
//     if first_index < second_index {
//         (&mut first_slice[first_index], &mut second_slice[0])
//     } else {
//         (&mut second_slice[0], &mut first_slice[second_index])
//     }
// }
