/**
 * NABC (St. Gall Notation) Parser
 * Parses NABC glyph descriptors according to the Gregorio specification
 */

import {
  NABCGlyphDescriptor,
  NABCBasicGlyph,
  NABCGlyphModifier,
  NABCSubpunctis,
  NABCPrepunctis,
  Range,
  Position
} from './types';

/**
 * Parse a single NABC snippet into a glyph descriptor
 */
export function parseNABCSnippet(nabc: string, startPos?: Position): NABCGlyphDescriptor | null {
  if (!nabc || nabc.trim().length === 0) {
    return null;
  }

  let pos = 0;
  // Remove all whitespace from NABC snippet
  const trimmed = nabc.trim().replace(/\s+/g, '');

  // Check for subpunctis/prepunctis first
  if (trimmed.startsWith('su') || trimmed.startsWith('pp')) {
    return parseSubpunctisPrepunctis(trimmed, startPos);
  }

  // Parse basic glyph descriptor (2 letters)
  const basicGlyph = parseBasicGlyph(trimmed.substring(pos, pos + 2));
  if (!basicGlyph) {
    return null;
  }
  pos += 2;

  const result: NABCGlyphDescriptor = {
    basicGlyph
  };

  // Parse modifiers (S, G, M, -, >, ~)
  const modifiers: NABCGlyphModifier[] = [];
  while (pos < trimmed.length) {
    const char = trimmed[pos];
    if (char === 'S') {
      modifiers.push(NABCGlyphModifier.MarkModification);
      pos++;
    } else if (char === 'G') {
      modifiers.push(NABCGlyphModifier.GroupingModification);
      pos++;
    } else if (char === 'M') {
      modifiers.push(NABCGlyphModifier.MelodicModification);
      pos++;
    } else if (char === '-') {
      modifiers.push(NABCGlyphModifier.Episema);
      pos++;
    } else if (char === '>') {
      modifiers.push(NABCGlyphModifier.AugmentiveLiquescence);
      pos++;
    } else if (char === '~') {
      modifiers.push(NABCGlyphModifier.DiminutiveLiquescence);
      pos++;
    } else if (/[1-9]/.test(char)) {
      // Variant number - skip for now
      pos++;
    } else {
      break;
    }
  }

  if (modifiers.length > 0) {
    result.modifiers = modifiers;
  }

  // Parse pitch descriptor (h + pitch letter)
  if (pos < trimmed.length && trimmed[pos] === 'h') {
    pos++;
    if (pos < trimmed.length && /[a-np]/.test(trimmed[pos])) {
      result.pitch = trimmed[pos];
      pos++;
    }
  }

  if (startPos) {
    result.range = {
      start: startPos,
      end: { line: startPos.line, character: startPos.character + trimmed.length }
    };
  }

  return result;
}

/**
 * Parse basic glyph descriptor (2-letter code)
 */
function parseBasicGlyph(code: string): NABCBasicGlyph | null {
  // Map all valid NABC glyph descriptors
  const glyphMap: Record<string, NABCBasicGlyph> = {
    'vi': NABCBasicGlyph.Virga,
    'pu': NABCBasicGlyph.Punctum,
    'ta': NABCBasicGlyph.Tractulus,
    'gr': NABCBasicGlyph.Gravis,
    'cl': NABCBasicGlyph.Clivis,
    'pe': NABCBasicGlyph.Pes,
    'po': NABCBasicGlyph.Porrectus,
    'to': NABCBasicGlyph.Torculus,
    'ci': NABCBasicGlyph.Climacus,
    'sc': NABCBasicGlyph.Scandicus,
    'pf': NABCBasicGlyph.PorrectusFlexus,
    'sf': NABCBasicGlyph.ScandicusFlexus,
    'tr': NABCBasicGlyph.TorculusResupinus,
    'st': NABCBasicGlyph.Stropha,
    'ds': NABCBasicGlyph.Distropha,
    'ts': NABCBasicGlyph.Tristropha,
    'tg': NABCBasicGlyph.Trigonus,
    'bv': NABCBasicGlyph.Bivirga,
    'tv': NABCBasicGlyph.Trivirga,
    'pr': NABCBasicGlyph.PressusMainor,
    'pi': NABCBasicGlyph.PressusMinor,
    'vs': NABCBasicGlyph.VirgaStrata,
    'or': NABCBasicGlyph.Oriscus,
    'sa': NABCBasicGlyph.Salicus,
    'pq': NABCBasicGlyph.PesQuassus,
    'ql': NABCBasicGlyph.Quilisma3Loops,
    'qi': NABCBasicGlyph.Quilisma2Loops,
    'pt': NABCBasicGlyph.PesStratus,
    'ni': NABCBasicGlyph.Nihil,
    'un': NABCBasicGlyph.Uncinus,
    'oc': NABCBasicGlyph.OriscusClivis
  };

  return glyphMap[code] || null;
}

