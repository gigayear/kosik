// Kosik Document Formatter
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

//! Converts elements to text blocks
//!
//! This module does not contain a driver.  It's just a collection of
//! conversions from [`ElementType`] variants to [`Block`] or
//! [`BlockList`].
//!
//! # Examples
//!
//! Flowing a paragraph into a 6-character-wide text block:
//! ```
//! use kosik::document::*;
//! use kosik::text::{Line, Segment};
//! use kosik::text::tokens::*;
//!
//! let mut elem = TextElement::new(P {
//!     indent: 0,
//!     line_spacing: LineSpacing::Double,
//!     left_margin: 10,
//!     right_margin: 15,
//! });
//!
//! elem.tokens.push(TokenType::Word(Token::from("foo")));
//! elem.tokens.push(TokenType::Space(Token::from(1)));
//! elem.tokens.push(TokenType::Word(Token::from("bar")));
//!
//! let block: Block = elem.into();
//! assert_eq!(block.lines.len(), 2);
//! ```

use std::iter::repeat;

use crate::document::*;
use crate::text;
use crate::lut::ROMAN_NUMERALS;

#[macro_use]
mod macros;

// container elements

impl From<ContainerElement<Authors>> for Block {
    fn from(elem: ContainerElement<Authors>) -> Self {
        let n = elem.children.len();
        
        let mut tokens: TokenList = Vec::with_capacity(n * 3 + 3);
        let mut footnotes: ElementList = Vec::new();

        tokens.push(TokenType::Word(Token::from("by")));
        tokens.push(TokenType::Space(Token::from(1)));

        for (i, child) in elem.children.into_iter().enumerate() {
            if i > 0 {
                if i == n - 1 {
                    tokens.push(TokenType::Space(Token::from(1)));
                    tokens.push(TokenType::Word(Token::from("and")));
                    tokens.push(TokenType::Space(Token::from(1)));

                } else {
                    tokens.push(TokenType::Symbol(Token::from(",")));
                    tokens.push(TokenType::Space(Token::from(1)));
                }
            }

            match child {
                ElementType::Person(child) => {
                    let (child_tokens, child_footnotes) = child.into();
                    tokens.extend(child_tokens.into_iter());
                    footnotes.extend(child_footnotes.into_iter());
                },
                _ => {},
            }
        }

        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let mut lines = text::linebreak_balance(&tokens[..], line_length);
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;

        for line in lines.iter_mut() {
            let n = line.length();
            line.column = center - n / 2 - n % 2;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 2,
            padding_after: 2,
            tag: Some(Tag::Head),
        }
    }
}

