# Complete Specification of NABC Extension for GABC Notation

This document describes the complete specification of the NABC (Cardine-Based Adiastematic Notation) extension for GABC notation, as specified in the official Gregorio documentation (GregorioNabcRef.tex).

## 1. Introduction

The `nabc` language provides the ability to describe some adiastematic neumes, currently just the St. Gallen and Laon (Metz notation family) styles. The language is partially based on Dom Eugène Cardine's Table of neumatic signs, but for more complex neumes doesn't always match how the neumes are called; instead attempts to make it easier to compose complex neumes from basic glyphs.

## 2. Basic Usage

To describe adiastematic neumes in `gabc`, the header should contain the line `nabc-lines: 1;`, like:

```
nabc-lines: 1;
(f3) AL(ef~|ta>)le(fg/hggf|peclhgpi)lú(ef~|ta>)ia.(f.|ta-) (,)
(ii//|bv-|gh!ivHG//|vi-hhppu2su1sut1|fhg/|to|eef.|pt) (;)
```

The `nabc` snippets are then separated by the `|` character from `gabc` snippets or other `nabc` snippets. Every `gabc` snippet may be followed by multiple `nabc` snippets. The maximum number of consecutive `nabc` snippets is the number declared in the header field `nabc-lines: x;`. After reaching that number of consecutive `nabc` snippets, another `gabc` snippet followed by `nabc` snippets can follow.

A single `nabc` snippet is not split into multiple lines, so for larger melismatic pieces it is desirable to synchronize the `gabc` snippets with corresponding `nabc` snippets.

- With `nabc-lines: 1;`, the `gabc` and `nabc` snippets form an alternating pattern, like `(gabc|nabc|gabc|nabc|gabc)`, in this case the last `gabc` snippet does not have any corresponding `nabc` neumes.
- With `nabc-lines: 2;`, the snippets ordering could be, for example, `(gabc|nabc1|nabc2|gabc|nabc1)`.

## 3. Structure of an NABC Snippet

Each `nabc` snippet consists of a sequence of *complex neume descriptors*. Each *complex neume descriptor* consists of:

1. Optionally, a *horizontal spacing adjustment descriptor*
2. A *complex glyph descriptor*
3. Optionally, a sequence of *subpunctis and prepunctis descriptors*
4. Optionally, a sequence of *significant letter descriptors*

## 4. Horizontal Spacing Adjustment Descriptor

