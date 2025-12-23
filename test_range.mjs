import { parseNABCSnippet } from './dist/parser/nabc-parser.js';

const startPos = { line: 0, character: 0 };
const result = parseNABCSnippet('vilsabc1', startPos);

console.log('Parse result for "vilsabc1":');
console.log('Number of glyphs:', result.length);

if (result.length > 0) {
  const glyph = result[0];
  console.log('\nFirst glyph:');
  console.log('- basicGlyph:', glyph.basicGlyph);
  console.log('- range:', JSON.stringify(glyph.range, null, 2));
  console.log('- significantLetters count:', glyph.significantLetters?.length || 0);
  
  if (glyph.significantLetters && glyph.significantLetters.length > 0) {
    console.log('\nFirst significant letter:');
    const letter = glyph.significantLetters[0];
    console.log('- type:', letter.type);
    console.log('- code:', letter.code);
    console.log('- position:', letter.position);
    console.log('- range:', JSON.stringify(letter.range, null, 2));
  }
}
