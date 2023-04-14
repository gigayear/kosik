// Kosik Document Module
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

//! In-memory representation of a document
//!
//! * The [`reader`] module builds an element tree, tokenizing the
//!   contents of each text element.
//!
//! * The [`formatter`] module breaks token lists into lines, and then
//!   forms the lines into text blocks.
//!
//! * The [`compositor`] module flows the text blocks into pages,
//!   splitting them when necessary, and adds a header to each page.
//!
//! * The [`writer`] module writes the pages to the standard output
//!   using the Latin-9 character set.

use std::cmp::max;
use std::error::Error;
use std::fmt::Debug;

use crate::text::Line;
use crate::text::Segment;
use crate::text::tokens::*;

pub mod reader;
pub mod formatter;
pub mod compositor;
pub mod writer;

// configuration

/// Character width in points
pub const CHAR_WIDTH: f32 = 7.2;

/// Line height in points
pub const LINE_HEIGHT: f32 = 12.0;

/// Default indent in spaces
pub const INDENT: usize = 5;

/// Left margin in spaces
pub const LEFT_MARGIN: usize = 10;

/// Right margin in spaces
pub const RIGHT_MARGIN: usize = 74;

/// Slug line height
pub const SLUG_LINE: usize = 62;

/// Line number of the top line
pub const TOP_LINE: usize = 59;

/// Line number of the middle of the page
pub const MIDDLE_LINE: usize = 27;

/// Line number of the bottom line
pub const BOTTOM_LINE: usize = 6;

/// Number of lines to skip after a part title
pub const PART_SKIP: usize = 5;

/// Number of lines to skip after a chapter title
pub const CHAPTER_SKIP: usize = 11;

/// Number of lines to skip after a section title
pub const SECTION_SKIP: usize = 5;

/// Sequence of composited pages plus slug line info
#[derive(Debug)]
pub struct Typescript {
    /// If there is contact information here, it will be printed in
    /// the top left corner of the title page.
    pub contact: Option<Block>,
    /// If there is word count information here, it will be printed in
    /// the top right corner of the title page, rounded to the nearest
    /// thousand.
    pub word_count: Option<usize>,
    /// This flag indicates whether the document contained any
    /// subdivisions:  parts, chapters or sectdions.
    pub has_structure: bool,
    /// A version of the document title formatted for the slug line.
    pub short_title: Segment,
    /// The surname of the first author listed in the head, formatted
    /// for the slug line.
    pub short_author_name: Segment,
    /// The page list to write to the output stream
    pub pages: PageList,
}

/// Numbered page including the page height, the lines to output, and
/// accompanying footnotes
#[derive(Debug)]
pub struct Page {
    /// Page number
    ///
    /// If the number is positive, a page number will be printed.  If
    /// it is less than or equal to zero, no page number will be
    /// printed.
    pub number: i32,
    /// Height of the page in lines
    ///
    /// Typewriter lines are 12 points high, 66 per page.  With at
    /// least 1-inch margins, that leaves 54 lines, or 27
    /// double-spaced lines.
    pub height: usize,
    /// Line data
    ///
    /// Blank lines are indicated by <tt>None</tt>.
    pub lines: Vec<Option<Line>>,
    /// Footer lines
    ///
    /// These lines are printed at the bottom of the page.
    pub footer: Vec<Option<Line>>,
}

/// Data type representing a sequence of pages
type PageList = Vec<Page>;

/// Document after line break but before page breaks
#[derive(Debug, Clone)]
pub struct Scroll {
    /// Page number of the first numbered page in the document to be
    /// composited
    pub first_page: i32,
    /// Document title formatted for the slug line
    pub short_title: Segment,
    /// Document author formatted for the slug line
    pub short_author_name: Segment,
    /// Block sequence of the document head
    pub head: BlockList,
    /// Block sequence of the document body
    pub body: BlockList,
}

