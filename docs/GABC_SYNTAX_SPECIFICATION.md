# Complete Specification of .gabc File Syntax

This document describes the complete syntax of `.gabc` files as specified in the official Gregorio documentation (GregorioRef.tex).

## 1. General File Structure

A `.gabc` file has the `.gabc` extension and has the following structure:

```
name: incipit;
gabc-copyright: copyright on this gabc file;
score-copyright: copyright on the source score;
author: if known; % maybe some additional comment
language: latin;
mode: 6;
mode-modifier: t.;
annotation: IN.;
annotation: 6;

%%

(clef) text(notes)
% another comment
com(notes)plex(notes) word(notes)
```

The file is divided into two sections separated by `%%`:

1. **Header section**: Contains metadata about the score
2. **Notation section**: Contains the score itself

Spaces in the notation section are significant, and the end of line is considered a space.

### Comments

The `%` character marks the beginning of a comment that ends at the end of the line. A comment also suppresses the end-of-line space. Comments may appear in any section of the document.

## 2. Headers

Headers have the form `name: value;`, where:
- The header name is composed of ASCII letters and numbers, optionally separated by dashes
- To write a value over several lines, omit the semicolon at the end of the first line and end the header value with `;;` (two semicolons)

### Headers with Special Meaning

#### `name`
Name of the piece, usually the incipit (first few words). **This field is required.**

#### `gabc-copyright`
Copyright notice (in English) of the gabc file, chosen by the person named in the transcriber field.

#### `score-copyright`
Copyright notice (in English) of the original score from which the gabc was transcribed.

#### `author`
Author of the piece, if known.

#### `language`
Language of the lyrics. Supported languages: Latin, English, Church Slavonic, Hungarian, Polish, Czech, Slovak. Can be specified by name or ISO 639 code.

#### `mode`
Mode of the piece. Normally an Arabic number between 1 and 8, but may be any text for unusual cases. The mode number will be converted to Roman numerals and placed above the initial, unless:
- There is a `\greannotation` defined immediately before `\gregorioscore`
- The `annotation` header field is defined

#### `mode-modifier`
Mode modifier of the piece. May be any \TeX{} code to be typeset after the mode.

#### `mode-differentia`
Mode or tone differentia of the piece. Typically expresses the variant of the psalm tone to use. May be any \TeX{} code to be typeset after the mode-modifier.

#### `annotation`
Text to appear above the initial letter. Usually an abbreviation of the office-part in the upper line and an indication of the mode (and differentia for antiphons) in the lower. Either one or two `annotation` fields may be used; if two are used, the first is the upper line, the second the lower.

#### `staff-lines`
Number of lines in the staff.

#### `nabc-lines`
Number of NABC lines in the staff; currently only 1 NABC line is supported.

#### `oriscus-orientation`
If set to `legacy`, Gregorio will use the older oriscus orientation semantics. Leave the header out to use the default oriscus orientation semantics.

#### `def-m`*n*
Defines \TeX{} code to be used for the numbered macro (from 0-9). See the macros section below.

### Other Suggested Headers

Although Gregorio ascribes no special meaning to them, other suggested headers include:
- `office-part`: Category of chant (antiphona, hymnus, responsorium, etc.)
- `occasion`: Liturgical occasion
- `meter`: Meter for hymns
- `commentary`: Notes about the source of the text
- `arranger`: Name of a modern arranger
- `date`: Date of composition
- `manuscript`: Manuscript identification
- `manuscript-reference`: Unique reference for the piece
- `manuscript-storage-place`: Location where the manuscript is held
- `book`: Name of the modern book
- `transcriber`: Name of the transcriber into gabc
- `transcription-date`: Date the gabc was written (format yyyymmdd)
- `user-notes`: Any additional text

## 3. Notation - Syllable Text Syntax

Text outside parentheses is considered syllable text. In general, this is the text that appears below the staff. Parenthesized note sections separate syllables.

### Special Characters

- `$`: Escape character that causes the following character to lose any special meaning. Use `$(` to insert parentheses in text. To insert a `$` in text, escape it (`$$`).
- `{` and `}`: For lyric centering (see lyric centering section)
- `[` and `]`: For translation text that appears below the lyric text

### Style Tags

HTML-like tags to apply basic styling:

