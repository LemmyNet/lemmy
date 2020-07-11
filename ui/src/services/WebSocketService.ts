import { wsUri } from '../env';
import {
  LoginForm,
  RegisterForm,
  UserOperation,
  CommunityForm,
  PostForm,
  SavePostForm,
  CommentForm,
  SaveCommentForm,
  CommentLikeForm,
  GetPostForm,
  GetPostsForm,
  CreatePostLikeForm,
  GetCommunityForm,
  FollowCommunityForm,
  GetFollowedCommunitiesForm,
  GetUserDetailsForm,
  ListCommunitiesForm,
  GetModlogForm,
  BanFromCommunityForm,
  AddModToCommunityForm,
  TransferCommunityForm,
  AddAdminForm,
  TransferSiteForm,
  BanUserForm,
  SiteForm,
  UserView,
  GetRepliesForm,
  GetUserMentionsForm,
  EditUserMentionForm,
  SearchForm,
  UserSettingsForm,
  DeleteAccountForm,
  PasswordResetForm,
  PasswordChangeForm,
  PrivateMessageForm,
  EditPrivateMessageForm,
  GetPrivateMessagesForm,
  GetCommentsForm,
  UserJoinForm,
  GetSiteConfig,
  SiteConfigForm,
  MessageType,
  WebSocketJsonResponse,
} from '../interfaces';
import { UserService } from './';
import { i18n } from '../i18next';
import { toast } from '../utils';
import { Observable } from 'rxjs';
import { share } from 'rxjs/operators';
import ReconnectingWebSocket from 'reconnecting-websocket';

export class WebSocketService {
  private static _instance: WebSocketService;
  public ws: ReconnectingWebSocket;
  public subject: Observable<any>;

  public admins: Array<UserView>;
  public banned: Array<UserView>;

  private constructor() {
    this.ws = new ReconnectingWebSocket(wsUri);
    let firstConnect = true;

    this.subject = Observable.create((obs: any) => {
      this.ws.onmessage = e => {
        obs.next(JSON.parse(e.data));
      };
      this.ws.onopen = () => {
        console.log(`Connected to ${wsUri}`);

        if (UserService.Instance.user) {
          this.userJoin();
        }

        if (!firstConnect) {
          let res: WebSocketJsonResponse = {
            reconnect: true,
          };
          obs.next(res);
        }

        firstConnect = false;
      };
    }).pipe(share());
  }

  public static get Instance() {
    return this._instance || (this._instance = new this());
  }

  public userJoin() {
    let form: UserJoinForm = { auth: UserService.Instance.auth };
    this.ws.send(this.wsSendWrapper(UserOperation.UserJoin, form));
  }

  public login(loginForm: LoginForm) {
    this.ws.send(this.wsSendWrapper(UserOperation.Login, loginForm));
  }

  public register(registerForm: RegisterForm) {
    this.ws.send(this.wsSendWrapper(UserOperation.Register, registerForm));
  }

  public createCommunity(communityForm: CommunityForm) {
    this.setAuth(communityForm);
    this.ws.send(
      this.wsSendWrapper(UserOperation.CreateCommunity, communityForm)
    );
  }

  public editCommunity(communityForm: CommunityForm) {
    this.setAuth(communityForm);
    this.ws.send(
      this.wsSendWrapper(UserOperation.EditCommunity, communityForm)
    );
  }

  public followCommunity(followCommunityForm: FollowCommunityForm) {
    this.setAuth(followCommunityForm);
    this.ws.send(
      this.wsSendWrapper(UserOperation.FollowCommunity, followCommunityForm)
    );
  }