/// Marker for special-purpose blocks
#[derive(Debug, Clone)]
pub enum Tag {
    /// Contact information is set aside for the writer.
    Contact,
    /// Head elements are marked but not extracted from the stream.
    Head,
    /// Table of contents elements are set aside by the compositor and
    /// formatted after the rest of the document is finished.
    ToC,
}

/// A text block
#[derive(Debug, Clone)]
pub struct Block {
    /// Formatted lines of text to be printed
    pub lines: Vec<Line>,
    /// List of footnotes appearing in the block, with a key for the
    /// compositor to use in its hash table
    pub footnotes: Vec<(String, BlockList)>,
    /// Line spacing can be <tt>single</tt> or <tt>double</tt>.
    pub line_spacing: LineSpacing,
    /// Number of blank lines preceding the block
    pub padding_before: i32,
    /// Number of blank lines following the block
    pub padding_after: usize,
    /// Marker for special-purpose blocks
    pub tag: Option<Tag>,
}

impl Block {
    /// Count the total number of lines this block will contain once
    /// it is rendered with the correct line spacing.
    pub fn count_lines(&self) -> usize {
        match self.line_spacing {
            LineSpacing::Double => {
                self.lines.len() * 2 - 1
            },
            LineSpacing::Single => {
                self.lines.len()
            },
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            lines: Vec::new(),
            footnotes: Vec::new(),
            line_spacing: LineSpacing::Single,
            padding_before: 0,
            padding_after: 0,
            tag: None,
        }
    }
}

/// Text block vector type
pub type BlockList = Vec<Block>;

/// Counts the total number of lines in a block list
pub fn count_lines(blocks: &BlockList) -> usize {
    let mut n: usize = 0;
    let mut last_padding_after: usize = 0;

    for (i, block) in blocks.iter().enumerate() {
        if i > 0 && block.padding_before >= 0 {
            n += max(block.padding_before as usize, last_padding_after);
        }

        n += block.count_lines();
        last_padding_after = block.padding_after;
    }

    n
}

// generic elements

/// Generic container element contains only other elements, no text
#[derive(Debug)]
pub struct ContainerElement<Attributes> {
    /// Parameter struct
    pub attributes: Attributes,
    /// Sequence of child elements
    pub children: ElementList,
}

impl<Attributes> ContainerElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
            children: Vec::new(),
        }
    }
}

/// Generic empty element contains only attributes, no content
#[derive(Debug)]
pub struct EmptyElement<Attributes> {
    /// Parameter struct
    pub attributes: Attributes,
}

impl<Attributes> EmptyElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
        }
    }
}

/// Generic text element contains mixed content, and footnote elements
/// are set aside
#[derive(Debug)]
pub struct TextElement<Attributes> {
    /// Parameter struct
    pub attributes: Attributes,
    /// Sequence of tokens
    pub tokens: TokenList,
    /// Sequence of footnote elements
    pub footnotes: ElementList,
}

impl<Attributes> TextElement<Attributes> {
    pub fn new(attributes: Attributes) -> Self {
        Self {
            attributes: attributes,
            tokens: Vec::new(),
            footnotes: Vec::new(),
        }
    }
}

// element type enum

/// Element type enum for in-memory representation of XML elements
#[derive(Debug)]
pub enum ElementType {
    Attribution(TextElement     <Attribution>),
    Authors    (ContainerElement<Authors    >),
    Backmatter (ContainerElement<Backmatter >),
    BibRef     (TextElement     <BibRef     >),
    Blockquote (ContainerElement<Blockquote >),
    Body       (ContainerElement<Body       >),
    Br         (EmptyElement    <Br         >),
    Chapter    (TextElement     <Chapter    >),
    Contact    (TextElement     <Contact    >),
    Div        (EmptyElement    <Div        >),
    Em         (TextElement     <Em         >),
    Footnote   (ContainerElement<Footnote   >),
    Frontmatter(ContainerElement<Frontmatter>),
    Gn         (TextElement     <Gn         >),
    Head       (ContainerElement<Head       >),
    Li         (ContainerElement<Li         >),
    Manuscript (ContainerElement<Manuscript >),
    NoteRef    (EmptyElement    <NoteRef    >),
    Ol         (ContainerElement<Ol         >),
    P          (TextElement     <P          >),
    PageBreak  (EmptyElement    <PageBreak  >),
    Part       (TextElement     <Part       >),
    Person     (ContainerElement<Person     >),
    Prefix     (TextElement     <Prefix     >),
    Section    (TextElement     <Section    >),
    Sn         (TextElement     <Sn         >),
    Sub        (TextElement     <Sub        >),
    Subtitle   (TextElement     <Subtitle   >),
    Suffix     (TextElement     <Suffix     >),
    Sup        (TextElement     <Sup        >),
    Title      (TextElement     <Title      >),
    Ul         (ContainerElement<Ul         >),
}