The *horizontal spacing adjustment descriptor* consists of a sequence of horizontal spacing adjustment characters `/` and `` ` ``.

| Character | Description |
|-----------|-------------|
| `//` | Move to the right by `nabclargerspace` |
| `/` | Move to the right by `nabcinterelementspace` |
| ` `` ` | Move to the left by `nabclargerspace` |
| `` ` `` | Move to the left by `nabcinterelementspace` |

## 5. Complex Glyph Descriptor

The *complex glyph descriptor* consists of:

1. A *glyph descriptor*, optionally followed by a sequence of other *glyph descriptors*, all separated by the `!` character. This is used to describe more complex glyphs, where certain basic glyphs are connected together.
2. Optionally, one or more *subpunctis and prepunctis descriptors*.
3. Optionally, one or more *significant letter descriptors*.

Each *glyph descriptor* consists of:

1. A *basic glyph descriptor*
2. Optionally, a *glyph modifier*
3. Optionally, a *pitch descriptor*

## 6. Basic Glyph Descriptors

### 6.1. St. Gall Neumes (gregall or gresgmodern)

The *basic glyph descriptor* is a two-letter string from the following table:

| Code | Name |
|------|------|
| `vi` | virga |
| `pu` | punctum |
| `ta` | tractulus |
| `gr` | gravis |
| `cl` | clivis |
| `pe` | pes |
| `po` | porrectus |
| `to` | torculus |
| `ci` | climacus |
| `sc` | scandicus |
| `pf` | porrectus flexus |
| `sf` | scandicus flexus |
| `tr` | torculus resupinus |
| `st` | stropha |
| `ds` | distropha |
| `ts` | tristropha |
| `tg` | trigonus |
| `bv` | bivirga |
| `tv` | trivirga |
| `pr` | pressus maior |
| `pi` | pressus minor |
| `vs` | virga strata |
| `or` | oriscus |
| `sa` | scandicus |
| `pq` | pes quassus |
| `ql` | quilisma (3 loops) |
| `qi` | quilisma (2 loops) |
| `pt` | pes stratus |
| `ni` | nihil |

### 6.2. Laon Neumes (grelaon)

The *basic glyph descriptor* for Laon neumes is very similar to the one used for St. Gall neumes, but with uncinus and oriscus-clivis added and stropha, 2 loops quilisma and gravis removed:

| Code | Name |
|------|------|
| `un` | uncinus |
| `vi` | virga |
| `pu` | punctum |
| `ta` | tractulus |
| `gr` | gravis |
| `cl` | clivis |
| `oc` | oriscus-clivis |
| `pe` | pes |
| `po` | porrectus |
| `to` | torculus |
| `ci` | climacus |
| `sc` | scandicus |
| `pf` | porrectus flexus |
| `sf` | scandicus flexus |
| `tr` | torculus resupinus |
| `ds` | distropha |
| `ts` | tristropha |
| `tg` | trigonus |
| `bv` | bivirga |
| `tv` | trivirga |
| `pr` | pressus maior |
| `pi` | pressus minor |
| `vs` | virga strata |
| `or` | oriscus |
| `sa` | scandicus |
| `pq` | pes quassus |
| `ql` | quilisma |
| `pt` | pes stratus |
| `ni` | nihil |

## 7. Glyph Modifiers

The *glyph modifiers* are a possibly empty sequence of the following characters, optionally followed by a number:

| Character | Description |
|-----------|-------------|
| `S` | Modification of the mark |
| `G` | Modification of the grouping (neumatic break) |
| `M` | Melodic modification |
| `-` | Addition of episema |
| `>` | Augmentive liquescence |
| `~` | Diminutive liquescence |

If Dom Cardine's table contains multiple glyphs with the same modifiers, a positive number is added afterwards. For example, for augmentive liquescent clivis, the table shows two different glyphs: the ancus `cl>` and then another neume - `cl>1`, the first neume does not contain any number after it, while the `1` indicates the first variant.

## 8. Pitch Descriptor

The *pitch descriptor* allows to specify the vertical position of the neume. There are no staves, so the vertical position is only rough. For pitches, the same letters as in `gabc` are used, `a` through `n` and `p`. If the *pitch descriptor* is missing, the default is `hf`, otherwise it consists of the letter `h` followed by the pitch letter.

Within the *complex glyph descriptor*, each *basic glyph descriptor* has its own pitch, but in the current fonts there are no glyphs with different relative pitches, so if you use a *pitch descriptor* on any of the *basic glyph descriptors* in the *complex glyph descriptor*, it is best to use the same one on all the other *basic glyph descriptors* in the same *complex glyph descriptor*.

## 9. Subpunctis and Prepunctis Descriptors

The *subpunctis and prepunctis descriptor* consists of the letters `su` for subpunctis or `pp` for prepunctis, followed optionally by a modifier letter from the following table and finally a mandatory positive number of repetitions. If the modifier letter is missing, it is a punctum.

### 9.1. For St. Gall Neumes

| Letter | Description |
|--------|-------------|
| `t` | tractulus |
| `u` | tractulus with episema |
| `v` | tractulus with double episema |
| `w` | gravis |
| `x` | liquescens stropha |
| `y` | gravis with episema |

### 9.2. For Laon Neumes

| Letter | Description |
|--------|-------------|
| `n` | uncinus |
| `q` | quilisma |
| `z` | virga |
| `x` | cephalicus |

Only subpunctis are normally used in neume classification; the prepunctis is a `nabc` concept to describe the rising sequence of punctis, tractulis, etc. in the left low corner of some neume. While, for example, `vipp2` describes the same neume as `sc`, the former form allows better control on how many punctis or tractulis or tractulis with episema, etc., there are.

**Examples:**
- `ppt3` stands for 3 raising tractulis with episema
- `su1sut1sux1` stands for a punctum, followed by tractulus, followed by liquescens stropha

## 10. Significant Letter Descriptors

The *significant letter descriptor* consists of the letters `ls`, followed by a shorthand of the significant letter or common group of them, followed by a number - a rough position relative to the *complex glyph descriptor* glyph with prepunctis and subpunctis attached to it. If more than one *significant letter descriptor* is used on the same *complex neume descriptor* for the same position, then they are ordered in the order they are written in the `nabc` snippet at that position.

### 10.1. Positions

The position numbers are:

| Number | Position |
|--------|----------|
| `1` | Left upper corner |
| `2` | Above the neume |
| `3` | Right upper corner |
| `4` | To the left of the neume |
| `5` | Inside the neume (Laon only) |
| `6` | To the right of the neume |
| `7` | Left bottom corner |
| `8` | Below the neume |
| `9` | Right bottom corner |

### 10.2. Shorthands for St. Gall Neumes

The shorthands, including the `ls` prefix, which should be followed by the above mentioned position digit:

| Code | Name |
|------|------|
| `lsal` | altius |
| `lsam` | altius mediocriter |
| `lsb` | bene |
| `lsc` | celeriter |
| `lscm` | celeriter mediocriter |
| `lsco` | coniunguntur |
| `lscw` | celeriter (wide form) |
| `lsd` | deprimatur |
| `lse` | equaliter |
| `lseq` | equaliter |
| `lsew` | equaliter (wide form) |
| `lsfid` | fideliter |
| `lsfr` | frendor |
| `lsg` | gutture |
| `lsi` | iusum |
| `lsim` | iusum mediocriter |
| `lsiv` | iusum valde |
| `lsk` | klenche |
| `lsl` | levare |
| `lslb` | levare bene |
| `lslc` | levare celeriter |
| `lslen` | leniter |
| `lslm` | levare mediocriter |
| `lslp` | levare parvum |
| `lslt` | levare tenere |
| `lsm` | mediocriter |
| `lsmoll` | molliter |
| `lsp` | parvum |
| `lspar` | paratim |
| `lspfec` | perfecte |
| `lspm` | parvum mediocriter |
| `lspulcre` | pulcre |
| `lss` | sursum |
| `lssb` | sursum bene |
| `lssc` | sursum celeriter |
| `lssimil` | similiter |
| `lssimul` | simul |
| `lssm` | sursum mediocriter |
| `lsst` | sursum tenere |
| `lssta` | statim |
| `lst` | tenere |
| `lstb` | tenere bene |
| `lstm` | tenere mediocriter |
| `lstw` | tenere (wide form) |
| `lsv` | valde |
| `lsvol` | volubiliter |
| `lsx` | expectare |

### 10.3. Shorthands for Laon Neumes

The shorthands, including the `ls` prefix, which should be followed by the above mentioned position digit:

| Code | Name |
|------|------|
| `lsa` | augete |
| `lsc` | celeriter |
| `lseq` | equaliter |
| `lseq-` | equaliter |
| `lsequ` | equaliter |
| `lsf` | fastigium |
| `lsh` | humiliter |
| `lshn` | humiliter nectum |
| `lshp` | humiliter parum |
| `lsl` | levare |
| `lsn` | non (tenere), negare, nectum, naturaliter |
| `lsnl` | non levare |
| `lsnt` | non tenere |
| `lsm` | mediocriter |
| `lsmd` | mediocriter |
| `lss` | sursum |
| `lssimp` | simpliciter |
| `lssimpl` | simpliciter |
| `lssp` | sursum parum |
| `lsst` | sursum tenere |
| `lst` | tenere |
| `lsth` | tenere humiliter |

### 10.4. Tironian Notes (Laon)

For the Tironian notes, the form of the *significant letter descriptor* starts with the `lt` prefix followed by letters from the following list, followed by the above mentioned position digit (`5` can't be used):

| Code | Name |
|------|------|
| `lti` | iusum |
| `ltdo` | deorsum |
| `ltdr` | devertit |
| `ltdx` | devexum |
| `ltps` | prode sub eam (trade subtus) |
| `ltqm` | quam mox |
| `ltsb` | sub |
| `ltse` | seorsum |
| `ltsj` | subjice |
| `ltsl` | saltim |
| `ltsn` | sonare |
| `ltsp` | supra |
| `ltsr` | sursum |
| `ltst` | saltate (salte) |
| `ltus` | ut supra |

## 11. Nihil

Nihil - `ni` is an artificial empty base glyph around which it is possible to place significant letters and/or subpunctis and/or prepunctis.

## 12. Commented Example

Here is a commented example of an `nabc` snippet:

```
```po////pe>2lse7lsl3qlhh!vshhppt1sut2ql>ppu3
```

