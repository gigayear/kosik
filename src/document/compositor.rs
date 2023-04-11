// Kosik Document Compositor
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

//! Flows formatted text blocks into pages, splitting them at page
//! breaks if necessary
//!
//! # Examples
//!
//! ```
//! use kosik::document::compositor::Compositor;
//! use kosik::document::Block;
//! use kosik::text::{Line, Segment};
//!
//! let mut block: Block = Default::default();
//! block.lines.push(Line::from(Segment::from("foo")));
//!
//! let mut compositor = Compositor::new(1, false);
//! compositor = compositor.run(vec![block]);
//!
//! assert_eq!(compositor.pages.len(), 1);
//! ```

use std::cmp::max;
use std::collections::HashMap;
use std::iter::repeat;

use crate::document::*;

/// Turns block lists into page lists
pub struct Compositor {
    /// If there is contact information in the document, it is set
    /// aside so it can be passed to the document writer later.
    pub contact: Option<Block>,
    /// Page list to be output by the document writer
    pub pages: PageList,
    footnotes: HashMap<String, BlockList>,
    first_page: i32,
    next_page_no: i32,
    has_structure: bool,
    last_padding_after: usize,
}

impl Compositor {
    /// Create a new compositor that will start numbering at the given
    /// page number.
    ///
    /// If <tt>has_structure</tt> is <tt>true</tt>, it means that
    /// there are subsections in the document, and therefore it will
    /// have a title page, which is unnumbered.  Otherwise, numbering
    /// should begin on the first page because some content from the
    /// body will appear on it.
    pub fn new(first_page: i32, has_structure: bool) -> Self {
        Self {
            contact: None,
            pages: Vec::new(),
            footnotes: HashMap::new(),
            first_page: first_page,
            next_page_no: -1,
            has_structure: has_structure,
            last_padding_after: 0,
        }
    }

    /// Flow a sequence of blocks into pages
    pub fn run(mut self, blocks: BlockList) -> Self {
        let mut toc: Vec<(i32, Block)> = Vec::new();
        
        if self.pages.is_empty() { // first page
            if self.has_structure {
                self.start_a_new_page();
                self.next_page_no = self.first_page;
            } else {
                self.next_page_no = self.first_page;
                self.start_a_new_page();
            }
        }

        let mut padding_before: i32 = 0;
        
        for block in blocks.into_iter() {
            if block.tag.is_some() {
                match block.tag {
                    Some(Tag::Contact) => {
                        self.contact = Some(block);
                    },
                    Some(Tag::Head) => {
                        self.compose(block, &mut padding_before);
                    },
                    Some(Tag::ToC) => {
                        toc.push((self.cur_page().number, block));
                    },
                    None => (),
                }
            } else {
                self.compose(block, &mut padding_before);
            }
        }

        if !toc.is_empty() {
            self.compose_toc(toc);
        }

        self
    }
    
    /// Consume a block, adding it to the current page
    fn compose(&mut self, block: Block, padding_before: &mut i32) {
        if block.padding_before < 0 {
            self.start_a_new_page();
            *padding_before = -block.padding_before - 1;
            self.last_padding_after = 0;

        } else {
            *padding_before = block.padding_before;
        };
            
        let padding = max(*padding_before as usize, self.last_padding_after);
        self.last_padding_after = block.padding_after;

        for _ in 0..padding {
            self.cur_page().lines.push(None);
        }
            
        self.compose_block(block);
    }

