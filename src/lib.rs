// Kosik Library
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
                                           
//! An XML-to-Postscript converter for creating typed manuscripts on
//! a home or office printer.
//!
//! Produces typewriter output in standard manuscript format.  Some
//! useful applications of manuscript format:
//!
//!   * Manuscripts are used for composition.
//!
//!   * Manuscripts are used for sharing texts with co-authors,
//!     readers, agents, and editors.
//!
//!   * Manuscripts are used for copy editing.
//!
//!   * Electronic or paper manuscripts can serve as a spec format for
//!     the production pipeline.
//!
//!   * Manuscript format is used to make hard copies for storage.
//!
//!   * Manuscripts are used for underground distribution.
//!
//! From the perspective of the author, preventing the loss of your
//! work is the most important job of a word processing system.  To
//! that end, XML is a reliable encoding because it uses plain text,
//! it is non-proprietary, it is compatible with version control, and
//! it is very widely used.
//!
//! This crate is named after an elephant from South Korea who is
//! reputed to possess the power of speech.
//!
//! Pronunciation (IPA): ‘kəʊ,ʃɪk
//!
//! # Examples
//!
//! Processing a valid document, an encoding of the short story
//! _Youth_, by Joseph Conrad:
//!
//! ```sh
//! $ head -4 conrad.sik
//! <?xml version="1.0" encoding="utf-8"?>
//! <manuscript
//!   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
//!   xsi:noNamespaceSchemaLocation="http://www.matchlock.com/kosik/manuscript.xsd">
//! $ wc -l conrad.sik
//! 1308 conrad.sik
//! $ xmllint --noout --schema manuscript.xsd conrad.sik
//! conrad.sik validates
//! $ kosik conrad.sik > conrad.ps
//! $ head -6 conrad.ps
//! %!PS
//! %%Title: Youth
//! %%Creator: kosik
//! %%DocumentFonts: Courier
//! %%BoundingBox: 0 0 612 792
//! %%Pages: 45
//! $
//! ```
//!
//! Output: [`conrad.ps`]
//!
//! Kosik can also show you its internal element representation using
//! the <tt>-e</tt> flag, and it works on fragments of the manuscript
//! schema:
//!
//! ```sh
//! $ cat minimal.sik
//! <br/>
//! $ kosik -e minimal.sik
//! EmptyElement { attributes: Br }
//! ```
//!
//! If you use the <tt>-b</tt> flag, Kosik will show you the internal
//! block representation.  In the example below, the output has been
//! formatted to make it more readable.  The actual output comes out
//! all on one line.
//!
//! ```sh
//! $ kosik -b minimal.sik
//! Block {
//!      lines: [
//!           Line {
//!                column: 10,
//!                segments: [Segment { text: "", ps: "() show " }],
//!                note_refs: []
//!           }
//!      ],
//!      footnotes: [],
//!      line_spacing: Single,
//!      padding_before: 0,
//!      padding_after: 0,
//!      tag: None
//! }
//! ```
//!
//! If you don't use either the <tt>-e</tt> nor the <tt>-e</tt> flags,
//! Kosik will render the individual element in Postscript.  In all
//! cases, a single top-level element is expected.
//!
//! [`conrad.ps`]: <http://www.matchlock.com/kosik/conrad.ps>

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

use lazy_static::lazy_static;

use crate::document::*;
use crate::document::reader::Reader;
use crate::document::compositor::Compositor;
use crate::document::writer::Writer;
use crate::text::*;
use crate::text::tokens::*;
    
pub mod document;
pub mod lut;
pub mod text;

//use crate::document::LEFT_MARGIN;
//use crate::document::RIGHT_MARGIN;

// configuration

lazy_static! {
    #[doc(hidden)]
    pub static ref PROGRAM_NAME: String = match get_program_name() {
        Some(name) => name,
        None => "kosik".to_string(),
    };

    /// Path to prologue.ps
    static ref PROLOGUE_FILE: PathBuf
        = PathBuf::from("/home/gene/share/kosik/prologue.ps");

    /// Path to roman_numerals.txt
    static ref ROMAN_NUMERALS_FILE: PathBuf
        = PathBuf::from("/home/gene/share/kosik/roman_numerals.txt");
}

/// Command-line arguments
#[derive(Parser, Default, Debug)]
#[clap(author="Gene Yu", version, about="Manuscript Typewriter")]
pub struct Arguments {
    /// An XML file conforming to the manuscript schema
    pub input_file: PathBuf,

    #[clap(short, long)]
    /// Show the internal element representation instead of the usual output.
    pub elements: bool,

    #[clap(short, long)]
    /// Show the internal block representation instead of the usual output.
    pub blocks: bool,
}

impl From<&str> for Arguments {
    // This method is for testing.
    fn from(s: &str) -> Self {
        Self {
            input_file: PathBuf::from(s),
            elements: false,
            blocks: false,
        }
    }
}

/// Read an XML input string and construct an element hierarchy from
/// its contents
///
/// # Examples
///
/// ```rust,no_run
/// # use std::path::PathBuf;
/// let args = kosik::Arguments::from("dummy.sik");
/// let root = kosik::read(&args).unwrap();
/// ```
pub fn read(args: &Arguments) -> Result<ElementType, Box<dyn Error>> {
    let xml_string = fs::read_to_string(&args.input_file).unwrap();
    let reader = Reader::new(&xml_string);
    reader.run().ok_or("No elements!".into())
}

