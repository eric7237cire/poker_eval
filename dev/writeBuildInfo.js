const fs = require('fs');
const path = require('path'); 
const { format } = require(path.join(__dirname, '../vue-poker/node_modules', 'date-fns'));

// Get the current date and time in ISO format
const currentDate = format(new Date(), 'yyyy/MM/dd HH:mm:ss');

const envVariableValue = process.env.GITHUB_SHA || 'Local';

// Create a string with the date, time, and environment variable value
const content = `Built: ${currentDate}\nCommit Hash: ${envVariableValue}`;

// Write the content to a text file (e.g., datetime.txt)
fs.writeFileSync('../vue-poker/public/build_info.txt', content, 'utf-8');

console.log('Date, time, and environment variable value written to build_info.txt');
