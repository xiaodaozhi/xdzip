// Owned test-only adapter for the pinned author's public ASAR API.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Readable } from 'node:stream';
import { createHash } from 'node:crypto';

const [reference, output] = process.argv.slice(2);
if (!reference || !output) throw new Error('Provide the pinned reference root and owned output');
const asar = await import(pathToFileURL(path.resolve(reference, 'node_modules/@electron/asar/lib/asar.js')).href);
fs.mkdirSync(output, { recursive: true });
const phrase = Buffer.from('abracadabra abracadabra\n'.repeat(6));
const unicode = Buffer.from('owned UTF-8 ASAR content\n资料\n');
const ordinary = new Map([['hello.txt', phrase], ['assets/资料.txt', unicode], ['empty.bin', Buffer.alloc(0)], ['copy.txt', phrase]]);
const streams = [{ type: 'directory', path: 'assets', unpacked: false }, { type: 'directory', path: 'empty-dir', unpacked: false }];
for (const [name, data] of ordinary) streams.push({ type: 'file', path: name, unpacked: false, stat: { size: data.length, mode: 0o100644 }, streamGenerator: () => Readable.from([data]) });
const target = path.join(output, 'ordinary.asar');
await asar.createPackageFromStreams(target, streams);

function wrap(name, header, body) {
    const json = Buffer.from(JSON.stringify(header));
    const padded = (json.length + 3) & ~3;
    const prefix = Buffer.alloc(16 + padded);
    prefix.writeUInt32LE(4, 0);
    prefix.writeUInt32LE(8 + padded, 4);
    prefix.writeUInt32LE(4 + padded, 8);
    prefix.writeUInt32LE(json.length, 12);
    json.copy(prefix, 16);
    fs.writeFileSync(path.join(output, name), Buffer.concat([prefix, body]));
}
function digest(bytes) { return createHash('sha256').update(bytes).digest('hex'); }
function integrity(bytes) {
    const blocks = [];
    for (let at = 0; at < bytes.length; at += 64) blocks.push(digest(bytes.subarray(at, at + 64)));
    // The empty file has the SHA-256 empty-message block in the author profile.
    if (!blocks.length) blocks.push(digest(Buffer.alloc(0)));
    return { algorithm: 'SHA256', hash: digest(bytes), blockSize: 64, blocks };
}
const raw = asar.getRawHeader(target);
const body = fs.readFileSync(target).subarray(8 + raw.headerSize);
const hashless = structuredClone(raw.header);
function stripIntegrity(node) {
    delete node.integrity;
    if (node.files) for (const child of Object.values(node.files)) stripIntegrity(child);
}
stripIntegrity(hashless);
wrap('without-integrity.asar', hashless, body);
const record = { size: phrase.length, offset: '0', integrity: integrity(phrase) };
wrap('shared.asar', { files: { 'hello.txt': record, 'copy.txt': structuredClone(record), 'empty-dir': { files: {} } } }, phrase);
wrap('executable.asar', { files: { 'run.txt': { ...record, executable: true } } }, phrase);
wrap('external.asar', { files: { 'outside.bin': { size: phrase.length, unpacked: true, integrity: integrity(phrase) } } }, Buffer.alloc(0));
wrap('link.asar', { files: { 'link.txt': { link: 'hello.txt' }, 'hello.txt': record } }, phrase);
wrap('empty.asar', { files: {} }, Buffer.alloc(0));

for (const name of ['ordinary.asar', 'without-integrity.asar']) {
    const file = path.join(output, name);
    asar.uncache(file);
    for (const [name, expected] of ordinary) {
        if (!asar.extractFile(file, name).equals(expected)) throw new Error('Independent ordinary payload mismatch');
    }
}
for (const name of ['hello.txt', 'copy.txt']) {
    if (!asar.extractFile(path.join(output, 'shared.asar'), name).equals(phrase)) throw new Error('Independent shared payload mismatch');
}
if (!asar.extractFile(path.join(output, 'executable.asar'), 'run.txt').equals(phrase)) throw new Error('Independent executable payload mismatch');
if (!asar.extractFile(path.join(output, 'link.asar'), 'hello.txt').equals(phrase)) throw new Error('Independent link target mismatch');
if (!asar.statFile(path.join(output, 'link.asar'), 'link.txt', false).link) throw new Error('Independent link metadata mismatch');
if (!asar.statFile(path.join(output, 'external.asar'), 'outside.bin', false).unpacked) throw new Error('Independent external metadata mismatch');
if (Object.keys(asar.getRawHeader(path.join(output, 'empty.asar')).header.files).length) throw new Error('Independent empty tree mismatch');
console.log(JSON.stringify({ ordinaryHeader: raw.header, cases: 7, phraseBytes: phrase.length }));
