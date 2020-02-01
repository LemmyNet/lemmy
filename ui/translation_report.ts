import { en } from './src/translations/en';
import { eo } from './src/translations/eo';
import { es } from './src/translations/es';
import { de } from './src/translations/de';
import { fa } from './src/translations/fa';
import { zh } from './src/translations/zh';
import { fr } from './src/translations/fr';
import { sv } from './src/translations/sv';
import { ru } from './src/translations/ru';
import { nl } from './src/translations/nl';
import { it } from './src/translations/it';
import { fi } from './src/translations/fi';
import { ca } from './src/translations/ca';
import fs from 'fs';

const files = [
  { t: ca, n: 'ca' },
  { t: de, n: 'de' },
  { t: fa, n: 'fa' },
  { t: eo, n: 'eo' },
  { t: es, n: 'es' },
  { t: fi, n: 'fi' },
  { t: fr, n: 'fr' },
  { t: it, n: 'it' },
  { t: nl, n: 'nl' },
  { t: ru, n: 'ru' },
  { t: sv, n: 'sv' },
  { t: zh, n: 'zh' },
];
const masterKeys = Object.keys(en.translation);

const readmePath = '../README.md';

const open = '<!-- translations -->';
const close = '<!-- translationsstop -->';

const readmeTxt = fs.readFileSync(readmePath, { encoding: 'utf8' });

const before = readmeTxt.split(open)[0];
const after = readmeTxt.split(close)[1];

function difference(a: Array<string>, b: Array<string>): Array<string> {
  return a.filter(x => !b.includes(x));
}

const report =
  'lang | done | missing\n' +
  '---- | ---- | -------\n' +
  files
    .map(file => {
      const keys = Object.keys(file.t.translation);
      const pct: number = (keys.length / masterKeys.length) * 100;
      const missing = difference(masterKeys, keys);
      return `${file.n} | ${pct.toFixed(0)}% | ${missing}`;
    })
    .join('\n');

const alteredReadmeTxt = `${before}${open}\n\n${report}\n${close}${after}`;

fs.writeFileSync(readmePath, alteredReadmeTxt);
