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
import { fi } from './src/translations/fi';
import fs from 'fs';

const files = [
  { t: de, n: 'de' },
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

const difference = (a: Array<string>, b: Array<string>): Array<string> => a.filter(x => !b.includes(x));

const report = 
  'lang | done | missing\n' +
  '---- | ---- | -------\n' +
  files.map(file => {
    const keys = Object.keys(file.t.translation);
    const pct: number = (keys.length / masterKeys.length) * 100;
    const missing = difference(masterKeys, keys);
    return `${file.n} | ${pct.toFixed(0)}% | ${missing}`;
  }).join("\n");

const alteredReadmeTxt = `${before}${open}\n\n${report}\n${close}${after}`;

fs.writeFileSync(readmePath, alteredReadmeTxt);
