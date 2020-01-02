import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { WebSocketService, UserService } from '../services';
import {
  Post,
  CreatePostLikeForm,
  PostForm as PostFormI,
  SavePostForm,
  CommunityUser,
  UserView,
  BanType,
  BanFromCommunityForm,
  BanUserForm,
  AddModToCommunityForm,
  AddAdminForm,
  TransferSiteForm,
  TransferCommunityForm,
} from '../interfaces';
import { MomentTime } from './moment-time';
import { PostForm } from './post-form';
import {
  mdToHtml,
  canMod,
  isMod,
  isImage,
  isVideo,
  getUnixTime,
  pictshareAvatarThumbnail,
  showAvatars,
} from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface PostListingState {
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  showBanDialog: boolean;
  banReason: string;
  banExpires: string;
  banType: BanType;
  showConfirmTransferSite: boolean;
  showConfirmTransferCommunity: boolean;
  imageExpanded: boolean;
  viewSource: boolean;
}

interface PostListingProps {
  post: Post;
  showCommunity?: boolean;
  showBody?: boolean;
  viewOnly?: boolean;
  moderators?: Array<CommunityUser>;
  admins?: Array<UserView>;
}

export class PostListing extends Component<PostListingProps, PostListingState> {
  private emptyState: PostListingState = {
    showEdit: false,
    showRemoveDialog: false,
    removeReason: null,
    showBanDialog: false,
    banReason: null,
    banExpires: null,
    banType: BanType.Community,
    showConfirmTransferSite: false,
    showConfirmTransferCommunity: false,
    imageExpanded: false,
    viewSource: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handlePostLike = this.handlePostLike.bind(this);
    this.handlePostDisLike = this.handlePostDisLike.bind(this);
    this.handleEditPost = this.handleEditPost.bind(this);
    this.handleEditCancel = this.handleEditCancel.bind(this);
  }

  render() {
    return (
      <div class="row">
        {!this.state.showEdit ? (
          this.listing()
        ) : (
          <div class="col-12">
            <PostForm
              post={this.props.post}
              onEdit={this.handleEditPost}
              onCancel={this.handleEditCancel}
            />
          </div>
        )}
      </div>
    );
  }

