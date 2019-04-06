import { UserOperation, Comment } from './interfaces';
import * as markdown_it from 'markdown-it';

export let repoUrl = 'https://github.com/dessalines/lemmy';
export let wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/service/ws/';

export function msgOp(msg: any): UserOperation {
  let opStr: string = msg.op;
  return UserOperation[opStr];
}

var md = new markdown_it({
  html: true,
  linkify: true,
  typographer: true
});

export function hotRank(comment: Comment): number {
  // Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity

  let date: Date = new Date(comment.published + 'Z'); // Add Z to convert from UTC date
  let now: Date = new Date();
  let hoursElapsed: number = (now.getTime() - date.getTime()) / 36e5;

  let rank = (10000 * Math.sign(comment.score) * Math.log10(1 + Math.abs(comment.score))) / Math.pow(hoursElapsed + 2, 1.8);

  // console.log(`Comment: ${comment.content}\nRank: ${rank}\nScore: ${comment.score}\nHours: ${hoursElapsed}`);

  return rank;
}

export function mdToHtml(text: string) {
  return {__html: md.render(text)};
}
