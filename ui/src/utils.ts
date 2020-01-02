import 'moment/locale/es';
import 'moment/locale/eo';
import 'moment/locale/de';
import 'moment/locale/zh-cn';
import 'moment/locale/fr';
import 'moment/locale/sv';
import 'moment/locale/ru';
import 'moment/locale/nl';
import 'moment/locale/it';

import {
  UserOperation,
  Comment,
  User,
  SortType,
  ListingType,
  SearchType,
} from './interfaces';
import { UserService } from './services/UserService';
import * as markdown_it from 'markdown-it';
import * as markdownitEmoji from 'markdown-it-emoji/light';
import * as markdown_it_container from 'markdown-it-container';
import * as twemoji from 'twemoji';
import * as emojiShortName from 'emoji-short-name';

export const repoUrl = 'https://github.com/dessalines/lemmy';
export const markdownHelpUrl = 'https://commonmark.org/help/';
export const archiveUrl = 'https://archive.is';

export const postRefetchSeconds: number = 60 * 1000;
export const fetchLimit: number = 20;
export const mentionDropdownFetchLimit = 6;

export function randomStr() {
  return Math.random()
    .toString(36)
    .replace(/[^a-z]+/g, '')
    .substr(2, 10);
}

export function msgOp(msg: any): UserOperation {
  let opStr: string = msg.op;
  return UserOperation[opStr];
}

export const md = new markdown_it({
  html: false,
  linkify: true,
  typographer: true,
})
  .use(markdown_it_container, 'spoiler', {
    validate: function(params: any) {
      return params.trim().match(/^spoiler\s+(.*)$/);
    },

    render: function(tokens: any, idx: any) {
      var m = tokens[idx].info.trim().match(/^spoiler\s+(.*)$/);

      if (tokens[idx].nesting === 1) {
        // opening tag
        return (
          '<details><summary>' + md.utils.escapeHtml(m[1]) + '</summary>\n'
        );
      } else {
        // closing tag
        return '</details>\n';
      }
    },
  })
  .use(markdownitEmoji, {
    defs: objectFlip(emojiShortName),
  });

md.renderer.rules.emoji = function(token, idx) {
  return twemoji.parse(token[idx].content);
};

export function hotRank(comment: Comment): number {
  // Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity

  let date: Date = new Date(comment.published + 'Z'); // Add Z to convert from UTC date
  let now: Date = new Date();
  let hoursElapsed: number = (now.getTime() - date.getTime()) / 36e5;

  let rank =
    (10000 * Math.log10(Math.max(1, 3 + comment.score))) /
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

var imageRegex = new RegExp(
  `(http)?s?:?(\/\/[^"']*\.(?:png|jpg|jpeg|gif|png|svg))`
);
var videoRegex = new RegExp(`(http)?s?:?(\/\/[^"']*\.(?:mp4))`);

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
  let re = /^(([^<>()\[\]\\.,;:\s@"]+(\.[^<>()\[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/;
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

export function routeSearchTypeToEnum(type: string): SearchType {
  return SearchType[capitalizeFirstLetter(type)];
}

export async function getPageTitle(url: string) {
  let res = await fetch(`https://textance.herokuapp.com/title/${url}`);
  let data = await res.text();
  return data;
}

export function debounce(
  func: any,
  wait: number = 1000,
  immediate: boolean = false
) {
  // 'private' variable for instance
  // The returned function will be able to reference this due to closure.
  // Each call to the returned function will share this common timer.
  let timeout: number;

  // Calling debounce returns a new anonymous function
  return function() {
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
    timeout = setTimeout(function() {
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

export const languages = [
  { code: 'en', name: 'English' },
  { code: 'eo', name: 'Esperanto' },
  { code: 'es', name: 'Español' },
  { code: 'de', name: 'Deutsch' },
  { code: 'zh', name: '中文' },
  { code: 'fr', name: 'Français' },
  { code: 'sv', name: 'Svenska' },
  { code: 'ru', name: 'Русский' },
  { code: 'nl', name: 'Nederlands' },
  { code: 'it', name: 'Italiano' },
];

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
  return navigator.language || navigator.userLanguage;
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
  } else {
    lang = 'en';
  }
  return lang;
}

export const themes = [
  'litera',
  'minty',
  'solar',
  'united',
  'cyborg',
  'darkly',
  'journal',
  'sketchy',
  'vaporwave',
  'vaporwave-dark',
];

export function setTheme(theme: string = 'darkly') {
  // unload all the other themes
  for (var i = 0; i < themes.length; i++) {
    let styleSheet = document.getElementById(themes[i]);
    if (styleSheet) {
      styleSheet.setAttribute('disabled', 'disabled');
    }
  }

  // Load the theme dynamically
  if (!document.getElementById(theme)) {
    var head = document.getElementsByTagName('head')[0];
    var link = document.createElement('link');
    link.id = theme;
    link.rel = 'stylesheet';
    link.type = 'text/css';
    link.href = `/static/assets/css/themes/${theme}.min.css`;
    link.media = 'all';
    head.appendChild(link);
  }
  document.getElementById(theme).removeAttribute('disabled');
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
  let out = `${split[0]}pictshare/96x96${split[1]}`;
  return out;
}

export function showAvatars(): boolean {
  return (
    (UserService.Instance.user && UserService.Instance.user.show_avatars) ||
    !UserService.Instance.user
  );
}