- `<b>...</b>`: Bold text
- `<c>...</c>`: Colored text using the `gregoriocolor` color
- `<i>...</i>`: Italic text
- `<sc>...</sc>`: Small capitals
- `<tt>...</tt>`: Teletype (monospaced)
- `<ul>...</ul>`: Underlined text

### Syllable Controls

Tags to control the interaction between syllables or line breaks:

- `<clear>` or `<clear/>`: Indicates that the syllable may not extend into the notation of the previous syllable
- `<e>...</e>`: Elision (typeset in italics by default)
- `<eu>...</eu>`: "Euouae" text for tone differentia marking
- `<nlba>...</nlba>`: No line break area
- `<pr>` or `<pr/>`: Protrusion (may push into the margin). Optionally followed by `:` and a number between 0 and 1 indicating how much of the marked text may push into the margin (e.g.: `<pr:.5>`)

### Other Tags

- `<alt>...</alt>`: "Above lines" text that will be typeset above the staff lines
- `<sp>...</sp>`: Special character, as defined by `\gresetspecial`
- `<v>...</v>`: Text passed directly to \TeX{}, without preprocessing as gabc text

### Lyric Centering

Gregorio centers the text of each syllable around the first note of each syllable. Three basic modes:

- `syllable`: The entire syllable is centered around the first note
- `firstletter`: The first letter of the syllable is centered around the first note
- `vowel`: The vowel sound of the syllable is centered around the first note (default)

All modes allow you to force the centering with curly brackets, for example `a{b}c` will center the notes around `b`.

### Translation Text

Translation text, enclosed in square brackets `[` and `]`, appears below the lyric text (by default) in italics. It is aligned to the syllable where it appears in the gabc file, unless the special sequence `[/]` appears in a later syllable, indicating that the translation text should be centered between those two points.

### Predefined Special Characters

The following special characters are defined by default:

- `ae`: æ ligature
- `oe`: œ ligature
- `'ae`: Accented æ (ǽ)
- `'oe`: Accented œ (œ́)
- `'æ`: Accented æ (ǽ)
- `'œ`: Accented œ (œ́)
- `A/`: A with a bar (Ā), typically used to signify the antiphon
- `R/`: R with a bar (R̄), typically used to signify the refrain or response
- `V/`: V with a bar (V̄), typically used to signify the verse
- `*`: Character produced by `\GreStar` (✠)
- `+`: Character produced by `\GreDagger` (†)
- `-`: Zero-width hyphen
- `\`: Backslash, avoiding \TeX{} interpretation
- `&`: Ampersand, avoiding \TeX{} interpretation
- `#`: Hash mark, avoiding \TeX{} interpretation
- `_`: Underscore, avoiding \TeX{} interpretation
- `~`: Centered tilde (a "math" tilde)

## 4. Notation - Note Syntax

In the notation section of the gabc file, notes and other figures that appear on the staff are specified within parentheses.

### Pitches

Pitches are represented by a single letter from `a` to `n` and `p`. Each letter represents the typographical position of the pitch, regardless of the clef position.

The available letters depend on the number of staff lines (set by the `staff-lines` header): three pitches are allowed above the top staff line.

- 2-line staff: supports `a`-`i`
- 3-line staff: supports `a`-`k`
- 4-line staff (default): supports `a`-`m`
- 5-line staff: supports `a`-`n` and `p`

### One-Note Neumes

A lowercase pitch letter represents a *punctum quadratum* (square note). A capitalized pitch letter represents a *punctum inclinatum* (diamond-shaped note).

Other shapes are created by appending various characters to the pitch letter:

| GABC | Description |
|------|------------|
| *pc* | punctum quadratum |
| *Pc* | punctum inclinatum (automatic leaning based on surrounding pitches) |
| *Pc*0 | left-leaning (descending) punctum inclinatum |
| *Pc*1 | right-leaning (ascending) punctum inclinatum |
| *Pc*2 | non-leaning (unison) punctum inclinatum |
| *pc*o | oriscus |
| *pc*w | quilisma |
| *pc*v | virga (stem on right) |
| *pc*V | virga reversa (stem on left) |
| *pc*s | stropha |
| *pc*~ | liquescent deminutus (small note) |
| *pc*< | augmented liquescent |
| *pc*> | diminished liquescent |
| *pc*= | linea |
| *pc*r | cavum (hollow note) |
| *pc*R | punctum quadratum surrounded by lines |
| *pc*r0 | punctum cavum surrounded by lines |
| *pc*x | flat |
| *pc*# | sharp |
| *pc*y | natural |
| *pc*x? | parenthesized flat |
| *pc*#? | parenthesized sharp |
| *pc*y? | parenthesized natural |
| *pc*X | soft flat |
| *pc*## | soft sharp |
| *pc*Y | soft natural |

