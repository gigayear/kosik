// Kosik Document Writer
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

//! Writes formatted and composed pages to the standard output.
//!
//! # Examples
//!
//! ```rust,no_run
//! use kosik::document::{Page, Typescript};
//! use kosik::document::writer::Writer;
//! use kosik::text::{Line, Segment};
//!
//! let typescript = Typescript {
//!     contact: None,
//!     word_count: None,
//!     has_structure: false,
//!     short_title: Segment::from("WORKING TITLE"),
//!     short_author_name: Segment::from("ANONYMOUS"),
//!     pages: vec![Page {
//!         number: 1,
//!         height: 54,
//!         lines: vec![Some(Line::from(Segment::from("foo")))],
//!         footer: Vec::new(),
//!     }],
//! };
//!
//! let mut writer = Writer::new(&typescript);
//! let result = writer.run();
//! ```
use std::fs;
use std::io::{self, Write};
use std::str;

use encoding::{Encoding, EncoderTrap};
use encoding::all::ISO_8859_15;
use math::round;
use regex::Regex;
use thousands::Separable;

use crate::PROGRAM_NAME;
use crate::PROLOGUE_FILE;
use crate::document::*;

/// Output driver
pub struct Writer<'a> {
    typescript: &'a Typescript,
    real_page_no: usize,
}

impl<'a> Writer<'_> {
    /// Creates a document writer
    pub fn new(typescript: &'a Typescript) -> Writer<'a> {
        Writer {
            typescript: typescript,
            real_page_no: 1,
        }
    }

    /// Writes the document to the standard output
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_prologue()?;

        for (i, page) in self.typescript.pages.iter().enumerate() {
            self.start_a_new_page(page.number)?;

            if i == 0 {
                if self.typescript.contact.is_some() {
                    self.write_contact()?;
                }

                if self.typescript.word_count.is_some() {
                    self.write_word_count()?;
                }
            }

            let mut y = (TOP_LINE as f32 * LINE_HEIGHT as f32).round() as i32;

            for line in page.lines.iter() {
                match line {
                    Some(line) => {
                        let x = (line.column as f32 * CHAR_WIDTH).round() as i32;

                        writeln(&format!("{} {} moveto {}", x, y, line.ps()))?;

                        y -= LINE_HEIGHT.round() as i32;
                    },
                    None => {
                        y -= LINE_HEIGHT.round() as i32;
                    },
                }
            }

            if !page.footer.is_empty() {
                let x = (LEFT_MARGIN as f32 * CHAR_WIDTH).round() as i32;
                y = ((BOTTOM_LINE + page.footer.len() + 2) as f32 * LINE_HEIGHT).round() as i32;

                writeln(&format!("{} {} moveto (____________________) show ", x, y))?;

                y -= (2.0 * LINE_HEIGHT).round() as i32;

                for line in page.footer.iter() {
                    match line {
			Some(line) => {
		            writeln(&format!("{} {} moveto {}", x, y, line.ps()))?;
                            y -= LINE_HEIGHT.round() as i32;
			},
			None => {
                            y -= LINE_HEIGHT.round() as i32;
			},
                    }
                }
            }

            writeln("page-end")?;
        }

        writeln("%%Trailer")
    }

    #[doc(hidden)]
    fn write_prologue(&mut self) -> Result<(), Box<dyn Error>> {
        let   title_pat = Regex::new(r"@title@")?;
        let creator_pat = Regex::new(r"@creator@")?;
        let   pages_pat = Regex::new(r"@pages@")?;

        let creator = PROGRAM_NAME.to_string();
	
        let num_pages = format!("{}", self.typescript.pages.len());
        let mut prologue = fs::read_to_string(&*PROLOGUE_FILE)?;

        prologue = title_pat.replace
            (&prologue, &self.typescript.short_title.text).to_string();
        prologue = creator_pat.replace(&prologue, &creator).to_string();
        prologue = pages_pat.replace(&prologue, &num_pages).to_string();

        write(&prologue)
    }

    #[doc(hidden)]
    fn write_contact(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(block) = &self.typescript.contact {
            let mut y = (TOP_LINE as f32 * LINE_HEIGHT as f32).round() as i32;

            for (i, line) in block.lines.iter().enumerate() {
                if i > 0 && block.line_spacing == LineSpacing::Double {
                    y -= LINE_HEIGHT.round() as i32;
                }
            
                let x = (line.column as f32 * CHAR_WIDTH).round() as i32;

                write(&format!("{} {} moveto {}", x, y, line.ps()))?;
                y -= LINE_HEIGHT.round() as i32;
            }
        }

        Ok(())
    }
        
    #[doc(hidden)]
    fn write_word_count(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(word_count) = self.typescript.word_count {
            let n = if word_count > 1000 {
                // nearest thousand
                (round::half_to_even(word_count as f64 / 10000.0, 1)
                 * 10000.0) as i32
            } else {
                // nearest hundred
                (round::half_to_even(word_count as f64 / 1000.0, 1)
                 * 1000.0) as i32
            };
            
            let s = format!("Approx. {} words", n.separate_with_commas());
            
            let line = Line {
                column: RIGHT_MARGIN - s.chars().count(),
                segments: vec![Segment::from(s)],
                note_refs: Vec::new(),
            };
                
            let x = (line.column as f32 * CHAR_WIDTH).round() as i32;
            let y = (TOP_LINE as f32 * LINE_HEIGHT as f32).round() as i32;
            write(&format!("{} {} moveto {}", x, y, line.ps()))?;
        }

        Ok(())
    }

    #[doc(hidden)]
    fn start_a_new_page(&mut self, page_no: i32) -> Result<(), Box<dyn Error>> {
        writeln(&format!("%%Page: {} {}", self.real_page_no, self.real_page_no))?;
        writeln("page-begin")?;

        self.real_page_no += 1;

        if page_no > 1
            || (page_no == 1 && self.typescript.has_structure)
            || (page_no == 1
                && !self.typescript.has_structure
                && self.typescript.contact.is_none()
                && self.typescript.word_count.is_none())
        {
            // write slug line
            let x = (LEFT_MARGIN as f32 * CHAR_WIDTH).round() as i32;
            let y = (SLUG_LINE as f32 * LINE_HEIGHT).round() as i32; 

            write(&format!("{} {} moveto ", x, y))?;
            write(&self.typescript.short_author_name.ps)?;
            write(&format!("(/) show "))?;
            write(&self.typescript.short_title.ps)?;
            writeln(&format!("(/{}) show ", page_no))
        } else {
            Ok(())
        }
    }
}

/// Converts UTF-8 characters to ISO/IEC 8859-15 and writes them to
/// the standard output
fn write(text: &str) -> Result<(), Box<dyn Error>> {
    let chars = ISO_8859_15.encode(text, EncoderTrap::Replace)?;
    io::stdout().write_all(&chars)?;
    Ok(())
}

/// Converts UTF-8 characters to ISO/IEC 8859-15 and writes them to
/// the standard output, appending a newline
fn writeln(text: &str) -> Result<(), Box<dyn Error>> {
    let mut chars = ISO_8859_15.encode(text, EncoderTrap::Replace)?;
    chars.push(b'\n');
    io::stdout().write_all(&chars)?;
    Ok(())
}
    
