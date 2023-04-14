// Kosik Document Reader
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

//! Reads a manuscript into memory
//!
//! This module reads manuscripts that are encoded in XML.  The driver
//! is a pushdown automaton that accepts valid instances of the
//! [manuscript schema].  Let _M_ = (Σ, Γ, _Q_, Δ, s, _F_) where
//!
//! * Σ, the **tape alphabet**, consists of the complete set of XML
//!   elements, plus the blank symbol β,
//!
//! * Γ, the **stack alphabet**, contains the parsed data from each
//!   XML element wrapped in a [`State`] struct, plus the blank symbol
//!   γ,
//!
//! * _Q_, the set of **states**, contains the parametrized [`State`]
//!   structs, plus the initial state _s_,
//!
//! * Δ, the set of **transitions**, is specified by the grammar
//!   described in the [manuscript schema],
//!
//! * _s_, the **initial state**, is the state of the machine before
//!   the first start tag or after the last end tag,
//!
//! * _F_, the set of **final states**, contains only _s_, because the
//!   stack is only empty in state _s_ (due to the fact that a valid
//!   XML document contains a single top-level element).
//!
//! Data is collected for each element while it is open, and the
//! collected data is then pushed to an [`ElementType`] variant when
//! the state is popped off the stack.  Parent states have to do
//! something with the result.  Container elements store children for
//! later traversal.  Text elements extract the tokens and add them to
//! their own token lists, discarding the children.  Footnotes are
//! stored separately and combined in the [`compositor`].  The output
//! is an [`ElementType`] hierarchy with an
//! [`ElementType::Manuscript`] at the root.
//! 
//! Thanks to [Ross] for insight into rusty pushdown automata!
//!
//! [manuscript schema]: <http://www.matchlock.com/kosik/manuscript.xsd>
//! [Ross]: <https://medium.com/swlh/rust-pushdown-automata-d37c2b1ae0c6>

use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::name::QName;

use std::str;

use crate::document::*;
use crate::text::parser::Parser;

#[macro_use]
mod macros;

/// Stack alphabet
pub enum State {
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

impl State {
    fn on_enter(&self) {}

    fn on_exit(self) -> ElementType {
        match self {
            State::Attribution(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Attribution(elem)
            },
            State::Authors(elem) => {
                ElementType::Authors(elem)
            },
            State::Backmatter(elem) => {
                ElementType::Backmatter(elem)
            },
            State::BibRef(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::BibRef(elem)
            },
            State::Blockquote(mut elem) => {
                for child in elem.children.iter_mut() {
                    match child {
                        ElementType::P(child) => {
                            State::trim_whitespace(&mut child.tokens);
                        },
                        _ => {},
                    }
                }
                
                ElementType::Blockquote(elem)
            },
            State::Body(elem) => {
                ElementType::Body(elem)
            },
            State::Br(elem) => {
                ElementType::Br(elem)
            },
            State::Chapter(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Chapter(elem)
            },
            State::Contact(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Contact(elem)
            },
            State::Div(elem) => {
                ElementType::Div(elem)
            },
            State::Em(elem) => {
                ElementType::Em(elem)
            },
            State::Footnote(mut elem) => {
                for child in elem.children.iter_mut() {
                    match child {
                        ElementType::P(child) => {
                            State::trim_whitespace(&mut child.tokens);
                        },
                        _ => {},
                    }
                }
                
                ElementType::Footnote(elem)
            },
            State::Frontmatter(elem) => {
                ElementType::Frontmatter(elem)
            },
            State::Gn(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Gn(elem)
            },
            State::Head(elem) => {
                ElementType::Head(elem)
            },
            State::Li(mut elem) => {
                for child in elem.children.iter_mut() {
                    match child {
                        ElementType::P(child) => {
                            State::trim_whitespace(&mut child.tokens);
                        },
                        _ => {},
                    }
                }
                
                ElementType::Li(elem)
            },
            State::Manuscript(elem) => {
                ElementType::Manuscript(elem)
            },
            State::NoteRef(elem) => {
                ElementType::NoteRef(elem)
            },
            State::Ol(elem) => {
                ElementType::Ol(elem)
            },
            State::P(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::P(elem)
            },
            State::PageBreak(elem) => {
                ElementType::PageBreak(elem)
            },
             State::Part(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Part(elem)
            },
            State::Person(elem) => {
                ElementType::Person(elem)
            },
            State::Prefix(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Prefix(elem)
            },
            State::Section(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Section(elem)
            },
            State::Sn(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Sn(elem)
            },
            State::Sub(elem) => {
                ElementType::Sub(elem)
            },
            State::Subtitle(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Subtitle(elem)
            },
            State::Suffix(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Suffix(elem)
            },
            State::Sup(elem) => {
                ElementType::Sup(elem)
            },
            State::Title(mut elem) => {
                State::trim_whitespace(&mut elem.tokens);
                ElementType::Title(elem)
            },
            State::Ul(elem) => {
                ElementType::Ul(elem)
            },
        }
    }

