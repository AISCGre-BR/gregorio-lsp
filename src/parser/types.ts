/**
 * GABC Parser Types
 * Types for parsing GABC (Gregorio) notation files
 */

export interface Position {
  line: number;
  character: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface ParsedDocument {
  headers: Map<string, string>;
  notation: NotationSection;
  comments: Comment[];
  errors: ParseError[];
}

export interface Comment {
  text: string;
  range: Range;
}

export interface ParseError {
  message: string;
  range: Range;
  severity: 'error' | 'warning' | 'info';
}

export interface NotationSection {
  syllables: Syllable[];
  range: Range;
}

export interface Syllable {
  text: string;
  textWithStyles?: string; // Text including style tags like <b>, <i>, etc.
  notes: NoteGroup[];
  range: Range;
  clef?: Clef;
  bar?: Bar;
  lineBreak?: LineBreak;
}

export interface NoteGroup {
  gabc: string;
  nabc?: string[];
  nabcParsed?: NABCGlyphDescriptor[];
  range: Range;
  notes: Note[];
}

export interface Note {
  pitch: string;
  shape: NoteShape;
  modifiers: NoteModifier[];
  range: Range;
}

export enum NoteShape {
  Punctum = 'punctum',
  PunctumInclinatum = 'punctum_inclinatum',
  Virga = 'virga',
  VirgaReversa = 'virga_reversa',
  Oriscus = 'oriscus',
  Quilisma = 'quilisma',
  Stropha = 'stropha',
  Liquescent = 'liquescent',
  Cavum = 'cavum',
  Linea = 'linea',
  Flat = 'flat',
  Sharp = 'sharp',
  Natural = 'natural'
}

export interface NoteModifier {
  type: ModifierType;
  value?: string;
}

export enum ModifierType {
  InitioDebilis = 'initio_debilis',
  PunctumMora = 'punctum_mora',
  HorizontalEpisema = 'horizontal_episema',
  VerticalEpisema = 'vertical_episema',
  Liquescent = 'liquescent',
  Oriscus = 'oriscus',
  Quilisma = 'quilisma',
  Fusion = 'fusion',
  Cavum = 'cavum',
  Strata = 'strata'
}

export interface Clef {
  type: 'c' | 'f';
  line: number;
  hasFlat: boolean;
  range: Range;
}

export interface Bar {
  type: 'virgula' | 'divisio_minima' | 'divisio_minor' | 'divisio_maior' | 'divisio_finalis' | 'dominican';
  range: Range;
}

export interface LineBreak {
  type: 'manual' | 'suggested';
  range: Range;
}

export interface StyleTag {
  type: 'bold' | 'italic' | 'color' | 'small_caps' | 'teletype' | 'underline';
  range: Range;
}

// ========================================
// NABC (St. Gall Notation) Types
// ========================================

export interface NABCGlyphDescriptor {
  basicGlyph: NABCBasicGlyph;
  modifiers?: NABCGlyphModifier[];
  pitch?: string;
  subpunctis?: NABCSubpunctis;
  prepunctis?: NABCPrepunctis;
  range?: Range;
}

export enum NABCBasicGlyph {
  // Single notes
  Virga = 'vi',
  Punctum = 'pu',
  Tractulus = 'ta',
  Gravis = 'gr',
  
  // Two-note neumes
  Clivis = 'cl',
  Pes = 'pe',
  
  // Three-note neumes
  Porrectus = 'po',
  Torculus = 'to',
  Climacus = 'ci',
  Scandicus = 'sc',
  
  // Four-note neumes
  PorrectusFlexus = 'pf',
  ScandicusFlexus = 'sf',
  TorculusResupinus = 'tr',
  
  // Stropha variants
  Stropha = 'st',
  Distropha = 'ds',
  Tristropha = 'ts',
  
  // Other neumes
  Trigonus = 'tg',
  Bivirga = 'bv',
  Trivirga = 'tv',
  PressusMainor = 'pr',
  PressusMinor = 'pi',
  VirgaStrata = 'vs',
  Oriscus = 'or',
  Salicus = 'sa',
  PesQuassus = 'pq',
  Quilisma3Loops = 'ql',
  Quilisma2Loops = 'qi',
  PesStratus = 'pt',
  Nihil = 'ni',
  Uncinus = 'un',
  OriscusClivis = 'oc'
}

export enum NABCGlyphModifier {
  MarkModification = 'S',
  GroupingModification = 'G',
  MelodicModification = 'M',
  Episema = '-',
  AugmentiveLiquescence = '>',
  DiminutiveLiquescence = '~'
}

export interface NABCSubpunctis {
  count?: number;
  modifier?: 'S' | 'G' | 'M';
  range?: Range;
}

export interface NABCPrepunctis {
  count?: number;
  modifier?: 'S' | 'G' | 'M';
  range?: Range;
}