    fn compose_toc(&mut self, blocks: Vec<(i32, Block)>) {
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let s = Segment::from("Table of Contents");
        let n = s.text.chars().count();
        let header = Line {
            column: center - n / 2 - n % 2,
            segments: vec![s],
            note_refs: Vec::new(),
        };
        
        let mut padding_before: i32 = 0;
        
        self.next_page_no = -1;
        self.compose(Block {
            lines: vec![header],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: -1,
            padding_after: CHAPTER_SKIP,
            tag: Some(Tag::ToC),
        }, &mut padding_before);

        for (page_no, mut block) in blocks.into_iter() {
            if let Some(_) = block.lines.first() {
                let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;

                let mut line = block.lines.remove(0);
                let n = line.length();
                
                let page_no_string = format!("{}", page_no);
                let p = page_no_string.chars().count();
                
                let mut spaces_remaining = line_length - n - p;

                let before_pad = if n % 2 == 1 {
                    spaces_remaining -= 1;
                    " ".to_string()
                } else {
                    spaces_remaining -= 2;
                    "  ".to_string()
                };

                let after_pad = if p % 2 == 0 {
                    " ".to_string()
                } else {
                    "".to_string()
                };
                
                let dots = repeat(". ")
                    .take(spaces_remaining / 2)
                    .collect::<String>();

                line.segments.push(
                    Segment::from(format!("{}{}{}{}", before_pad, dots,
                                          after_pad, page_no))
                );
                
                block.lines.insert(0, line);
                
                let remainder = self.cur_page().height as i32
                    - self.cur_page().lines.len() as i32
                    - 1 // for the current line
                    - 1; // for the ToC entry separator
                
                // If the block is about to be split, start a new page
                // instead.
                if remainder < block.count_lines() as i32 {
                    self.start_a_new_page();
                    self.last_padding_after = 0;
                }

                self.compose(block, &mut padding_before);
            }
        }
    }
    
    fn start_a_new_page(&mut self) {
        let page = Page {
	    number: self.next_page_no,
	    height: TOP_LINE - BOTTOM_LINE + 1,
	    lines: Vec::new(),
            footer: Vec::new(),
        };

        self.pages.push(page);
	self.next_page_no += 1;
    }

    fn cur_page(&mut self) -> &mut Page {
	assert!(!self.pages.is_empty());
	self.pages.iter_mut().last().unwrap()
    }

    fn compose_block(&mut self, block: Block) {
        // Transfer footnotes to the hash map.
        for (label, footnote) in block.footnotes {
            self.footnotes.insert(label, footnote);
        }

        let page_height = block.lines.len();
        
        for (i, line) in block.lines.into_iter().enumerate() {
            if !line.note_refs.is_empty() { // There are footnotes on this line.
                // Count the total number of footnote lines.
                let mut footer_height: usize = 0;
                let mut j = 0;
        
                for label in line.note_refs.iter() {
                    if j > 0 {
                        footer_height += 1;
                    }
                    
                    if let Some(footnote) = self.footnotes.get(label) {
                        footer_height += count_lines(footnote);
                        j += 1;
                    }
                }
                
                let mut remainder = self.cur_page().height as i32
                    - self.cur_page().lines.len() as i32
                    - 1 // for the current line
                    - footer_height as i32
                    - 2; // for the footnote separator

                if !self.cur_page().footer.is_empty() {
                    remainder -= 1; // skip a space between footnotes
                    remainder -= self.cur_page().footer.len() as i32;
                }

                if remainder < 1 {
                    self.start_a_new_page();
                }

                // Add any footnotes to the current page.
                for label in line.note_refs.iter() {
                    // Note references with no attached footnotes are
                    // filtered out here.
                    if let Some(blocks) = self.footnotes.remove(label) {
                        // Skip a space between footnotes.
                        if !self.cur_page().footer.is_empty() {
                            self.cur_page().footer.push(None);
                        }
                    
                        let m = blocks.len();
                        for (j, block) in  blocks.into_iter().enumerate() {
                            let n = block.lines.len();
                            for (k, line) in block.lines.into_iter().enumerate() {
                                self.cur_page().footer.push(Some(line));

                                // If this is not the last line and we
                                // are double spacing, add a blank
                                // line.
                                if (j < m - 1 || k < n - 1) &&
                                    block.line_spacing == LineSpacing::Double
                                {
                                    self.cur_page().footer.push(None);
                                }
                            }
                        }
                    }
                }
            }

            // Now back to the current line.  Remember that the
            // height of the current page may have changed.
            let mut remainder = self.cur_page().height as i32 -
		self.cur_page().lines.len() as i32 - 1;

            if !self.cur_page().footer.is_empty() {
                remainder -= self.cur_page().footer.len() as i32 + 2;
            }

            if remainder < 1 {
                self.start_a_new_page();
            }

            // Push the current line onto the current page.
            self.cur_page().lines.push(Some(line));

            // If that was not the last line, and we are double
            // spacing, and there is space for at least one more line,
            // add a space.
            if i < page_height - 1
                && block.line_spacing == LineSpacing::Double
                && self.cur_page().height - self.cur_page().lines.len() > 1
            {
                self.cur_page().lines.push(None);
            }
        }
    }
}
