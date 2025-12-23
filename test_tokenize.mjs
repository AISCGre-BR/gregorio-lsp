import { parseNABCSnippet } from './dist/parser/nabc-parser.js';

// Simular o que o tokenizer faz
const text = "exemplo com vilsabc1 no meio";
const lines = text.split('\n');
const startPos = { line: 0, character: 12 }; // posição onde 'vilsabc1' começa

const glyphs = parseNABCSnippet('vilsabc1', startPos);
console.log('Number of glyphs:', glyphs.length);

if (glyphs.length > 0) {
  const glyph = glyphs[0];
  console.log('\nGlyph range:', glyph.range);
  
  const glyphText = lines[glyph.range.start.line]?.substring(
    glyph.range.start.character, 
    glyph.range.end.character
  ) || '';
  console.log('Glyph text extracted:', glyphText);
  
  if (glyph.significantLetters && glyph.significantLetters.length > 0) {
    const letter = glyph.significantLetters[0];
    console.log('\nLetter range:', letter.range);
    
    const letterText = lines[letter.range.start.line]?.substring(
      letter.range.start.character,
      letter.range.end.character
    ) || '';
    console.log('Letter text extracted:', letterText);
  }
}
