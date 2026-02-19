#!/usr/bin/env node
const fs = require('fs');

try {
    // Read input from stdin
    const inputData = fs.readFileSync(0, 'utf-8');
    if (!inputData) process.exit(0);

    const json = JSON.parse(inputData);
    const userInput = json.input || '';

    // Process
    const response = { response: `Hello from Node! You said: ${userInput}` };

    // Write response to stdout
    console.log(JSON.stringify(response));
} catch (e) {
    console.error(JSON.stringify({ error: e.message }));
    process.exit(1);
}