/// Data type for a list of elements
pub type ElementList = Vec<ElementType>;

// attribute values

/// Enum for manually setting the amount of line spacing to use for a
/// block of text
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LineSpacing {
    Single = 1,
    Double = 2,
}

impl From<&str> for LineSpacing {
    fn from(s: &str) -> Self {
        match s {
            "double" => LineSpacing::Double,
            _ => LineSpacing::Single,
        }
    }
}

// elements with no attributes

/// The main body of the document
#[derive(Debug)]
pub struct Body {}

/// Mandatory line break
#[derive(Debug)]
pub struct Br {}

/// Scene divider
///
/// Manuscript format dictates that a scene divider is a single
/// <tt>#</tt> character (<tt>U+0023</tt>) centered on the page, with
/// one blank line before and after.
#[derive(Debug)]
pub struct Div {}

/// Emphasis
#[derive(Debug)]
pub struct Em {}

/// Given name.  Multiple given names are allowed, so middle names
/// should use this element.
#[derive(Debug)]
pub struct Gn {}

/// Document header containing identifying information
///
/// This is an element-only container holding [`Title`], [`Subtitle`],
/// and [`Authors`] in sequence.
#[derive(Debug)]
pub struct Head {}

/// Mandatory page break
#[derive(Debug)]
pub struct PageBreak {}

/// An element-only container holding personal name components
///
/// # Examples
///
/// Common name:
///
/// ```xml
/// <person><gn>Joseph</gn><sn>Conrad</sn></person>
/// ```
/// 
/// Output:
///
/// <pre>Joseph Conrad</pre>
///
/// Full boat:
///
/// ```xml
/// <person>
///   <prefix>Dr.</prefix>
///   <gn>Martin</gn><gn>Luther</gn>
///   <sn>King</sn>
///   <suffix comma="true">Jr.</suffix>
///   <footnote label="*">Dexter Avenue Baptist Church</footnote>
///   <footnote label="×">Southern Christian Leadership Conference</footnote>
/// </person>
/// ```
/// 
/// Output:
/// <pre>Dr. Martin Luther King, Jr.<sup>*×</sup></pre>
#[derive(Debug)]
pub struct Person {}

/// The prefix of a personal name, such as Mr., Ms., Dr., etc.
#[derive(Debug)]
pub struct Prefix {}

/// Surname
#[derive(Debug)]
pub struct Sn {}

/// Subscript
///
/// Shifts a half a line down for the duration of the element's
/// contents, just as you would do on a typewriter.
#[derive(Debug)]
pub struct Sub {}

/// Superscript
///
/// Shifts a half a line up for the duration of the element's
/// contents, just as you would do on a typewriter.
#[derive(Debug)]
pub struct Sup {}

// elements with attributes

