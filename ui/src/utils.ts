import { UserOperation } from './interfaces';

export let repoUrl = 'https://github.com/dessalines/rust-reddit-fediverse';
export let wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/service/ws/';

export function msgOp(msg: any): UserOperation {
  let opStr: string = msg.op;
  return UserOperation[opStr];
}
