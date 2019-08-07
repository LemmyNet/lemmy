import * as i18next from 'i18next';

// https://github.com/nimbusec-oss/inferno-i18next/blob/master/tests/T.test.js#L66
const resources = {
	en: {
		translation: {
      subscribed_to_communities:'Subscribed to <1>communities</1>',
      create_a_community: 'Create a community',
      trending_communities:'Trending <1>communities</1>',
      edit: 'edit',
      number_of_users:'{{count}} Users',
      number_of_posts:'{{count}} Posts',
      number_of_comments:'{{count}} Comments',
      modlog: 'Modlog',
      admins: 'admins',
      powered_by: 'Powered by',
      landing_0: 'Lemmy is a <1>link aggregator</1> / reddit alternative, intended to work in the <2>fediverse</2>.<3></3>Its self-hostable, has live-updating comment threads, and is tiny (<4>~80kB</4>). Federation into the ActivityPub network is on the roadmap. <5></5>This is a <6>very early beta version</6>, and a lot of features are currently broken or missing. <7></7>Suggest new features or report bugs <8>here.</8><9></9>Made with <10>Rust</10>, <11>Actix</11>, <12>Inferno</12>, <13>Typescript</13>.',


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