*pc* represents a pitch character and *Pc* represents a capitalized pitch character.

### Alterations (Flats, Naturals, and Sharps)

Flats, sharps, and naturals have "soft" versions (`X`, `##`, and `Y`, respectively). Soft flats are printed if no previous flat (on the same pitch) is in effect. Similarly for soft sharps. Soft naturals are printed if a previous flat or sharp (on the same pitch) is in effect.

### Oriscus Orientation

Under the default rules for oriscus orientation, the direction of an oriscus (pointing upwards or downwards) depends on whether the first non-unison note that follows is higher or lower than the oriscus. However, if Gregorio\TeX{} does not produce the desired oriscus, the direction may be explicitly selected using `o0` or `O0` for a downwards-pointing oriscus and `o1` or `O1` for an upwards-pointing oriscus.

### Complex Neumes

Neumes with more than one note are simply made by chaining notes together without worrying about the connections between them.

In addition to the characters for one-note neumes, some additional characters are available to adjust the shape:

| GABC | Description |
|------|------------|
| `-`*pc* | (prior to the pitch it modifies) initio debilis |
| *pc*`O` | oriscus scapus (an oriscus with stems that connect to the note prior to it) |
| *pc*`q` | quadratum (for making a "square" pes shape) |
| *pc*`W` | quilisma quadratum (similarly, for making a "square" quilisma shape) |
| *pc*`ss` | distropha |
| *pc*`sss` | tristropha |
| *pc*`vv` | bivirga |
| *pc*`vvv` | trivirga |

### Neume Fusion

Neume fusion allows for the composition of new shapes based on a set of primitive neumes. Placing the `@` character between two notes will attempt to use the fusion rules to fuse the notes together.

Placing the `@` character before a primitive that would get a stem will suppress the stem.

As a convenience, a sequence of notes enclosed within `@[` and `]` will be fused automatically based on an algorithm that breaks up the notes into the above primitives.

### Neume Spacing

For musical phrases that consist of multiple neumes, various spaces may be added to the notes:

| GABC | Description |
|------|------------|
| `/0` | a half space that is considered part of the same neume |
| `/!` | a small separation that is considered part of the same neume |
| `/` | a small separation between neumes |
| `//` | a medium separation between neumes |
| *space* | a large separation between neumes |
| `/[`*factor*`]` | a space with the size of the large separation scaled by the given *factor* (which may be negative, resulting in a backspace) |
| `!` | if alone, a zero-width space used when Gregorio\TeX{} does not break the chain of notes in the correct place; if followed by a space, makes the space that follows a non-breaking space |

### Shape Hints

In some cases, an alternate form of a shape is desired. To do this, put a `[shape:`*hint*`]` after the figure that needs to be altered.

| *hint* | Description |
|--------|------------|
| `stroke` | Renders a clivis/flexus as a stroke (like in a porrectus) rather than as two notes |

### Additional Symbols

Puncta mora, episemata, and other symbols may also be added to a note by adding various other characters:

| GABC | Description |
|------|------------|
| *pc*`.` | punctum mora |
| *pc*`_` | horizontal episema (see horizontal episemata section below) |
| *pc*`'` | vertical episema / ictus (automatic placement) |
| *pc*`'0` | vertical episema / ictus below the note |
| *pc*`'1` | vertical episema / ictus above the note |
| *pc*`r1` | accent above staff |
| *pc*`r2` | grave accent above staff |
| *pc*`r3` | circle above staff |
| *pc*`r4` | lower semicircle above staff |
| *pc*`r5` | upper semicircle above staff |
| *pc*`r6` | musica ficta flat |
| *pc*`r7` | musica ficta natural |
| *pc*`r8` | musica ficta sharp |

### Rhythmic Signs

#### Punctum Mora

The *punctum mora* (dots after the note) are denoted by adding a period (`.`) after the note. Next to a note on a line, Gregorio will place the dot above or below the line according to the context, but you can force its position with `.0` (below) or `.1` (above). When there are two *punctum mora* after a neume, the character is simply doubled (`..`). You cannot place more than two *punctum mora* after a note.

