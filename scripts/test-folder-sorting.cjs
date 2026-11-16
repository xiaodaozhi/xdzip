// Exercise the actual HTML query function without starting a browser.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const source = fs.readFileSync(process.argv[2] || path.join(__dirname, '../design/html/app.js'), 'utf8');
const start = source.indexOf('function visibleEntries() {');
const end = source.indexOf('\nfunction fileIcon(', start);
assert(start >= 0 && end > start);
const query = source.slice(start, end);
const entries = [
    {id: 'file-a', parent: '', name: 'a.txt', size: 1, compressed: 1, type: 'A', modified: '2025-01-01'},
    {id: 'folder-z', parent: '', name: 'z-folder', folder: true},
    {id: 'file-z', parent: '', name: 'z.txt', size: 9, compressed: 8, type: 'Z', modified: '2026-01-01'},
    {id: 'folder-b', parent: '', name: 'b-folder', folder: true},
    {id: 'child', parent: 'folder-z', name: 'child.txt'},
];
function rows(sort, sortKey, text = '') {
    const context = {entries, state: {archiveState: 'ready', path: '', query: text, sort, sortKey},
        withinDirectory: () => true, archivePath: entry => entry.name};
    vm.runInNewContext(query + '\nresult = visibleEntries();', context);
    return Array.from(context.result);
}
assert.deepEqual(rows(0, 'name').map(entry => entry.id), ['folder-z', 'folder-b', 'file-a', 'file-z']);
assert.deepEqual(rows(1, 'name').map(entry => entry.id), ['folder-b', 'folder-z', 'file-a', 'file-z']);
assert.deepEqual(rows(-1, 'name').map(entry => entry.id), ['folder-z', 'folder-b', 'file-z', 'file-a']);
for (const text of ['', 'folder', '.txt']) {
    for (const key of ['name', 'size', 'compressed', 'type', 'modified']) {
        for (const direction of [0, 1, -1]) {
            let filesStarted = false;
            for (const entry of rows(direction, key, text)) {
                assert(!entry.folder || !filesStarted, `${text}/${key}/${direction}`);
                filesStarted ||= !entry.folder;
            }
        }
    }
}
console.log('HTML default order and five sort fields keep directories first in both directions and search');
