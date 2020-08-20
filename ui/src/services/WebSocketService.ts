import { wsUri } from '../env';
import {
  LemmyWebsocket,
  LoginForm,
  RegisterForm,
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
  GetSiteForm,
  SiteConfigForm,
  MarkAllAsReadForm,
  WebSocketJsonResponse,
} from 'lemmy-js-client';
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
  private client = new LemmyWebsocket();

  private constructor() {
    this.ws = new ReconnectingWebSocket(wsUri);
    let firstConnect = true;

    this.subject = Observable.create((obs: any) => {
      this.ws.onmessage = e => {
        obs.next(JSON.parse(e.data));
      };
      this.ws.onopen = () => {
        console.log(`Connected to ${wsUri}`);

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
    this.ws.send(this.client.userJoin(form));
  }

  public login(form: LoginForm) {
    this.ws.send(this.client.login(form));
  }

  public register(form: RegisterForm) {
    this.ws.send(this.client.register(form));
  }

  public getCaptcha() {
    this.ws.send(this.client.getCaptcha());
  }

  public createCommunity(form: CommunityForm) {
    this.setAuth(form); // TODO all these setauths at some point would be good to make required
    this.ws.send(this.client.createCommunity(form));
  }

  public editCommunity(form: CommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.editCommunity(form));
  }

  public deleteCommunity(form: DeleteCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.deleteCommunity(form));
  }

  public removeCommunity(form: RemoveCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.removeCommunity(form));
  }

  public followCommunity(form: FollowCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.followCommunity(form));
  }

  public listCommunities(form: ListCommunitiesForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.listCommunities(form));
  }

  public getFollowedCommunities() {
    let form: GetFollowedCommunitiesForm = { auth: UserService.Instance.auth };
    this.ws.send(this.client.getFollowedCommunities(form));
  }

  public listCategories() {
    this.ws.send(this.client.listCategories());
  }

  public createPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(this.client.createPost(form));
  }

  public getPost(form: GetPostForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.getPost(form));
  }

  public getCommunity(form: GetCommunityForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.getCommunity(form));
  }

  public createComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(this.client.createComment(form));
  }

  public editComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(this.client.editComment(form));
  }

  public deleteComment(form: DeleteCommentForm) {
    this.setAuth(form);
    this.ws.send(this.client.deleteComment(form));
  }

  public removeComment(form: RemoveCommentForm) {
    this.setAuth(form);
    this.ws.send(this.client.removeComment(form));
  }

  public markCommentAsRead(form: MarkCommentAsReadForm) {
    this.setAuth(form);
    this.ws.send(this.client.markCommentAsRead(form));
  }

  public likeComment(form: CommentLikeForm) {
    this.setAuth(form);
    this.ws.send(this.client.likeComment(form));
  }

  public saveComment(form: SaveCommentForm) {
    this.setAuth(form);
    this.ws.send(this.client.saveComment(form));
  }

  public getPosts(form: GetPostsForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.getPosts(form));
  }

  public getComments(form: GetCommentsForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.getComments(form));
  }

  public likePost(form: CreatePostLikeForm) {
    this.setAuth(form);
    this.ws.send(this.client.likePost(form));
  }

  public editPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(this.client.editPost(form));
  }

  public deletePost(form: DeletePostForm) {
    this.setAuth(form);
    this.ws.send(this.client.deletePost(form));
  }

  public removePost(form: RemovePostForm) {
    this.setAuth(form);
    this.ws.send(this.client.removePost(form));
  }

  public lockPost(form: LockPostForm) {
    this.setAuth(form);
    this.ws.send(this.client.lockPost(form));
  }

  public stickyPost(form: StickyPostForm) {
    this.setAuth(form);
    this.ws.send(this.client.stickyPost(form));
  }

  public savePost(form: SavePostForm) {
    this.setAuth(form);
    this.ws.send(this.client.savePost(form));
  }

  public banFromCommunity(form: BanFromCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.banFromCommunity(form));
  }

  public addModToCommunity(form: AddModToCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.addModToCommunity(form));
  }

  public transferCommunity(form: TransferCommunityForm) {
    this.setAuth(form);
    this.ws.send(this.client.transferCommunity(form));
  }

  public transferSite(form: TransferSiteForm) {
    this.setAuth(form);
    this.ws.send(this.client.transferSite(form));
  }

  public banUser(form: BanUserForm) {
    this.setAuth(form);
    this.ws.send(this.client.banUser(form));
  }

  public addAdmin(form: AddAdminForm) {
    this.setAuth(form);
    this.ws.send(this.client.addAdmin(form));
  }

  public getUserDetails(form: GetUserDetailsForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.getUserDetails(form));
  }

  public getReplies(form: GetRepliesForm) {
    this.setAuth(form);
    this.ws.send(this.client.getReplies(form));
  }

  public getUserMentions(form: GetUserMentionsForm) {
    this.setAuth(form);
    this.ws.send(this.client.getUserMentions(form));
  }

  public markUserMentionAsRead(form: MarkUserMentionAsReadForm) {
    this.setAuth(form);
    this.ws.send(this.client.markUserMentionAsRead(form));
  }

  public getModlog(form: GetModlogForm) {
    this.ws.send(this.client.getModlog(form));
  }

  public createSite(form: SiteForm) {
    this.setAuth(form);
    this.ws.send(this.client.createSite(form));
  }

  public editSite(form: SiteForm) {
    this.setAuth(form);
    this.ws.send(this.client.editSite(form));
  }

  public getSite(form: GetSiteForm = {}) {
    this.setAuth(form, false);
    this.ws.send(this.client.getSite(form));
  }

  public getSiteConfig() {
    let form: GetSiteConfig = {};
    this.setAuth(form);
    this.ws.send(this.client.getSiteConfig(form));
  }

  public search(form: SearchForm) {
    this.setAuth(form, false);
    this.ws.send(this.client.search(form));
  }

  public markAllAsRead() {
    let form: MarkAllAsReadForm;
    this.setAuth(form);
    this.ws.send(this.client.markAllAsRead(form));
  }

  public saveUserSettings(form: UserSettingsForm) {
    this.setAuth(form);
    this.ws.send(this.client.saveUserSettings(form));
  }

  public deleteAccount(form: DeleteAccountForm) {
    this.setAuth(form);
    this.ws.send(this.client.deleteAccount(form));
  }

  public passwordReset(form: PasswordResetForm) {
    this.ws.send(this.client.passwordReset(form));
  }

  public passwordChange(form: PasswordChangeForm) {
    this.ws.send(this.client.passwordChange(form));
  }

  public createPrivateMessage(form: PrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.client.createPrivateMessage(form));
  }

  public editPrivateMessage(form: EditPrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.client.editPrivateMessage(form));
  }

  public deletePrivateMessage(form: DeletePrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(this.client.deletePrivateMessage(form));
  }

  public markPrivateMessageAsRead(form: MarkPrivateMessageAsReadForm) {
    this.setAuth(form);
    this.ws.send(this.client.markPrivateMessageAsRead(form));
  }

  public getPrivateMessages(form: GetPrivateMessagesForm) {
    this.setAuth(form);
    this.ws.send(this.client.getPrivateMessages(form));
  }

  public saveSiteConfig(form: SiteConfigForm) {
    this.setAuth(form);
    this.ws.send(this.client.saveSiteConfig(form));
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
