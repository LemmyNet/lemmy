const fs = require('fs');

exports.setVersion =  function() {
  let revision = require('child_process')
    .execSync('git describe --tags --long')
    .toString().trim();
  let line = `export let version: string = "${revision}";`;
  fs.writeFileSync("./src/version.ts", line);
}

this.setVersion()
