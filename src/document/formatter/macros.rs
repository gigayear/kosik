// Kosik Formatter Macros
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

macro_rules! format_toc_entry {
    ($elem:ident, $tag:expr) => {
        {
            let tag_length = $tag.chars().count();

            let indent = if $elem.attributes.depth == 2 {
                INDENT * 3
            } else if $elem.attributes.depth == 1 {
                INDENT * 2
            } else {
                INDENT 
            };

            // Filter note references.
            let tokens = $elem.tokens.iter().filter_map(|t| match t {
                TokenType::NoteRef(_) => None,
                t => Some(t.clone()),
            }).collect::<TokenList>();
            
            let line_length = RIGHT_MARGIN - LEFT_MARGIN - INDENT * 2 - indent;
            let mut lines = text::linebreak_fill(&tokens[..], line_length);
            let spaces = repeat(' ').take(indent).collect::<String>();

            for (i, line) in lines.iter_mut().enumerate() {
                line.column = LEFT_MARGIN;
                
                if i > 0 {
                    line.segments.insert(0, Segment::from(&spaces[..]));
                } else {
                    let spaces_before = repeat(' ')
                        .take(indent - INDENT)
                        .collect::<String>();
                    
                    let spaces_after = if INDENT as i32 - tag_length as i32 - 2 > 0 {
                        repeat(' ').take(INDENT - tag_length - 2).collect::<String>()
                    } else {
                        "".to_string()
                    };

                    let prefix = Segment::from(format!("{}{}. {}", spaces_before,
                                                       $tag, spaces_after));
                    line.segments.insert(0, prefix);
                }
            }
            
            Block {
                lines: lines,
                footnotes: Vec::new(),
                line_spacing: LineSpacing::Single,
                padding_before: 0,
                padding_after: 1,
                tag: Some(Tag::ToC),
            }
        }
    };
    ($label:expr) => {
        {
            let line = Line {
                column:  LEFT_MARGIN,
                segments: vec![Segment::from($label)],
                note_refs: Vec::new(),
            };
                
            Block {
                lines: vec![line],
                footnotes: Vec::new(),
                line_spacing: LineSpacing::Single,
                padding_before: 0,
                padding_after: 1,
                tag: Some(Tag::ToC),
            }
        }
    };
}
