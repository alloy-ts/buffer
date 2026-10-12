import { File, Buffer, atob, btoa, isAscii, isLatin1, isUtf8 } from '../index.js';

console.log('--- File & Utility Example ---');

const file = new File(['file content line 1\nline 2'], 'example.txt', {
  type: 'text/plain',
  lastModified: Date.now(),
});

console.log('File name:', file.name);
console.log('File size:', file.size);
console.log('File lastModified:', file.lastModified);
console.log('File text:', await file.text());

// Base64 helpers
const encoded = btoa('Hello World');
console.log('btoa:', encoded);
console.log('atob:', atob(encoded));

// Encoding validation
console.log('isAscii("Hello"):', isAscii(Buffer.fromString('Hello').bytes()));
console.log('isLatin1("café"):', isLatin1('café'));
console.log('isUtf8(UTF8 bytes):', isUtf8(Buffer.fromString('UTF8 string').bytes()));
