const fs = require('node:fs');
const value = require('@fixture/lib');
if (value !== 'peer:patched') throw new Error(`workspace link drift: ${value}`);
try { require.resolve('@fixture/darwin-only'); throw new Error('platform optional package incorrectly installed'); }
catch (error) { if (error.code !== 'MODULE_NOT_FOUND') throw error; }
fs.mkdirSync('dist', {recursive:true});
fs.writeFileSync('dist/index.html', `<h1>${value}</h1>\n`);
