import * as i18next from 'i18next';

const resources = {
	en: {
		translation: {
      trending: 'NO',
      subscribed_to_communities:'Subscribed to <1>communities</1>',
      create_a_community: 'Create a community',






      foo: 'foo',
			bar: '<1>bar</1>',
			baz: '<1>{{count}}</1>',
			qux: 'qux<1></1>',
			qux_plural: 'quxes<1></1>',
			quux: '<1>{{name, uppercase}}</1>',
			userMessagesUnread: 'Hello <1>{{name}}</1>, you have {{count}} unread messages. <3>Go to messages</3>.',
			userMessagesUnread_plural: 'Hello <1>{{name}}</1>, you have {{count}} unread messages. <3>Go to messages</3>.'
		},
	},
};

function format(value: any, format: any, lng: any) {
	if (format === 'uppercase') return value.toUpperCase();
	return value;
}

i18next.init({
	lng: 'en',
	resources,
	interpolation: {
		format: format
	}
});

export { i18next, resources };
