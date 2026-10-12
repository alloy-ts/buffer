import { Blob, Buffer } from '../index.js';

console.log('--- Blob Example ---');

const blob = new Blob([Buffer.fromString('Hello '), 'World!'], { type: 'text/plain' });

console.log('Blob size:', blob.size);
console.log('Blob type:', blob.type);

const text = await blob.text();
console.log('Blob text:', text);

const sliced = blob.slice(0, 5, 'text/plain');
console.log('Sliced size:', sliced.size);
console.log('Sliced text:', await sliced.text());
