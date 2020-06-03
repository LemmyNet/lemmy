import 'moment/locale/es';
import 'moment/locale/el';
import 'moment/locale/eu';
import 'moment/locale/eo';
import 'moment/locale/de';
import 'moment/locale/zh-cn';
import 'moment/locale/fr';
import 'moment/locale/sv';
import 'moment/locale/ru';
import 'moment/locale/nl';
import 'moment/locale/it';
import 'moment/locale/fi';
import 'moment/locale/ca';
import 'moment/locale/fa';
import 'moment/locale/pt-br';
import 'moment/locale/ja';
import 'moment/locale/ka';
import 'moment/locale/hi';

import {
  UserOperation,
  Comment,
  CommentNode as CommentNodeI,
  Post,
  PrivateMessage,
  User,
  SortType,
  CommentSortType,
  ListingType,
  DataType,
  SearchType,
  WebSocketResponse,
  WebSocketJsonResponse,
  SearchForm,
  SearchResponse,
  CommentResponse,
  PostResponse,
} from './interfaces';
import { UserService, WebSocketService } from './services';

import Tribute from 'tributejs/src/Tribute.js';
import markdown_it from 'markdown-it';
import markdownitEmoji from 'markdown-it-emoji/light';
import markdown_it_container from 'markdown-it-container';
import twemoji from 'twemoji';
import emojiShortName from 'emoji-short-name';
import Toastify from 'toastify-js';
import tippy from 'tippy.js';
import EmojiButton from '@joeattardi/emoji-button';

export const repoUrl = 'https://github.com/LemmyNet/lemmy';
export const helpGuideUrl = '/docs/about_guide.html';
export const markdownHelpUrl = `${helpGuideUrl}#markdown-guide`;
export const sortingHelpUrl = `${helpGuideUrl}#sorting`;
export const archiveUrl = 'https://archive.is';

export const postRefetchSeconds: number = 60 * 1000;
export const fetchLimit: number = 20;
export const mentionDropdownFetchLimit = 10;

export const languages = [
  { code: 'ca', name: 'Català' },
  { code: 'en', name: 'English' },
  { code: 'el', name: 'Ελληνικά' },
  { code: 'eu', name: 'Euskara' },
  { code: 'eo', name: 'Esperanto' },
  { code: 'es', name: 'Español' },
  { code: 'de', name: 'Deutsch' },
  { code: 'ka', name: 'ქართული ენა' },
  { code: 'hi', name: 'मानक हिन्दी' },
  { code: 'fa', name: 'فارسی' },
  { code: 'ja', name: '日本語' },
  { code: 'pt_BR', name: 'Português Brasileiro' },
  { code: 'zh', name: '中文' },
  { code: 'fi', name: 'Suomi' },
  { code: 'fr', name: 'Français' },
  { code: 'sv', name: 'Svenska' },
  { code: 'ru', name: 'Русский' },
  { code: 'nl', name: 'Nederlands' },
  { code: 'it', name: 'Italiano' },
];

export const themes = [
  'litera',
  'materia',
  'minty',
  'solar',
  'united',
  'cyborg',
  'darkly',
  'journal',
  'sketchy',
  'vaporwave',
  'vaporwave-dark',
  'i386',
];

export const emojiPicker = new EmojiButton({
  // Use the emojiShortName from native
  style: 'twemoji',
  theme: 'dark',
  position: 'auto-start',
  // TODO i18n
});

export function randomStr() {
  return Math.random()
    .toString(36)
    .replace(/[^a-z]+/g, '')
    .substr(2, 10);
}

export function wsJsonToRes(msg: WebSocketJsonResponse): WebSocketResponse {
  let opStr: string = msg.op;
  return {
    op: UserOperation[opStr],
    data: msg.data,
  };
}

