import * as Cookies from 'js-cookie';
import { User } from '../interfaces';
import * as jwt_decode from 'jwt-decode';
import { Subject } from 'rxjs';

export class UserService {
  private static _instance: UserService;
  private user: User;
  public sub: Subject<User> = new Subject<User>();

  private constructor() {
    let jwt = Cookies.get("jwt");
    if (jwt) {
      this.setUser(jwt);
    } else {
      console.log('No JWT cookie found.');
    }

  }

  public login(jwt: string) {
    this.setUser(jwt);
    Cookies.set("jwt", jwt);
    console.log("jwt cookie set");
  }

  public logout() {
    this.user = null;
    Cookies.remove("jwt");
    console.log("Logged out.");
    this.sub.next(null);
  }

  public get loggedIn(): boolean {
    return this.user !== undefined;
  }

  public get auth(): string {
    return Cookies.get("jwt");
  }

  private setUser(jwt: string) {
    this.user = jwt_decode(jwt);
    this.sub.next(this.user);
  }

  public static get Instance(){
    return this._instance || (this._instance = new this());
  }
}
