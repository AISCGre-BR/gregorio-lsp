//! Core types shared across parsers and validators.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

impl Position {
    pub const fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub const fn zero() -> Self {
        Self {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// A suggested text replacement for an auto-fixable diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFix {
    /// The range in the source document to replace.
    pub range: Range,
    /// The replacement text.
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub range: Range,
    pub severity: Severity,
    pub code: Option<String>,
    /// Optional auto-fix: a single text replacement that resolves the diagnostic.
    pub fix: Option<TextFix>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, range: Range, severity: Severity) -> Self {
        Self {
            message: message.into(),
            range,
            severity,
            code: None,
            fix: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_fix(mut self, fix: TextFix) -> Self {
        self.fix = Some(fix);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub text: String,
    pub range: Range,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    /// Headers in insertion order, last value wins for duplicates (mirrors TS Map behavior).
    pub headers: HeaderMap,
    pub notation: NotationSection,
    pub comments: Vec<Comment>,
    pub errors: Vec<ParseError>,
}

/// Insertion-order preserving header map (lowercased keys).
#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    entries: Vec<(String, String)>,
    index: BTreeMap<String, usize>,
    /// Keys that were inserted more than once (one entry per overwrite, in insertion order).
    /// Use to detect duplicate header definitions.
    pub duplicate_keys: Vec<String>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let key = name.into().to_lowercase();
        let val = value.into();
        if let Some(&idx) = self.index.get(&key) {
            self.entries[idx].1 = val;
            self.duplicate_keys.push(key);
        } else {
            self.index.insert(key.clone(), self.entries.len());
            self.entries.push((key, val));
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        let key = name.to_lowercase();
        self.index.get(&key).map(|&i| self.entries[i].1.as_str())
    }

    pub fn has(&self, name: &str) -> bool {
        self.index.contains_key(&name.to_lowercase())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct NotationSection {
    pub syllables: Vec<Syllable>,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct Syllable {
    pub text: String,
    pub text_with_styles: Option<String>,
    /// Range covering only the syllable text (before the opening `(`).
    pub text_range: Range,
    pub notes: Vec<NoteGroup>,
    pub range: Range,
    pub clef: Option<Clef>,
    pub bar: Option<Bar>,
    pub line_break: Option<LineBreak>,
}

#[derive(Debug, Clone)]
pub struct NoteGroup {
    pub gabc: String,
    pub nabc: Option<Vec<String>>,
    pub nabc_parsed: Option<Vec<NabcGlyphDescriptor>>,
    pub range: Range,
    pub notes: Vec<Note>,
    pub custos: Option<Custos>,
    pub attributes: Option<Vec<GabcAttribute>>,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: char,
    pub shape: NoteShape,
    pub modifiers: Vec<NoteModifier>,
    pub range: Range,
    pub fusion: bool,
}

#[derive(Debug, Clone)]
pub struct Custos {
    pub kind: CustosKind,
    pub pitch: Option<char>,
    pub range: Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustosKind {
    Auto,
    Explicit,
}

#[derive(Debug, Clone)]
pub struct GabcAttribute {
    pub name: String,
    pub value: Option<String>,
    pub range: Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteShape {
    Punctum,
    PunctumInclinatum,
    Virga,
    VirgaReversa,
    Oriscus,
    Quilisma,
    Stropha,
    Liquescent,
    Cavum,
    Linea,
    Flat,
    Sharp,
    Natural,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteModifier {
    pub kind: ModifierType,
    pub value: Option<String>,
}

impl NoteModifier {
    pub const fn simple(kind: ModifierType) -> Self {
        Self { kind, value: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierType {
    InitioDebilis,
    PunctumMora,
    HorizontalEpisema,
    VerticalEpisema,
    Liquescent,
    Oriscus,
    OriscusScapus,
    Quilisma,
    Fusion,
    Cavum,
    Strata,
    Quadratum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClefKind {
    C,
    F,
}

#[derive(Debug, Clone)]
pub struct Clef {
    pub kind: ClefKind,
    pub line: u8,
    pub has_flat: bool,
    pub range: Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarType {
    Virgula,
    DivisioMinima,
    DivisioMinor,
    DivisioMaior,
    DivisioFinalis,
    Dominican,
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub kind: BarType,
    pub range: Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineBreakKind {
    Manual,
    Suggested,
}

#[derive(Debug, Clone)]
pub struct LineBreak {
    pub kind: LineBreakKind,
    pub range: Range,
}

// =========================================================================
// NABC Types
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NabcBasicGlyph {
    Virga,
    Punctum,
    Tractulus,
    Gravis,
    Clivis,
    Pes,
    Porrectus,
    Torculus,
    Climacus,
    Scandicus,
    PorrectusFlexus,
    ScandicusFlexus,
    TorculusResupinus,
    Stropha,
    Distropha,
    Tristropha,
    Trigonus,
    Bivirga,
    Trivirga,
    PressusMainor,
    PressusMinor,
    VirgaStrata,
    Oriscus,
    Salicus,
    PesQuassus,
    Quilisma3Loops,
    Quilisma2Loops,
    PesStratus,
    Nihil,
    Uncinus,
    OriscusClivis,
}

impl NabcBasicGlyph {
    pub fn from_code(code: &str) -> Option<Self> {
        use NabcBasicGlyph::*;
        Some(match code {
            "vi" => Virga,
            "pu" => Punctum,
            "ta" => Tractulus,
            "gr" => Gravis,
            "cl" => Clivis,
            "pe" => Pes,
            "po" => Porrectus,
            "to" => Torculus,
            "ci" => Climacus,
            "sc" => Scandicus,
            "pf" => PorrectusFlexus,
            "sf" => ScandicusFlexus,
            "tr" => TorculusResupinus,
            "st" => Stropha,
            "ds" => Distropha,
            "ts" => Tristropha,
            "tg" => Trigonus,
            "bv" => Bivirga,
            "tv" => Trivirga,
            "pr" => PressusMainor,
            "pi" => PressusMinor,
            "vs" => VirgaStrata,
            "or" => Oriscus,
            "sa" => Salicus,
            "pq" => PesQuassus,
            "ql" => Quilisma3Loops,
            "qi" => Quilisma2Loops,
            "pt" => PesStratus,
            "ni" => Nihil,
            "un" => Uncinus,
            "oc" => OriscusClivis,
            _ => return None,
        })
    }

    pub fn code(self) -> &'static str {
        use NabcBasicGlyph::*;
        match self {
            Virga => "vi",
            Punctum => "pu",
            Tractulus => "ta",
            Gravis => "gr",
            Clivis => "cl",
            Pes => "pe",
            Porrectus => "po",
            Torculus => "to",
            Climacus => "ci",
            Scandicus => "sc",
            PorrectusFlexus => "pf",
            ScandicusFlexus => "sf",
            TorculusResupinus => "tr",
            Stropha => "st",
            Distropha => "ds",
            Tristropha => "ts",
            Trigonus => "tg",
            Bivirga => "bv",
            Trivirga => "tv",
            PressusMainor => "pr",
            PressusMinor => "pi",
            VirgaStrata => "vs",
            Oriscus => "or",
            Salicus => "sa",
            PesQuassus => "pq",
            Quilisma3Loops => "ql",
            Quilisma2Loops => "qi",
            PesStratus => "pt",
            Nihil => "ni",
            Uncinus => "un",
            OriscusClivis => "oc",
        }
    }

    pub fn all() -> &'static [NabcBasicGlyph] {
        use NabcBasicGlyph::*;
        &[
            Virga,
            Punctum,
            Tractulus,
            Gravis,
            Clivis,
            Pes,
            Porrectus,
            Torculus,
            Climacus,
            Scandicus,
            PorrectusFlexus,
            ScandicusFlexus,
            TorculusResupinus,
            Stropha,
            Distropha,
            Tristropha,
            Trigonus,
            Bivirga,
            Trivirga,
            PressusMainor,
            PressusMinor,
            VirgaStrata,
            Oriscus,
            Salicus,
            PesQuassus,
            Quilisma3Loops,
            Quilisma2Loops,
            PesStratus,
            Nihil,
            Uncinus,
            OriscusClivis,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NabcGlyphModifier {
    MarkModification,      // S
    GroupingModification,  // G
    MelodicModification,   // M
    Episema,               // -
    AugmentiveLiquescence, // >
    DiminutiveLiquescence, // ~
}

impl NabcGlyphModifier {
    pub fn as_char(self) -> char {
        use NabcGlyphModifier::*;
        match self {
            MarkModification => 'S',
            GroupingModification => 'G',
            MelodicModification => 'M',
            Episema => '-',
            AugmentiveLiquescence => '>',
            DiminutiveLiquescence => '~',
        }
    }
}

#[derive(Debug, Clone)]
pub struct NabcSubpunctis {
    pub count: u8,
    pub modifier: Option<char>,
    pub range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct NabcPrepunctis {
    pub count: u8,
    pub modifier: Option<char>,
    pub range: Option<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NabcLetterKind {
    Significant, // ls
    Tironian,    // lt
}

#[derive(Debug, Clone)]
pub struct NabcSignificantLetter {
    pub kind: NabcLetterKind,
    pub code: String,
    pub position: u8, // 1..=9
    pub range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct NabcGlyphDescriptor {
    pub basic_glyph: NabcBasicGlyph,
    pub modifiers: Option<Vec<NabcGlyphModifier>>,
    pub pitch: Option<char>,
    pub subpunctis: Option<NabcSubpunctis>,
    pub prepunctis: Option<NabcPrepunctis>,
    pub significant_letters: Option<Vec<NabcSignificantLetter>>,
    pub range: Option<Range>,
    pub fusion: Option<Box<NabcGlyphDescriptor>>,
}

impl NabcGlyphDescriptor {
    pub fn new(basic_glyph: NabcBasicGlyph) -> Self {
        Self {
            basic_glyph,
            modifiers: None,
            pitch: None,
            subpunctis: None,
            prepunctis: None,
            significant_letters: None,
            range: None,
            fusion: None,
        }
    }
}

// =========================================================================
// NABC code tables (ls/lt)
// =========================================================================

pub const NABC_ST_GALL_CODES: &[&str] = &[
    "al", "am", "b", "c", "cm", "co", "cw", "d", "e", "eq", "ew", "fid", "fr", "g", "i", "im",
    "iv", "k", "l", "lb", "lc", "len", "lm", "lp", "lt", "m", "moll", "p", "par", "pfec", "pm",
    "pulcre", "s", "sb", "sc", "simil", "simul", "sm", "st", "sta", "t", "tb", "tm", "tw", "v",
    "vol", "x",
];

pub const NABC_LAON_CODES: &[&str] = &[
    "a", "c", "eq", "eq-", "equ", "f", "h", "hn", "hp", "l", "n", "nl", "nt", "m", "md", "s",
    "simp", "simpl", "sp", "st", "t", "th",
];

pub const NABC_TIRONIAN_CODES: &[&str] = &[
    "i", "do", "dr", "dx", "ps", "qm", "sb", "se", "sj", "sl", "sn", "sp", "sr", "st", "us",
];

pub fn is_nabc_st_gall_code(code: &str) -> bool {
    NABC_ST_GALL_CODES.contains(&code)
}

pub fn is_nabc_laon_code(code: &str) -> bool {
    NABC_LAON_CODES.contains(&code)
}

pub fn is_nabc_tironian_code(code: &str) -> bool {
    NABC_TIRONIAN_CODES.contains(&code)
}

pub fn is_nabc_significant_letter_code(code: &str) -> bool {
    is_nabc_st_gall_code(code) || is_nabc_laon_code(code)
}
