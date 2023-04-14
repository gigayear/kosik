# kosik

An XML-to-Postscript converter for creating typed manuscripts on
a home or office printer.

Produces typewriter output in standard manuscript format.  Some
useful applications of manuscript format:

  * Manuscripts are used for composition.

  * Manuscripts are used for sharing texts with co-authors,
    readers, agents, and editors.

  * Manuscripts are used for copy editing.

  * Electronic or paper manuscripts can serve as a spec format for
    the production pipeline.

  * Manuscript format is used to make hard copies for storage.

  * Manuscripts are used for underground distribution.

From the perspective of the author, preventing the loss of your
work is the most important job of a word processing system.  To
that end, XML is a reliable encoding because it uses plain text,
it is non-proprietary, it is compatible with version control, and
it is very widely used.

This crate is named after an elephant from South Korea who is
reputed to possess the power of speech.

Pronunciation (IPA): ‘kəʊ,ʃɪk

## Examples

Processing a valid document, an encoding of the short story
_Youth_, by Joseph Conrad:

```sh
$ head -4 conrad.sik
<?xml version="1.0" encoding="utf-8"?>
<manuscript
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:noNamespaceSchemaLocation="http://www.matchlock.com/kosik/manuscript.xsd">
$ wc -l conrad.sik
1308 conrad.sik
$ xmllint --noout --schema manuscript.xsd conrad.sik
conrad.sik validates
$ kosik conrad.sik > conrad.ps
$ head -6 conrad.ps
%!PS
%%Title: Youth
%%Creator: kosik
%%DocumentFonts: Courier
%%BoundingBox: 0 0 612 792
%%Pages: 45
$
```

Output: [`conrad.pdf`]

Kosik can also show you its internal element representation using
the <tt>-e</tt> flag, and it works on fragments of the manuscript
schema:

```sh
$ cat minimal.sik
<br/>
$ kosik -e minimal.sik
EmptyElement { attributes: Br }
```

If you use the <tt>-b</tt> flag, Kosik will show you the internal
block representation.  In the example below, the output has been
formatted to make it more readable.  The actual output comes out
all on one line.

```sh
$ kosik -b minimal.sik
Block {
     lines: [
          Line {
               column: 10,
               segments: [Segment { text: "", ps: "() show " }],
               note_refs: []
          }
     ],
     footnotes: [],
     line_spacing: Single,
     padding_before: 0,
     padding_after: 0,
     tag: None
}
```

If you don't use either the <tt>-e</tt> nor the <tt>-e</tt> flags,
Kosik will render the individual element in Postscript.  In all
cases, a single top-level element is expected.

[`conrad.pdf`]: <http://www.matchlock.com/kosik/conrad.pdf>
