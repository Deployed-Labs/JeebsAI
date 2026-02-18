#!/usr/bin/env node
const fs = require('fs');

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const obj = JSON.parse(input || '{}');
    const resp = obj.input ? `Node hello: ${obj.input}` : 'Node hello: (no input)';
    console.log(JSON.stringify({ response: resp }));
  } catch (err) {
    console.log(JSON.stringify({ response: 'invalid input' }));
  }
});
