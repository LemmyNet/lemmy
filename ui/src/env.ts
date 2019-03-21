// export const endpoint = window.location.origin;
export const endpoint = "http://localhost:8080";
export let wsUri = (window.location.protocol=='https:') ? 'wss://' : 'ws://' + endpoint.substr(7) + '/service/ws';
