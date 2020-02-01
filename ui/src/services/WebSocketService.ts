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
  Site,
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
  MessageType,
} from '../interfaces';
import { webSocket } from 'rxjs/webSocket';
import { Subject } from 'rxjs';
import { retryWhen, delay } from 'rxjs/operators';
import { UserService } from './';
import { i18n } from '../i18next';
import { toast } from '../utils';

export class WebSocketService {
  private static _instance: WebSocketService;
  public subject: Subject<any>;

  public site: Site;
  public admins: Array<UserView>;
  public banned: Array<UserView>;

  private constructor() {
    this.subject = webSocket(wsUri);

    // Necessary to not keep reconnecting
    this.subject
      .pipe(
        retryWhen(errors =>
          errors.pipe(
            delay(1000)
            // take(999)
          )
        )
      )
      .subscribe();

    console.log(`Connected to ${wsUri}`);
  }

  public static get Instance() {
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
    this.subject.next(
      this.wsSendWrapper(UserOperation.CreateCommunity, communityForm)
    );
  }

  public editCommunity(communityForm: CommunityForm) {
    this.setAuth(communityForm);
    this.subject.next(
      this.wsSendWrapper(UserOperation.EditCommunity, communityForm)
    );
  }

  public followCommunity(followCommunityForm: FollowCommunityForm) {
    this.setAuth(followCommunityForm);
    this.subject.next(
      this.wsSendWrapper(UserOperation.FollowCommunity, followCommunityForm)
    );
  }

  public listCommunities(form: ListCommunitiesForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.ListCommunities, form));
  }

  public getFollowedCommunities() {
    let form: GetFollowedCommunitiesForm = { auth: UserService.Instance.auth };
    this.subject.next(
      this.wsSendWrapper(UserOperation.GetFollowedCommunities, form)
    );
  }

  public listCategories() {
    this.subject.next(
      this.wsSendWrapper(UserOperation.ListCategories, {})
    );
  }

  public createPost(postForm: PostForm) {
    this.setAuth(postForm);
    this.subject.next(this.wsSendWrapper(UserOperation.CreatePost, postForm));
  }

  public getPost(form: GetPostForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.GetPost, form));
  }

  public getCommunity(form: GetCommunityForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.GetCommunity, form));
  }

  public createComment(commentForm: CommentForm) {
    this.setAuth(commentForm);
    this.subject.next(
      this.wsSendWrapper(UserOperation.CreateComment, commentForm)
    );
  }

  public editComment(commentForm: CommentForm) {
    this.setAuth(commentForm);
    this.subject.next(
      this.wsSendWrapper(UserOperation.EditComment, commentForm)
    );
  }

  public likeComment(form: CommentLikeForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.CreateCommentLike, form)
    );
  }

  public saveComment(form: SaveCommentForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.SaveComment, form));
  }

  public getPosts(form: GetPostsForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.GetPosts, form));
  }

  public likePost(form: CreatePostLikeForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.CreatePostLike, form));
  }

  public editPost(postForm: PostForm) {
    this.setAuth(postForm);
    this.subject.next(this.wsSendWrapper(UserOperation.EditPost, postForm));
  }

  public savePost(form: SavePostForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.SavePost, form));
  }

  public banFromCommunity(form: BanFromCommunityForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.BanFromCommunity, form));
  }

  public addModToCommunity(form: AddModToCommunityForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.AddModToCommunity, form)
    );
  }

  public transferCommunity(form: TransferCommunityForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.TransferCommunity, form)
    );
  }

  public transferSite(form: TransferSiteForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.TransferSite, form));
  }

  public banUser(form: BanUserForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.BanUser, form));
  }

  public addAdmin(form: AddAdminForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.AddAdmin, form));
  }

  public getUserDetails(form: GetUserDetailsForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.GetUserDetails, form));
  }

  public getReplies(form: GetRepliesForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.GetReplies, form));
  }

  public getUserMentions(form: GetUserMentionsForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.GetUserMentions, form));
  }

  public editUserMention(form: EditUserMentionForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.EditUserMention, form));
  }

  public getModlog(form: GetModlogForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.GetModlog, form));
  }

  public createSite(siteForm: SiteForm) {
    this.setAuth(siteForm);
    this.subject.next(this.wsSendWrapper(UserOperation.CreateSite, siteForm));
  }

  public editSite(siteForm: SiteForm) {
    this.setAuth(siteForm);
    this.subject.next(this.wsSendWrapper(UserOperation.EditSite, siteForm));
  }

  public getSite() {
    this.subject.next(this.wsSendWrapper(UserOperation.GetSite, {}));
  }

  public search(form: SearchForm) {
    this.setAuth(form, false);
    this.subject.next(this.wsSendWrapper(UserOperation.Search, form));
  }

  public markAllAsRead() {
    let form = {};
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.MarkAllAsRead, form));
  }

  public saveUserSettings(userSettingsForm: UserSettingsForm) {
    this.setAuth(userSettingsForm);
    this.subject.next(
      this.wsSendWrapper(UserOperation.SaveUserSettings, userSettingsForm)
    );
  }

  public deleteAccount(form: DeleteAccountForm) {
    this.setAuth(form);
    this.subject.next(this.wsSendWrapper(UserOperation.DeleteAccount, form));
  }

  public passwordReset(form: PasswordResetForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.PasswordReset, form));
  }

  public passwordChange(form: PasswordChangeForm) {
    this.subject.next(this.wsSendWrapper(UserOperation.PasswordChange, form));
  }

  public createPrivateMessage(form: PrivateMessageForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.CreatePrivateMessage, form)
    );
  }

  public editPrivateMessage(form: EditPrivateMessageForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.EditPrivateMessage, form)
    );
  }

  public getPrivateMessages(form: GetPrivateMessagesForm) {
    this.setAuth(form);
    this.subject.next(
      this.wsSendWrapper(UserOperation.GetPrivateMessages, form)
    );
  }

  private wsSendWrapper(op: UserOperation, data: MessageType) {
    let send = { op: UserOperation[op], data: data };
    console.log(send);
    return send;
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
  WebSocketService.Instance.subject.unsubscribe();
  WebSocketService.Instance.subject = null;
};
