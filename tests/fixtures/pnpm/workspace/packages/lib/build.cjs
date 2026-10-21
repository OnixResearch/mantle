const fs = require('node:fs');
const value = require('@fixture/consumer');
if (value !== 'peer:patched') throw new Error(`patch/peer resolution drift: ${value}`);
if (require('@fixture/git-archive') !== 'git-archive') throw new Error('Git archive source drift');
fs.mkdirSync('dist', {recursive:true});
fs.writeFileSync('dist/index.js', `module.exports = ${JSON.stringify(value)};\n`);
