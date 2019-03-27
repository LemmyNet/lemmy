import { wsUri } from '../env';
import { LoginForm, RegisterForm, UserOperation, CommunityForm, PostForm, CommentForm } from '../interfaces';
import { webSocket } from 'rxjs/webSocket';
import { Subject } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserService } from './';

export class WebSocketService {
  private static _instance: WebSocketService;
  public subject: Subject<any>;

  private constructor() {
    this.subject = webSocket(wsUri);

    // Even tho this isn't used, its necessary to not keep reconnecting
    this.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe();

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
    this.setAuth(communityForm);
    this.subject.next(this.wsSendWrapper(UserOperation.CreateCommunity, communityForm));
  }

  public listCommunities() {
    this.subject.next(this.wsSendWrapper(UserOperation.ListCommunities, undefined));
  }

  public createPost(postForm: PostForm) {
    this.setAuth(postForm);
    this.subject.next(this.wsSendWrapper(UserOperation.CreatePost, postForm));
  }

  public getPost(postId: number) {
    this.subject.next(this.wsSendWrapper(UserOperation.GetPost, {id: postId}));
  }

  public getCommunity(communityId: number) {
    this.subject.next(this.wsSendWrapper(UserOperation.GetCommunity, {id: communityId}));
  }

  public createComment(commentForm: CommentForm) {
    this.setAuth(commentForm);
    this.subject.next(this.wsSendWrapper(UserOperation.CreateComment, commentForm));
  }

  private wsSendWrapper(op: UserOperation, data: any) {
    let send = { op: UserOperation[op], data: data };
    console.log(send);
    return send;
  }

  private setAuth(obj: any) {
    obj.auth = UserService.Instance.auth;
    if (obj.auth == null) {
      alert("Not logged in.");
      throw "Not logged in";
    }
  }
}
