import { wsUri } from './env';
import { LoginForm, RegisterForm, UserOperation } from './interfaces';

export class WebSocketService {
  private static _instance: WebSocketService;
  private _ws;
  private conn: WebSocket;

  private constructor() {
    console.log("Creating WSS");
    this.connect();
    console.log(wsUri);
  }

  public static get Instance(){
    return this._instance || (this._instance = new this());
  }

  private connect() {
    this.disconnect();
    this.conn = new WebSocket(wsUri);
    console.log('Connecting...');
    this.conn.onopen = (() => {
      console.log('Connected.');
    });
    this.conn.onmessage = (e => {
      console.log('Received: ' + e.data);
    });
    this.conn.onclose = (() => {
      console.log('Disconnected.');
      this.conn = null;
    });
  }
  private disconnect() {
    if (this.conn != null) {
      console.log('Disconnecting...');
      this.conn.close();
      this.conn = null;
    }
  }
  
  public login(loginForm: LoginForm) {
    this.conn.send(this.wsSendWrapper(UserOperation.Login, loginForm));
  }

  public register(registerForm: RegisterForm) {
    this.conn.send(this.wsSendWrapper(UserOperation.Register, registerForm));
  }

  private wsSendWrapper(op: UserOperation, data: any): string {
    let send = { op: UserOperation[op], data: data };
    console.log(send);
    return JSON.stringify(send);
  }


}
