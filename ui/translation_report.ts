import { en } from './src/translations/en';
import { es } from './src/translations/es';
import { de } from './src/translations/de';
import { zh } from './src/translations/zh';
import { fr } from './src/translations/fr';
import { sv } from './src/translations/sv';
import { ru } from './src/translations/ru';

let files = [
  {t: es, n: 'es'}, 
  {t: de, n: 'de'}, 
  {t: zh, n: 'zh'}, 
  {t: fr, n: 'fr'}, 
  {t: sv, n: 'sv'}, 
  {t: ru, n: 'ru'}, 
];
let masterKeys = Object.keys(en.translation);

let report = 'lang | missing | percent\n';
report += '--- | --- | ---\n';

for (let file of files) {
  let keys = Object.keys(file.t.translation);
  let pct: number = (keys.length / masterKeys.length * 100);
  let missing = difference(masterKeys, keys);
  report += `${file.n} | ${missing} | ${pct.toFixed(0)}%\n`;
}

console.log(report);

function difference(a: Array<string>, b: Array<string>): Array<string> {
  return a.filter(x => !b.includes(x));
}
