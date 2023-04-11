// Kosik Formatter Lookup Tables
// Copyright (C) 2023 Gene Yu
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <https://www.gnu.org/licenses/>.

//! Lookup table for Roman numerals
//!
//! # Examples
//!
//! ```
//! use kosik::lut::ROMAN_NUMERALS;
//! if let Some(s) = ROMAN_NUMERALS.numeral(9) {
//!     assert_eq!(s, "IX");
//! }
//! ```

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use lazy_static::lazy_static;

use crate::ROMAN_NUMERALS_FILE;

lazy_static! {
    #[doc(hidden)]
    pub static ref ROMAN_NUMERALS: RomanNumerals
        = RomanNumerals::new(&ROMAN_NUMERALS_FILE).unwrap();
}

/// An ordered list of Roman numerals
pub struct RomanNumerals {
    numerals: Vec<String>,
}

impl RomanNumerals {
    /// Parses the Roman numeral data file and stores the entries in
    /// a vector.
    pub fn new(filename: &PathBuf) -> Result<RomanNumerals, Box<dyn Error>> {
	let text = fs::read_to_string(filename)?; 

	Ok(RomanNumerals {
            numerals: text.split_whitespace().map(|s| s.to_string()).collect(),
	})
    }

    /// This function returns the Roman numeral (in string form)
    /// corresponding to the input integer.  The range goes from 1 to
    /// the number of lines in the data file.  If the index is out of
    /// range, the function returns <tt>None</tt>.
    pub fn numeral(&self, i: usize) -> Option<&str> {
	if i >= 1 && i < self.numerals.len() {
	    Some(&self.numerals[i - 1])
	} else {
	    None
	}
    }
}
