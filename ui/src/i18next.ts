import * as i18n from 'i18next';

// https://github.com/nimbusec-oss/inferno-i18next/blob/master/tests/T.test.js#L66
//
// TODO don't forget to add moment locales for new languages.
const resources = {
	en: {
		translation: {
      post: 'post',
      edit: 'edit',
      reply: 'reply',
      cancel: 'Cancel',
      unlock: 'unlock',
      lock: 'lock',
      link: 'link',
      mod: 'mod',
      mods: 'mods',
      moderates: 'Moderates',
      admin: 'admin',
      admins: 'admins',
      modlog: 'Modlog',
      remove: 'remove',
      removed: 'removed',
      locked: 'locked',
      reason: 'Reason',
      remove_as_mod: 'remove as mod',
      appoint_as_mod: 'appoint as mod',
      remove_as_admin: 'remove as admin',
      appoint_as_admin: 'appoint as admin',
      mark_as_read: 'mark as read',
      mark_as_unread: 'mark as unread',
      remove_comment: 'Remove Comment',
      remove_community: 'Remove Community',
      delete: 'delete',
      deleted: 'deleted',
      restore: 'restore',
      ban: 'ban',
      unban: 'unban',
      ban_from_site: 'ban from site',
      unban_from_site: 'unban from site',
      save: 'save',
      unsave: 'unsave',
      create: 'create',
      subscribed_to_communities:'Subscribed to <1>communities</1>',
      create_a_community: 'Create a community',
      create_community: 'Create Community',
      create_a_post: 'Create a post',
      create_post: 'Create Post',
      trending_communities:'Trending <1>communities</1>',
      number_of_users:'{{count}} Users',
      number_of_subscribers:'{{count}} Subscribers',
      number_of_posts:'{{count}} Posts',
      number_of_comments:'{{count}} Comments',
      number_of_points:'{{count}} Points',
      powered_by: 'Powered by',
      landing_0: 'Lemmy is a <1>link aggregator</1> / reddit alternative, intended to work in the <2>fediverse</2>.<3></3>Its self-hostable, has live-updating comment threads, and is tiny (<4>~80kB</4>). Federation into the ActivityPub network is on the roadmap. <5></5>This is a <6>very early beta version</6>, and a lot of features are currently broken or missing. <7></7>Suggest new features or report bugs <8>here.</8><9></9>Made with <10>Rust</10>, <11>Actix</11>, <12>Inferno</12>, <13>Typescript</13>.',
      list_of_communities: 'List of communities',
      name: 'Name',
      title: 'Title',
      category: 'Category',
      subscribers: 'Subscribers',
      both: 'Both',
      posts: 'Posts',
      comments: 'Comments',
      saved: 'Saved',
      unsubscribe: 'Unsubscribe',
      subscribe: 'Subscribe',
      prev: 'Prev',
      next: 'Next',
      sidebar: 'Sidebar',
      community_reqs: 'lowercase, underscores, and no spaces.',
      sort_type: 'Sort type',
      hot: 'Hot',
      new: 'New',
      top_day: 'Top day',
      week: 'Week',
      month: 'Month',
      year: 'Year',
      all: 'All',
      top: 'Top',
      
      api: 'API',
      sponsors: 'Sponsors',
      sponsors_of_lemmy: 'Sponsors of Lemmy',
      sponsor_message: 'Lemmy is free, <1>open-source</1> software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project. Thank you to the following people:',
      support_on_patreon: 'Support on Patreon',
      general_sponsors:'General Sponsors are those that pledged $10 to $39 to Lemmy.',
      bitcoin: 'Bitcoin',
      ethereum: 'Ethereum',
      code: 'Code',

      inbox: 'Inbox',
      inbox_for: 'Inbox for <1>{{user}}</1>',
      mark_all_as_read: 'mark all as read',
      type: 'Type',
      unread: 'Unread',
      reply_sent: 'Reply sent',
      
      communities: 'Communities',
      search: 'Search',
      overview: 'Overview',
      view: 'View',
      logout: 'Logout',
      login_sign_up: 'Login / Sign up',
      notifications_error: 'Desktop notifications not available in your browser. Try Firefox or Chrome.',
      unread_messages: 'Unread Messages',

      email_or_username: 'Email or Username',
      password: 'Password',
      verify_password: 'Verify Password',
      login: 'Login',
      sign_up: 'Sign Up',
      username: 'Username',
      email: 'Email',
      optional: 'Optional',

      url: 'URL',
      body: 'Body',
      copy_suggested_title: 'copy suggested title: {{title}}',
      related_posts: 'These posts might be related',
      community: 'Community',

      expand_here: 'Expand here',
      remove_post: 'Remove Post',

      no_posts: 'No Posts.',
      subscribe_to_communities: 'Subscribe to some <1>communities</1>.',

      chat: 'Chat',

      no_results: 'No results.',
      
      setup: 'Setup',
      lemmy_instance_setup: 'Lemmy Instance Setup',
      setup_admin: 'Set Up Site Administrator',

      your_site: 'your site',
      modified: 'modified',


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

i18n
.init({
  fallbackLng: 'en',
	resources,
	interpolation: {
		format: format
	}
});

export { i18n, resources };
