const fs = require('fs');
const path = require('path'); 

// Get the current date and time in ISO format
const currentDate = new Date();

const githubSha = process.env.GITHUB_SHA || 'Local';


const bi = {
    build_date: currentDate.getTime(),
    github_sha: githubSha,
}
// Create a string with the date, time, and environment variable value
const content = JSON.stringify(bi);

// Write the content to a text file (e.g., datetime.txt)
fs.writeFileSync('../vue-poker/public/build_info.txt', content, 'utf-8');

console.log('Date, time, and environment variable value written to build_info.txt');