#### Ictus

The *ictus* (also called the vertical *episema*) is denoted by an apostrophe (`'`) after the note. Gregorio will place the episema above or below the note according to the context, however you can force its position with `'0` (always below) and `'1` (always above).

The *ictus* can be present more than once in a neume, but only once per note.

#### Horizontal Episemata

For the horizontal *episema*, type an underscore (`_`) after every note that is under an episema. A horizontal episema may be adjusted with the addition of numbers. Multiple numbers may be added to combine their effects:

| GABC | Description |
|------|------------|
| `0` | place the episema below the note |
| `1` | place the episema above the note |
| `2` | disable bridging the episema with the following episema |
| `3` | use a small episema, aligned left |
| `4` | use a small episema, aligned center |
| `5` | use a small episema, aligned right |

### Separation Bars

Bars separate sections of the chant:

| GABC | Description |
|------|------------|
| `` ` `` | virgula |
| `` `0 `` | virgula on the ledger line above the staff |
| `^` | divisio "minimis" (eighth bar) |
| `^0` | divisio "minimis" (eighth bar) |
| `,` | divisio minima (quarter bar) |
| `,0` | divisio minima on the ledger line above the staff |
| `;` | divisio minor (half bar) |
| `:` | divisio maior (full bar) |
| `:?` | dotted divisio maior |
| `::` | divisio finalis |
| `;`*n* | Dominican bar, where *n* is the position, from 1-8; depending on the number of staff lines, some Dominican bars will not be available |

Bars can also take the following characters for additional symbols:

| GABC | Description |
|------|------------|
| `'` | vertical episema |
| `_` | bar brace |

### Clefs

The syntax for a clef is a letter corresponding to the clef symbol, `c` or `f`, followed optionally by `b` if the clef should have a flat (only supported for C-clefs: `cb1`, `cb2`, `cb3`, `cb4`), followed by a number from `1` up to the number of staff lines indicating the line on which the clef is typeset.

**Supported clefs:**
- C-clefs: `c1`, `c2`, `c3`, `c4`
- F-clefs: `f1`, `f2`, `f3`, `f4`
- C-clefs with flat: `cb1`, `cb2`, `cb3`, `cb4`

Two clefs may be typeset at the same time by linking the two clefs with a `@`. The two clefs will be typeset in such a way that they don't collide with each other.

### Custos

Gregorio\TeX{} typesets the custos automatically by default. This may be switched off using the `\greseteolcustos` command. Additionally, a custos whose pitch depends on the note that follows may be typeset anywhere by entering `z0` or at any pitch by entering a `+` after the desired pitch character (e.g., `g+` for a custos at the `g` pitch).

`[nocustos]` will prevent a custos from appearing at the point where specified, if line formatting causes a line break there. The `[nocustos]` tag must appear *before* spaces appearing at the point specified or it will have no effect.

### Line Breaks

Gabc has a few codes to control line breaks:

| GABC | Description |
|------|------------|
| `z` | insert a justified line break whose custos depends on the `\greseteolcustos` configuration |
| `z+` | insert a justified line break with a forced automatic custos |
| `z-` | insert a justified line break without a custos |
| `Z` | insert a ragged line break whose custos depends on the `\greseteolcustos` configuration |
| `Z+` | insert a ragged line break with a forced automatic custos |
| `Z-` | insert a ragged line break without a custos |
| `<nlba>` | mark the beginning of a set of neumes where no line breaks are allowed |
| `</nlba>` | mark the end of a set of neumes where no line breaks are allowed |

A line break at the very end of a score is discouraged, and whether the last line is justified or ragged should be specified by `\gresetlastline` instead.

### Choral Signs

Choral signs (text near the note in the staff) may be typeset by using `[cs:`*choral sign text*`]`.

An NABC choral sign may be typeset by using `[cn:`*nabc neume code*`]`.

### Braces

To typeset a brace, use `[`*type*`:`*n*`;`*size*`]` after some neume. The *type* may be `ob` for a round brace above the staff, `ub` for a round brace below the staff, `ocb` for a curly brace above the staff, or `ocba` for a curly brace with accent above the staff. If *n* is `0`, the brace will begin at the end of the neume. If *n* is `1`, the brace will begin at the start of the neume. The *size* should be a \TeX{} size unit.

