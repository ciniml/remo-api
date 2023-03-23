// Parser options
// Copyright 2022-2023 Kenta Ida 
// SPDX-License-Identifier: MIT
//

use core::str::FromStr;

use heapless::String;
use crate::common_types::ModelNodeParseError;

pub struct ParserOptions {
    /// Truncate strings if the length is too long to hold.
    truncate_too_long_string: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            truncate_too_long_string: true,
        }
    }
}

/// Copy string as long as the storage can hold.
pub fn copy_string_possible<const N: usize>(s: &str) -> String<N> {
    let mut string = String::new();
    for c in s.chars() {
        if let Err(_) = string.push(c) {
            break;
        }
    }
    string
}

/// Copy string. Truncate extra characters which cannot be held by the storage if options.truncate_too_long_string is true.
pub fn copy_string_option<const N: usize>(s: &str, options: &ParserOptions) -> Result<String<N>, ModelNodeParseError> {
    if options.truncate_too_long_string {
        Ok(copy_string_possible(s))
    } else {
        String::from_str(s).map_err(|_| ModelNodeParseError::StringTooLong)
    }
}