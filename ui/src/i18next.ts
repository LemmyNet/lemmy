import i18next from 'i18next';
import { getLanguage } from './utils';
import XHR from 'i18next-xhr-backend';

function format(value: any, format: any, lng: any): any {
  return format === 'uppercase' ? value.toUpperCase() : value;
}

i18next
  .use(XHR)
  .init({
    debug: true,
    //load: 'languageOnly',

    // initImmediate: false,
    lng: getLanguage(),
    fallbackLng: 'en',
    interpolation: { format },
    backend: {
      loadPath: '/static/assets/translations/{{lng}}.json',
    }
});

export { i18next as i18n, resources };
