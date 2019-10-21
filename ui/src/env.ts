let host = `${window.location.hostname}`;
let port = `${window.location.port == '4444' ? '8536' : window.location.port}`;
let endpoint = `${host}:${port}`;
export let wsUri = `${
  window.location.protocol == 'https:' ? 'wss://' : 'ws://'
}${endpoint}/api/v1/ws`;
