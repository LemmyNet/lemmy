export const endpoint = `${window.location.hostname}:8536`;
export let wsUri = `${(window.location.protocol=='https:') ? 'wss://' : 'ws://'}${endpoint}/service/ws`;
