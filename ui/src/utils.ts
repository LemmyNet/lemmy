import 'moment/locale/es';
import 'moment/locale/eo';
import 'moment/locale/de';
import 'moment/locale/zh-cn';
import 'moment/locale/fr';
import 'moment/locale/sv';
import 'moment/locale/ru';
import 'moment/locale/nl';

import { UserOperation, Comment, User, SortType, ListingType, SearchType } from './interfaces';
import * as markdown_it from 'markdown-it';
import * as markdownitEmoji from 'markdown-it-emoji/light';
import * as markdown_it_container from 'markdown-it-container';
import * as twemoji from 'twemoji';
import * as emojiShortName from 'emoji-short-name';

export const repoUrl = 'https://github.com/dessalines/lemmy';
export const imageUploadUrl = 'https://postimages.org/';
export const markdownHelpUrl = 'https://commonmark.org/help/';

export const fetchLimit: number = 20;
export const mentionDropdownFetchLimit = 6;

export function randomStr() {return Math.random().toString(36).replace(/[^a-z]+/g, '').substr(2, 10)}

export function msgOp(msg: any): UserOperation {
  let opStr: string = msg.op;
  return UserOperation[opStr];
}

export const md = new markdown_it({
  html: false,
  linkify: true,
  typographer: true
}).use(markdown_it_container, 'spoiler', {
  validate: function(params: any) {
    return params.trim().match(/^spoiler\s+(.*)$/);
  },

  render: function (tokens: any, idx: any) {
    var m = tokens[idx].info.trim().match(/^spoiler\s+(.*)$/);

    if (tokens[idx].nesting === 1) {
      // opening tag
      return '<details><summary>' + md.utils.escapeHtml(m[1]) + '</summary>\n';

    } else {
      // closing tag
      return '</details>\n';
    }
  }
}).use(markdownitEmoji, {
  defs: objectFlip(emojiShortName)
});

md.renderer.rules.emoji = function(token, idx) {
  return twemoji.parse(token[idx].content);
};

export function hotRank(comment: Comment): number {
  // Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity

  let date: Date = new Date(comment.published + 'Z'); // Add Z to convert from UTC date
  let now: Date = new Date();
  let hoursElapsed: number = (now.getTime() - date.getTime()) / 36e5;

  let rank = (10000 *  Math.log10(Math.max(1, 3 + comment.score))) / Math.pow(hoursElapsed + 2, 1.8);

  // console.log(`Comment: ${comment.content}\nRank: ${rank}\nScore: ${comment.score}\nHours: ${hoursElapsed}`);

  return rank;
}

export function mdToHtml(text: string) {
  return {__html: md.render(text)};
}

export function getUnixTime(text: string): number { 
  return text ? new Date(text).getTime()/1000 : undefined;
}

export function addTypeInfo<T>(arr: Array<T>, name: string): Array<{type_: string, data: T}> {  
  return arr.map(e => {return {type_: name, data: e}});
}

export function canMod(user: User, modIds: Array<number>, creator_id: number): boolean {
  // You can do moderator actions only on the mods added after you.
  if (user) {
    let yourIndex = modIds.findIndex(id => id == user.id);
    if (yourIndex == -1) {
      return false;
    } else { 
      modIds = modIds.slice(0, yourIndex+1); // +1 cause you cant mod yourself
      return !modIds.includes(creator_id);
    }
  } else {
    return false;
  }
}

export function isMod(modIds: Array<number>, creator_id: number): boolean {
  return modIds.includes(creator_id);
}


var imageRegex = new RegExp(`(http)?s?:?(\/\/[^"']*\.(?:png|jpg|jpeg|gif|png|svg))`);
var videoRegex = new RegExp(`(http)?s?:?(\/\/[^"']*\.(?:mp4))`);

export function isImage(url: string) {
  return imageRegex.test(url);
}

export function isVideo(url: string) {
  return videoRegex.test(url);
}

export function validURL(str: string) {
  var pattern = new RegExp('^(https?:\\/\\/)?'+ // protocol
    '((([a-z\\d]([a-z\\d-]*[a-z\\d])*)\\.)+[a-z]{2,}|'+ // domain name
    '((\\d{1,3}\\.){3}\\d{1,3}))'+ // OR ip (v4) address
    '(\\:\\d+)?(\\/[-a-z\\d%_.~+]*)*'+ // port and path
    '(\\?[;&a-z\\d%_.~+=-]*)?'+ // query string
    '(\\#[-a-z\\d_]*)?$','i'); // fragment locator
  return !!pattern.test(str);
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

export function debounce(func: any, wait: number = 500, immediate: boolean = false) {
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
  }
}

export function getLanguage(): string {
  return (navigator.language || navigator.userLanguage);
}

export function objectFlip(obj: any) {
  const ret = {};
  Object.keys(obj).forEach((key) => {
    ret[obj[key]] = key;
  });
  return ret;
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
  } else {
    lang = 'en';
  }
  return lang;
}
