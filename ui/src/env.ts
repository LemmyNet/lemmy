const host = `${window.location.hostname}`;
const port = `${
  window.location.port == '4444' ? '8540' : window.location.port
}`;
const endpoint = `${host}:${port}`;

export const wsUri = `${
  window.location.protocol == 'https:' ? 'wss://' : 'ws://'
}${endpoint}/api/v1/ws`;