  public listCommunities(form: ListCommunitiesForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.ListCommunities, form));
  }

  public getFollowedCommunities() {
    let form: GetFollowedCommunitiesForm = { auth: UserService.Instance.auth };
    this.ws.send(
      this.wsSendWrapper(UserOperation.GetFollowedCommunities, form)
    );
  }

  public listCategories() {
    this.ws.send(this.wsSendWrapper(UserOperation.ListCategories, {}));
  }

  public createPost(postForm: PostForm) {
    this.setAuth(postForm);
    this.ws.send(this.wsSendWrapper(UserOperation.CreatePost, postForm));
  }

  public getPost(form: GetPostForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetPost, form));
  }

  public getCommunity(form: GetCommunityForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetCommunity, form));
  }

  public createComment(commentForm: CommentForm) {
    this.setAuth(commentForm);
    this.ws.send(this.wsSendWrapper(UserOperation.CreateComment, commentForm));
  }

  public editComment(commentForm: CommentForm) {
    this.setAuth(commentForm);
    this.ws.send(this.wsSendWrapper(UserOperation.EditComment, commentForm));
  }

  public likeComment(form: CommentLikeForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreateCommentLike, form));
  }

  public saveComment(form: SaveCommentForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.SaveComment, form));
  }

  public getPosts(form: GetPostsForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetPosts, form));
  }

  public getComments(form: GetCommentsForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetComments, form));
  }

  public likePost(form: CreatePostLikeForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreatePostLike, form));
  }

  public editPost(postForm: PostForm) {
    this.setAuth(postForm);
    this.ws.send(this.wsSendWrapper(UserOperation.EditPost, postForm));
  }

  public savePost(form: SavePostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.SavePost, form));
  }

  public banFromCommunity(form: BanFromCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.BanFromCommunity, form));
  }

  public addModToCommunity(form: AddModToCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.AddModToCommunity, form));
  }

  public transferCommunity(form: TransferCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.TransferCommunity, form));
  }

  public transferSite(form: TransferSiteForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.TransferSite, form));
  }

  public banUser(form: BanUserForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.BanUser, form));
  }

  public addAdmin(form: AddAdminForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.AddAdmin, form));
  }

  public getUserDetails(form: GetUserDetailsForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetUserDetails, form));
  }

  public getReplies(form: GetRepliesForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.GetReplies, form));
  }

  public getUserMentions(form: GetUserMentionsForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.GetUserMentions, form));
  }

  public editUserMention(form: EditUserMentionForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.EditUserMention, form));
  }

  public getModlog(form: GetModlogForm) {
    this.ws.send(this.wsSendWrapper(UserOperation.GetModlog, form));
  }

  public createSite(siteForm: SiteForm) {
    this.setAuth(siteForm);
    this.ws.send(this.wsSendWrapper(UserOperation.CreateSite, siteForm));
  }

  public editSite(siteForm: SiteForm) {
    this.setAuth(siteForm);
    this.ws.send(this.wsSendWrapper(UserOperation.EditSite, siteForm));
  }

  public getSite() {
    this.ws.send(this.wsSendWrapper(UserOperation.GetSite, {}));
  }

  public getSiteConfig() {
    let siteConfig: GetSiteConfig = {};
    this.setAuth(siteConfig);
    this.ws.send(this.wsSendWrapper(UserOperation.GetSiteConfig, siteConfig));
  }

  public search(form: SearchForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.Search, form));
  }

  public markAllAsRead() {
    let form = {};
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.MarkAllAsRead, form));
  }

  public saveUserSettings(userSettingsForm: UserSettingsForm) {
    this.setAuth(userSettingsForm);
    this.ws.send(
      this.wsSendWrapper(UserOperation.SaveUserSettings, userSettingsForm)
    );
  }

  public deleteAccount(form: DeleteAccountForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.DeleteAccount, form));
  }

  public passwordReset(form: PasswordResetForm) {
    this.ws.send(this.wsSendWrapper(UserOperation.PasswordReset, form));
  }

  public passwordChange(form: PasswordChangeForm) {
    this.ws.send(this.wsSendWrapper(UserOperation.PasswordChange, form));
  }

  public createPrivateMessage(form: PrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreatePrivateMessage, form));
  }

  public editPrivateMessage(form: EditPrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.EditPrivateMessage, form));
  }

  public getPrivateMessages(form: GetPrivateMessagesForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.GetPrivateMessages, form));
  }

  public saveSiteConfig(form: SiteConfigForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.SaveSiteConfig, form));
  }

  private wsSendWrapper(op: UserOperation, data: MessageType) {
    let send = { op: UserOperation[op], data: data };
    console.log(send);
    return JSON.stringify(send);
  }

  private setAuth(obj: any, throwErr: boolean = true) {
    obj.auth = UserService.Instance.auth;
    if (obj.auth == null && throwErr) {
      toast(i18n.t('not_logged_in'), 'danger');
      throw 'Not logged in';
    }
  }
}

window.onbeforeunload = () => {
  WebSocketService.Instance.ws.close();
};