  listing() {
    let post = this.props.post;
    return (
      <div class="listing col-12">
        <div
          className={`vote-bar mr-2 float-left small text-center ${this.props
            .viewOnly && 'no-click'}`}
        >
          <button
            className={`btn p-0 ${
              post.my_vote == 1 ? 'text-info' : 'text-muted'
            }`}
            onClick={linkEvent(this, this.handlePostLike)}
          >
            <svg class="icon upvote">
              <use xlinkHref="#icon-arrow-up"></use>
            </svg>
          </button>
          <div class={`font-weight-bold text-muted`}>{post.score}</div>
          {WebSocketService.Instance.site.enable_downvotes && (
            <button
              className={`btn p-0 ${
                post.my_vote == -1 ? 'text-danger' : 'text-muted'
              }`}
              onClick={linkEvent(this, this.handlePostDisLike)}
            >
              <svg class="icon downvote">
                <use xlinkHref="#icon-arrow-down"></use>
              </svg>
            </button>
          )}
        </div>
        {post.url && isImage(post.url) && !post.nsfw && !post.community_nsfw && (
          <span
            title={i18n.t('expand_here')}
            class="pointer"
            onClick={linkEvent(this, this.handleImageExpandClick)}
          >
            <img
              class="mx-2 mt-1 float-left img-fluid thumbnail rounded"
              src={post.url}
            />
          </span>
        )}
        {post.url && isVideo(post.url) && (
          <video
            playsinline
            muted
            loop
            controls
            class="mx-2 mt-1 float-left"
            height="100"
            width="150"
          >
            <source src={post.url} type="video/mp4" />
          </video>
        )}
        <div className="ml-4">
          <div className="post-title">
            <h5 className="mb-0 d-inline">
              {post.url ? (
                <a
                  className="text-body"
                  href={post.url}
                  target="_blank"
                  title={post.url}
                >
                  {post.name}
                </a>
              ) : (
                <Link
                  className="text-body"
                  to={`/post/${post.id}`}
                  title={i18n.t('comments')}
                >
                  {post.name}
                </Link>
              )}
            </h5>
            {post.url && (
              <small>
                <a
                  className="ml-2 text-muted font-italic"
                  href={post.url}
                  target="_blank"
                  title={post.url}
                >
                  {new URL(post.url).hostname}
                </a>
              </small>
            )}
            {post.url && isImage(post.url) && (
              <>
                {!this.state.imageExpanded ? (
                  <span
                    class="text-monospace pointer ml-2 text-muted small"
                    title={i18n.t('expand_here')}
                    onClick={linkEvent(this, this.handleImageExpandClick)}
                  >
                    [+]
                  </span>
                ) : (
                  <span>
                    <span
                      class="text-monospace pointer ml-2 text-muted small"
                      onClick={linkEvent(this, this.handleImageExpandClick)}
                    >
                      [-]
                    </span>
                    <div>
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleImageExpandClick)}
                      >
                        <img class="img-fluid" src={post.url} />
                      </span>
                    </div>
                  </span>
                )}
              </>
            )}
            {post.removed && (
              <small className="ml-2 text-muted font-italic">
                <T i18nKey="removed">#</T>
              </small>
            )}
            {post.deleted && (
              <small className="ml-2 text-muted font-italic">
                <T i18nKey="deleted">#</T>
              </small>
            )}
            {post.locked && (
              <small className="ml-2 text-muted font-italic">
                <T i18nKey="locked">#</T>
              </small>
            )}
            {post.stickied && (
              <small className="ml-2 text-muted font-italic">
                <T i18nKey="stickied">#</T>
              </small>
            )}
            {post.nsfw && (
              <small className="ml-2 text-muted font-italic">
                <T i18nKey="nsfw">#</T>
              </small>
            )}
          </div>
        </div>
        <div className="details ml-4">
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <span>{i18n.t('by')} </span>
              <Link className="text-info" to={`/u/${post.creator_name}`}>
                {post.creator_avatar && showAvatars() && (
                  <img
                    height="32"
                    width="32"
                    src={pictshareAvatarThumbnail(post.creator_avatar)}
                    class="rounded-circle mr-1"
                  />
                )}
                <span>{post.creator_name}</span>
              </Link>
              {this.isMod && (
                <span className="mx-1 badge badge-light">
                  <T i18nKey="mod">#</T>
                </span>
              )}
              {this.isAdmin && (
                <span className="mx-1 badge badge-light">
                  <T i18nKey="admin">#</T>
                </span>
              )}
              {(post.banned_from_community || post.banned) && (
                <span className="mx-1 badge badge-danger">
                  <T i18nKey="banned">#</T>
                </span>
              )}
              {this.props.showCommunity && (
                <span>
                  <span> {i18n.t('to')} </span>
                  <Link to={`/c/${post.community_name}`}>
                    {post.community_name}
                  </Link>
                </span>
              )}
            </li>
            <li className="list-inline-item">
              <span>
                <MomentTime data={post} />
              </span>
            </li>
            <li className="list-inline-item">
              <span>
                (<span className="text-info">+{post.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{post.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <Link className="text-muted" to={`/post/${post.id}`}>
                <T
                  i18nKey="number_of_comments"
                  interpolation={{ count: post.number_of_comments }}
                >
                  #
                </T>
              </Link>
            </li>
          </ul>
          <ul class="list-inline mb-1 text-muted small font-weight-bold">
            {UserService.Instance.user && (
              <>
                {this.props.showBody && (
                  <>
                    <li className="list-inline-item mr-2">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleSavePostClick)}
                      >
                        {post.saved ? i18n.t('unsave') : i18n.t('save')}
                      </span>
                    </li>
                    <li className="list-inline-item mr-2">
                      <Link
                        className="text-muted"
                        to={`/create_post${this.crossPostParams}`}
                      >
                        <T i18nKey="cross_post">#</T>
                      </Link>
                    </li>
                  </>
                )}
                {this.myPost && (
                  <>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleEditClick)}
                      >
                        <T i18nKey="edit">#</T>
                      </span>
                    </li>
                    <li className="list-inline-item mr-2">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleDeleteClick)}
                      >
                        {!post.deleted ? i18n.t('delete') : i18n.t('restore')}
                      </span>
                    </li>
                  </>
                )}
                {this.canModOnSelf && (
                  <>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleModLock)}
                      >
                        {post.locked ? i18n.t('unlock') : i18n.t('lock')}
                      </span>
                    </li>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleModSticky)}
                      >
                        {post.stickied ? i18n.t('unsticky') : i18n.t('sticky')}
                      </span>
                    </li>
                  </>
                )}
                {/* Mods can ban from community, and appoint as mods to community */}
                {(this.canMod || this.canAdmin) && (
                  <li className="list-inline-item">
                    {!post.removed ? (
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleModRemoveShow)}
                      >
                        <T i18nKey="remove">#</T>
                      </span>
                    ) : (
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleModRemoveSubmit)}
                      >
                        <T i18nKey="restore">#</T>
                      </span>
                    )}
                  </li>
                )}
                {this.canMod && (
                  <>
                    {!this.isMod && (
                      <li className="list-inline-item">
                        {!post.banned_from_community ? (
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleModBanFromCommunityShow
                            )}
                          >
                            <T i18nKey="ban">#</T>
                          </span>
                        ) : (
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleModBanFromCommunitySubmit
                            )}
                          >
                            <T i18nKey="unban">#</T>
                          </span>
                        )}
                      </li>
                    )}
                    {!post.banned_from_community && (
                      <li className="list-inline-item">
                        <span
                          class="pointer"
                          onClick={linkEvent(
                            this,
                            this.handleAddModToCommunity
                          )}
                        >
                          {this.isMod
                            ? i18n.t('remove_as_mod')
                            : i18n.t('appoint_as_mod')}
                        </span>
                      </li>
                    )}
                  </>
                )}
                {/* Community creators and admins can transfer community to another mod */}
                {(this.amCommunityCreator || this.canAdmin) && this.isMod && (
                  <li className="list-inline-item">
                    {!this.state.showConfirmTransferCommunity ? (
                      <span
                        class="pointer"
                        onClick={linkEvent(
                          this,
                          this.handleShowConfirmTransferCommunity
                        )}
                      >
                        <T i18nKey="transfer_community">#</T>
                      </span>
                    ) : (
                      <>
                        <span class="d-inline-block mr-1">
                          <T i18nKey="are_you_sure">#</T>
                        </span>
                        <span
                          class="pointer d-inline-block mr-1"
                          onClick={linkEvent(
                            this,
                            this.handleTransferCommunity
                          )}
                        >
                          <T i18nKey="yes">#</T>
                        </span>
                        <span
                          class="pointer d-inline-block"
                          onClick={linkEvent(
                            this,
                            this.handleCancelShowConfirmTransferCommunity
                          )}
                        >
                          <T i18nKey="no">#</T>
                        </span>
                      </>
                    )}
                  </li>
                )}
                {/* Admins can ban from all, and appoint other admins */}
                {this.canAdmin && (
                  <>
                    {!this.isAdmin && (
                      <li className="list-inline-item">
                        {!post.banned ? (
                          <span
                            class="pointer"
                            onClick={linkEvent(this, this.handleModBanShow)}
                          >
                            <T i18nKey="ban_from_site">#</T>
                          </span>
                        ) : (
                          <span
                            class="pointer"
                            onClick={linkEvent(this, this.handleModBanSubmit)}
                          >
                            <T i18nKey="unban_from_site">#</T>
                          </span>
                        )}
                      </li>
                    )}
                    {!post.banned && (
                      <li className="list-inline-item">
                        <span
                          class="pointer"
                          onClick={linkEvent(this, this.handleAddAdmin)}
                        >
                          {this.isAdmin
                            ? i18n.t('remove_as_admin')
                            : i18n.t('appoint_as_admin')}
                        </span>
                      </li>
                    )}
                  </>
                )}
                {/* Site Creator can transfer to another admin */}
                {this.amSiteCreator && this.isAdmin && (
                  <li className="list-inline-item">
                    {!this.state.showConfirmTransferSite ? (
                      <span
                        class="pointer"
                        onClick={linkEvent(
                          this,
                          this.handleShowConfirmTransferSite
                        )}
                      >
                        <T i18nKey="transfer_site">#</T>
                      </span>
                    ) : (
                      <>
                        <span class="d-inline-block mr-1">
                          <T i18nKey="are_you_sure">#</T>
                        </span>
                        <span
                          class="pointer d-inline-block mr-1"
                          onClick={linkEvent(this, this.handleTransferSite)}
                        >
                          <T i18nKey="yes">#</T>
                        </span>
                        <span
                          class="pointer d-inline-block"
                          onClick={linkEvent(
                            this,
                            this.handleCancelShowConfirmTransferSite
                          )}
                        >
                          <T i18nKey="no">#</T>
                        </span>
                      </>
                    )}
                  </li>
                )}
              </>
            )}
            {this.props.showBody && post.body && (
              <li className="list-inline-item">
                <span
                  className="pointer"
                  onClick={linkEvent(this, this.handleViewSource)}
                >
                  <T i18nKey="view_source">#</T>
                </span>
              </li>
            )}
          </ul>
          {this.state.showRemoveDialog && (
            <form
              class="form-inline"
              onSubmit={linkEvent(this, this.handleModRemoveSubmit)}
            >
              <input
                type="text"
                class="form-control mr-2"
                placeholder={i18n.t('reason')}
                value={this.state.removeReason}
                onInput={linkEvent(this, this.handleModRemoveReasonChange)}
              />
              <button type="submit" class="btn btn-secondary">
                <T i18nKey="remove_post">#</T>
              </button>
            </form>
          )}
          {this.state.showBanDialog && (
            <form onSubmit={linkEvent(this, this.handleModBanBothSubmit)}>
              <div class="form-group row">
                <label class="col-form-label">
                  <T i18nKey="reason">#</T>
                </label>
                <input
                  type="text"
                  class="form-control mr-2"
                  placeholder={i18n.t('reason')}
                  value={this.state.banReason}
                  onInput={linkEvent(this, this.handleModBanReasonChange)}
                />
              </div>
              {/* TODO hold off on expires until later */}
              {/* <div class="form-group row"> */}
              {/*   <label class="col-form-label">Expires</label> */}
              {/*   <input type="date" class="form-control mr-2" placeholder={i18n.t('expires')} value={this.state.banExpires} onInput={linkEvent(this, this.handleModBanExpiresChange)} /> */}
              {/* </div> */}
              <div class="form-group row">
                <button type="submit" class="btn btn-secondary">
                  {i18n.t('ban')} {post.creator_name}
                </button>
              </div>
            </form>
          )}
          {this.props.showBody && post.body && (
            <>
              {this.state.viewSource ? (
                <pre>{post.body}</pre>
              ) : (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(post.body)}
                />
              )}
            </>
          )}
        </div>
      </div>
    );
  }

  private get myPost(): boolean {
    return (
      UserService.Instance.user &&
      this.props.post.creator_id == UserService.Instance.user.id
    );
  }

  get isMod(): boolean {
    return (
      this.props.moderators &&
      isMod(
        this.props.moderators.map(m => m.user_id),
        this.props.post.creator_id
      )
    );
  }

  get isAdmin(): boolean {
    return (
      this.props.admins &&
      isMod(this.props.admins.map(a => a.id), this.props.post.creator_id)
    );
  }

  get canMod(): boolean {
    if (this.props.admins && this.props.moderators) {
      let adminsThenMods = this.props.admins
        .map(a => a.id)
        .concat(this.props.moderators.map(m => m.user_id));

      return canMod(
        UserService.Instance.user,
        adminsThenMods,
        this.props.post.creator_id
      );
    } else {
      return false;
    }
  }

  get canModOnSelf(): boolean {
    if (this.props.admins && this.props.moderators) {
      let adminsThenMods = this.props.admins
        .map(a => a.id)
        .concat(this.props.moderators.map(m => m.user_id));

      return canMod(
        UserService.Instance.user,
        adminsThenMods,
        this.props.post.creator_id,
        true
      );
    } else {
      return false;
    }
  }

  get canAdmin(): boolean {
    return (
      this.props.admins &&
      canMod(
        UserService.Instance.user,
        this.props.admins.map(a => a.id),
        this.props.post.creator_id
      )
    );
  }

  get amCommunityCreator(): boolean {
    return (
      this.props.moderators &&
      UserService.Instance.user &&
      this.props.post.creator_id != UserService.Instance.user.id &&
      UserService.Instance.user.id == this.props.moderators[0].user_id
    );
  }

  get amSiteCreator(): boolean {
    return (
      this.props.admins &&
      UserService.Instance.user &&
      this.props.post.creator_id != UserService.Instance.user.id &&
      UserService.Instance.user.id == this.props.admins[0].id
    );
  }

  handlePostLike(i: PostListing) {
    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: i.props.post.my_vote == 1 ? 0 : 1,
    };
    WebSocketService.Instance.likePost(form);
  }

  handlePostDisLike(i: PostListing) {
    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: i.props.post.my_vote == -1 ? 0 : -1,
    };
    WebSocketService.Instance.likePost(form);
  }

  handleEditClick(i: PostListing) {
    i.state.showEdit = true;
    i.setState(i.state);
  }

  handleEditCancel() {
    this.state.showEdit = false;
    this.setState(this.state);
  }

  // The actual editing is done in the recieve for post
  handleEditPost() {
    this.state.showEdit = false;
    this.setState(this.state);
  }

  handleDeleteClick(i: PostListing) {
    let deleteForm: PostFormI = {
      body: i.props.post.body,
      community_id: i.props.post.community_id,
      name: i.props.post.name,
      url: i.props.post.url,
      edit_id: i.props.post.id,
      creator_id: i.props.post.creator_id,
      deleted: !i.props.post.deleted,
      nsfw: i.props.post.nsfw,
      auth: null,
    };
    WebSocketService.Instance.editPost(deleteForm);
  }

  handleSavePostClick(i: PostListing) {
    let saved = i.props.post.saved == undefined ? true : !i.props.post.saved;
    let form: SavePostForm = {
      post_id: i.props.post.id,
      save: saved,
    };

    WebSocketService.Instance.savePost(form);
  }

  get crossPostParams(): string {
    let params = `?name=${this.props.post.name}`;
    if (this.props.post.url) {
      params += `&url=${this.props.post.url}`;
    }
    if (this.props.post.body) {
      params += `&body=${this.props.post.body}`;
    }
    return params;
  }

  handleModRemoveShow(i: PostListing) {
    i.state.showRemoveDialog = true;
    i.setState(i.state);
  }

  handleModRemoveReasonChange(i: PostListing, event: any) {
    i.state.removeReason = event.target.value;
    i.setState(i.state);
  }

  handleModRemoveSubmit(i: PostListing) {
    event.preventDefault();
    let form: PostFormI = {
      name: i.props.post.name,
      community_id: i.props.post.community_id,
      edit_id: i.props.post.id,
      creator_id: i.props.post.creator_id,
      removed: !i.props.post.removed,
      reason: i.state.removeReason,
      nsfw: i.props.post.nsfw,
      auth: null,
    };
    WebSocketService.Instance.editPost(form);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }

  handleModLock(i: PostListing) {
    let form: PostFormI = {
      name: i.props.post.name,
      community_id: i.props.post.community_id,
      edit_id: i.props.post.id,
      creator_id: i.props.post.creator_id,
      nsfw: i.props.post.nsfw,
      locked: !i.props.post.locked,
      auth: null,
    };
    WebSocketService.Instance.editPost(form);
  }

  handleModSticky(i: PostListing) {
    let form: PostFormI = {
      name: i.props.post.name,
      community_id: i.props.post.community_id,
      edit_id: i.props.post.id,
      creator_id: i.props.post.creator_id,
      nsfw: i.props.post.nsfw,
      stickied: !i.props.post.stickied,
      auth: null,
    };
    WebSocketService.Instance.editPost(form);
  }

  handleModBanFromCommunityShow(i: PostListing) {
    i.state.showBanDialog = true;
    i.state.banType = BanType.Community;
    i.setState(i.state);
  }

  handleModBanShow(i: PostListing) {
    i.state.showBanDialog = true;
    i.state.banType = BanType.Site;
    i.setState(i.state);
  }

  handleModBanReasonChange(i: PostListing, event: any) {
    i.state.banReason = event.target.value;
    i.setState(i.state);
  }

  handleModBanExpiresChange(i: PostListing, event: any) {
    i.state.banExpires = event.target.value;
    i.setState(i.state);
  }

  handleModBanFromCommunitySubmit(i: PostListing) {
    i.state.banType = BanType.Community;
    i.setState(i.state);
    i.handleModBanBothSubmit(i);
  }

  handleModBanSubmit(i: PostListing) {
    i.state.banType = BanType.Site;
    i.setState(i.state);
    i.handleModBanBothSubmit(i);
  }

  handleModBanBothSubmit(i: PostListing) {
    event.preventDefault();

    if (i.state.banType == BanType.Community) {
      let form: BanFromCommunityForm = {
        user_id: i.props.post.creator_id,
        community_id: i.props.post.community_id,
        ban: !i.props.post.banned_from_community,
        reason: i.state.banReason,
        expires: getUnixTime(i.state.banExpires),
      };
      WebSocketService.Instance.banFromCommunity(form);
    } else {
      let form: BanUserForm = {
        user_id: i.props.post.creator_id,
        ban: !i.props.post.banned,
        reason: i.state.banReason,
        expires: getUnixTime(i.state.banExpires),
      };
      WebSocketService.Instance.banUser(form);
    }

    i.state.showBanDialog = false;
    i.setState(i.state);
  }

  handleAddModToCommunity(i: PostListing) {
    let form: AddModToCommunityForm = {
      user_id: i.props.post.creator_id,
      community_id: i.props.post.community_id,
      added: !i.isMod,
    };
    WebSocketService.Instance.addModToCommunity(form);
    i.setState(i.state);
  }

  handleAddAdmin(i: PostListing) {
    let form: AddAdminForm = {
      user_id: i.props.post.creator_id,
      added: !i.isAdmin,
    };
    WebSocketService.Instance.addAdmin(form);
    i.setState(i.state);
  }

  handleShowConfirmTransferCommunity(i: PostListing) {
    i.state.showConfirmTransferCommunity = true;
    i.setState(i.state);
  }

  handleCancelShowConfirmTransferCommunity(i: PostListing) {
    i.state.showConfirmTransferCommunity = false;
    i.setState(i.state);
  }

  handleTransferCommunity(i: PostListing) {
    let form: TransferCommunityForm = {
      community_id: i.props.post.community_id,
      user_id: i.props.post.creator_id,
    };
    WebSocketService.Instance.transferCommunity(form);
    i.state.showConfirmTransferCommunity = false;
    i.setState(i.state);
  }

  handleShowConfirmTransferSite(i: PostListing) {
    i.state.showConfirmTransferSite = true;
    i.setState(i.state);
  }

  handleCancelShowConfirmTransferSite(i: PostListing) {
    i.state.showConfirmTransferSite = false;
    i.setState(i.state);
  }

  handleTransferSite(i: PostListing) {
    let form: TransferSiteForm = {
      user_id: i.props.post.creator_id,
    };
    WebSocketService.Instance.transferSite(form);
    i.state.showConfirmTransferSite = false;
    i.setState(i.state);
  }

  handleImageExpandClick(i: PostListing) {
    i.state.imageExpanded = !i.state.imageExpanded;
    i.setState(i.state);
  }

  handleViewSource(i: PostListing) {
    i.state.viewSource = !i.state.viewSource;
    i.setState(i.state);
  }
}
