import i18next from 'i18next';
import { getLanguage } from './utils';
import { en } from './translations/en';
import { eo } from './translations/eo';
import { es } from './translations/es';
import { de } from './translations/de';
import { fr } from './translations/fr';
import { sv } from './translations/sv';
import { ru } from './translations/ru';
import { zh } from './translations/zh';
import { nl } from './translations/nl';
import { it } from './translations/it';
import { fi } from './translations/fi';
import { ca } from './translations/ca';
import { fa } from './translations/fa';

// https://github.com/nimbusec-oss/inferno-i18next/blob/master/tests/T.test.js#L66
const resources = {
  en,
  eo,
  es,
  de,
  zh,
  fr,
  sv,
  ru,
  nl,
  it,
  fi,
  ca,
  fa,
};

function format(value: any, format: any, lng: any): any {
  return format === 'uppercase' ? value.toUpperCase() : value;
}

i18next.init({
  debug: false,
  // load: 'languageOnly',

  // initImmediate: false,
  lng: getLanguage(),
  fallbackLng: 'en',
  resources,
  interpolation: { format },
});

export { i18next as i18n, resources };
