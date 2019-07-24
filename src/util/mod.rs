//! Utilites contains useful functions that are unrelated to any of the main game modules.

extern crate num;
use self::num::Num;

/// Modulus function.
/// In Rust the `%` operator is the remainder, not modulus.
pub fn modulus<T: Num + PartialOrd + Copy>(a: T, b: T) -> T {
    ((a % b) + b) % b
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
