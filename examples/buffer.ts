import { Buffer } from '../index.js';

console.log('--- Buffer Example ---');

// Allocate buffer
const buf1 = Buffer.alloc(10, 65); // Fill with 'A'
console.log('buf1 string:', buf1.toString('utf8', 0, 10)); // AAAAAAAAAA

// Create from string
const buf2 = Buffer.fromString('Hello, NAPI-RS Buffer!');
console.log('buf2 length:', buf2.length);
console.log('buf2 string:', buf2.toString());

// Write numeric values
const buf3 = Buffer.alloc(8);
buf3.writeInt32BE(0x12345678, 0);
buf3.writeFloatBE(3.14159, 4);

console.log('Read Int32BE:', buf3.readInt32BE(0).toString(16));
console.log('Read FloatBE:', buf3.readFloatBE(4).toFixed(5));

// Copy & slice operations
const target = Buffer.alloc(5);
buf2.copy(target, 0, 0, 5);
console.log('Copied target:', target.toString());

// Encoding checks
console.log('isEncoding("utf8"):', Buffer.isEncoding('utf8'));
console.log('isEncoding("invalid"):', Buffer.isEncoding('invalid'));