**Analysis:**

- ` ``` `: Stands for a negative horizontal skip (move to the left) by `nabcinterelementspace` and `nabclargerspace`.
- `po`: Is a basic porrectus, at the default pitch (height `hf`).
- `////`: Stands for a horizontal skip (move to the right) by two times `nabclargerspace`.
- `pe>2lse7lsl3`: Stands for the 3rd augmentive liquescent form of podatus (epiphonus with tractulus) at the default pitch, with equaliter in the low left corner and levare in the upper right corner.
- `qlhh!vshhppt1sut2`: Stands for a 3 loop quilisma joined with virga strata, both at relative pitch 2 above the default one, with a single tractulus before it and two subpunctis - tractulis.
- `ql>ppu3`: Is a 3 loop liquescent quilisma with 3 tractulis with episema before it.

## 13. Differences between St. Gall and Laon

### 13.1. Basic Glyphs

- **St. Gall**: Includes `st` (stropha), `qi` (quilisma with 2 loops), `gr` (gravis)
- **Laon**: Includes `un` (uncinus), `oc` (oriscus-clivis), but does not include `st`, `qi` (only `ql`), and `gr` is removed from the main list

### 13.2. Subpunctis and Prepunctis

- **St. Gall**: Uses modifiers `t`, `u`, `v`, `w`, `x`, `y`
- **Laon**: Uses modifiers `n`, `q`, `z`, `x`

### 13.3. Significant Letters

- **St. Gall**: Uses a more extensive set of shorthands
- **Laon**: Uses a different set of shorthands and includes Tironian notes with `lt` prefix
- **Laon**: Allows position `5` (inside the neume) for significant letters

## 14. Integration with GABC

NABC snippets are integrated into GABC notation using the `|` character as a separator. They appear after GABC snippets and are processed together to produce the final notation that combines diastematic neumes (GABC) with adiastematic neumes (NABC).

## 15. Complete List of NABC Codes

This section contains a comprehensive list of **1863 NABC codes** that are supported in the Gregorio fonts (gregall, gresgmodern, and grelaon). These codes are organized alphabetically.

**Note:** This list includes codes based on the documented patterns and aliases found in the Gregorio source code. For the most up-to-date and complete list with rendered glyphs, please refer to the tables in GregorioNabcRef.pdf (pages 7-13).

### All NABC Codes (Alphabetical Order)