#[doc(hidden)]
#[macro_use]
mod fragments;

/// Write an element hierarchy to the standard output in Postscript
///
/// # Examples
///
/// ```rust,no_run
/// # use std::path::PathBuf;
/// # let args = kosik::Arguments::from("test.sik");
/// let root = kosik::read(&args).unwrap();
/// kosik::write(root, &args);
/// ```
pub fn write(elem: ElementType, args: &Arguments)
             -> Result<(), Box<dyn Error>>
{
    match elem {
        ElementType::Attribution(elem) => {
            write_block!(elem, "attribution", &args);
        },
        ElementType::Authors(elem) => {
            write_block!(elem, "authors", &args);
        },
        ElementType::Backmatter(elem) => {
            write_container!(elem, "backmatter", &args);
        },
        ElementType::BibRef(elem) => {
            write_block!(elem, "bibRef", &args);
        },
        ElementType::Blockquote(elem) => {
            write_container!(elem, "blockquote", &args);
        },
        ElementType::Body(elem) => {
            write_container!(elem, "body", &args);
        },
        ElementType::Br(elem) => {
            write_block!(elem, "br", &args);
        },
        ElementType::Chapter(elem) => {
            write_container!(elem, "chapter", &args);
        },
        ElementType::Contact(elem) => {
            write_block!(elem, "contact", &args);
        },
        ElementType::Div(elem) => {
            write_block!(elem, "div", &args);
        },
        ElementType::Em(elem) => {
            write_block!(elem, "em", &args);
        },
        ElementType::Footnote(elem) => {
            let wrapper = TextElement {
                attributes: P {
                    indent: 0,
                    line_spacing: LineSpacing::Double,
                    left_margin: LEFT_MARGIN,
                    right_margin: RIGHT_MARGIN,
                },
                tokens: vec![TokenType::NoteRef(Token {
                    data: NoteRefData {
                        text: elem.attributes.label.clone(),
                    },
                    dpy: DisplayFlags::SUP,
                    frm: Default::default(),
                })],
                footnotes: vec![ElementType::Footnote(elem)],
            };
            
            write_block!(wrapper, "footnote", &args);
        },
        ElementType::Frontmatter(elem) => {
            write_container!(elem, "frontmatter", &args);
        },
        ElementType::Gn(elem) => {
            write_block!(elem, "gn", &args);
        },
        ElementType::Head(elem) => {
            write_container!(elem, "head", &args);
        },
        ElementType::Li(elem) => {
            write_container!(elem, "li", &args);
        },
        ElementType::Manuscript(elem) => {
            if args.elements {
                println!("{:?}", &elem);

                if !args.blocks {
                    return Ok(());
                }
            }

            let first_page = elem.attributes.first_page;
            let word_count = elem.attributes.word_count;
            let has_structure = elem.attributes.has_structure;
            
            let short_title = match elem.short_title() {
                Some(segment) => segment,
                None => Segment {
                    text: "Working Title".to_string(),
                    ps: "(WORKING TITLE) show ".to_string(),
                },
            };
            
            let short_author_name = match elem.short_author_name() {
                Some(segment) => segment,
                None => Segment::from("ANONYMOUS"),
            };
            
            let blocks: BlockList = elem.into();

            if args.blocks {
                println!("{:?}", &blocks);
            }

            if args.elements || args.blocks {
                return Ok(());
            }
                
            let mut compositor = Compositor::new(first_page, has_structure);
            compositor = compositor.run(blocks);
            
            let typescript = Typescript {
                contact: compositor.contact,
                word_count: Some(word_count),
                has_structure: has_structure,
                short_title: short_title,
                short_author_name: short_author_name,
                pages: compositor.pages,
            };

            let mut writer = Writer::new(&typescript);
            writer.run()?;
        },
        ElementType::NoteRef(elem) => {
            write_block!(elem, "noteRef", &args);
        },
        ElementType::Ol(elem) => {
            write_container!(elem, "ol", &args);
        },
        ElementType::P(elem) => {
            write_block!(elem, "p", &args);
        },
        ElementType::PageBreak(elem) => {
            write_block!(elem, "pageBreak", &args);
        },
        ElementType::Part(elem) => {
            write_container!(elem, "part", &args);
        },
        ElementType::Person(elem) => {
            write_container!(elem, "person", &args);
        },
        ElementType::Prefix(elem) => {
            write_block!(elem, "prefix", &args);
        },
        ElementType::Section(elem) => {
            write_container!(elem, "section", &args);
        },
        ElementType::Sn(elem) => {
            write_block!(elem, "sn", &args);
        },
        ElementType::Sub(elem) => {
            write_block!(elem, "sub", &args);
        },
        ElementType::Subtitle(elem) => {
            write_block!(elem, "subtitle", &args);
        },
        ElementType::Suffix(elem) => {
            write_block!(elem, "prefix", &args);
        },
        ElementType::Sup(elem) => {
            write_block!(elem, "sup", &args);
        },
        ElementType::Title(elem) => {
            write_block!(elem, "title", &args);
        },
        ElementType::Ul(elem) => {
            write_container!(elem, "ul", &args);
        },
    }
    
    Ok(())
}

#[doc(hidden)]
fn get_program_name() -> Option<String> {
    env::current_exe().ok()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}