/// Right-justified block for an attribution following a blockquote
#[derive(Debug)]
pub struct Attribution {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Sequence of authors
#[derive(Debug)]
pub struct Authors {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Appendix, Epilogue, Postscript, Bibliography, etc.
#[derive(Debug)]
pub struct Backmatter {
    /// Name of section
    pub label: String,
}

/// Paragraph with a hanging indent for bibliography references
#[derive(Debug)]
pub struct BibRef {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Paragraph with narrow margins for quotations
#[derive(Debug)]
pub struct Blockquote {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Chapter name
#[derive(Debug)]
pub struct Chapter {
    /// Chapter number.  This attribute is set automatically, but can
    /// be overriden using ab XML attribute.
    pub number: i32,
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
    /// Subdivision depth.  This attribute is set during
    /// post-processing by the document reader, once it is known how
    /// many levels the document contains.
    pub depth: i32,
}

/// Contact information
///
/// Contact information flows into a block half the width of the page,
/// but it is intended to be used with line breaks.
///
/// # Examples
///
/// ```xml
/// <contact>MATCHLOCK PRESS<br/>P.O.\ Box 90606<br/>Brooklyn, NY 11209</contact>
/// ```
///
/// Output:
///
/// <pre>
/// MATCHLOCK PRESS
/// P.O. Box 90606
/// Brooklyn, NY 11209
/// </pre>
#[derive(Debug)]
pub struct Contact {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Footnote
#[derive(Debug)]
pub struct Footnote {
    /// Footnote label defaults to automatic numbering, but man be
    /// overridden by an XML attribute.
    pub label: String,
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute.  This setting applies to all child list elements,
    /// unless overridden by the child.
    pub line_spacing: LineSpacing,
}

/// Forward, Introduction, Preface, etc.
#[derive(Debug)]
pub struct Frontmatter {
    /// Name of section
    pub label: String,
}

/// List item
#[derive(Debug)]
pub struct Li {
    /// List item number.  Only used by ordered lists, <tt>None</tt>
    /// for unordered lists
    pub number: Option<i32>,
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Document root
#[derive(Debug)]
pub struct Manuscript {
    /// Sets the page number of the first numbered page (not including
    /// the title page, if any)
    pub first_page: i32,
    /// The number of word tokens in the document
    pub word_count: usize,
    /// True if any subdivision elements appear in the document (part,
    /// chapter or section)
    pub has_structure: bool,
}

/// Note reference
#[derive(Debug)]
pub struct NoteRef {
    /// Note reference.  A symbol or character identifying the
    /// reference will appear in superscript mode
    pub label: String,
}

/// Ordered list element
///
/// # Examples
///
/// ```xml
/// <p indent="0">Caesar's Audience</p>
/// <ol>
///   <li>Friends</li>
///   <li>Romans</li>
///   <li>Countrymen</li>
/// </ol>
/// ```
///
/// Output:
/// ```text
/// Caesar's Audience
///
///   1. Friends
///
///   2. Romans
///
///   3. Countrymen
/// ```
#[derive(Debug)]
pub struct Ol {
    /// The list item sequence number is initialized to this value,
    /// but it may be overridden by individual list items
    pub start_no: i32,
    /// Controls line spacing for the entire list, but may be
    /// overridden by individual list items
    pub line_spacing: LineSpacing,
}

/// Paragraph
#[derive(Debug)]
pub struct P {
    /// Indent default to five spaces
    pub indent: usize,
    /// Defaults to <tt>double</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
    /// Left margin is one inch from the left edge of the page
    pub left_margin: usize,
    /// Right margin is one inch from the right edge of the page
    pub right_margin: usize,
}

/// Level 0 subdivision
#[derive(Debug)]
pub struct Part {
    /// Part number.  This attribute is set automatically, but may be
    /// overriden using an XML attribute.
    pub number: i32,
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
    /// Subdivision depth.  This attribute is set during
    /// post-processing by the document reader, once it is known how
    /// many levels the document contains.
    pub depth: i32,
}

/// Level 2 subdivision
#[derive(Debug)]
pub struct Section {
    /// Section number.  This attribute is set automatically, but may be
    /// overriden using an XML attribute.
    pub number: i32,
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
    /// Number of blank lines preceding a section header
    pub padding_before: i32,
    /// Subdivision depth.  This attribute is set during
    /// post-processing by the document reader, once it is known how
    /// many levels the document contains.
    pub depth: i32,
}

/// Document subtitle
#[derive(Debug)]
pub struct Subtitle {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Personal name suffix, such as Jr. or III
#[derive(Debug)]
pub struct Suffix {
    /// True if a comma should proceed the suffix when printed
    pub comma: bool,
}

/// Document title
#[derive(Debug)]
pub struct Title {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

/// Unordered list
///
/// # Examples
///
/// ```xml
/// <p indent="0">Caesar's Audience</p>
/// <ul>
///   <li>Friends</li>
///   <li>Romans</li>
///   <li>Countrymen</li>
/// </ul>
/// ```
///
/// Output:
/// ```text
/// Caesar's Audience
///
///    * Friends
///
///    * Romans
///
///    * Countrymen
/// ```
#[derive(Debug)]
pub struct Ul {
    /// Defaults to <tt>single</tt>, but may be overridden by an XML
    /// attribute
    pub line_spacing: LineSpacing,
}

// parametrized text elements

impl TextElement<Title> {
    // Return the first line of the title (with ellipses if shortened)
    // in a Segment with the <tt>text</tt> in mixed case, but the
    // Postscript output in all-uppercase.
    pub fn short_title(&self) -> Option<Segment> {
        let line_length = RIGHT_MARGIN - LEFT_MARGIN - 4 * INDENT;
        let text_length: usize = self.tokens.iter()
            .fold(0, |sum, token| sum + token.length());
        let line_count = text_length / line_length + 1;
        let cutoff = text_length / line_count;
        let mut x: usize = 0;

        // Find the index j of the first line break.
        let n = self.tokens.len();
        let mut j: usize = self.tokens.len();
        
        for (i, token) in self.tokens.iter().enumerate() {
            let len = token.length();
            let frm = token.format_flags();

            if frm.intersects(FormatFlags::MLB) {
                j = i;
                break;
                
            } else if frm.intersects(FormatFlags::DLB) {                
                if x + len >= cutoff {
                    if frm.intersects(FormatFlags::DOB) {
                        j = i;
                    } else {
                        j = i + 1
                    };

                    break;
                } else {
                    x += len;
                }
            } else {
                x += len;
            }
        }

        if j > 0 {
            // Copy j tokens, clearing all display flags and
            // converting words to uppercase.
            let mut tokens: TokenList = Vec::with_capacity(j);

            // Construct a mixed-case version of the title for the
            // %%Title line in the Postscript file.
            let mut plaintext = String::new();

            for token in (&self.tokens[0..j]).iter() {
                match token {
                    TokenType::Close(token) => {
                        plaintext.push_str(&token.data.text);
                        
                        tokens.push(TokenType::Close(Token {
                            data: CloseData {
                                text: token.data.text.clone(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::LineBreak(_) => {},
                    TokenType::NoteRef(_) => {},
                    TokenType::Open(token) => {
                        plaintext.push_str(&token.data.text);

                        tokens.push(TokenType::Open(Token {
                            data: OpenData {
                                text: token.data.text.clone(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Punct(token) => {
                        plaintext.push_str(&token.data.text);

                        tokens.push(TokenType::Punct(Token {
                            data: PunctData {
                                text: token.data.text.clone(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Space(token) => {
                        plaintext.push_str(&token.data.text);

                        tokens.push(TokenType::Space(Token {
                            data: SpaceData {
                                text: token.data.text.clone(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Symbol(token) => {
                        plaintext.push_str(&token.data.text);

                        tokens.push(TokenType::Symbol(Token {
                            data: SymbolData {
                                text: token.data.text.clone(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Word(token) => {
                        plaintext.push_str(&token.data.text);

                        tokens.push(TokenType::Word(Token {
                            data: WordData {
                                text: token.data.text.to_uppercase(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                }
            }
            
            if j < n {
                plaintext.push_str(" . . .");
                
                tokens.push(TokenType::Space(Token::from(1)));
                tokens.push(TokenType::Punct(Token {
                    data: PunctData {
                        text: ". . .".to_string(),
                    },
                    dpy: Default::default(),
                    frm: FormatFlags::DLB,
                }));
            }
            
            let mut segment: Segment = (&tokens[..]).into();
            segment.text = plaintext;
            Some(segment)

        } else {
            None
        }
    }
}

// parametrized container elements

impl ContainerElement<Authors> {
    /// Returns the surname of the first listed author, converted to all-uppercase
    pub fn short_author_name(&self) -> Option<Segment> {
        if let Some(sn) = self.first_sn() {
            // Copy tokens, clearing all display flags and converting
            // Words to uppercase.
            let mut tokens: TokenList = Vec::with_capacity(sn.tokens.len());

            for token in sn.tokens.iter() {
                match token {
                    TokenType::Close(token) => {
                        tokens.push(TokenType::Close(Token {
                            data: token.data.clone(),
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::LineBreak(_) => {},
                    TokenType::NoteRef(_) => {},
                    TokenType::Open(token) => {
                        tokens.push(TokenType::Open(Token {
                            data: token.data.clone(),
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Punct(token) => {
                        tokens.push(TokenType::Punct(Token {
                            data: token.data.clone(),
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Space(token) => {
                        tokens.push(TokenType::Space(Token {
                            data: token.data.clone(),
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Symbol(token) => {
                        tokens.push(TokenType::Symbol(Token {
                            data: token.data.clone(),
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                    TokenType::Word(token) => {
                        tokens.push(TokenType::Word(Token {
                            data: WordData {
                                text: token.data.text.to_uppercase(),
                            },
                            dpy: Default::default(),
                            frm: token.frm,
                        }));
                    },
                }
            }
            
            Some((&tokens[..]).into())

        } else {
            None
        }
    }

    /// Navigates to the surname of the first listed author
    pub fn first_sn(&self) -> Option<&TextElement<Sn>> {
        for person in self.children.iter() {
            match person {
                ElementType::Person(person) => {
                    for sn in person.children.iter() {
                        match sn {
                            ElementType::Sn(sn) => {
                                return Some(sn);
                            },
                            _ => {},
                        }
                    }
                },
                _ => {},
            }
        }

        None
    }
}

impl ContainerElement<Head> {
    /// Navigates to the document title
    pub fn title(&self) -> Option<&TextElement<Title>> {
        for child in self.children.iter() {
            match child {
                ElementType::Title(elem) => {
                    return Some(elem);
                },
                _ => {},
            }
        }

        None
    }

    /// Navigates to the author container
    pub fn authors(&self) -> Option<&ContainerElement<Authors>> {
        for child in self.children.iter() {
            match child {
                ElementType::Authors(elem) => {
                    return Some(elem);
                },
                _ => {},
            }
        }

        None
    }
}

impl ContainerElement<Manuscript> {
    /// Navigates to the head element
    pub fn head(&self) -> Option<&ContainerElement<Head>> {
        for child in self.children.iter() {
            match child {
                ElementType::Head(elem) => {
                    return Some(elem);
                },
                _ => {},
            }
        }

        None
    }

    /// Navigates to the body element
    pub fn body(&mut self) -> Option<&mut ContainerElement<Body>> {
        for child in self.children.iter_mut() {
            match child {
                ElementType::Body(elem) => {
                    return Some(elem);
                },
                _ => {},
            }
        }

        None
    }

    /// Navigates to the Title element and calls its short_title member
    /// function.
    pub fn short_title(&self) -> Option<Segment> {
        self.head()
            .and_then(|x| x.title())
            .and_then(|x| x.short_title())
    }

    /// Navigates to the [`Authors`] element and calls its short_author_name
    /// member function.
    pub fn short_author_name(&self) -> Option<Segment> {
        self.head()
            .and_then(|x| x.authors())
            .and_then(|x| x.short_author_name())
    }
    
}