```
bv                   bv-                  bv-1                 bv-2
bv>                  bv>1                 bv>2                 bvG
bvG1                 bvG2                 bvM                  bvM1
bvM2                 bvS                  bvS1                 bvS2
bv~                  bv~1                 bv~2                 ci
ci-                  ci-1                 ci-2                 ci1
ci>                  ci>1                 ci>2                 ciG
ciG1                 ciG2                 ciM                  ciM-
ciM1                 ciM2                 ciS                  ciS1
ciS2                 ciSppt1              cilsal1              cilsal2
cilsal3              cilsal7              cilsal8              cilsal9
cilsam1              cilsam2              cilsam3              cilsam7
cilsam8              cilsam9              cilsb1               cilsb2
cilsb3               cilsb7               cilsb8               cilsb9
cilsc1               cilsc2               cilsc3               cilsc7
cilsc8               cilsc9               cilse1               cilse2
cilse3               cilse7               cilse8               cilse9
cilsfr1              cilsfr2              cilsfr3              cilsfr7
cilsfr8              cilsfr9              cilsi1               cilsi2
cilsi3               cilsi7               cilsi8               cilsi9
cilsiv1              cilsiv2              cilsiv3              cilsiv7
cilsiv8              cilsiv9              cilsl1               cilsl2
cilsl3               cilsl7               cilsl8               cilsl9
cilsm1               cilsm2               cilsm3               cilsm7
cilsm8               cilsm9               cilss1               cilss2
cilss3               cilss7               cilss8               cilss9
cilst1               cilst2               cilst3               cilst7
cilst8               cilst9               cipp1                cippt1
cippt1lsc3           cippu1               cippu1lsc3           cippu2
cippu3               cisu1                cisut1               cisuu1
cisuv1               cisux1               ci~                  ci~1
ci~2                 cl                   cl!po>               cl!po>ppt1
cl-                  cl-1                 cl-2                 cl-ppt2
cl>                  cl>1                 cl>2                 cl>pp2
clG                  clG1                 clG2                 clM
clM1                 clM2                 clMpp2               clS
clS1                 clS2                 clSsu2               clSsut1
cllsal1              cllsal2              cllsal3              cllsal7
cllsal8              cllsal9              cllsam1              cllsam2
cllsam3              cllsam7              cllsam8              cllsam9
cllsb1               cllsb2               cllsb3               cllsb7
cllsb8               cllsb9               cllsc1               cllsc2
cllsc3               cllsc7               cllsc8               cllsc9
cllse1               cllse2               cllse3               cllse7
cllse8               cllse9               cllsfr1              cllsfr2
cllsfr3              cllsfr7              cllsfr8              cllsfr9
cllsi1               cllsi2               cllsi3               cllsi7
cllsi8               cllsi9               cllsiv1              cllsiv2
cllsiv3              cllsiv7              cllsiv8              cllsiv9
cllsl1               cllsl2               cllsl3               cllsl7
cllsl8               cllsl9               cllsm1               cllsm2
cllsm3               cllsm7               cllsm8               cllsm9
cllss1               cllss2               cllss3               cllss7
cllss8               cllss9               cllst1               cllst2
cllst3               cllst7               cllst8               cllst9
clpp1                clpp2                clpp2lsc2            clppt1
clppt2               clppu1               clppu2               clppu3
clsu1                clsut1               clsuu1               clsuv1
clsux1               cl~                  cl~1                 cl~2
ds                   ds-                  ds-1                 ds-2
ds>                  ds>1                 ds>2                 dsG
dsG1                 dsG2                 dsM                  dsM1
dsM2                 dsS                  dsS1                 dsS2
dslsal1              dslsal2              dslsal3              dslsal7
dslsal8              dslsal9              dslsam1              dslsam2
dslsam3              dslsam7              dslsam8              dslsam9
dslsb1               dslsb2               dslsb3               dslsb7
dslsb8               dslsb9               dslsc1               dslsc2
dslsc3               dslsc7               dslsc8               dslsc9
dslse1               dslse2               dslse3               dslse7
dslse8               dslse9               dslsfr1              dslsfr2
dslsfr3              dslsfr7              dslsfr8              dslsfr9
dslsi1               dslsi2               dslsi3               dslsi7
dslsi8               dslsi9               dslsiv1              dslsiv2
dslsiv3              dslsiv7              dslsiv8              dslsiv9
dslsl1               dslsl2               dslsl3               dslsl7
dslsl8               dslsl9               dslsm1               dslsm2
dslsm3               dslsm7               dslsm8               dslsm9
dslss1               dslss2               dslss3               dslss7
dslss8               dslss9               dslst1               dslst2
dslst3               dslst7               dslst8               dslst9
dspp1                dsppt1               dsppu1               dsppu2
dsppu3               dssu1                dssut1               dssuu1
dssuv1               dssux1               ds~                  ds~1
ds~2                 gr                   gr-                  gr-1
gr-2                 gr>                  gr>1                 gr>2
grG                  grG1                 grG2                 grM
grM1                 grM2                 grS                  grS1
grS2                 grlsal1              grlsal2              grlsal3
grlsal7              grlsal8              grlsal9              grlsam1
grlsam2              grlsam3              grlsam7              grlsam8
grlsam9              grlsb1               grlsb2               grlsb3
grlsb7               grlsb8               grlsb9               grlsc1
grlsc2               grlsc3               grlsc7               grlsc8
grlsc9               grlse1               grlse2               grlse3
grlse7               grlse8               grlse9               grlsfr1
grlsfr2              grlsfr3              grlsfr7              grlsfr8
grlsfr9              grlsi1               grlsi2               grlsi3
grlsi7               grlsi8               grlsi9               grlsiv1
grlsiv2              grlsiv3              grlsiv7              grlsiv8
grlsiv9              grlsl1               grlsl2               grlsl3
grlsl7               grlsl8               grlsl9               grlsm1
grlsm2               grlsm3               grlsm7               grlsm8
grlsm9               grlss1               grlss2               grlss3
grlss7               grlss8               grlss9               grlst1
grlst2               grlst3               grlst7               grlst8
grlst9               grpp1                grppt1               grppu1
grppu2               grppu3               grsu1                grsut1
grsuu1               grsuv1               grsux1               gr~
gr~1                 gr~2                 ni                   ni-
ni-1                 ni-2                 ni>                  ni>1
ni>2                 niG                  niG1                 niG2
niM                  niM1                 niM2                 niS
niS1                 niS2                 ni~                  ni~1
ni~2                 oc                   oc-                  oc-1
oc-2                 oc>                  oc>1                 oc>2
ocG                  ocG1                 ocG2                 ocM
ocM1                 ocM2                 ocS                  ocS1
ocS2                 oc~                  oc~1                 oc~2
or                   or-                  or-1                 or-2
or>                  or>1                 or>2                 orG
orG1                 orG2                 orM                  orM1
orM2                 orS                  orS1                 orS2
or~                  or~1                 or~2                 pe
pe!cl                pe!clhgpi            pe-                  pe-1
pe-2                 pe>                  pe>1                 pe>2
peG                  peG1                 peG2                 peM
peM1                 peM2                 peS                  peS1
peS2                 pelsal1              pelsal2              pelsal3
pelsal7              pelsal8              pelsal9              pelsam1
pelsam2              pelsam3              pelsam7              pelsam8
pelsam9              pelsb1               pelsb2               pelsb3
pelsb7               pelsb8               pelsb9               pelsc1
pelsc2               pelsc3               pelsc7               pelsc8
pelsc9               pelse1               pelse2               pelse3
pelse7               pelse8               pelse9               pelsfr1
pelsfr2              pelsfr3              pelsfr7              pelsfr8
pelsfr9              pelsi1               pelsi2               pelsi3
pelsi7               pelsi8               pelsi9               pelsiv1
pelsiv2              pelsiv3              pelsiv7              pelsiv8
pelsiv9              pelsl1               pelsl2               pelsl3
pelsl7               pelsl8               pelsl9               pelsm1
pelsm2               pelsm3               pelsm7               pelsm8
pelsm9               pelss1               pelss2               pelss3
pelss7               pelss8               pelss9               pelst1
pelst2               pelst3               pelst7               pelst8
pelst9               pepp1                peppt1               peppu1
peppu2               peppu3               pesu1                pesut1
pesuu1               pesuv1               pesux1               pe~
pe~1                 pe~2                 pf                   pf!cl~
pf!cl~ppt1           pf-                  pf-1                 pf-2
pf>                  pf>1                 pf>2                 pfG
pfG1                 pfG2                 pfM                  pfM1
pfM2                 pfS                  pfS1                 pfS2
pflsal1              pflsal2              pflsal3              pflsal7
pflsal8              pflsal9              pflsam1              pflsam2
pflsam3              pflsam7              pflsam8              pflsam9
pflsb1               pflsb2               pflsb3               pflsb7
pflsb8               pflsb9               pflsc1               pflsc2
pflsc3               pflsc7               pflsc8               pflsc9
pflse1               pflse2               pflse3               pflse7
pflse8               pflse9               pflsfr1              pflsfr2
pflsfr3              pflsfr7              pflsfr8              pflsfr9
pflsi1               pflsi2               pflsi3               pflsi7
pflsi8               pflsi9               pflsiv1              pflsiv2
pflsiv3              pflsiv7              pflsiv8              pflsiv9
pflsl1               pflsl2               pflsl3               pflsl7
pflsl8               pflsl9               pflsm1               pflsm2
pflsm3               pflsm7               pflsm8               pflsm9
pflss1               pflss2               pflss3               pflss7
pflss8               pflss9               pflst1               pflst2
pflst3               pflst7               pflst8               pflst9
pfpp1                pfppt1               pfppu1               pfppu2
pfppu3               pfsu1                pfsut1               pfsuu1
pfsuv1               pfsux1               pf~                  pf~1
pf~2                 pi                   pi-                  pi-1
pi-2                 pi>                  pi>1                 pi>2
piG                  piG1                 piG2                 piM
piM1                 piM2                 piS                  piS1
piS2                 pi~                  pi~1                 pi~2
po                   po-                  po-1                 po-2
po>                  po>1                 po>2                 poG
poG1                 poG2                 poM                  poM1
poM2                 poS                  poS1                 poS2
polsal1              polsal2              polsal3              polsal7
polsal8              polsal9              polsam1              polsam2
polsam3              polsam7              polsam8              polsam9
polsb1               polsb2               polsb3               polsb7
polsb8               polsb9               polsc1               polsc2
polsc3               polsc7               polsc8               polsc9
polse1               polse2               polse3               polse7
polse8               polse9               polsfr1              polsfr2
polsfr3              polsfr7              polsfr8              polsfr9
polsi1               polsi2               polsi3               polsi7
polsi8               polsi9               polsiv1              polsiv2
polsiv3              polsiv7              polsiv8              polsiv9
polsl1               polsl2               polsl3               polsl7
polsl8               polsl9               polsm1               polsm2
polsm3               polsm7               polsm8               polsm9
polss1               polss2               polss3               polss7
polss8               polss9               polst1               polst2
polst3               polst7               polst8               polst9
popp1                poppt1               poppu1               poppu2
poppu3               posu1                posut1               posuu1
posuv1               posux1               po~                  po~1
po~2                 ppu1                 pq                   pq-
pq-1                 pq-2                 pq>                  pq>1
pq>2                 pqG                  pqG1                 pqG2
pqM                  pqM1                 pqM2                 pqS
pqS1                 pqS2                 pq~                  pq~1
pq~2                 pr                   pr-                  pr-1
pr-2                 pr>                  pr>1                 pr>2
prG                  prG1                 prG2                 prM
prM1                 prM2                 prS                  prS1
prS2                 pr~                  pr~1                 pr~2
pt                   pt-                  pt-1                 pt-2
pt>                  pt>1                 pt>2                 ptG
ptG1                 ptG2                 ptM                  ptM1
ptM2                 ptS                  ptS1                 ptS2
pt~                  pt~1                 pt~2                 pu
pu-                  pu-1                 pu-2                 pu>
pu>1                 pu>2                 puG                  puG1
puG2                 puM                  puM1                 puM2
puS                  puS1                 puS2                 pulsal1
pulsal2              pulsal3              pulsal7              pulsal8
pulsal9              pulsam1              pulsam2              pulsam3
pulsam7              pulsam8              pulsam9              pulsb1
pulsb2               pulsb3               pulsb7               pulsb8
pulsb9               pulsc1               pulsc2               pulsc3
pulsc7               pulsc8               pulsc9               pulse1
pulse2               pulse3               pulse7               pulse8
pulse9               pulsfr1              pulsfr2              pulsfr3
pulsfr7              pulsfr8              pulsfr9              pulsi1
pulsi2               pulsi3               pulsi7               pulsi8
pulsi9               pulsiv1              pulsiv2              pulsiv3
pulsiv7              pulsiv8              pulsiv9              pulsl1
pulsl2               pulsl3               pulsl7               pulsl8
pulsl9               pulsm1               pulsm2               pulsm3
pulsm7               pulsm8               pulsm9               pulss1
pulss2               pulss3               pulss7               pulss8
pulss9               pulst1               pulst2               pulst3
pulst7               pulst8               pulst9               pupp1
puppt1               puppu1               puppu2               puppu3
pusu1                pusut1               pusuu1               pusuv1
pusux1               pu~                  pu~1                 pu~2
qi                   qi-                  qi-1                 qi-2
qi>                  qi>1                 qi>2                 qiG
qiG1                 qiG2                 qiM                  qiM1
qiM2                 qiS                  qiS1                 qiS2
qi~                  qi~1                 qi~2                 ql
ql-                  ql-1                 ql-2                 ql>
ql>1                 ql>2                 qlG                  qlG1
qlG2                 qlM                  qlM1                 qlM2
qlS                  qlS1                 qlS2                 qlhh!vshh
qlhh!vshhppt1sut2    ql~                  ql~1                 ql~2
sa                   sa-                  sa-1                 sa-2
sa>                  sa>1                 sa>2                 saG
saG1                 saG2                 saM                  saM1
saM2                 saS                  saS1                 saS2
sa~                  sa~1                 sa~2                 sc
sc-                  sc-1                 sc-2                 sc1
sc>                  sc>1                 sc>2                 scG
scG1                 scG2                 scM                  scM1
scM2                 scS                  scS1                 scS2
sclsal1              sclsal2              sclsal3              sclsal7
sclsal8              sclsal9              sclsam1              sclsam2
sclsam3              sclsam7              sclsam8              sclsam9
sclsb1               sclsb2               sclsb3               sclsb7
sclsb8               sclsb9               sclsc1               sclsc2
sclsc3               sclsc7               sclsc8               sclsc9
sclse1               sclse2               sclse3               sclse7
sclse8               sclse9               sclsfr1              sclsfr2
sclsfr3              sclsfr7              sclsfr8              sclsfr9
sclsi1               sclsi2               sclsi3               sclsi7
sclsi8               sclsi9               sclsiv1              sclsiv2
sclsiv3              sclsiv7              sclsiv8              sclsiv9
sclsl1               sclsl2               sclsl3               sclsl7
sclsl8               sclsl9               sclsm1               sclsm2
sclsm3               sclsm7               sclsm8               sclsm9
sclss1               sclss2               sclss3               sclss7
sclss8               sclss9               sclst1               sclst2
sclst3               sclst7               sclst8               sclst9
scpp1                scppt1               scppu1               scppu2
scppu3               scsu1                scsut1               scsuu1
scsuv1               scsux1               sc~                  sc~1
sc~2                 sf                   sf-                  sf-1
sf-2                 sf>                  sf>1                 sf>2
sfG                  sfG1                 sfG2                 sfM
sfM1                 sfM2                 sfS                  sfS1
sfS2                 sflsal1              sflsal2              sflsal3
sflsal7              sflsal8              sflsal9              sflsam1
sflsam2              sflsam3              sflsam7              sflsam8
sflsam9              sflsb1               sflsb2               sflsb3
sflsb7               sflsb8               sflsb9               sflsc1
sflsc2               sflsc3               sflsc7               sflsc8
sflsc9               sflse1               sflse2               sflse3
sflse7               sflse8               sflse9               sflsfr1
sflsfr2              sflsfr3              sflsfr7              sflsfr8
sflsfr9              sflsi1               sflsi2               sflsi3
sflsi7               sflsi8               sflsi9               sflsiv1
sflsiv2              sflsiv3              sflsiv7              sflsiv8
sflsiv9              sflsl1               sflsl2               sflsl3
sflsl7               sflsl8               sflsl9               sflsm1
sflsm2               sflsm3               sflsm7               sflsm8
sflsm9               sflss1               sflss2               sflss3
sflss7               sflss8               sflss9               sflst1
sflst2               sflst3               sflst7               sflst8
sflst9               sfpp1                sfppt1               sfppu1
sfppu2               sfppu3               sfsu1                sfsut1
sfsuu1               sfsuv1               sfsux1               sf~
sf~1                 sf~2                 st                   st-
st-1                 st-2                 st>                  st>1
st>2                 stG                  stG1                 stG2
stM                  stM1                 stM2                 stS
stS1                 stS2                 stlsal1              stlsal2
stlsal3              stlsal7              stlsal8              stlsal9
stlsam1              stlsam2              stlsam3              stlsam7
stlsam8              stlsam9              stlsb1               stlsb2
stlsb3               stlsb7               stlsb8               stlsb9
stlsc1               stlsc2               stlsc3               stlsc7
stlsc8               stlsc9               stlse1               stlse2
stlse3               stlse7               stlse8               stlse9
stlsfr1              stlsfr2              stlsfr3              stlsfr7
stlsfr8              stlsfr9              stlsi1               stlsi2
stlsi3               stlsi7               stlsi8               stlsi9
stlsiv1              stlsiv2              stlsiv3              stlsiv7
stlsiv8              stlsiv9              stlsl1               stlsl2
stlsl3               stlsl7               stlsl8               stlsl9
stlsm1               stlsm2               stlsm3               stlsm7
stlsm8               stlsm9               stlss1               stlss2
stlss3               stlss7               stlss8               stlss9
stlst1               stlst2               stlst3               stlst7
stlst8               stlst9               stpp1                stppt1
stppu1               stppu2               stppu3               stsu1
stsut1               stsuu1               stsuv1               stsux1
st~                  st~1                 st~2                 suu1
ta                   ta-                  ta-1                 ta-2
ta>                  ta>1                 ta>2                 ta>ppt1
taG                  taG1                 taG2                 taM
taM1                 taM2                 taS                  taS1
taS2                 talsal1              talsal2              talsal3
talsal7              talsal8              talsal9              talsam1
talsam2              talsam3              talsam7              talsam8
talsam9              talsb1               talsb2               talsb3
talsb7               talsb8               talsb9               talsc1
talsc2               talsc3               talsc7               talsc8
talsc9               talse1               talse2               talse3
talse7               talse8               talse9               talsfr1
talsfr2              talsfr3              talsfr7              talsfr8
talsfr9              talsi1               talsi2               talsi3
talsi7               talsi8               talsi9               talsiv1
talsiv2              talsiv3              talsiv7              talsiv8
talsiv9              talsl1               talsl2               talsl3
talsl7               talsl8               talsl9               talsm1
talsm2               talsm3               talsm7               talsm8
talsm9               talss1               talss2               talss3
talss7               talss8               talss9               talst1
talst2               talst3               talst7               talst8
talst9               tapp1                tappt1               tappu1
tappu2               tappu3               tasu1                tasut1
tasuu1               tasuv1               tasux1               ta~
ta~1                 ta~2                 tg                   tg-
tg-1                 tg-2                 tg>                  tg>1
tg>2                 tgG                  tgG1                 tgG2
tgM                  tgM1                 tgM2                 tgS
tgS1                 tgS2                 tg~                  tg~1
tg~2                 to                   to-                  to-1
to-2                 to>                  to>1                 to>2
toG                  toG1                 toG2                 toM
toM1                 toM2                 toS                  toS1
toS2                 tolsal1              tolsal2              tolsal3
tolsal7              tolsal8              tolsal9              tolsam1
tolsam2              tolsam3              tolsam7              tolsam8
tolsam9              tolsb1               tolsb2               tolsb3
tolsb7               tolsb8               tolsb9               tolsc1
tolsc2               tolsc3               tolsc7               tolsc8
tolsc9               tolse1               tolse2               tolse3
tolse7               tolse8               tolse9               tolsfr1
tolsfr2              tolsfr3              tolsfr7              tolsfr8
tolsfr9              tolsi1               tolsi2               tolsi3
tolsi7               tolsi8               tolsi9               tolsiv1
tolsiv2              tolsiv3              tolsiv7              tolsiv8
tolsiv9              tolsl1               tolsl2               tolsl3
tolsl7               tolsl8               tolsl9               tolsm1
tolsm2               tolsm3               tolsm7               tolsm8
tolsm9               tolss1               tolss2               tolss3
tolss7               tolss8               tolss9               tolst1
tolst2               tolst3               tolst7               tolst8
tolst9               topp1                toppt1               toppu1
toppu2               toppu3               tosu1                tosut1
tosuu1               tosuv1               tosux1               to~
to~1                 to~2                 tr                   tr-
tr-1                 tr-2                 tr>                  tr>1
tr>2                 trG                  trG1                 trG2
trM                  trM1                 trM2                 trS
trS1                 trS2                 trlsal1              trlsal2
trlsal3              trlsal7              trlsal8              trlsal9
trlsam1              trlsam2              trlsam3              trlsam7
trlsam8              trlsam9              trlsb1               trlsb2
trlsb3               trlsb7               trlsb8               trlsb9
trlsc1               trlsc2               trlsc3               trlsc7
trlsc8               trlsc9               trlse1               trlse2
trlse3               trlse7               trlse8               trlse9
trlsfr1              trlsfr2              trlsfr3              trlsfr7
trlsfr8              trlsfr9              trlsi1               trlsi2
trlsi3               trlsi7               trlsi8               trlsi9
trlsiv1              trlsiv2              trlsiv3              trlsiv7
trlsiv8              trlsiv9              trlsl1               trlsl2
trlsl3               trlsl7               trlsl8               trlsl9
trlsm1               trlsm2               trlsm3               trlsm7
trlsm8               trlsm9               trlss1               trlss2
trlss3               trlss7               trlss8               trlss9
trlst1               trlst2               trlst3               trlst7
trlst8               trlst9               trpp1                trppt1
trppu1               trppu2               trppu3               trsu1
trsut1               trsuu1               trsuv1               trsux1
tr~                  tr~1                 tr~2                 ts
ts-                  ts-1                 ts-2                 ts>
ts>1                 ts>2                 tsG                  tsG1
tsG2                 tsM                  tsM1                 tsM2
tsS                  tsS1                 tsS2                 ts~
ts~1                 ts~2                 tv                   tv-
tv-1                 tv-2                 tv>                  tv>1
tv>2                 tvG                  tvG1                 tvG2
tvM                  tvM1                 tvM2                 tvS
tvS1                 tvS2                 tv~                  tv~1
tv~2                 un                   un-                  un-1
un-2                 un>                  un>1                 un>2
unG                  unG1                 unG2                 unM
unM1                 unM2                 unS                  unS1
unS2                 un~                  un~1                 un~2
vi                   vi-                  vi-1                 vi-2
vi-ppt2              vi-su1suw1           vi-su2               vi>
vi>1                 vi>2                 vi>pp2               viG
viG1                 viG2                 viM                  viM1
viM2                 viS                  viS1                 viS2
vilsal1              vilsal2              vilsal3              vilsal7
vilsal8              vilsal9              vilsam1              vilsam2
vilsam3              vilsam7              vilsam8              vilsam9
vilsb1               vilsb2               vilsb3               vilsb7
vilsb8               vilsb9               vilsc1               vilsc2
vilsc3               vilsc7               vilsc8               vilsc9
vilse1               vilse2               vilse3               vilse7
vilse8               vilse9               vilsfr1              vilsfr2
vilsfr3              vilsfr7              vilsfr8              vilsfr9
vilsi1               vilsi2               vilsi3               vilsi7
vilsi8               vilsi9               vilsiv1              vilsiv2
vilsiv3              vilsiv7              vilsiv8              vilsiv9
vilsl1               vilsl2               vilsl3               vilsl7
vilsl8               vilsl9               vilsm1               vilsm2
vilsm3               vilsm7               vilsm8               vilsm9
vilss1               vilss2               vilss3               vilss7
vilss8               vilss9               vilst1               vilst2
vilst3               vilst7               vilst8               vilst9
vipp1                vipp2                vipp3                vippt1
vippt1su1sut1        vippt1su2            vippt1su2lsc3        vippt2
vippu1               vippu1su2            vippu1su2lsc3        vippu2
vippu3               visu1                visu1sut1            visu1suw1
visu1sux1            visu2                visu2lsc3            visu3
visut1               visut2               visuu1               visuv1
visux1               vi~                  vi~1                 vi~2
vs                   vs-                  vs-1                 vs-2
vs>                  vs>1                 vs>2                 vsG
vsG1                 vsG2                 vsM                  vsM1
vsM2                 vsS                  vsS1                 vsS2
vs~                  vs~1                 vs~2
```

**Total: 1863 codes**

## References

This specification is based on the official Gregorio documentation, available at:
- GregorioNabcRef.tex (source code)
- GregorioNabcRef.pdf (compiled document available at ctan.org)

For more information, see:
- [Gregorio project website](http://gregorio-project.github.io/)
- [GitHub repository](http://github.com/gregorio-project/gregorio)
- Dom Eugène Cardine's Table of neumatic signs