impl From<ContainerElement<Backmatter>> for BlockList {
    fn from(elem: ContainerElement<Backmatter>) -> BlockList {
        let center =  LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let mut headline = Line::from(Segment::from(&elem.attributes.label[..]));
        let n = headline.length();
        headline.column = center - n / 2 - n % 2;

        let mut blocks: BlockList = Vec::with_capacity(elem.children.len() + 1);

        blocks.push(Block {
            lines: vec![headline],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: -1,
            padding_after: CHAPTER_SKIP,
            tag: None,
        });

        let toc_entry = format_toc_entry!(elem.attributes.label);
        blocks.push(toc_entry);
        
        for child in elem.children {
            match child {
                ElementType::Attribution(child) => {
                    blocks.push(child.into());
                },
                ElementType::BibRef(child) => {
                    blocks.push(child.into());
                },
                ElementType::Blockquote(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Br(child) => {
                    blocks.push(child.into());
                },
                ElementType::Div(child) => {
                    blocks.push(child.into());
                },
                ElementType::Ol(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::P(child) => {
                    blocks.push(child.into());
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                ElementType::Ul(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                _ => {},
            }
        }
        
        blocks
    }
}

impl From<ContainerElement<Blockquote>> for BlockList {
    fn from(elem: ContainerElement<Blockquote>) -> Self {
        let p_count = elem.children.len();
        let mut blocks: BlockList = Vec::with_capacity(p_count);

        for (i, child) in elem.children.into_iter().enumerate() {
            match child {
                ElementType::P(child) => {
                    let mut block: Block = child.into();
                    
                    if i == p_count - 1 { // last paragraph
                        block.padding_after = 1;
                    }

                    blocks.push(block);
                }
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                _ => (),
            }
        }
        
        blocks
    }
}

impl From<ContainerElement<Body>> for BlockList {
    fn from(elem: ContainerElement<Body>) -> BlockList {
        let mut blocks: BlockList = Vec::with_capacity(elem.children.len());
        
        for child in elem.children {
            match child {
                ElementType::Attribution(child) => {
                    blocks.push(child.into());
                },
                ElementType::Blockquote(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Br(child) => {
                    blocks.push(child.into());
                },
                ElementType::Chapter(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Div(child) => {
                    blocks.push(child.into());
                },
                ElementType::Ol(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::P(child) => {
                    blocks.push(child.into());
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                ElementType::Part(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Section(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Ul(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                _ => {},
            }
        }

        blocks
    }
}

impl From<ContainerElement<Frontmatter>> for BlockList {
    fn from(elem: ContainerElement<Frontmatter>) -> BlockList {
        let center =  LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let mut headline = Line::from(Segment::from(&elem.attributes.label[..]));
        let n = headline.length();
        headline.column = center - n / 2 - n % 2;

        let mut blocks: BlockList = Vec::with_capacity(elem.children.len() + 1);

        blocks.push(Block {
            lines: vec![headline],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: -1,
            padding_after: CHAPTER_SKIP,
            tag: None,
        });

        let toc_entry = format_toc_entry!(elem.attributes.label);
        blocks.push(toc_entry);

        for child in elem.children {
            match child {
                ElementType::Attribution(child) => {
                    blocks.push(child.into());
                },
                ElementType::Blockquote(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Br(child) => {
                    blocks.push(child.into());
                },
                ElementType::Div(child) => {
                    blocks.push(child.into());
                },
                ElementType::Ol(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::P(child) => {
                    blocks.push(child.into());
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                ElementType::Ul(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                _ => {},
            }
        }

        blocks
    }
}

impl From<ContainerElement<Head>> for BlockList {
    fn from(elem: ContainerElement<Head>) -> BlockList {
        let mut title: Option<Block> = None;
        let mut subtitle: Option<Block> = None;
        let mut authors: Option<Block> = None;
        let mut contact: Option<Block> = None;
        let mut block_count: usize = 0;
        let mut line_count: usize = 0;
        let n = elem.children.len();
        
        for (i, child) in elem.children.into_iter().enumerate() {
            match child {
                ElementType::Authors(child) => {
                    let block: Block = child.into();
                    block_count += 1;
                    line_count += block.count_lines();
                    authors = Some(block);
                },
                ElementType::Contact(child) => {
                    let block: Block = child.into();
                    contact = Some(block);
                },
                ElementType::Title(child) => {
                    let block: Block = child.into();
                    block_count += 1;

                    if i < n - 1 {
                        line_count += block.count_lines() + 2;
                    }
                    
                    title = Some(block);
                },
                ElementType::Subtitle(child) => {
                    let block: Block = child.into();
                    block_count += 1;

                    if i < n - 1 {
                        line_count += block.count_lines() + 2;
                    }

                    subtitle = Some(block);
                },
                _ => {},
            }
        }

        let mut blocks: BlockList = Vec::with_capacity(block_count);
        
        if contact.is_some() {
            blocks.push(contact.unwrap());
        }

        if title.is_some() {
            let mut block = title.unwrap();
            //block.padding_before = (MIDDLE_LINE - line_count / 2 - line_count % 2) as i32;
            block.padding_before = (MIDDLE_LINE - line_count) as i32;
            blocks.push(block);
        }

        if subtitle.is_some() {
            blocks.push(subtitle.unwrap());
        }

        if authors.is_some() {
            blocks.push(authors.unwrap());
        }

        blocks
    }
}

impl From<ContainerElement<Li>> for BlockList {
    fn from(elem: ContainerElement<Li>) -> Self {
        let p_count = elem.children.len();
        let mut blocks: BlockList = Vec::with_capacity(p_count);
        let prefix: String;
        
        if let Some(n) = elem.attributes.number { // ordered
            let indent = repeat(' ').take(INDENT).collect::<String>();
            let label = format!("{}", n);
            let w = label.chars().count();
            let n = max(INDENT - w - 2, 0);

            if n > 0 {
                let pad = repeat(' ').take(n).collect::<String>();
                prefix = format!("{}{}. {}", indent, label, pad);
            } else {
                prefix = format!("{}{}. ", indent, label);
            }
            
        } else { // unordered
            let indent = repeat(' ').take(INDENT).collect::<String>();
            let n = max(INDENT - 2, 0);

            if n > 0 {
                let pad = repeat(' ').take(n).collect::<String>();
                prefix = format!("{}* {}", indent, pad);
            } else {
                prefix = format!("{}* ", indent);
            }
        }

        let indent = repeat(' ').take(INDENT * 2).collect::<String>();

        for (i, child) in elem.children.into_iter().enumerate() {
            match child {
                ElementType::P(mut child) => {
                    if i == 0 { // first paragraph
                        child.attributes.indent = 0;
                    }
                    
                    let mut block: Block = child.into();
                    
                    for (j, line) in block.lines.iter_mut().enumerate() {
                        line.column -= INDENT * 2;

                        if i == 0 && j == 0 {
                            line.segments.insert(0, Segment::from(&prefix[..]));
                        } else {
                            line.segments.insert(0, Segment::from(&indent[..]));
                        }
                    }

                    if i == p_count - 1 { // last paragraph
                        block.padding_after = 1;
                    }

                    blocks.push(block);
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                _ => (),
            }
        }
        
        blocks
    }
}

impl From<ContainerElement<Manuscript>> for BlockList {
    fn from(elem: ContainerElement<Manuscript>) -> Self {
        let mut blocks: BlockList = Vec::new();
        
        for child in elem.children {
            match child {
                ElementType::Backmatter(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Body(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Frontmatter(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::Head(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                _ => {},
            }
        }

        blocks
    }
}

impl From<ContainerElement<Ol>> for BlockList {
    fn from(elem: ContainerElement<Ol>) -> BlockList {
        let mut blocks: BlockList = Vec::with_capacity(elem.children.len());
            
        for child in elem.children {
            match child {
                ElementType::Li(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                _ => {},
            }
        }

        blocks
    }
}

impl From<ContainerElement<Person>> for BlockList {
    fn from(elem: ContainerElement<Person>) -> Self {
        let (tokens, footnotes) = elem.into();
        
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let mut lines = text::linebreak_balance(&tokens[..], line_length);
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;

        for line in lines.iter_mut() {
            let n = line.length();
            line.column = center - n / 2 - n % 2;
        }

        vec![Block {
            lines: lines,
            footnotes: format_footnotes(footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 3,
            padding_after: 3,
            tag: Some(Tag::Head),
        }]
    }
}

impl From<ContainerElement<Person>> for (TokenList, ElementList) {
    fn from(elem: ContainerElement<Person>) -> Self {
        let mut tokens: TokenList = Vec::with_capacity(elem.children.len());
        let mut footnotes: ElementList = Vec::new();

        for (i, child) in elem.children.into_iter().enumerate() {
            match child {
                ElementType::Footnote(child) => {
                    tokens.push(TokenType::NoteRef(Token {
                        data: NoteRefData {
                            text: child.attributes.label.clone(),
                        },
                        dpy: DisplayFlags::SUP,
                        frm: Default::default(),
                    }));
                    footnotes.push(ElementType::Footnote(child));
                },
                ElementType::Gn(child) => {
                    if i > 0 {
                        tokens.push(TokenType::Space(Token::from(1)));
                    }

                    tokens.extend(child.tokens.into_iter());
                    footnotes.extend(child.footnotes.into_iter());
                },
                ElementType::NoteRef(child) => {
                    tokens.push(TokenType::NoteRef(Token {
                        data: NoteRefData {
                            text: child.attributes.label.clone(),
                        },
                        dpy: DisplayFlags::SUP,
                        frm: Default::default(),
                    }));
                },
                ElementType::Prefix(child) => {
                    if i > 0 {
                        tokens.push(TokenType::Space(Token::from(1)));
                    }

                    tokens.extend(child.tokens.into_iter());
                    footnotes.extend(child.footnotes.into_iter());
                },
                ElementType::Sn(child) => {
                    if i > 0 {
                        tokens.push(TokenType::Space(Token::from(1)));
                    }

                    tokens.extend(child.tokens.into_iter());
                    footnotes.extend(child.footnotes.into_iter());
                },
                ElementType::Suffix(child) => {
                    if child.attributes.comma {
                        tokens.push(TokenType::Punct(Token::from(",")));
                    }
                    
                    if i > 0 {
                        tokens.push(TokenType::Space(Token::from(1)));
                    }

                    tokens.extend(child.tokens.into_iter());
                    footnotes.extend(child.footnotes.into_iter());
                },
                _ => {},
            }
        }

        (tokens, footnotes)
    }
}

impl From<ContainerElement<Ul>> for BlockList {
    fn from(elem: ContainerElement<Ul>) -> BlockList {
        let mut blocks: BlockList = Vec::with_capacity(elem.children.len());
            
        for child in elem.children {
            match child {
                ElementType::Li(child) => {
                    let child_blocks: BlockList = child.into();
                    blocks.extend(child_blocks.into_iter());
                },
                ElementType::PageBreak(child) => {
                    blocks.push(child.into());
                },
                _ => {},
            }
        }

        blocks
    }
}

// text elements

impl From<TextElement<Attribution>> for Block {
    fn from(elem: TextElement<Attribution>) -> Self {
        let tokens = elem.tokens;
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let mut lines = text::linebreak_balance(&tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = RIGHT_MARGIN - line.length();
        }

        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 1,
            padding_after: 1,
            tag: None,
        }
    }
}

impl From<TextElement<BibRef>> for Block {
    fn from(elem: TextElement<BibRef>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_hang(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }

        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 0,
            padding_after: 1,
            tag: None,
        }
    }
}

impl From<TextElement<Chapter>> for BlockList {
    fn from(elem: TextElement<Chapter>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let center =  LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let tag = format!("{}", elem.attributes.number); 
        let headtext = format!("Chapter {}", &tag);

        let mut headline = Line::from(Segment::from(headtext));
        let n = headline.length();
        headline.column = center - n / 2 - n % 2;

        let mut blocks = vec![Block {
            lines: vec![headline],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: -1,
            padding_after: if !elem.tokens.is_empty() {
                2
            } else {
                CHAPTER_SKIP
            },
            tag: None,
        }];

        if !elem.tokens.is_empty() {
            let mut lines = text::linebreak_balance(&elem.tokens[..], line_length);

            for line in lines.iter_mut() {
                let n = line.length();
                line.column = center - n / 2 - n % 2;
            }

            blocks.push(Block {
                lines: lines,
                footnotes: format_footnotes(elem.footnotes),
                line_spacing: elem.attributes.line_spacing,
                padding_before: 0,
                padding_after: CHAPTER_SKIP,
                tag: None,
            });

            let toc_entry = format_toc_entry!(elem, tag);
            blocks.push(toc_entry);
        }

        blocks
    }
}

impl From<TextElement<Contact>> for Block {
    fn from(elem: TextElement<Contact>) -> Self {
        let line_length = (RIGHT_MARGIN - LEFT_MARGIN) / 2 + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }

        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 0,
            padding_after: 2,
            tag: Some(Tag::Contact),
        }
    }
}

impl From<TextElement<Em>> for Block {
    fn from(elem: TextElement<Em>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

impl From<TextElement<Gn>> for Block {
    fn from(elem: TextElement<Gn>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: Some(Tag::Head),
        }
    }
}

impl From<TextElement<P>> for Block {
    fn from(elem: TextElement<P>) -> Self {
        let mut tokens = elem.tokens;

        if elem.attributes.indent > 0 {
            tokens.insert(0, TokenType::Space(Token::from(elem.attributes.indent)));
        }

        let line_length = elem.attributes.right_margin
            - elem.attributes.left_margin + 1;
        let mut lines = text::linebreak_fill(&tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = elem.attributes.left_margin;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 0,
            padding_after: if elem.attributes.line_spacing == LineSpacing::Double {
                1
            } else {
                0
            },
            tag: None,
        }
    }
}

impl From<TextElement<Part>> for BlockList {
    fn from(elem: TextElement<Part>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let center =  LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let tag;
        
        if let Some(roman_numeral) =
            ROMAN_NUMERALS.numeral(elem.attributes.number as usize)
        {
            tag = format!("{}", roman_numeral);
        } else {
            tag = format!("{}", elem.attributes.number);
        }

        let headtext = format!("Part {}", &tag);

        let mut blocks: BlockList = Vec::with_capacity(1);
        let mut height: usize = 1;
        let mut headline = Line::from(Segment::from(headtext));
        let n = headline.length();
        headline.column = center - n / 2 - n % 2;

        if !elem.tokens.is_empty() {
            let mut lines = text::linebreak_balance(&elem.tokens[..], line_length);
            height += 2 + lines.len();
            
            for line in lines.iter_mut() {
                let n = line.length();
                line.column = center - n / 2 - n % 2;
            }

            blocks.push(Block {
                lines: lines,
                footnotes: format_footnotes(elem.footnotes),
                line_spacing: elem.attributes.line_spacing,
                padding_before: 1,
                padding_after: PART_SKIP,
                tag: None,
            });

            let toc_entry = format_toc_entry!(elem, tag);
            blocks.push(toc_entry);
        }

        //let padding_before = -((MIDDLE_LINE - height / 2 - height % 2 + 1) as i32);
        let padding_before = -((MIDDLE_LINE - height + 1) as i32);
        blocks.insert(0, Block {
            lines: vec![headline],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: padding_before,
            padding_after: if !elem.tokens.is_empty() {
                2
            } else {
                PART_SKIP
            },
            tag: None,
        });

        blocks
    }
}

impl From<TextElement<Prefix>> for Block {
    fn from(elem: TextElement<Prefix>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: Some(Tag::Head),
        }
    }
}

impl From<TextElement<Section>> for BlockList {
    fn from(elem: TextElement<Section>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let center =  LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;
        let tag;
        
        if let Some(ch) = char::from_u32('@' as u32 + elem.attributes.number as u32) {
            tag = format!("{}", ch);
        } else {
            tag = format!("{}", &elem.attributes.number);
        }

        let headtext = format!("Section {}", &tag);
        let mut headline = Line::from(Segment::from(headtext));
        let n = headline.length();
        headline.column = center - n / 2 - n % 2;

        let mut blocks = vec![Block {
            lines: vec![headline],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: elem.attributes.padding_before,
            padding_after: if !elem.tokens.is_empty() {
                2
            } else {
                SECTION_SKIP
            },
            tag: None,
        }];

        if !elem.tokens.is_empty() {
            let mut lines = text::linebreak_balance(&elem.tokens[..], line_length);

            for line in lines.iter_mut() {
                let n = line.length();
                line.column = center - n / 2 - n % 2;
            }

            blocks.push(Block {
                lines: lines,
                footnotes: format_footnotes(elem.footnotes),
                line_spacing: elem.attributes.line_spacing,
                padding_before: 1,
                padding_after: SECTION_SKIP,
                tag: None,
            });

            let toc_entry = format_toc_entry!(elem, tag);
            blocks.push(toc_entry);
        }

        blocks
    }
}

impl From<TextElement<Sn>> for Block {
    fn from(elem: TextElement<Sn>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: Some(Tag::Head),
        }
    }
}

impl From<TextElement<Sub>> for Block {
    fn from(elem: TextElement<Sub>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

impl From<TextElement<Subtitle>> for Block {
    fn from(elem: TextElement<Subtitle>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let mut lines = text::linebreak_balance(&elem.tokens[..], line_length);
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;

        for line in lines.iter_mut() {
            let n = line.length();
            line.column = center - n / 2 - n % 2;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 0,
            padding_after: 2,
            tag: Some(Tag::Head),
        }
    }
}

impl From<TextElement<Suffix>> for Block {
    fn from(elem: TextElement<Suffix>) -> Self {
        let mut tokens = elem.tokens;
        
        if elem.attributes.comma {
            tokens.insert(0, TokenType::Space(Token::from(1)));
            tokens.insert(0, TokenType::Punct(Token::from(",")));
        }
                                
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: Some(Tag::Head),
        }
    }
}

impl From<TextElement<Sup>> for Block {
    fn from(elem: TextElement<Sup>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN + 1;
        let mut lines = text::linebreak_fill(&elem.tokens[..], line_length);

        for line in lines.iter_mut() {
            line.column = LEFT_MARGIN;
        }
        
        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

impl From<TextElement<Title>> for Block {
    fn from(elem: TextElement<Title>) -> Self {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT + 1;
        let mut lines = text::linebreak_balance(&elem.tokens[..], line_length);
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;

        for line in lines.iter_mut() {
            let n = line.length();
            line.column = center - n / 2 - n % 2;
        }

        Block {
            lines: lines,
            footnotes: format_footnotes(elem.footnotes),
            line_spacing: elem.attributes.line_spacing,
            padding_before: 0,
            padding_after: 2,
            tag: Some(Tag::Head),
        }
    }
}

// empty elements

impl From<EmptyElement<Br>> for Block {
    fn from(_: EmptyElement<Br>) -> Self {
        Block {
            lines: vec![Line {
                column: LEFT_MARGIN,
                segments: vec![Segment {
                    text: "".to_string(),
                    ps: "() show ".to_string(),
                }],
                note_refs: Vec::new(),
            }],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

impl From<EmptyElement<Div>> for Block {
    fn from(_: EmptyElement<Div>) -> Self {
        let center = LEFT_MARGIN + (RIGHT_MARGIN - LEFT_MARGIN) / 2;

        Block {
            lines: vec![Line {
                column: center,
                segments: vec![Segment {
                    text: "#".to_string(),
                    ps: "(#) show ".to_string(),
                }],
                note_refs: Vec::new(),
            }],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: 1,
            padding_after: 1,
            tag: None,
        }
    }
}

impl From<EmptyElement<NoteRef>> for Block {
    fn from(elem: EmptyElement<NoteRef>) -> Self {
        Block {
            lines: vec![Line {
                column: LEFT_MARGIN,
                segments: vec![Segment::from(&elem.attributes.label[..])],
                note_refs: Vec::new(),
            }],
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

impl From<EmptyElement<PageBreak>> for Block {
    fn from(_: EmptyElement<PageBreak>) -> Self {
        Block {
            lines: Vec::new(),
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: -1,
            padding_after: 0,
            tag: None,
        }
    }
}

// functions

#[doc(hidden)]
fn format_footnotes(elements: ElementList) -> Vec<(String, BlockList)> {
    let mut footnotes: Vec<(String, BlockList)> = Vec::with_capacity(elements.len());
        
    for elem in elements {
        match elem {
            ElementType::Footnote(footnote) => {
                let key = footnote.attributes.label.clone();
                let mut blocks: BlockList = Vec::new();

                for (i, child) in footnote.children.into_iter().enumerate() {
                    match child {
                        ElementType::P(mut p) => {
                            if i == 0 {
                                p.attributes.indent = 0;
                                
                                let label = format!("{}", footnote.attributes.label);
                                let mut n = INDENT - 1;
                                
                                if label.len() > 1 {
                                    n -= label.chars().count() - 1;
                                }

                                let spaces = repeat(' ').take(n).collect::<String>();
                                let prefix = format!("{}{}", spaces, label);

                                let token = Token::new(WordData::from(prefix),
                                                       DisplayFlags::SUP,
                                                       Default::default());
                                
                                p.tokens.insert(0, TokenType::Word(token));
                            }
                            
                            blocks.push(p.into());
                        },
                        _ => {},
                    }
                }

                footnotes.push((key, blocks));
            },
            _ => {},
        }
    }

    footnotes
}
