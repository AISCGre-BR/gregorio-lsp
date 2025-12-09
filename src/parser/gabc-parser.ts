/**
 * GABC Parser - Fallback TypeScript Implementation
 * Parses .gabc files according to the GABC specification
 */

import {
  ParsedDocument,
  ParseError,
  NotationSection,
  Syllable,
  NoteGroup,
  Note,
  NoteShape,
  ModifierType,
  Clef,
  Bar,
  Position,
  Range,
  Comment
} from './types';

export class GabcParser {
  private text: string;
  private pos: number;
  private line: number;
  private character: number;
  private errors: ParseError[];
  private comments: Comment[];

  constructor(text: string) {
    this.text = text;
    this.pos = 0;
    this.line = 0;
    this.character = 0;
    this.errors = [];
    this.comments = [];
  }

  parse(): ParsedDocument {
    const headers = this.parseHeaders();
    const notation = this.parseNotation();

    return {
      headers,
      notation,
      comments: this.comments,
      errors: this.errors
    };
  }

  private parseHeaders(): Map<string, string> {
    const headers = new Map<string, string>();
    let inHeader = true;

    while (inHeader && this.pos < this.text.length) {
      this.skipWhitespaceAndComments();

      // Check for header separator
      if (this.peek(2) === '%%') {
        this.advance(2);
        inHeader = false;
        break;
      }

      // Parse header line
      const headerStart = this.getCurrentPosition();
      const nameMatch = this.matchRegex(/^([a-zA-Z0-9-]+):/);
      
      if (nameMatch) {
        const name = nameMatch[1];
        this.advance(nameMatch[0].length);
        
        // Parse header value
        let value = '';
        let multiline = false;

        while (this.pos < this.text.length) {
          const char = this.peek();
          
          if (char === '%') {
            this.parseComment();
            continue;
          }

          if (char === ';') {
            this.advance(1);
            if (this.peek() === ';') {
              this.advance(1);
              multiline = false;
              break;
            }
            break;
          }

          if (char === '\n') {
            if (!multiline && value.trim().length > 0) {
              multiline = true;
            }
            this.advance(1);
            continue;
          }

          value += char;
          this.advance(1);
        }

        headers.set(name.toLowerCase(), value.trim());
      } else {
        this.advance(1);
      }
    }

    return headers;
  }

  private parseNotation(): NotationSection {
    const start = this.getCurrentPosition();
    const syllables: Syllable[] = [];

    while (this.pos < this.text.length) {
      this.skipWhitespaceAndComments();

      if (this.pos >= this.text.length) {
        break;
      }

      // Check for parentheses (note group)
      if (this.peek() === '(') {
        const syllable = this.parseSyllable();
        if (syllable) {
          syllables.push(syllable);
        }
      } else {
        // Parse text outside parentheses
        const textStart = this.getCurrentPosition();
        let text = '';
        
        while (this.pos < this.text.length && this.peek() !== '(' && this.peek() !== '%') {
          text += this.peek();
          this.advance(1);
        }

        if (text.trim().length > 0) {
          // Create syllable with text only
          const textEnd = this.getCurrentPosition();
          syllables.push({
            text: text.trim(),
            notes: [],
            range: { start: textStart, end: textEnd }
          });
        }
      }
    }

    const end = this.getCurrentPosition();

    return {
      syllables,
      range: { start, end }
    };
  }

  private parseSyllable(): Syllable | null {
    const start = this.getCurrentPosition();
    const notes: NoteGroup[] = [];
    let clef: Clef | undefined;
    let bar: Bar | undefined;

    if (this.peek() !== '(') {
      return null;
    }

    this.advance(1); // Skip '('

    const noteStart = this.getCurrentPosition();
    let gabcContent = '';
    let nabcSnippets: string[] = [];

    while (this.pos < this.text.length && this.peek() !== ')') {
      const char = this.peek();

      if (char === '|') {
        // NABC separator
        if (gabcContent.trim().length > 0) {
          // Save GABC content
          const noteGroup = this.parseNoteGroup(gabcContent, nabcSnippets, noteStart);
          if (noteGroup) {
            notes.push(noteGroup);
          }
          gabcContent = '';
          nabcSnippets = [];
        }
        this.advance(1);
        
        // Parse NABC snippet
        const nabcStart = this.pos;
        let nabcContent = '';
        while (this.pos < this.text.length && this.peek() !== ')' && this.peek() !== '|') {
          nabcContent += this.peek();
          this.advance(1);
        }
        nabcSnippets.push(nabcContent);
      } else {
        gabcContent += char;
        this.advance(1);
      }
    }

    if (this.peek() === ')') {
      this.advance(1);
    }

    // Parse final note group
    if (gabcContent.trim().length > 0) {
      const noteGroup = this.parseNoteGroup(gabcContent, nabcSnippets, noteStart);
      if (noteGroup) {
        notes.push(noteGroup);
      }
    }

    // Parse clef if present
    clef = this.parseClef(gabcContent);
    bar = this.parseBar(gabcContent);

    const end = this.getCurrentPosition();

    return {
      text: '',
      notes,
      range: { start, end },
      clef,
      bar
    };
  }

