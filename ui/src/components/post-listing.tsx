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
  FramelyData,
} from '../interfaces';
import { MomentTime } from './moment-time';
import { PostForm } from './post-form';
import { IFramelyCard } from './iframely-card';
import {
  mdToHtml,
  canMod,
  isMod,
  isImage,
  isVideo,
  getUnixTime,
  pictshareAvatarThumbnail,
  showAvatars,
  imageThumbnailer,
} from '../utils';
import { i18n } from '../i18next';

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
  my_vote: number;
  score: number;
  upvotes: number;
  downvotes: number;
  url: string;
  iframely: FramelyData;
  thumbnail: string;
}

interface PostListingProps {
  post: Post;
  showCommunity?: boolean;
  showBody?: boolean;
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
    my_vote: this.props.post.my_vote,
    score: this.props.post.score,
    upvotes: this.props.post.upvotes,
    downvotes: this.props.post.downvotes,
    url: this.props.post.url,
    iframely: null,
    thumbnail: null,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handlePostLike = this.handlePostLike.bind(this);
    this.handlePostDisLike = this.handlePostDisLike.bind(this);
    this.handleEditPost = this.handleEditPost.bind(this);
    this.handleEditCancel = this.handleEditCancel.bind(this);

    if (this.state.url) {
      this.setThumbnail();
      this.fetchIframely();
    }
  }

  componentWillReceiveProps(nextProps: PostListingProps) {
    this.state.my_vote = nextProps.post.my_vote;
    this.state.upvotes = nextProps.post.upvotes;
    this.state.downvotes = nextProps.post.downvotes;
    this.state.score = nextProps.post.score;

    if (nextProps.post.url !== this.state.url) {
      this.state.url = nextProps.post.url;
      if (this.state.url) {
        this.setThumbnail();
        this.fetchIframely();
      } else {
        this.state.iframely = null;
        this.state.thumbnail = null;
      }
    }

    this.setState(this.state);
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

  imgThumbnail() {
    let post = this.props.post;
    return (
      <object
        className={`img-fluid thumbnail rounded ${(post.nsfw ||
          post.community_nsfw) &&
          'img-blur'}`}
        data={imageThumbnailer(this.state.thumbnail)}
      ></object>
    );
  }

  listing() {
    let post = this.props.post;
    return (
      <div class="listing col-12">
        <div className={`vote-bar mr-2 float-left small text-center`}>
          <button
            className={`vote-animate btn btn-link p-0 ${
              this.state.my_vote == 1 ? 'text-info' : 'text-muted'
            }`}
            onClick={linkEvent(this, this.handlePostLike)}
          >
            <svg class="icon upvote">
              <use xlinkHref="#icon-arrow-up"></use>
            </svg>
          </button>
          <div class={`font-weight-bold text-muted`}>{this.state.score}</div>
          {WebSocketService.Instance.site.enable_downvotes && (
            <button
              className={`vote-animate btn btn-link p-0 ${
                this.state.my_vote == -1 ? 'text-danger' : 'text-muted'
              }`}
              onClick={linkEvent(this, this.handlePostDisLike)}
            >
              <svg class="icon downvote">
                <use xlinkHref="#icon-arrow-down"></use>
              </svg>
            </button>
          )}
        </div>
        {this.state.thumbnail && !this.state.imageExpanded && (
          <div class="mx-2 mt-1 float-left position-relative">
            {isImage(this.state.url) ? (
              <span
                class="text-body pointer"
                title={i18n.t('expand_here')}
                onClick={linkEvent(this, this.handleImageExpandClick)}
              >
                {this.imgThumbnail()}
                <svg class="icon rounded link-overlay hover-link">
                  <use xlinkHref="#icon-image"></use>
                </svg>
              </span>
            ) : (
              <a
                className="text-body"
                href={this.state.url}
                target="_blank"
                title={this.state.url}
              >
                {this.imgThumbnail()}
                <svg class="icon rounded link-overlay hover-link">
                  <use xlinkHref="#icon-external-link"></use>
                </svg>
              </a>
            )}
          </div>
        )}
        {this.state.url && isVideo(this.state.url) && (
          <video
            playsinline
            muted
            loop
            controls
            class="mx-2 mt-1 float-left"
            height="100"
            width="150"
          >
            <source src={this.state.url} type="video/mp4" />
          </video>
        )}
        <div className="ml-4">
          <div className="post-title">
            <h5 className="mb-0 d-inline">
              {this.props.showBody && this.state.url ? (
                <a
                  className="text-body"
                  href={this.state.url}
                  target="_blank"
                  title={this.state.url}
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
            {this.state.url &&
              !(
                new URL(this.state.url).hostname == window.location.hostname
              ) && (
                <small class="d-inline-block">
                  <a
                    className="ml-2 text-muted font-italic"
                    href={this.state.url}
                    target="_blank"
                    title={this.state.url}
                  >
                    {new URL(this.state.url).hostname}
                    <svg class="ml-1 icon">
                      <use xlinkHref="#icon-external-link"></use>
                    </svg>
                  </a>
                </small>
              )}
            {this.state.thumbnail && (
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
                        <object
                          class="img-fluid img-expanded"
                          data={this.state.thumbnail}
                        >
                          <svg class="icon rounded placeholder">
                            <use xlinkHref="#icon-external-link"></use>
                          </svg>
                        </object>
                      </span>
                    </div>
                  </span>
                )}
              </>
            )}
            {post.removed && (
              <small className="ml-2 text-muted font-italic">
                {i18n.t('removed')}
              </small>
            )}
            {post.deleted && (
              <small className="ml-2 text-muted font-italic">
                {i18n.t('deleted')}
              </small>
            )}
            {post.locked && (
              <small className="ml-2 text-muted font-italic">
                {i18n.t('locked')}
              </small>
            )}
            {post.stickied && (
              <small className="ml-2 text-muted font-italic">
                {i18n.t('stickied')}
              </small>
            )}
            {post.nsfw && (
              <small className="ml-2 text-muted font-italic">
                {i18n.t('nsfw')}
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
                <span className="mx-1 badge badge-light">{i18n.t('mod')}</span>
              )}
              {this.isAdmin && (
                <span className="mx-1 badge badge-light">
                  {i18n.t('admin')}
                </span>
              )}
              {(post.banned_from_community || post.banned) && (
                <span className="mx-1 badge badge-danger">
                  {i18n.t('banned')}
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
                (<span className="text-info">+{this.state.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{this.state.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <Link className="text-muted" to={`/post/${post.id}`}>
                {i18n.t('number_of_comments', {
                  count: post.number_of_comments,
                })}
              </Link>
            </li>
          </ul>
          <ul class="list-inline mb-1 text-muted small">
            {this.props.post.duplicates && (
              <>
                <li className="list-inline-item mr-2">
                  {i18n.t('cross_posted_to')}
                </li>
                {this.props.post.duplicates.map(post => (
                  <li className="list-inline-item mr-2">
                    <Link to={`/post/${post.id}`}>{post.community_name}</Link>
                  </li>
                ))}
              </>
            )}
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
                        {i18n.t('cross_post')}
                      </Link>
                    </li>
                  </>
                )}
                {this.myPost && this.props.showBody && (
                  <>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleEditClick)}
                      >
                        {i18n.t('edit')}
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
                        {i18n.t('remove')}
                      </span>
                    ) : (
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleModRemoveSubmit)}
                      >
                        {i18n.t('restore')}
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
                            {i18n.t('ban')}
                          </span>
                        ) : (
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleModBanFromCommunitySubmit
                            )}
                          >
                            {i18n.t('unban')}
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
                        {i18n.t('transfer_community')}
                      </span>
                    ) : (
                      <>
                        <span class="d-inline-block mr-1">
                          {i18n.t('are_you_sure')}
                        </span>
                        <span
                          class="pointer d-inline-block mr-1"
                          onClick={linkEvent(
                            this,
                            this.handleTransferCommunity
                          )}
                        >
                          {i18n.t('yes')}
                        </span>
                        <span
                          class="pointer d-inline-block"
                          onClick={linkEvent(
                            this,
                            this.handleCancelShowConfirmTransferCommunity
                          )}
                        >
                          {i18n.t('no')}
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
                            {i18n.t('ban_from_site')}
                          </span>
                        ) : (
                          <span
                            class="pointer"
                            onClick={linkEvent(this, this.handleModBanSubmit)}
                          >
                            {i18n.t('unban_from_site')}
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
                        {i18n.t('transfer_site')}
                      </span>
                    ) : (
                      <>
                        <span class="d-inline-block mr-1">
                          {i18n.t('are_you_sure')}
                        </span>
                        <span
                          class="pointer d-inline-block mr-1"
                          onClick={linkEvent(this, this.handleTransferSite)}
                        >
                          {i18n.t('yes')}
                        </span>
                        <span
                          class="pointer d-inline-block"
                          onClick={linkEvent(
                            this,
                            this.handleCancelShowConfirmTransferSite
                          )}
                        >
                          {i18n.t('no')}
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
                  {i18n.t('view_source')}
                </span>
              </li>
            )}
          </ul>
          {this.state.url && this.props.showBody && this.state.iframely && (
            <IFramelyCard iframely={this.state.iframely} />
          )}
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
                {i18n.t('remove_post')}
              </button>
            </form>
          )}
          {this.state.showBanDialog && (
            <form onSubmit={linkEvent(this, this.handleModBanBothSubmit)}>
              <div class="form-group row">
                <label class="col-form-label" htmlFor="post-listing-reason">
                  {i18n.t('reason')}
                </label>
                <input
                  type="text"
                  id="post-listing-reason"
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
      isMod(
        this.props.admins.map(a => a.id),
        this.props.post.creator_id
      )
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

  fetchIframely() {
    fetch(`/iframely/oembed?url=${this.state.url}`)
      .then(res => res.json())
      .then(res => {
        this.state.iframely = res;
        this.setState(this.state);

        // Store and fetch the image in pictshare
        if (
          this.state.iframely.thumbnail_url &&
          isImage(this.state.iframely.thumbnail_url)
        ) {
          fetch(
            `/pictshare/api/geturl.php?url=${this.state.iframely.thumbnail_url}`
          )
            .then(res => res.json())
            .then(res => {
              let url = `${window.location.origin}/pictshare/${res.url}`;
              if (res.filetype == 'mp4') {
                url += '/raw';
              }
              this.state.thumbnail = url;
              this.setState(this.state);
            });
        }
      })
      .catch(error => {
        console.error(`Iframely service not set up properly. ${error}`);
      });
  }

  setThumbnail() {
    let simpleImg = isImage(this.state.url);
    if (simpleImg) {
      this.state.thumbnail = this.state.url;
    } else {
      this.state.thumbnail = null;
    }
    this.setState(this.state);
  }

  handlePostLike(i: PostListing) {
    let new_vote = i.state.my_vote == 1 ? 0 : 1;

    if (i.state.my_vote == 1) {
      i.state.score--;
      i.state.upvotes--;
    } else if (i.state.my_vote == -1) {
      i.state.downvotes--;
      i.state.upvotes++;
      i.state.score += 2;
    } else {
      i.state.upvotes++;
      i.state.score++;
    }

    i.state.my_vote = new_vote;

    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: i.state.my_vote,
    };

    WebSocketService.Instance.likePost(form);
    i.setState(i.state);
  }

  handlePostDisLike(i: PostListing) {
    let new_vote = i.state.my_vote == -1 ? 0 : -1;

    if (i.state.my_vote == 1) {
      i.state.score -= 2;
      i.state.upvotes--;
      i.state.downvotes++;
    } else if (i.state.my_vote == -1) {
      i.state.downvotes--;
      i.state.score++;
    } else {
      i.state.downvotes++;
      i.state.score--;
    }

    i.state.my_vote = new_vote;

    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: i.state.my_vote,
    };

    WebSocketService.Instance.likePost(form);
    i.setState(i.state);
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
    let params = `?title=${this.props.post.name}`;
    if (this.state.url) {
      params += `&url=${this.state.url}`;
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
