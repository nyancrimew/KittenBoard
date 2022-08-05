// TODO: move constants to more reasonable locations in the future
// constants from defines.h

/// The max number of the keys in one keyboard layout
pub const MAX_KEY_COUNT_IN_A_KEYBOARD: usize = 64;

pub const KEYCODE_SPACE: char = ' ';
pub const KEYCODE_SINGLE_QUOTE: char = '\'';
pub const KEYCODE_HYPHEN_MINUS: char = '-';

pub const MAX_PERCENTILE: i32 = 100;

pub const MAX_VALUE_FOR_WEIGHTING: i32 = 10000000;

pub enum DoubleLetterLevel {
    None,
    DoubleLetter,
    Strong,
}
