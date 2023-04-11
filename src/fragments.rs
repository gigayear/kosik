// Kosik Fragments Module
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

macro_rules! write_block {
    ($elem:ident, $name:literal, &$args:ident) => {
        if $args.elements {
            println!("{:?}", &$elem);

            if !$args.blocks {
                return Ok(());
            }
        }

        let block: Block = $elem.into();

        if $args.blocks {
            println!("{:?}", &block);
        }

        if $args.elements || $args.blocks {
            return Ok(());
        }

        let mut compositor = Compositor::new(1, false);
        compositor = compositor.run(vec![block]);

        let typescript = Typescript {
            contact: None,
            word_count: None,
            has_structure: false,
            short_title: Segment::from(&$name[..]),
            short_author_name: Segment::from(&(*PROGRAM_NAME)[..]),
            pages: compositor.pages,
        };

        let mut writer = Writer::new(&typescript);
        writer.run()?;
    };
}

macro_rules! write_container {
    ($elem:ident, $name:literal, &$args:ident) => {
        if $args.elements {
            println!("{:?}", &$elem);

            if !$args.blocks {
                return Ok(());
            }
        }

        let blocks: BlockList = $elem.into();

        if $args.blocks {
            println!("{:?}", &blocks);
        }

        if $args.elements || $args.blocks {
            return Ok(());
        }

        let mut compositor = Compositor::new(1, false);
        compositor = compositor.run(blocks);

        let typescript = Typescript {
            contact: None,
            word_count: None,
            has_structure: false,
            short_title: Segment::from(&$name[..]),
            short_author_name: Segment::from(&(*PROGRAM_NAME)[..]),
            pages: compositor.pages,
        };

        let mut writer = Writer::new(&typescript);
        writer.run()?;
    };
}