  private parseNoteGroup(gabc: string, nabc: string[], start: Position): NoteGroup | null {
    const notes: Note[] = [];
    const end = this.getCurrentPosition();

    // Parse individual notes from GABC string
    let i = 0;
    while (i < gabc.length) {
      const char = gabc[i];

      // Check for pitch letters
      if (/[a-np]/.test(char.toLowerCase())) {
        const noteStart = { ...start };
        const pitch = char.toLowerCase();
        let shape = NoteShape.Punctum;
        const modifiers: any[] = [];

        i++;

        // Parse modifiers
        while (i < gabc.length) {
          const mod = gabc[i];

          if (mod === 'o') {
            shape = NoteShape.Oriscus;
            i++;
          } else if (mod === 'w') {
            shape = NoteShape.Quilisma;
            i++;
          } else if (mod === 'v') {
            shape = NoteShape.Virga;
            i++;
          } else if (mod === 'V') {
            shape = NoteShape.VirgaReversa;
            i++;
          } else if (mod === 's') {
            shape = NoteShape.Stropha;
            i++;
          } else if (mod === 'r') {
            shape = NoteShape.Cavum;
            i++;
          } else if (mod === '~' || mod === '<' || mod === '>') {
            shape = NoteShape.Liquescent;
            modifiers.push({ type: ModifierType.Liquescent });
            i++;
          } else if (mod === '.') {
            modifiers.push({ type: ModifierType.PunctumMora });
            i++;
          } else if (mod === '_') {
            modifiers.push({ type: ModifierType.HorizontalEpisema });
            i++;
          } else if (mod === "'") {
            modifiers.push({ type: ModifierType.VerticalEpisema });
            i++;
          } else if (mod === '-') {
            modifiers.push({ type: ModifierType.InitioDebilis });
            i++;
          } else {
            break;
          }
        }

        const noteEnd = { ...this.getCurrentPosition() };

        notes.push({
          pitch,
          shape,
          modifiers,
          range: { start: noteStart, end: noteEnd }
        });
      } else {
        i++;
      }
    }

    return {
      gabc,
      nabc: nabc.length > 0 ? nabc : undefined,
      range: { start, end },
      notes
    };
  }

  private parseClef(content: string): Clef | undefined {
    const clefMatch = content.match(/^(c|f)(b)?([1-4])/);
    if (clefMatch) {
      return {
        type: clefMatch[1] as 'c' | 'f',
        line: parseInt(clefMatch[3]),
        hasFlat: !!clefMatch[2],
        range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() }
      };
    }
    return undefined;
  }

  private parseBar(content: string): Bar | undefined {
    const trimmed = content.trim();
    
    if (trimmed === '`' || trimmed === '`0') {
      return { type: 'virgula', range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() } };
    }
    if (trimmed === ',' || trimmed === ',0') {
      return { type: 'divisio_minima', range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() } };
    }
    if (trimmed === ';') {
      return { type: 'divisio_minor', range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() } };
    }
    if (trimmed === ':') {
      return { type: 'divisio_maior', range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() } };
    }
    if (trimmed === '::') {
      return { type: 'divisio_finalis', range: { start: this.getCurrentPosition(), end: this.getCurrentPosition() } };
    }

    return undefined;
  }

  private parseComment(): void {
    if (this.peek() !== '%') {
      return;
    }

    const start = this.getCurrentPosition();
    this.advance(1); // Skip '%'

    let text = '';
    while (this.pos < this.text.length && this.peek() !== '\n') {
      text += this.peek();
      this.advance(1);
    }

    const end = this.getCurrentPosition();
    this.comments.push({
      text,
      range: { start, end }
    });
  }

  private skipWhitespaceAndComments(): void {
    while (this.pos < this.text.length) {
      const char = this.peek();

      if (char === '%') {
        this.parseComment();
        continue;
      }

      if (/\s/.test(char)) {
        this.advance(1);
        continue;
      }

      break;
    }
  }

  private peek(length: number = 1): string {
    return this.text.substring(this.pos, this.pos + length);
  }

  private advance(count: number): void {
    for (let i = 0; i < count && this.pos < this.text.length; i++) {
      if (this.text[this.pos] === '\n') {
        this.line++;
        this.character = 0;
      } else {
        this.character++;
      }
      this.pos++;
    }
  }

  private getCurrentPosition(): Position {
    return { line: this.line, character: this.character };
  }

  private matchRegex(regex: RegExp): RegExpMatchArray | null {
    const remaining = this.text.substring(this.pos);
    return remaining.match(regex);
  }

  addError(message: string, range: Range, severity: 'error' | 'warning' | 'info' = 'error'): void {
    this.errors.push({ message, range, severity });
  }
}