Alternately, use `[`*type*`:`*n*`{]` followed by `[`*type*`:`*n*`}]` after some later neume. The *type* and *n* are as above, but this form typesets a brace with the endpoints thus defined.

### Stem length for the bottom lines

Gregorio will determine the length of the stem for most neumes. Some manual input might be needed for notes on the bottom staff line (*d*). Most of the time they will take a short form, but when a ledger line is drawn below these notes, they should take a long form. To solve this problem, you can add `[ll:0]` to the note carrying the stem to get its short form, or `[ll:1]` to force its long form.

### Custom Ledger Lines

To specify a custom ledger line, use `[oll:`*left*`;`*right*`]` to create an over-the-staff ledger line with specified lengths to the left and right of the point where it is introduced. If *left* is `0`, the ledger line will start at the introduction point. If *left* is `1`, the ledger line will start at the `additionaallineswidth` distance to the left of the introduction point. Otherwise, the line will start at the *left* distance (taken to be an explicit length, with \TeX{} units required) to the left of the introduction point. When using this form, *right* must be an explicit length to the right of the introduction point at which to end the line.

Alternately, use `[oll:`*left*`{`*right*`]` to specify the start of an over-the-staff ledger line, followed by `[oll:]}` at some point later to specify its end. When using this form, *left* has the same meaning as before. However, *right* takes on similar values as *left*, which are instead applied to the right of the specified endpoint.

Use `ull` instead of `oll` (with either form) to create an under-the-staff ledger line.

### Simple Slurs

To specify a simple slur, use `[oslur:`*shift*`;`*width*`,`*height*`]` to create an over-the-notes slur with the specified *width* and *height*. If *shift* is `0`, the slur will start on the right side of the note to which it is attached. If *shift* is `1`, the slur will start one punctum's width to the left of the right side of the note to which it is attached. If *shift* is `2`, the slur will start one-half punctum's width to the left of the right side of the note to which it is attached.

Alternately, use `[oslur:`*shift*`{]` to specify the start of an over-the-notes slur, followed by `[oslur:`*shift*`}]` at some point later to specify its end. When using this form, *shift* has the same meaning as before, but applies to both ends of the slur.

Use `uslur` instead of `oslur` (with either form) to create an under-the-staff slur.

### Horizontal episema placement for very high and low notes

Gregorio places horizontal episema under `c` and above `k` (or the note above the topmost line when staff does not have exactly 4 lines) closer to the notes when no ledger line is present. The heuristics used by Gregorio are not perfect so it may be necessary to make the presence or absence of ledger line explicit for horizontal episema placement. This is done in the exact same way as for stem length: place `[ll:0]` or `[ll:1]` on the note carrying the episema, to force Gregorio to consider the absence or presence of a ledger line in episema placement.

### Horizontal Episema Tuning

The horizontal episema position within the space can be adjusted should the defaults not be satisfactory.

There are five tunable dimensions:
- `overhepisemalowshift`: The shift for positioning a horizontal episema that is over a note in a low position in the space
- `overhepisemahighhift`: The shift for positioning a horizontal episema that is over a note in a high position in the space
- `underhepisemalowshift`: The shift for positioning a horizontal episema that is under a note in a low position in the space
- `underhepisemahighhift`: The shift for positioning a horizontal episema that is under a note in a high position in the space
- `hepisemamiddleshift`: The shift for centering the horizontal episema in the middle of a space

In addition, gabc allows you to adjust the positioning of a given episema by appending `[oh:`*p*`]` (for the episema over the note) or `[uh:`*p*`]` (for the episema under the note). Here, *p* is an optional position specifier followed by an optional nudge. However at least one or the other must be specified.

The position specifier allows you to select which of the five tunable dimensions will be used for the base position:
- *omitted*: Uses the default shift based on the position of the episema relative to the note
- `m`: Uses `hepisemamiddleshift`
- `l`: Uses `overhepisemalowshift` or `underhepisemalowshift` depending on whether the episema is over or under the note
- `h`: Uses `overhepisemahighshift` or `underhepisemahighhift` depending on whether the episema is over or under the note
- `ol`: Uses `overhepisemalowshift`
- `oh`: Uses `overhepisemahighshift`
- `ul`: Uses `underhepisemalowshift`
- `uh`: Uses `underhepisemahighhift`

