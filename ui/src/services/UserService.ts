import * as Cookies from 'js-cookie';
import { User, LoginResponse } from '../interfaces';
import { setTheme } from '../utils';
import * as jwt_decode from 'jwt-decode';
import { Subject } from 'rxjs';

export class UserService {
  private static _instance: UserService;
  public user: User;
  public sub: Subject<{ user: User; unreadCount: number }> = new Subject<{
    user: User;
    unreadCount: number;
  }>();

  private constructor() {
    let jwt = Cookies.get('jwt');
    if (jwt) {
      this.setUser(jwt);
    } else {
      setTheme();
      console.log('No JWT cookie found.');
    }
  }

  public login(res: LoginResponse) {
    this.setUser(res.jwt);
    Cookies.set('jwt', res.jwt, { expires: 365 });
    console.log('jwt cookie set');
  }

  public logout() {
    this.user = undefined;
    Cookies.remove('jwt');
    setTheme();
    this.sub.next({ user: undefined, unreadCount: 0 });
    console.log('Logged out.');
  }

  public get auth(): string {
    return Cookies.get('jwt');
  }

  private setUser(jwt: string) {
    this.user = jwt_decode(jwt);
    if (this.user.theme != 'darkly') {
      setTheme(this.user.theme);
    }
    this.sub.next({ user: this.user, unreadCount: 0 });
    console.log(this.user);
  }

  public static get Instance() {
    return this._instance || (this._instance = new this());
  }
}
