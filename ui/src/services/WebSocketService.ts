import { wsUri } from '../env';
import { LoginForm, RegisterForm, UserOperation, CommunityForm } from '../interfaces';
import { webSocket } from 'rxjs/webSocket';
import { Subject } from 'rxjs';
import { UserService } from './';

export class WebSocketService {
  private static _instance: WebSocketService;
  public subject: Subject<{}>;

  private constructor() {
    this.subject = webSocket(wsUri);
    console.log(`Connected to ${wsUri}`);
  }

  public static get Instance(){
    return this._instance || (this._instance = new this());
  }
   
  public login(loginForm: LoginForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.Login, loginForm));
  }

  public register(registerForm: RegisterForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.Register, registerForm));
  }

  public createCommunity(communityForm: CommunityForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.CreateCommunity, communityForm, UserService.Instance.auth));
  }

  private wsSendWrapper(op: UserOperation, data: any, auth?: string) {
    let send = { op: UserOperation[op], data: data, auth: auth };
    console.log(send);
    return send;
  }
}
