import { en } from './src/translations/en';
import { eo } from './src/translations/eo';
import { es } from './src/translations/es';
import { de } from './src/translations/de';
import { zh } from './src/translations/zh';
import { fr } from './src/translations/fr';
import { sv } from './src/translations/sv';
import { ru } from './src/translations/ru';
import { nl } from './src/translations/nl';
import { it } from './src/translations/it';

let files = [
  { t: de, n: 'de' },
  { t: eo, n: 'eo' },
  { t: es, n: 'es' },
  { t: fr, n: 'fr' },
  { t: it, n: 'it' },
  { t: nl, n: 'nl' },
  { t: ru, n: 'ru' },
  { t: sv, n: 'sv' },
  { t: zh, n: 'zh' },
];
let masterKeys = Object.keys(en.translation);

let report = 'lang | done | missing\n';
report += '--- | --- | ---\n';

for (let file of files) {
  let keys = Object.keys(file.t.translation);
  let pct: number = (keys.length / masterKeys.length) * 100;
  let missing = difference(masterKeys, keys);
  report += `${file.n} | ${pct.toFixed(0)}% | ${missing} \n`;
}

console.log(report);

function difference(a: Array<string>, b: Array<string>): Array<string> {
  return a.filter(x => !b.includes(x));
}
