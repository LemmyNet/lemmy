import { wsUri } from '../env';
import {
  wsSendSearch,
  wsSendGetPost,
  wsSendBanUser,
  wsSendGetSite,
  wsSendUserJoin,
  wsSendRegister,
  wsSendLogin,
  wsSendGetPosts,
  wsSendLikePost,
  wsSendEditPost,
  wsSendLockPost,
  wsSendSavePost,
  wsSendAddAdmin,
  wsSendEditSite,
  wsSendGetModlog,
  wsSendGetCaptcha,
  wsSendCreatePost,
  wsSendDeletePost,
  wsSendRemovePost,
  wsSendStickyPost,
  wsSendGetReplies,
  wsSendCreateSite,
  wsSendEditComment,
  wsSendLikeComment,
  wsSendSaveComment,
  wsSendGetComments,
  wsSendGetCommunity,
  wsSendTransferSite,
  wsSendEditCommunity,
  wsSendCreateComment,
  wsSendDeleteComment,
  wsSendRemoveComment,
  wsSendGetSiteConfig,
  wsSendMarkAllAsRead,
  wsSendDeleteAccount,
  wsSendPasswordReset,
  wsSendPasswordChange,
  wsSendListCategories,
  wsSendGetUserDetails,
  wsSendGetUserMentions,
  wsSendSaveSiteConfig,
  wsSendDeleteCommunity,
  wsSendCreateCommunity,
  wsSendRemoveCommunity,
  wsSendFollowCommunity,
  wsSendListCommunities,
  wsSendBanFromCommunity,
  wsSendSaveUserSettings,
  wsSendMarkCommentAsRead,
  wsSendGetFollowedCommunities,
  wsSendAddModToCommunity,
  wsSendTransferCommunity,
  wsSendMarkUserMentionAsRead,
  wsSendDeletePrivateMessage,
  wsSendEditPrivateMessage,
  wsSendGetPrivateMessages,
  wsSendCreatePrivateMessage,
  wsSendMarkPrivateMessageAsRead,
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
    this.ws.send(wsSendUserJoin(form));
  }

  public login(form: LoginForm) {
    this.ws.send(wsSendLogin(form));
  }

  public register(form: RegisterForm) {
    this.ws.send(wsSendRegister(form));
  }

  public getCaptcha() {
    this.ws.send(wsSendGetCaptcha());
  }

  public createCommunity(form: CommunityForm) {
    this.setAuth(form); // TODO all these setauths at some point would be good to make required
    this.ws.send(wsSendCreateCommunity(form));
  }

  public editCommunity(form: CommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendEditCommunity(form));
  }

  public deleteCommunity(form: DeleteCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendDeleteCommunity(form));
  }

  public removeCommunity(form: RemoveCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendRemoveCommunity(form));
  }

  public followCommunity(form: FollowCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendFollowCommunity(form));
  }

  public listCommunities(form: ListCommunitiesForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendListCommunities(form));
  }

  public getFollowedCommunities() {
    let form: GetFollowedCommunitiesForm = { auth: UserService.Instance.auth };
    this.ws.send(wsSendGetFollowedCommunities(form));
  }

  public listCategories() {
    this.ws.send(wsSendListCategories());
  }

  public createPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(wsSendCreatePost(form));
  }

  public getPost(form: GetPostForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetPost(form));
  }

  public getCommunity(form: GetCommunityForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetCommunity(form));
  }

  public createComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(wsSendCreateComment(form));
  }

  public editComment(form: CommentForm) {
    this.setAuth(form);
    this.ws.send(wsSendEditComment(form));
  }

  public deleteComment(form: DeleteCommentForm) {
    this.setAuth(form);
    this.ws.send(wsSendDeleteComment(form));
  }

  public removeComment(form: RemoveCommentForm) {
    this.setAuth(form);
    this.ws.send(wsSendRemoveComment(form));
  }

  public markCommentAsRead(form: MarkCommentAsReadForm) {
    this.setAuth(form);
    this.ws.send(wsSendMarkCommentAsRead(form));
  }

  public likeComment(form: CommentLikeForm) {
    this.setAuth(form);
    this.ws.send(wsSendLikeComment(form));
  }

  public saveComment(form: SaveCommentForm) {
    this.setAuth(form);
    this.ws.send(wsSendSaveComment(form));
  }

  public getPosts(form: GetPostsForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetPosts(form));
  }

  public getComments(form: GetCommentsForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetComments(form));
  }

  public likePost(form: CreatePostLikeForm) {
    this.setAuth(form);
    this.ws.send(wsSendLikePost(form));
  }

  public editPost(form: PostForm) {
    this.setAuth(form);
    this.ws.send(wsSendEditPost(form));
  }

  public deletePost(form: DeletePostForm) {
    this.setAuth(form);
    this.ws.send(wsSendDeletePost(form));
  }

  public removePost(form: RemovePostForm) {
    this.setAuth(form);
    this.ws.send(wsSendRemovePost(form));
  }

  public lockPost(form: LockPostForm) {
    this.setAuth(form);
    this.ws.send(wsSendLockPost(form));
  }

  public stickyPost(form: StickyPostForm) {
    this.setAuth(form);
    this.ws.send(wsSendStickyPost(form));
  }

  public savePost(form: SavePostForm) {
    this.setAuth(form);
    this.ws.send(wsSendSavePost(form));
  }

  public banFromCommunity(form: BanFromCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendBanFromCommunity(form));
  }

  public addModToCommunity(form: AddModToCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendAddModToCommunity(form));
  }

  public transferCommunity(form: TransferCommunityForm) {
    this.setAuth(form);
    this.ws.send(wsSendTransferCommunity(form));
  }

  public transferSite(form: TransferSiteForm) {
    this.setAuth(form);
    this.ws.send(wsSendTransferSite(form));
  }

  public banUser(form: BanUserForm) {
    this.setAuth(form);
    this.ws.send(wsSendBanUser(form));
  }

  public addAdmin(form: AddAdminForm) {
    this.setAuth(form);
    this.ws.send(wsSendAddAdmin(form));
  }

  public getUserDetails(form: GetUserDetailsForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetUserDetails(form));
  }

  public getReplies(form: GetRepliesForm) {
    this.setAuth(form);
    this.ws.send(wsSendGetReplies(form));
  }

  public getUserMentions(form: GetUserMentionsForm) {
    this.setAuth(form);
    this.ws.send(wsSendGetUserMentions(form));
  }

  public markUserMentionAsRead(form: MarkUserMentionAsReadForm) {
    this.setAuth(form);
    this.ws.send(wsSendMarkUserMentionAsRead(form));
  }

  public getModlog(form: GetModlogForm) {
    this.ws.send(wsSendGetModlog(form));
  }

  public createSite(form: SiteForm) {
    this.setAuth(form);
    this.ws.send(wsSendCreateSite(form));
  }

  public editSite(form: SiteForm) {
    this.setAuth(form);
    this.ws.send(wsSendEditSite(form));
  }

  public getSite(form: GetSiteForm = {}) {
    this.setAuth(form, false);
    this.ws.send(wsSendGetSite(form));
  }

  public getSiteConfig() {
    let form: GetSiteConfig = {};
    this.setAuth(form);
    this.ws.send(wsSendGetSiteConfig(form));
  }

  public search(form: SearchForm) {
    this.setAuth(form, false);
    this.ws.send(wsSendSearch(form));
  }

  public markAllAsRead() {
    let form = {};
    this.setAuth(form);
    this.ws.send(wsSendMarkAllAsRead(form));
  }

  public saveUserSettings(form: UserSettingsForm) {
    this.setAuth(form);
    this.ws.send(wsSendSaveUserSettings(form));
  }

  public deleteAccount(form: DeleteAccountForm) {
    this.setAuth(form);
    this.ws.send(wsSendDeleteAccount(form));
  }

  public passwordReset(form: PasswordResetForm) {
    this.ws.send(wsSendPasswordReset(form));
  }

  public passwordChange(form: PasswordChangeForm) {
    this.ws.send(wsSendPasswordChange(form));
  }

  public createPrivateMessage(form: PrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(wsSendCreatePrivateMessage(form));
  }

  public editPrivateMessage(form: EditPrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(wsSendEditPrivateMessage(form));
  }

  public deletePrivateMessage(form: DeletePrivateMessageForm) {
    this.setAuth(form);
    this.ws.send(wsSendDeletePrivateMessage(form));
  }

  public markPrivateMessageAsRead(form: MarkPrivateMessageAsReadForm) {
    this.setAuth(form);
    this.ws.send(wsSendMarkPrivateMessageAsRead(form));
  }

  public getPrivateMessages(form: GetPrivateMessagesForm) {
    this.setAuth(form);
    this.ws.send(wsSendGetPrivateMessages(form));
  }

  public saveSiteConfig(form: SiteConfigForm) {
    this.setAuth(form);
    this.ws.send(wsSendSaveSiteConfig(form));
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