/**
 * Parse subpunctis or prepunctis descriptor
 */
function parseSubpunctisPrepunctis(nabc: string, startPos?: Position): NABCGlyphDescriptor | null {
  const type = nabc.substring(0, 2);
  let pos = 2;

  // Parse optional count (digit)
  let count: number | undefined;
  if (pos < nabc.length && /[1-9]/.test(nabc[pos])) {
    count = parseInt(nabc[pos]);
    pos++;
  }

  // Parse optional modifier (S, G, M)
  let modifier: 'S' | 'G' | 'M' | undefined;
  if (pos < nabc.length && /[SGM]/.test(nabc[pos])) {
    modifier = nabc[pos] as 'S' | 'G' | 'M';
    pos++;
  }

  // Create appropriate descriptor
  const descriptor: NABCGlyphDescriptor = {
    basicGlyph: NABCBasicGlyph.Punctum, // Use punctum as placeholder
  };

  if (type === 'su') {
    descriptor.subpunctis = { count, modifier };
  } else if (type === 'pp') {
    descriptor.prepunctis = { count, modifier };
  }

  if (startPos) {
    descriptor.range = {
      start: startPos,
      end: { line: startPos.line, character: startPos.character + nabc.length }
    };
  }

  return descriptor;
}

/**
 * Parse all NABC snippets from an array
 */
export function parseNABCSnippets(nabcArray: string[], startPos?: Position): NABCGlyphDescriptor[] {
  return nabcArray
    .map((nabc, index) => {
      const pos = startPos ? {
        line: startPos.line,
        character: startPos.character + index * 10 // Approximate offset
      } : undefined;
      return parseNABCSnippet(nabc, pos);
    })
    .filter((d): d is NABCGlyphDescriptor => d !== null);
}

/**
 * Validate NABC glyph descriptor
 */
export function validateNABCDescriptor(descriptor: NABCGlyphDescriptor): string[] {
  const errors: string[] = [];

  // Check for valid basic glyph
  if (!descriptor.basicGlyph && !descriptor.subpunctis && !descriptor.prepunctis) {
    errors.push('NABC descriptor must have a basic glyph, subpunctis, or prepunctis');
  }

  // Check pitch descriptor format
  if (descriptor.pitch && !/[a-np]/.test(descriptor.pitch)) {
    errors.push(`Invalid NABC pitch descriptor: ${descriptor.pitch}`);
  }

  // Check modifier combinations
  if (descriptor.modifiers && descriptor.modifiers.length > 0) {
    const hasLiquescence = descriptor.modifiers.some(
      m => m === NABCGlyphModifier.AugmentiveLiquescence || 
           m === NABCGlyphModifier.DiminutiveLiquescence
    );
    
    // Liquescence modifiers should be mutually exclusive
    if (descriptor.modifiers.includes(NABCGlyphModifier.AugmentiveLiquescence) &&
        descriptor.modifiers.includes(NABCGlyphModifier.DiminutiveLiquescence)) {
      errors.push('NABC descriptor cannot have both augmentive and diminutive liquescence');
    }
  }

  return errors;
}

/**
 * Get all valid NABC glyph descriptor codes
 */
export function getAllNABCGlyphCodes(): string[] {
  return Object.values(NABCBasicGlyph);
}

/**
 * Check if a string is a valid NABC glyph descriptor
 */
export function isValidNABCGlyph(code: string): boolean {
  return parseBasicGlyph(code) !== null;
}