export const md = new markdown_it({
  html: false,
  linkify: true,
  typographer: true,
})
  .use(markdown_it_container, 'spoiler', {
    validate: function (params: any) {
      return params.trim().match(/^spoiler\s+(.*)$/);
    },

    render: function (tokens: any, idx: any) {
      var m = tokens[idx].info.trim().match(/^spoiler\s+(.*)$/);

      if (tokens[idx].nesting === 1) {
        // opening tag
        return `<details><summary> ${md.utils.escapeHtml(m[1])} </summary>\n`;
      } else {
        // closing tag
        return '</details>\n';
      }
    },
  })
  .use(markdownitEmoji, {
    defs: objectFlip(emojiShortName),
  });

md.renderer.rules.emoji = function (token, idx) {
  return twemoji.parse(token[idx].content);
};

export function hotRankComment(comment: Comment): number {
  return hotRank(comment.score, comment.published);
}

export function hotRankPost(post: Post): number {
  return hotRank(post.score, post.newest_activity_time);
}

export function hotRank(score: number, timeStr: string): number {
  // Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
  let date: Date = new Date(timeStr + 'Z'); // Add Z to convert from UTC date
  let now: Date = new Date();
  let hoursElapsed: number = (now.getTime() - date.getTime()) / 36e5;

  let rank =
    (10000 * Math.log10(Math.max(1, 3 + score))) /
    Math.pow(hoursElapsed + 2, 1.8);

  // console.log(`Comment: ${comment.content}\nRank: ${rank}\nScore: ${comment.score}\nHours: ${hoursElapsed}`);

  return rank;
}

export function mdToHtml(text: string) {
  return { __html: md.render(text) };
}

export function getUnixTime(text: string): number {
  return text ? new Date(text).getTime() / 1000 : undefined;
}

export function addTypeInfo<T>(
  arr: Array<T>,
  name: string
): Array<{ type_: string; data: T }> {
  return arr.map(e => {
    return { type_: name, data: e };
  });
}

export function canMod(
  user: User,
  modIds: Array<number>,
  creator_id: number,
  onSelf: boolean = false
): boolean {
  // You can do moderator actions only on the mods added after you.
  if (user) {
    let yourIndex = modIds.findIndex(id => id == user.id);
    if (yourIndex == -1) {
      return false;
    } else {
      // onSelf +1 on mod actions not for yourself, IE ban, remove, etc
      modIds = modIds.slice(0, yourIndex + (onSelf ? 0 : 1));
      return !modIds.includes(creator_id);
    }
  } else {
    return false;
  }
}

export function isMod(modIds: Array<number>, creator_id: number): boolean {
  return modIds.includes(creator_id);
}

