fs = require('fs');

fs.mkdirSync('src/translations/', { recursive: true });
fs.readdir('translations', (err, files) => {
  files.forEach(filename => {
    const lang = filename.split('.')[0];
    try {
      const json = JSON.parse(
        fs.readFileSync('translations/' + filename, 'utf8')
      );
      var data = `export const ${lang} = {\n  translation: {`;
      for (var key in json) {
        if (key in json) {
          const value = json[key].replace(/"/g, '\\"');
          data = `${data}\n    ${key}: "${value}",`;
        }
      }
      data += '\n  },\n};';
      const target = 'src/translations/' + lang + '.ts';
      fs.writeFileSync(target, data);
    } catch (err) {
      console.error(err);
    }
  });
});