    fn on_pause(&self) {}

    fn on_resume(mut self, child: ElementType) -> Self {
        match self {
            State::Attribution(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Authors(ref mut elem) => {
                elem.children.push(child);
            },
            State::Backmatter(ref mut elem) => {
                elem.children.push(child);
            },
            State::BibRef(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Blockquote(ref mut elem) => {
                resume_mixed_content!(elem, child, LEFT_MARGIN + INDENT,
                                      RIGHT_MARGIN - INDENT);
            },
            State::Body(ref mut elem) => {
                elem.children.push(child);
            },
            State::Chapter(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Contact(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Em(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Footnote(ref mut elem) => {
                resume_mixed_content!(elem, child, LEFT_MARGIN, RIGHT_MARGIN);
            },
            State::Frontmatter(ref mut elem) => {
                elem.children.push(child);
            },
            State::Gn(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Head(ref mut elem) => {
                elem.children.push(child);
            },
            State::Li(ref mut elem) => {
                resume_mixed_content!(elem, child, LEFT_MARGIN + INDENT,
                                      RIGHT_MARGIN);
            },
            State::Manuscript(ref mut elem) => {
                elem.children.push(child);
            },
            State::Ol(ref mut elem) => {
                elem.children.push(child);
            },
            State::P(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Part(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Person(ref mut elem) => {
                elem.children.push(child);
            },
            State::Prefix(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Section(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Sn(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Subtitle(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Suffix(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Title(ref mut elem) => {
                State::resume_text_element(elem, child);
            },
            State::Ul(ref mut elem) => {
                elem.children.push(child);
            },
            _ => {},
        }

        self
    }

    fn resume_text_element<T>(elem: &mut TextElement<T>, child: ElementType) {
        match child {
            ElementType::Br(_) => {
                let token = Token {
                    data: LineBreakData {},
                    dpy: Default::default(),
                    frm: FormatFlags::MLB,
                };
                elem.tokens.push(TokenType::LineBreak(token));
            },
            ElementType::Em(child) => {
                elem.tokens.extend(child.tokens.into_iter());
            },
            ElementType::Footnote(child) => {
                let token = Token {
                    data: NoteRefData {
                        text: child.attributes.label.clone(),
                    },
                    dpy: DisplayFlags::SUP,
                    frm: Default::default(),
                };
                elem.tokens.push(TokenType::NoteRef(token));
                elem.footnotes.push(ElementType::Footnote(child));
            },
            ElementType::NoteRef(child) => {
                let token = Token::<NoteRefData> {
                    data: NoteRefData {
                        text: child.attributes.label.clone(),
                    },
                    dpy: DisplayFlags::SUP,
                    frm: Default::default(),
                };
                elem.tokens.push(TokenType::NoteRef(token));
            },
            ElementType::Sub(child) => {
                elem.tokens.extend(child.tokens.into_iter());
            },
            ElementType::Sup(child) => {
                elem.tokens.extend(child.tokens.into_iter());
            },
            _ => {},
        }
    }

    fn trim_whitespace(tokens: &mut TokenList) {
        if let Some(TokenType::Space(_)) = tokens.first() {
            tokens.remove(0);
        }

        if let Some(TokenType::Space(_)) = tokens.last() {
            tokens.pop();
        }
    }

    fn contains_only_whitespace(tokens: &[TokenType]) -> bool {
        for token in tokens {
            match token {
                TokenType::Space(_) => (),
                _ => return false,
            }
        }

        true
    }
}

/// Input driver
///
/// Accumulates a hierarchy of [`ElementType`] variants.
pub struct Reader<'a> {
    /// A [`quick_xml`] reader
    xml_reader: quick_xml::Reader<&'a [u8]>,
    stack: Vec<State>,
    next_note_no: i32,
    next_part_no: i32,
    next_chapter_no: i32,
    next_section_no: i32,
    next_li_no: Option<i32>,
    has_parts: bool,
    has_chapters: bool,
    has_sections: bool,

    /// Element accumulator
    pub root: Option<ElementType>,
    /// Word token counter
    pub word_count: usize,
}

impl<'a> Reader<'a> {
    /// Construct a new reader from an XML string
    ///
    /// # Examples
    ///
    /// ```
    /// use kosik::document::reader::Reader;
    /// let reader = Reader::new("<em>Ulysses</em>");
    /// assert!(reader.root.is_none());
    /// ```
    pub fn new(xml_string: &'a str) -> Self {
        Reader {
            xml_reader: quick_xml::Reader::from_str(xml_string),
            stack: Vec::with_capacity(16),
            next_note_no: 1,
            next_part_no: 1,
            next_chapter_no: 1,
            next_section_no: 1,
            next_li_no: None,
            has_parts: false,
            has_chapters: false,
            has_sections: false,
            root: None,
            word_count: 0,
        }
    }

    /// Push a state onto the stack
    fn push(&mut self, next: State) {
        if let Some(prev) = self.stack.last() {
            prev.on_pause();
        }

        next.on_enter();
        self.stack.push(next);
    }

    /// Pop a state off the stack
    fn pop(&mut self) {
        if let Some(prev) = self.stack.pop() {
            let elem = prev.on_exit();

            if let Some(next) = self.stack.pop() {
                self.stack.push(next.on_resume(elem));

            } else {
                self.root = Some(elem);
            }
        }
    }

    /// Process XML events
    ///
    /// # Examples
    ///
    /// ```
    /// # use kosik::document::reader::Reader;
    /// let reader = Reader::new("<em>Ulysses</em>");
    /// let root = reader.run();
    /// assert!(root.is_some());
    /// ```
    pub fn run(mut self) -> Option<ElementType> {
        loop {
            match self.xml_reader.read_event().unwrap() {
                Event::Start(ref event) => {
                    match event.local_name().into_inner() {
                        b"attribution" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Attribution {
                                line_spacing: line_spacing,
                            });

                            self.push(State::Attribution(elem));
                        },
                        b"authors" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = ContainerElement::new(Authors {
                                line_spacing: line_spacing,
                            });
                            
                            self.push(State::Authors(elem));
                        },
                        b"backmatter" => {
                            let elem = ContainerElement::new(Backmatter {
                                label: fetch_string_attr!(event, b"label")
                                    .unwrap_or("BACKMATTER".to_string()),
                            });

                            self.push(State::Backmatter(elem));
                        },
                        b"bibRef" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(BibRef {
                                line_spacing: line_spacing,
                            });

                            self.push(State::BibRef(elem));
                        },
                        b"blockquote" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);
                                
                            let elem = ContainerElement::new(Blockquote {
                                line_spacing: line_spacing,
                            });

                            self.push(State::Blockquote(elem));
                        },
                        b"body" => {
                            let elem = ContainerElement::new(Body {});
                            self.push(State::Body(elem));
                        },
                        b"chapter" => {
                            let number;
                            
	                    if let Some(n)
                                = fetch_numeric_attr!(event, b"number", i32)
                            {
	                        number = n;
                                self.next_chapter_no = number + 1;

	                    } else {
                                number = self.next_chapter_no;
                                self.next_chapter_no += 1;
	                    }

                            self.next_section_no = 1; // reset section number

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Chapter{
                                number: number,
                                line_spacing: line_spacing,
                                depth: -1,
                            });

                            self.has_chapters = true;
                            self.push(State::Chapter(elem));
                        },
                        b"contact" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Contact{
                                line_spacing: line_spacing,
                            });

                            self.push(State::Contact(elem));
                        },
                        b"em" => {
                            let elem = TextElement::new(Em {});
                            self.push(State::Em(elem));
                        },
                        b"footnote" => {
                            let label;
                            
	                    if let Some(s) = fetch_string_attr!(event, b"label") {
	                        label = s;

                                if let Ok(n) = label.parse::<i32>() {
                                    self.next_note_no = n + 1;
                                }
                                
	                    } else {
                                label = format!("{}", self.next_note_no);
                                self.next_note_no += 1;
                            }

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = ContainerElement::new(Footnote {
                                label: label,
                                line_spacing: line_spacing,
                            });

                            self.push(State::Footnote(elem));
                        },
                        b"frontmatter" => {
                            let elem = ContainerElement::new(Frontmatter {
                                label: fetch_string_attr!(event, b"label")
                                    .unwrap_or(r"FRONTMATTER".to_string()),
                            });

                            self.push(State::Frontmatter(elem));
                        },
                        b"gn" => {
                            let elem = TextElement::new(Gn {});
                            self.push(State::Gn(elem));
                        },
                        b"head" => {
                            let elem = ContainerElement::new(Head {});
                            self.push(State::Head(elem));
                        },
                        b"li" => {
                            let mut number: Option<i32> = None;
                            
                            if let Some(n) = self.next_li_no {
	                        if let Some(n) = fetch_numeric_attr!(event, b"number", i32) {
	                            number = Some(n);
                                    self.next_li_no = Some(n + 1);
	                        } else {
                                    number = Some(n);
                                    self.next_li_no = Some(n + 1);
                                }
                            }

                            let mut line_spacing = LineSpacing::Single;

                            if let Some(state) = self.stack.last() {
                                match state {
                                    State::Ol(parent) => {
                                        line_spacing = parent.attributes.line_spacing;
                                    },
                                    State::Ul(parent) => {
                                        line_spacing = parent.attributes.line_spacing;
                                    },
                                    _ => (),
                                }
                            }

	                    if let Some(value) =
                                fetch_enum_attr!(event, b"lineSpacing", LineSpacing,
                                                 |x| LineSpacing::from(x))
                            {
	                        line_spacing = value;
	                    }
                            
                            let elem = ContainerElement::new(Li {
                                number: number,
                                line_spacing: line_spacing,
                            });

                            self.push(State::Li(elem));
                        },
                        b"manuscript" => {
                            let first_page = fetch_numeric_attr!(
                                event, b"firstPage", i32
                            ).unwrap_or(1);

                            let elem = ContainerElement::new(Manuscript {
                                first_page: first_page,
                                word_count: 0,
                                has_structure: false,
                            });
                            
                            self.push(State::Manuscript(elem));
                        },
                        b"ol" => {
                            let start_no = fetch_numeric_attr!(
                                event, b"startNo", i32
                            ).unwrap_or(1);

                            self.next_li_no = Some(start_no);

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = ContainerElement::new(Ol {
                                start_no: start_no,
                                line_spacing: line_spacing,
                            });

                            self.push(State::Ol(elem));
                        },
                        b"p" => {
                            let indent = fetch_numeric_attr!(event, b"indent", usize)
                                .unwrap_or(INDENT);
                            
                            let mut line_spacing = LineSpacing::Double;
                            let mut left_margin = LEFT_MARGIN;
                            let mut right_margin = RIGHT_MARGIN;
                            
                            if let Some(state) = self.stack.last() {
                                match state {
                                    State::Blockquote(parent) => {
                                        line_spacing = parent.attributes.line_spacing;
                                        left_margin += INDENT;
                                        right_margin -= INDENT;
                                    },
                                    State::Footnote(parent) => {
                                        line_spacing = parent.attributes.line_spacing;
                                    },
                                    State::Li(parent) => {
                                        line_spacing = parent.attributes.line_spacing;
                                        left_margin += INDENT * 2;
                                    },
                                    _ => (),
                                }
                            }

	                    if let Some(value) =
                                fetch_enum_attr!(event, b"lineSpacing", LineSpacing,
                                                 |x| LineSpacing::from(x))
                            {
	                        line_spacing = value;
                            }

                            let elem = TextElement::new(P {
                                indent: indent,
                                line_spacing: line_spacing,
                                left_margin: left_margin,
                                right_margin: right_margin,
                            });

                            self.push(State::P(elem));
                        },
                        b"part" => {
                            let number;
                            
	                    if let Some(n) = fetch_numeric_attr!(event, b"number", i32) {
	                        number = n;
                                self.next_part_no = number + 1;
	                    } else {
                                number = self.next_part_no;
                                self.next_part_no += 1;
	                    }

                            self.next_chapter_no = 1; // reset chapter number
                            self.next_section_no = 1; // reset section number

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Part {
                                number: number,
                                line_spacing: line_spacing,
                                depth: -1,
                            });

                            self.has_parts = true;
                            self.push(State::Part(elem));
                        },
                        b"person" => {
                            let elem = ContainerElement::new(Person {});
                            self.push(State::Person(elem));
                        },
                        b"prefix" => {
                            let elem = TextElement::new(Prefix {});
                            self.push(State::Prefix(elem));
                        },
                        b"section" => {
                            let mut padding_before: i32 = -1;
                            
                            if let Some(State::Body(parent))
                                = self.stack.last()
                            {
                                if let Some(ElementType::Chapter(_))
                                    = parent.children.last()
                                {
                                    padding_before = 0;
                                }
                            }
                            
                            let number;
                            
	                    if let Some(n)
                                = fetch_numeric_attr!(event, b"number", i32)
                            {
	                        number = n;
                                self.next_section_no = number + 1;

	                    } else {
                                number = self.next_section_no;
                                self.next_section_no += 1;
	                    }

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);
                            
                            let elem = TextElement::new(Section {
                                number: number,
                                line_spacing: line_spacing,
                                padding_before: padding_before,
                                depth: -1,
                            });
                            
                            self.has_sections = true;
                            self.push(State::Section(elem));
                        },
                        b"sn" => {
                            let elem = TextElement::new(Sn {});
                            self.push(State::Sn(elem));
                        },
                        b"sub" => {
                            let elem = TextElement::new(Sub {});
                            self.push(State::Sub(elem));
                        },
                        b"subtitle" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Subtitle {
                                line_spacing: line_spacing,
                            });
                            
                            self.push(State::Subtitle(elem));
                        },
                        b"suffix" => {
                            let comma = fetch_bool_attr!(event, b"comma")
                                .unwrap_or(false);

                            let elem = TextElement::new(Suffix {
                                comma: comma,
                            });
                            
                            self.push(State::Suffix(elem));
                        },
                        b"sup" => {
                            let elem = TextElement::new(Sup {});
                            self.push(State::Sup(elem));
                        },
                        b"title" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Title {
                                line_spacing: line_spacing,
                            });

                            self.push(State::Title(elem));
                        },
                        b"ul" => {
                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = ContainerElement::new(Ul {
                                line_spacing: line_spacing,
                            });

                            self.push(State::Ul(elem));
                        },
                        _ => {},
                    }
                },
                Event::End(_) => self.pop(),
	        Event::Empty(ref event) => {
                    match event.local_name().into_inner() {
                        b"br" => {
                            self.push(State::Br(EmptyElement::new(Br {})));
                            self.pop();
                        },
                        b"chapter" => {
                            let number;
                            
	                    if let Some(n)
                                = fetch_numeric_attr!(event, b"number", i32)
                            {
	                        number = n;
                                self.next_chapter_no = number + 1;

	                    } else {
                                number = self.next_chapter_no;
                                self.next_chapter_no += 1;
	                    }

                            self.next_section_no = 1; // reset section number

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Chapter {
                                number: number,
                                line_spacing: line_spacing,
                                depth: -1,
                            });

                            self.push(State::Chapter(elem));
                            self.pop();
                        },
                        b"div" => {
                            self.push(State::Div(EmptyElement::new(Div {})));
                            self.pop();
                        },
                        b"noteRef" => {
                            let elem = EmptyElement::new(NoteRef {
                                label: fetch_string_attr!(event, b"label")
                                    .unwrap_or("*".to_string()),
                            });

                            self.push(State::NoteRef(elem));
                            self.pop();
                        },
                        b"pageBreak" => {
                            self.push(State::PageBreak(EmptyElement::new(PageBreak {})));
                            self.pop();
                        },
                        b"part" => {
                            let number;
                            
	                    if let Some(n) = fetch_numeric_attr!(event, b"number", i32) {
	                        number = n;
                                self.next_part_no = number + 1;
	                    } else {
                                number = self.next_part_no;
                                self.next_part_no += 1;
	                    }

                            self.next_chapter_no = 1; // reset chapter number
                            self.next_section_no = 1; // reset section number

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Part {
                                number: number,
                                line_spacing: line_spacing,
                                depth: -1,
                            });

                            self.push(State::Part(elem));
                            self.pop();
                        },
                        b"section" => {
                            let mut padding_before: i32 = -1;
                            
                            if let Some(State::Body(parent))
                                = self.stack.last()
                            {
                                if let Some(ElementType::Chapter(_))
                                    = parent.children.last()
                                {
                                    padding_before = 0;
                                }
                            }

                            let number;
                            
	                    if let Some(n)
                                = fetch_numeric_attr!(event, b"number", i32)
                            {
	                        number = n;
                                self.next_section_no = number + 1;

	                    } else {
                                number = self.next_section_no;
                                self.next_section_no += 1;
	                    }

                            let line_spacing = fetch_enum_attr!(
                                event, b"lineSpacing", LineSpacing,
                                |x| LineSpacing::from(x)
                            ).unwrap_or(LineSpacing::Single);

                            let elem = TextElement::new(Section {
                                number: number,
                                line_spacing: line_spacing,
                                padding_before: padding_before,
                                depth: -1,
                            });
                            
                            self.push(State::Section(elem));
                            self.pop();
                        },
                        _ => {},
                    }
                },
	        Event::Text(ref event) => {
                    let n: usize;
                    
                    match self.stack.pop() {
                        Some(State::Attribution(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Attribution(elem));
                        },
                        Some(State::BibRef(mut elem)) => {
                            (n, elem.tokens) =
                                self.parse_text(event, elem.tokens,
                                                Default::default());

                            self.word_count += n;
                            self.stack.push(State::BibRef(elem));
                        },
                        Some(State::Blockquote(mut elem)) => {
                            if let Some(ElementType::P(_)) =
                                elem.children.last()
                            {
                                if let Some(ElementType::P(mut wrapper))
                                    = elem.children.pop()
                                {
                                    (n, wrapper.tokens) = self
                                        .parse_text(event, wrapper.tokens,
                                                    Default::default());
                                    
                                    self.word_count += n;
                                    elem.children.push(ElementType::P(wrapper));
                                }

                            } else {
                                let mut wrapper = TextElement::new(P {
                                    indent: 0,
                                    line_spacing: elem.attributes.line_spacing,
                                    left_margin: LEFT_MARGIN + INDENT,
                                    right_margin: RIGHT_MARGIN - INDENT,
                                });

                                (n, wrapper.tokens) = self
                                    .parse_text(event, wrapper.tokens,
                                                Default::default());

                                self.word_count += n;
                                elem.children.push(ElementType::P(wrapper));
                            }

                            self.stack.push(State::Blockquote(elem));
                        },
                        Some(State::Chapter(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Chapter(elem));
                        },
                        Some(State::Contact(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Contact(elem));
                        },
                        Some(State::Em(mut elem)) => {
                            (n, elem.tokens) = self.
                                parse_text(event, elem.tokens,
                                           DisplayFlags::EM);

                            self.word_count += n;
                            self.stack.push(State::Em(elem));
                        },
                        Some(State::Footnote(mut elem)) => {
                            if let Some(ElementType::P(_))
                                = elem.children.last()
                            {
                                if let Some(ElementType::P(mut wrapper))
                                    = elem.children.pop()
                                {
                                    (n, wrapper.tokens) = self.
                                        parse_text(event, wrapper.tokens,
                                                   Default::default());

                                    self.word_count += n;
                                    elem.children.push(ElementType::P(wrapper));
                                }

                            } else {
                                let mut wrapper = TextElement::new(P {
                                    indent: 0,
                                    line_spacing: elem.attributes.line_spacing,
                                    left_margin: LEFT_MARGIN,
                                    right_margin: RIGHT_MARGIN,
                                });

                                (n, wrapper.tokens) = self.
                                    parse_text(event, wrapper.tokens,
                                               Default::default());

                                self.word_count += n;
                                elem.children.push(ElementType::P(wrapper));
                            }

                            self.stack.push(State::Footnote(elem));
                        },
                        Some(State::Gn(mut elem)) => {
                            (n, elem.tokens) = self.
                                parse_text(event, elem.tokens,
                                           Default::default());
                            
                            self.word_count += n;
                            self.stack.push(State::Gn(elem));
                        },
                        Some(State::Li(mut elem)) => {
                            if let Some(ElementType::P(_))
                                = elem.children.last()
                            {
                                if let Some(ElementType::P(mut wrapper))
                                    = elem.children.pop()
                                {
                                    (n, wrapper.tokens) = self
                                        .parse_text(event, wrapper.tokens,
                                                    Default::default());

                                    self.word_count += n;
                                    elem.children.push(ElementType::P(wrapper));
                                }

                            } else {
                                let mut wrapper = TextElement::new(P {
                                    indent: 0,
                                    line_spacing: elem.attributes.line_spacing,
                                    left_margin: LEFT_MARGIN + 2 * INDENT,
                                    right_margin: RIGHT_MARGIN,
                                });

                                (n, wrapper.tokens) = self
                                    .parse_text(event, wrapper.tokens,
                                                Default::default());

                                self.word_count += n;
                                elem.children.push(ElementType::P(wrapper));
                            }

                            self.stack.push(State::Li(elem));
                        },
                        Some(State::P(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::P(elem));
                        },
                        Some(State::Part(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                           Default::default());
                            
                            self.word_count += n;
                            self.stack.push(State::Part(elem));
                        },
                        Some(State::Prefix(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Prefix(elem));
                        },
                        Some(State::Section(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Section(elem));
                        },
                        Some(State::Sub(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            DisplayFlags::SUB);

                            self.word_count += n;
                            self.stack.push(State::Sub(elem));
                        },
                        Some(State::Suffix(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Suffix(elem));
                        },
                        Some(State::Sn(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());

                            self.word_count += n;
                            self.stack.push(State::Sn(elem));
                        },
                        Some(State::Subtitle(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());
                            
                            self.word_count += n;
                            self.stack.push(State::Subtitle(elem));
                        },
                        Some(State::Sup(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            DisplayFlags::SUP);
                            
                            self.word_count += n;
                            self.stack.push(State::Sup(elem));
                        },
                        Some(State::Title(mut elem)) => {
                            (n, elem.tokens) = self
                                .parse_text(event, elem.tokens,
                                            Default::default());
                            
                            self.word_count += n;
                            self.stack.push(State::Title(elem));
                        },
                        Some(state) => self.stack.push(state),
                        None => (),
                    }
                },
	        Event::Comment(_) => (), // ignore comments
	        Event::CData(_) => (), // not handled
	        Event::Decl(_) => (), // ignore declaration
	        Event::PI(_) => (), // not handled
	        Event::DocType(_) => (), // not handled
	        Event::Eof => break,
            }
        }

        // post-processing
        
        if let Some(elem) = &mut self.root {
            match elem {
                ElementType::Manuscript(elem) => {
                    elem.attributes.word_count = self.word_count;

                    let part_depth = if self.has_parts {
                        elem.attributes.has_structure = true;
                        0
                    } else {
                        -1
                    };

                    let chapter_depth = if self.has_chapters {
                        elem.attributes.has_structure = true;
                        if part_depth >= 0 { 1 } else { 0 }
                    } else {
                        -1
                    };

                    let section_depth = if self.has_sections {
                        elem.attributes.has_structure = true;
                        if part_depth >= 0 && chapter_depth >= 0 { 2 } else { 1 }
                    } else {
                        -1
                    };

                    if let Some(body) = elem.body() {
                        for child in body.children.iter_mut() {
                            match child {
                                ElementType::Chapter(child) => {
                                    child.attributes.depth = chapter_depth;
                                },
                                ElementType::Part(child) => {
                                    child.attributes.depth = part_depth;
                                },
                                ElementType::Section(child) => {
                                    child.attributes.depth = section_depth;
                                },
                                _ => (),
                            }
                        }
                    }
                },
                _ => (),
            }
        }
        
        self.root
    }

    fn parse_text(&mut self, event: &BytesText, tokens: TokenList, dpy: DisplayFlags)
        -> (usize, TokenList)
    {
        let text = event.unescape().unwrap();
        let parser = Parser::new(&text, tokens, dpy);
        parser.run()
    }
}    