const imageRegex = new RegExp(
  /(http)?s?:?(\/\/[^"']*\.(?:jpg|jpeg|gif|png|svg|webp))/
);
const videoRegex = new RegExp(`(http)?s?:?(\/\/[^"']*\.(?:mp4))`);

export function isImage(url: string) {
  return imageRegex.test(url);
}

export function isVideo(url: string) {
  return videoRegex.test(url);
}

export function validURL(str: string) {
  try {
    return !!new URL(str);
  } catch {
    return false;
  }
}

export function validEmail(email: string) {
  let re = /^(([^\s"(),.:;<>@[\\\]]+(\.[^\s"(),.:;<>@[\\\]]+)*)|(".+"))@((\[(?:\d{1,3}\.){3}\d{1,3}])|(([\dA-Za-z\-]+\.)+[A-Za-z]{2,}))$/;
  return re.test(String(email).toLowerCase());
}

export function capitalizeFirstLetter(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function routeSortTypeToEnum(sort: string): SortType {
  if (sort == 'new') {
    return SortType.New;
  } else if (sort == 'hot') {
    return SortType.Hot;
  } else if (sort == 'topday') {
    return SortType.TopDay;
  } else if (sort == 'topweek') {
    return SortType.TopWeek;
  } else if (sort == 'topmonth') {
    return SortType.TopMonth;
  } else if (sort == 'topyear') {
    return SortType.TopYear;
  } else if (sort == 'topall') {
    return SortType.TopAll;
  }
}

export function routeListingTypeToEnum(type: string): ListingType {
  return ListingType[capitalizeFirstLetter(type)];
}

export function routeDataTypeToEnum(type: string): DataType {
  return DataType[capitalizeFirstLetter(type)];
}

export function routeSearchTypeToEnum(type: string): SearchType {
  return SearchType[capitalizeFirstLetter(type)];
}

export async function getPageTitle(url: string) {
  let res = await fetch(`/iframely/oembed?url=${url}`).then(res => res.json());
  let title = await res.title;
  return title;
}

export function debounce(
  func: any,
  wait: number = 1000,
  immediate: boolean = false
) {
  // 'private' variable for instance
  // The returned function will be able to reference this due to closure.
  // Each call to the returned function will share this common timer.
  let timeout: any;

  // Calling debounce returns a new anonymous function
  return function () {
    // reference the context and args for the setTimeout function
    var context = this,
      args = arguments;

    // Should the function be called now? If immediate is true
    //   and not already in a timeout then the answer is: Yes
    var callNow = immediate && !timeout;

    // This is the basic debounce behaviour where you can call this
    //   function several times, but it will only execute once
    //   [before or after imposing a delay].
    //   Each time the returned function is called, the timer starts over.
    clearTimeout(timeout);

    // Set the new timeout
    timeout = setTimeout(function () {
      // Inside the timeout function, clear the timeout variable
      // which will let the next execution run when in 'immediate' mode
      timeout = null;

      // Check if the function already ran with the immediate flag
      if (!immediate) {
        // Call the original function with apply
        // apply lets you define the 'this' object as well as the arguments
        //    (both captured before setTimeout)
        func.apply(context, args);
      }
    }, wait);

    // Immediate mode and no wait timer? Execute the function..
    if (callNow) func.apply(context, args);
  };
}

export function getLanguage(): string {
  let user = UserService.Instance.user;
  let lang = user && user.lang ? user.lang : 'browser';

  if (lang == 'browser') {
    return getBrowserLanguage();
  } else {
    return lang;
  }
}

export function getBrowserLanguage(): string {
  return navigator.language;
}

export function getMomentLanguage(): string {
  let lang = getLanguage();
  if (lang.startsWith('zh')) {
    lang = 'zh-cn';
  } else if (lang.startsWith('sv')) {
    lang = 'sv';
  } else if (lang.startsWith('fr')) {
    lang = 'fr';
  } else if (lang.startsWith('de')) {
    lang = 'de';
  } else if (lang.startsWith('ru')) {
    lang = 'ru';
  } else if (lang.startsWith('es')) {
    lang = 'es';
  } else if (lang.startsWith('eo')) {
    lang = 'eo';
  } else if (lang.startsWith('nl')) {
    lang = 'nl';
  } else if (lang.startsWith('it')) {
    lang = 'it';
  } else if (lang.startsWith('fi')) {
    lang = 'fi';
  } else if (lang.startsWith('ca')) {
    lang = 'ca';
  } else if (lang.startsWith('fa')) {
    lang = 'fa';
  } else if (lang.startsWith('pt')) {
    lang = 'pt-br';
  } else if (lang.startsWith('ja')) {
    lang = 'ja';
  } else if (lang.startsWith('ka')) {
    lang = 'ka';
  } else if (lang.startsWith('hi')) {
    lang = 'hi';
  } else if (lang.startsWith('el')) {
    lang = 'el';
  } else if (lang.startsWith('eu')) {
    lang = 'eu';
  } else {
    lang = 'en';
  }
  return lang;
}

export function setTheme(theme: string = 'darkly') {
  // unload all the other themes
  for (var i = 0; i < themes.length; i++) {
    let styleSheet = document.getElementById(themes[i]);
    if (styleSheet) {
      styleSheet.setAttribute('disabled', 'disabled');
    }
  }

  // Load the theme dynamically
  let cssLoc = `/static/assets/css/themes/${theme}.min.css`;
  loadCss(theme, cssLoc);
  document.getElementById(theme).removeAttribute('disabled');
}

export function loadCss(id: string, loc: string) {
  if (!document.getElementById(id)) {
    var head = document.getElementsByTagName('head')[0];
    var link = document.createElement('link');
    link.id = id;
    link.rel = 'stylesheet';
    link.type = 'text/css';
    link.href = loc;
    link.media = 'all';
    head.appendChild(link);
  }
}

export function objectFlip(obj: any) {
  const ret = {};
  Object.keys(obj).forEach(key => {
    ret[obj[key]] = key;
  });
  return ret;
}

export function pictshareAvatarThumbnail(src: string): string {
  // sample url: http://localhost:8535/pictshare/gs7xuu.jpg
  let split = src.split('pictshare');
  let out = `${split[0]}pictshare/webp/96${split[1]}`;
  return out;
}

export function showAvatars(): boolean {
  return (
    (UserService.Instance.user && UserService.Instance.user.show_avatars) ||
    !UserService.Instance.user
  );
}

// Converts to image thumbnail
export function pictshareImage(
  hash: string,
  thumbnail: boolean = false
): string {
  let root = `/pictshare`;

  // Necessary for other servers / domains
  if (hash.includes('pictshare')) {
    let split = hash.split('/pictshare/');
    root = `${split[0]}/pictshare`;
    hash = split[1];
  }

  let out = `${root}/webp/${thumbnail ? '192/' : ''}${hash}`;
  return out;
}

export function isCommentType(item: Comment | PrivateMessage): item is Comment {
  return (item as Comment).community_id !== undefined;
}

export function toast(text: string, background: string = 'success') {
  let backgroundColor = `var(--${background})`;
  Toastify({
    text: text,
    backgroundColor: backgroundColor,
    gravity: 'bottom',
    position: 'left',
  }).showToast();
}

export function messageToastify(
  creator: string,
  avatar: string,
  body: string,
  link: string,
  router: any
) {
  let backgroundColor = `var(--light)`;

  let toast = Toastify({
    text: `${body}<br />${creator}`,
    avatar: avatar,
    backgroundColor: backgroundColor,
    className: 'text-body',
    close: true,
    gravity: 'top',
    position: 'right',
    duration: 0,
    onClick: () => {
      if (toast) {
        toast.hideToast();
        router.history.push(link);
      }
    },
  }).showToast();
}

export function setupTribute(): Tribute {
  return new Tribute({
    collection: [
      // Emojis
      {
        trigger: ':',
        menuItemTemplate: (item: any) => {
          let shortName = `:${item.original.key}:`;
          let twemojiIcon = twemoji.parse(item.original.val);
          return `${twemojiIcon} ${shortName}`;
        },
        selectTemplate: (item: any) => {
          return `:${item.original.key}:`;
        },
        values: Object.entries(emojiShortName).map(e => {
          return { key: e[1], val: e[0] };
        }),
        allowSpaces: false,
        autocompleteMode: true,
        menuItemLimit: mentionDropdownFetchLimit,
        menuShowMinLength: 2,
      },
      // Users
      {
        trigger: '@',
        selectTemplate: (item: any) => {
          return `[/u/${item.original.key}](/u/${item.original.key})`;
        },
        values: (text: string, cb: any) => {
          userSearch(text, (users: any) => cb(users));
        },
        allowSpaces: false,
        autocompleteMode: true,
        menuItemLimit: mentionDropdownFetchLimit,
        menuShowMinLength: 2,
      },

      // Communities
      {
        trigger: '#',
        selectTemplate: (item: any) => {
          return `[/c/${item.original.key}](/c/${item.original.key})`;
        },
        values: (text: string, cb: any) => {
          communitySearch(text, (communities: any) => cb(communities));
        },
        allowSpaces: false,
        autocompleteMode: true,
        menuItemLimit: mentionDropdownFetchLimit,
        menuShowMinLength: 2,
      },
    ],
  });
}

let tippyInstance = tippy('[data-tippy-content]');

export function setupTippy() {
  tippyInstance.forEach(e => e.destroy());
  tippyInstance = tippy('[data-tippy-content]', {
    delay: [500, 0],
    // Display on "long press"
    touch: ['hold', 500],
  });
}

function userSearch(text: string, cb: any) {
  if (text) {
    let form: SearchForm = {
      q: text,
      type_: SearchType[SearchType.Users],
      sort: SortType[SortType.TopAll],
      page: 1,
      limit: mentionDropdownFetchLimit,
    };

    WebSocketService.Instance.search(form);

    this.userSub = WebSocketService.Instance.subject.subscribe(
      msg => {
        let res = wsJsonToRes(msg);
        if (res.op == UserOperation.Search) {
          let data = res.data as SearchResponse;
          let users = data.users.map(u => {
            return { key: u.name };
          });
          cb(users);
          this.userSub.unsubscribe();
        }
      },
      err => console.error(err),
      () => console.log('complete')
    );
  } else {
    cb([]);
  }
}

function communitySearch(text: string, cb: any) {
  if (text) {
    let form: SearchForm = {
      q: text,
      type_: SearchType[SearchType.Communities],
      sort: SortType[SortType.TopAll],
      page: 1,
      limit: mentionDropdownFetchLimit,
    };

    WebSocketService.Instance.search(form);

    this.communitySub = WebSocketService.Instance.subject.subscribe(
      msg => {
        let res = wsJsonToRes(msg);
        if (res.op == UserOperation.Search) {
          let data = res.data as SearchResponse;
          let communities = data.communities.map(u => {
            return { key: u.name };
          });
          cb(communities);
          this.communitySub.unsubscribe();
        }
      },
      err => console.error(err),
      () => console.log('complete')
    );
  } else {
    cb([]);
  }
}

export function getListingTypeFromProps(props: any): ListingType {
  return props.match.params.listing_type
    ? routeListingTypeToEnum(props.match.params.listing_type)
    : UserService.Instance.user
    ? UserService.Instance.user.default_listing_type
    : ListingType.All;
}

// TODO might need to add a user setting for this too
export function getDataTypeFromProps(props: any): DataType {
  return props.match.params.data_type
    ? routeDataTypeToEnum(props.match.params.data_type)
    : DataType.Post;
}

export function getSortTypeFromProps(props: any): SortType {
  return props.match.params.sort
    ? routeSortTypeToEnum(props.match.params.sort)
    : UserService.Instance.user
    ? UserService.Instance.user.default_sort_type
    : SortType.Hot;
}

export function getPageFromProps(props: any): number {
  return props.match.params.page ? Number(props.match.params.page) : 1;
}

export function editCommentRes(
  data: CommentResponse,
  comments: Array<Comment>
) {
  let found = comments.find(c => c.id == data.comment.id);
  if (found) {
    found.content = data.comment.content;
    found.updated = data.comment.updated;
    found.removed = data.comment.removed;
    found.deleted = data.comment.deleted;
    found.upvotes = data.comment.upvotes;
    found.downvotes = data.comment.downvotes;
    found.score = data.comment.score;
  }
}

export function saveCommentRes(
  data: CommentResponse,
  comments: Array<Comment>
) {
  let found = comments.find(c => c.id == data.comment.id);
  if (found) {
    found.saved = data.comment.saved;
  }
}

export function createCommentLikeRes(
  data: CommentResponse,
  comments: Array<Comment>
) {
  let found: Comment = comments.find(c => c.id === data.comment.id);
  if (found) {
    found.score = data.comment.score;
    found.upvotes = data.comment.upvotes;
    found.downvotes = data.comment.downvotes;
    if (data.comment.my_vote !== null) {
      found.my_vote = data.comment.my_vote;
    }
  }
}

export function createPostLikeFindRes(data: PostResponse, posts: Array<Post>) {
  let found = posts.find(c => c.id == data.post.id);
  if (found) {
    createPostLikeRes(data, found);
  }
}

export function createPostLikeRes(data: PostResponse, post: Post) {
  if (post) {
    post.score = data.post.score;
    post.upvotes = data.post.upvotes;
    post.downvotes = data.post.downvotes;
    if (data.post.my_vote !== null) {
      post.my_vote = data.post.my_vote;
    }
  }
}

export function editPostFindRes(data: PostResponse, posts: Array<Post>) {
  let found = posts.find(c => c.id == data.post.id);
  if (found) {
    editPostRes(data, found);
  }
}

export function editPostRes(data: PostResponse, post: Post) {
  if (post) {
    post.url = data.post.url;
    post.name = data.post.name;
    post.nsfw = data.post.nsfw;
  }
}

export function commentsToFlatNodes(
  comments: Array<Comment>
): Array<CommentNodeI> {
  let nodes: Array<CommentNodeI> = [];
  for (let comment of comments) {
    nodes.push({ comment: comment });
  }
  return nodes;
}

export function commentSort(tree: Array<CommentNodeI>, sort: CommentSortType) {
  // First, put removed and deleted comments at the bottom, then do your other sorts
  if (sort == CommentSortType.Top) {
    tree.sort(
      (a, b) =>
        +a.comment.removed - +b.comment.removed ||
        +a.comment.deleted - +b.comment.deleted ||
        b.comment.score - a.comment.score
    );
  } else if (sort == CommentSortType.New) {
    tree.sort(
      (a, b) =>
        +a.comment.removed - +b.comment.removed ||
        +a.comment.deleted - +b.comment.deleted ||
        b.comment.published.localeCompare(a.comment.published)
    );
  } else if (sort == CommentSortType.Old) {
    tree.sort(
      (a, b) =>
        +a.comment.removed - +b.comment.removed ||
        +a.comment.deleted - +b.comment.deleted ||
        a.comment.published.localeCompare(b.comment.published)
    );
  } else if (sort == CommentSortType.Hot) {
    tree.sort(
      (a, b) =>
        +a.comment.removed - +b.comment.removed ||
        +a.comment.deleted - +b.comment.deleted ||
        hotRankComment(b.comment) - hotRankComment(a.comment)
    );
  }

  // Go through the children recursively
  for (let node of tree) {
    if (node.children) {
      commentSort(node.children, sort);
    }
  }
}

export function commentSortSortType(tree: Array<CommentNodeI>, sort: SortType) {
  commentSort(tree, convertCommentSortType(sort));
}

function convertCommentSortType(sort: SortType): CommentSortType {
  if (
    sort == SortType.TopAll ||
    sort == SortType.TopDay ||
    sort == SortType.TopWeek ||
    sort == SortType.TopMonth ||
    sort == SortType.TopYear
  ) {
    return CommentSortType.Top;
  } else if (sort == SortType.New) {
    return CommentSortType.New;
  } else if (sort == SortType.Hot) {
    return CommentSortType.Hot;
  } else {
    return CommentSortType.Hot;
  }
}

export function postSort(
  posts: Array<Post>,
  sort: SortType,
  communityType: boolean
) {
  // First, put removed and deleted comments at the bottom, then do your other sorts
  if (
    sort == SortType.TopAll ||
    sort == SortType.TopDay ||
    sort == SortType.TopWeek ||
    sort == SortType.TopMonth ||
    sort == SortType.TopYear
  ) {
    posts.sort(
      (a, b) =>
        +a.removed - +b.removed ||
        +a.deleted - +b.deleted ||
        (communityType && +b.stickied - +a.stickied) ||
        b.score - a.score
    );
  } else if (sort == SortType.New) {
    posts.sort(
      (a, b) =>
        +a.removed - +b.removed ||
        +a.deleted - +b.deleted ||
        (communityType && +b.stickied - +a.stickied) ||
        b.published.localeCompare(a.published)
    );
  } else if (sort == SortType.Hot) {
    posts.sort(
      (a, b) =>
        +a.removed - +b.removed ||
        +a.deleted - +b.deleted ||
        (communityType && +b.stickied - +a.stickied) ||
        hotRankPost(b) - hotRankPost(a)
    );
  }
}

export const colorList: Array<string> = [
  hsl(0),
  hsl(100),
  hsl(150),
  hsl(200),
  hsl(250),
  hsl(300),
];

function hsl(num: number) {
  return `hsla(${num}, 35%, 50%, 1)`;
}

function randomHsl() {
  return `hsla(${Math.random() * 360}, 100%, 50%, 1)`;
}

export function previewLines(text: string, lines: number = 3): string {
  // Use lines * 2 because markdown requires 2 lines
  return text
    .split('\n')
    .slice(0, lines * 2)
    .join('\n');
}