The nudge is a \TeX{} dimension specification (number and units) that starts with `+` for a nudge upwards or `-` for a nudge downwards from the base position selected by the position specifier. If omitted, the episema will be drawn at the base position.

In addition, gabc also allows you to specify that a block of notes—possibly separated with spaces and in different syllables—should be considered a single unit when it comes to positioning the horizontal episema. To do this, put `[oh:`*p*`{]` (for the episema over the note) or `[uh:`*p*`{]` (for the episema under the note) before the first note of the block and the corresponding `[oh]}` or `[uh]}` after the last note of the block. When using this syntax, *p* is the position specifier as before, but is entirely optional, and when completely omitted, allows the `:` to also be omitted.

### Above Lines Text Within Notes

"Above lines text" may also be specified within the notes for better positioning. To do this, use `[alt:`*text*`]`.

### Verbatim \TeX{}

\TeX{} may be inserted directly within the notes. This is an advanced command and is not easy to use, but it can allow for some interesting tweaks and special effects.

To do this, use `[nv:`*tex code*`]` to insert \TeX{} code at the note level, `[gv:`*tex code*`]` to insert \TeX{} code at the glyph level, or `[ev:`*tex code*`]` to insert \TeX{} code at the element level.

Here, note, glyph, and element refer to how Gregorio understands the score.

### Macros

Macros may be defined using the `def-m`*n* headers, where *n* is 0-9, as described in the headers section above. Then they may be inserted into the score at the note level using `[nm`*n*`]`, at the glyph level using `[gm`*n*`]`, or at the element level using `[em`*n*`]`. For backwards compatibility reasons, Gregorio will also accept `[altm`*n*`]` to invoke the macro at the element level.

As before, note, glyph, and element refer to how Gregorio understands the score.

## 5. Vowel file

When run, Gregorio will look for a file named `gregorio-vowels.dat` in your working directory or amongst the GregorioTeX files. If it finds the language requested by the header (matched in a *case-sensitive* fashion) in one of these files (henceforth called vowel files), Gregorio will use the rules contained within for vowel centering. If it cannot find the requested language in any of the vowel files or is unable to parse the rules, Gregorio will fall back on the Latin rules.

The vowel file is a list of statements, each starting with a keyword and ending with a semicolon (`;`). Multiple statements with the same keyword are allowed, and all will apply. Comments start with a hash symbol (`#`) and end at the end of the line.

In general, Gregorio does no case folding, so the keywords and language names are case-sensitive and both upper- and lower-case characters should be listed after the keywords if they should both be considered in their given categories.

The keywords are:

### `alias`
The `alias` keyword indicates that a given name is an alias for a given language. The `alias` keyword must be followed by the name of the alias (enclosed in square brackets), the `to` keyword, the name of the target language (enclosed in square brackets), and a semicolon.

### `language`
The `language` keyword indicates that the rules which follow are for the specified language. It must be followed by the language name, enclosed in square brackets, and a semicolon. The language specified applies until the next language statement.

### `vowel`
The `vowel` keyword indicates that the characters which follow, until the next semicolon, should be considered vowels.

### `prefix`
The `prefix` keyword lists strings of characters which end in a vowel, but when followed by a sequence of vowels, *should not* be considered part of the vowel sound. These strings follow the keyword and must be separated by space and end with a semicolon. Examples of prefixes include *i* and *u* in Latin and *qu* in English.

### `suffix`
The `suffix` keyword lists strings of characters which don't start with a vowel, but when appearing after a sequence of vowels, *should* be considered part of the vowel sound. These strings follow the keyword and must be separated by space and end with a semicolon. Examples of suffixes include *w* and *we* in English and *y* in Spanish.

### `secondary`
The `secondary` keyword lists strings of characters which do not contain vowels, but for which, when there are no vowels present in a syllable, define the center of the syllable. These strings follow the keyword and must be separated by space and end with a semicolon. Examples of secondary sequences include *w* from Welsh loanwords in English and the syllabic consonants *l* and *r* in Czech.

## References

This specification is based on the official Gregorio documentation, available at:
- GregorioRef.tex (source code)
- GregorioRef.pdf (compiled document available at ctan.org)

For more information, see:
- [Gregorio project website](http://gregorio-project.github.io/)
- [GitHub repository](http://github.com/gregorio-project/gregorio)

