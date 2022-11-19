//! Utilites contains useful functions that are unrelated to any of the main game modules.

pub mod random;
pub mod timer;

pub use timer::Timer;

pub fn generate_gray_code(n: u8) -> Vec<u8> {
    let base: u8 = 2;
    let code_len: u8 = base.pow(u32::from(n));
    (0..code_len).map(|x| x ^ (x >> 1)).collect::<Vec<u8>>()
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
pub const fn _gray_to_binary32(mut x: u8) -> u8 {
    // x = x ^ (x >> 16);
    // x = x ^ (x >> 8);
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

/// Format text to fit within the given line width. Whenever the given width is reached or exceeded
/// a new line will be created. Returns a list of lines with a width <= `line_width`
///
/// # Arguments
///
/// * `text` - text to be formatted
/// * `line_width` - maximum length of a line of text.
pub fn text_to_width(text: &str, line_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current_line: Vec<&str> = Vec::new();
    let mut current_width = 0;
    for word in text.split(' ') {
        current_width += word.len() + 1;
        if current_width <= line_width as usize + 1 {
            current_line.push(word);
        } else {
            lines.push(current_line.join(" "));
            current_line.clear();
            current_line.push(word);
            current_width = word.len() + 1;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line.join(" "));
    }
    lines
}
