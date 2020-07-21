import { wsUri } from '../env';
import {
  LoginForm,
  RegisterForm,
  UserOperation,
  CommunityForm,
  DeleteCommunityForm,
  RemoveCommunityForm,
  PostForm,
  DeletePostForm,
  RemovePostForm,
  LockPostForm,
  StickyPostForm,
  SavePostForm,
  CommentForm,
  DeleteCommentForm,
  RemoveCommentForm,
  MarkCommentAsReadForm,
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
  MarkUserMentionAsReadForm,
  SearchForm,
  UserSettingsForm,
  DeleteAccountForm,
  PasswordResetForm,
  PasswordChangeForm,
  PrivateMessageForm,
  EditPrivateMessageForm,
  DeletePrivateMessageForm,
  MarkPrivateMessageAsReadForm,
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

  public createCommunity(form: CommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreateCommunity, form));
  }

  public editCommunity(form: CommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.EditCommunity, form));
  }

  public deleteCommunity(form: DeleteCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.DeleteCommunity, form));
  }

  public removeCommunity(form: RemoveCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.RemoveCommunity, form));
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

  public createPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreatePost, form));
  }

  public getPost(form: GetPostForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetPost, form));
  }

  public getCommunity(form: GetCommunityForm) {
    this.setAuth(form, false);
    this.ws.send(this.wsSendWrapper(UserOperation.GetCommunity, form));
  }

  public createComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.CreateComment, form));
  }

  public editComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.EditComment, form));
  }

  public deleteComment(form: DeleteCommentForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.DeleteComment, form));
  }

  public removeComment(form: RemoveCommentForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.RemoveComment, form));
  }

  public markCommentAsRead(form: MarkCommentAsReadForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.MarkCommentAsRead, form));
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

  public editPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.EditPost, form));
  }

  public deletePost(form: DeletePostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.DeletePost, form));
  }

  public removePost(form: RemovePostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.RemovePost, form));
  }

  public lockPost(form: LockPostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.LockPost, form));
  }

  public stickyPost(form: StickyPostForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.StickyPost, form));
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

  public markUserMentionAsRead(form: MarkUserMentionAsReadForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.MarkUserMentionAsRead, form));
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

  public deletePrivateMessage(form: DeletePrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.wsSendWrapper(UserOperation.DeletePrivateMessage, form));
  }

  public markPrivateMessageAsRead(form: MarkPrivateMessageAsReadForm) {
    this.setAuth(form);
    this.ws.send(
      this.wsSendWrapper(UserOperation.MarkPrivateMessageAsRead, form)
    );
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
