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
import { IFramelyCard } from './iframely-card';
import { UserListing } from './user-listing';
import {
  md,
  mdToHtml,
  canMod,
  isMod,
  isImage,
  isVideo,
  getUnixTime,
  pictrsImage,
  setupTippy,
  previewLines,
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
  showAdvanced: boolean;
  my_vote: number;
  score: number;
  upvotes: number;
  downvotes: number;
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
    showAdvanced: false,
    my_vote: this.props.post.my_vote,
    score: this.props.post.score,
    upvotes: this.props.post.upvotes,
    downvotes: this.props.post.downvotes,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handlePostLike = this.handlePostLike.bind(this);
    this.handlePostDisLike = this.handlePostDisLike.bind(this);
    this.handleEditPost = this.handleEditPost.bind(this);
    this.handleEditCancel = this.handleEditCancel.bind(this);
  }

  componentWillReceiveProps(nextProps: PostListingProps) {
    this.state.my_vote = nextProps.post.my_vote;
    this.state.upvotes = nextProps.post.upvotes;
    this.state.downvotes = nextProps.post.downvotes;
    this.state.score = nextProps.post.score;
    this.setState(this.state);
  }

  render() {
    return (
      <div class="">
        {!this.state.showEdit ? (
          <>
            {this.listing()}
            {this.body()}
          </>
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

  body() {
    return (
      <div class="row">
        <div class="col-12">
          {this.props.post.url &&
            this.props.showBody &&
            this.props.post.embed_title && (
              <IFramelyCard post={this.props.post} />
            )}
          {this.props.showBody && this.props.post.body && (
            <>
              {this.state.viewSource ? (
                <pre>{this.props.post.body}</pre>
              ) : (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(this.props.post.body)}
                />
              )}
            </>
          )}
        </div>
      </div>
    );
  }

  imgThumb(src: string) {
    let post = this.props.post;
    return (
      <img
        className={`img-fluid thumbnail rounded ${
          (post.nsfw || post.community_nsfw) && 'img-blur'
        }`}
        src={src}
      />
    );
  }

  getImage(thumbnail: boolean = false) {
    let post = this.props.post;
    if (isImage(post.url)) {
      if (post.url.includes('pictrs')) {
        return pictrsImage(post.url, thumbnail);
      } else if (post.thumbnail_url) {
        return pictrsImage(post.thumbnail_url, thumbnail);
      } else {
        return post.url;
      }
    } else if (post.thumbnail_url) {
      return pictrsImage(post.thumbnail_url, thumbnail);
    }
  }

  thumbnail() {
    let post = this.props.post;

    if (isImage(post.url)) {
      return (
        <span
          class="text-body pointer"
          data-tippy-content={i18n.t('expand_here')}
          onClick={linkEvent(this, this.handleImageExpandClick)}
        >
          {this.imgThumb(this.getImage(true))}
          <svg class="icon mini-overlay">
            <use xlinkHref="#icon-image"></use>
          </svg>
        </span>
      );
    } else if (post.thumbnail_url) {
      return (
        <a
          className="text-body"
          href={post.url}
          target="_blank"
          title={post.url}
        >
          {this.imgThumb(this.getImage(true))}
          <svg class="icon mini-overlay">
            <use xlinkHref="#icon-external-link"></use>
          </svg>
        </a>
      );
    } else if (post.url) {
      if (isVideo(post.url)) {
        return (
          <div class="embed-responsive embed-responsive-16by9">
            <video
              playsinline
              muted
              loop
              controls
              class="embed-responsive-item"
            >
              <source src={post.url} type="video/mp4" />
            </video>
          </div>
        );
      } else {
        return (
          <a
            className="text-body"
            href={post.url}
            target="_blank"
            title={post.url}
          >
            <svg class="icon thumbnail">
              <use xlinkHref="#icon-external-link"></use>
            </svg>
          </a>
        );
      }
    } else {
      return (
        <Link
          className="text-body"
          to={`/post/${post.id}`}
          title={i18n.t('comments')}
        >
          <svg class="icon thumbnail">
            <use xlinkHref="#icon-message-square"></use>
          </svg>
        </Link>
      );
    }
  }

  listing() {
    let post = this.props.post;
    return (
      <div class="row">
        <div className={`vote-bar col-1 pr-0 small text-center`}>
          <button
            className={`btn-animate btn btn-link p-0 ${
              this.state.my_vote == 1 ? 'text-info' : 'text-muted'
            }`}
            onClick={linkEvent(this, this.handlePostLike)}
            data-tippy-content={i18n.t('upvote')}
          >
            <svg class="icon upvote">
              <use xlinkHref="#icon-arrow-up1"></use>
            </svg>
          </button>
          <div
            class={`unselectable pointer font-weight-bold text-muted px-1`}
            data-tippy-content={this.pointsTippy}
          >
            {this.state.score}
          </div>
          {WebSocketService.Instance.site.enable_downvotes && (
            <button
              className={`btn-animate btn btn-link p-0 ${
                this.state.my_vote == -1 ? 'text-danger' : 'text-muted'
              }`}
              onClick={linkEvent(this, this.handlePostDisLike)}
              data-tippy-content={i18n.t('downvote')}
            >
              <svg class="icon downvote">
                <use xlinkHref="#icon-arrow-down1"></use>
              </svg>
            </button>
          )}
        </div>
        {!this.state.imageExpanded && (
          <div class="col-3 col-sm-2 pr-0 mt-1">
            <div class="position-relative">{this.thumbnail()}</div>
          </div>
        )}
        <div
          class={`${this.state.imageExpanded ? 'col-12' : 'col-8 col-sm-9'}`}
        >
          <div class="row">
            <div className="col-12">
              <div className="post-title">
                <h5 className="mb-0 d-inline">
                  {this.props.showBody && post.url ? (
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
                {post.url &&
                  !(new URL(post.url).hostname == window.location.hostname) && (
                    <small class="d-inline-block">
                      <a
                        className="ml-2 text-muted font-italic"
                        href={post.url}
                        target="_blank"
                        title={post.url}
                      >
                        {new URL(post.url).hostname}
                        <svg class="ml-1 icon icon-inline">
                          <use xlinkHref="#icon-external-link"></use>
                        </svg>
                      </a>
                    </small>
                  )}
                {(isImage(post.url) || this.props.post.thumbnail_url) && (
                  <>
                    {!this.state.imageExpanded ? (
                      <span
                        class="text-monospace unselectable pointer ml-2 text-muted small"
                        data-tippy-content={i18n.t('expand_here')}
                        onClick={linkEvent(this, this.handleImageExpandClick)}
                      >
                        <svg class="icon icon-inline">
                          <use xlinkHref="#icon-plus-square"></use>
                        </svg>
                      </span>
                    ) : (
                      <span>
                        <span
                          class="text-monospace unselectable pointer ml-2 text-muted small"
                          onClick={linkEvent(this, this.handleImageExpandClick)}
                        >
                          <svg class="icon icon-inline">
                            <use xlinkHref="#icon-minus-square"></use>
                          </svg>
                        </span>
                        <div>
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleImageExpandClick
                            )}
                          >
                            <img
                              class="img-fluid img-expanded"
                              src={this.getImage()}
                            />
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
                  <small
                    className="unselectable pointer ml-2 text-muted font-italic"
                    data-tippy-content={i18n.t('deleted')}
                  >
                    <svg class={`icon icon-inline text-danger`}>
                      <use xlinkHref="#icon-trash"></use>
                    </svg>
                  </small>
                )}
                {post.locked && (
                  <small
                    className="unselectable pointer ml-2 text-muted font-italic"
                    data-tippy-content={i18n.t('locked')}
                  >
                    <svg class={`icon icon-inline text-danger`}>
                      <use xlinkHref="#icon-lock"></use>
                    </svg>
                  </small>
                )}
                {post.stickied && (
                  <small
                    className="unselectable pointer ml-2 text-muted font-italic"
                    data-tippy-content={i18n.t('stickied')}
                  >
                    <svg class={`icon icon-inline text-success`}>
                      <use xlinkHref="#icon-pin"></use>
                    </svg>
                  </small>
                )}
                {post.nsfw && (
                  <small className="ml-2 text-muted font-italic">
                    {i18n.t('nsfw')}
                  </small>
                )}
              </div>
            </div>
          </div>
          <div class="row">
            <div className="details col-12">
              <ul class="list-inline mb-0 text-muted small">
                <li className="list-inline-item">
                  <span>{i18n.t('by')} </span>
                  <UserListing
                    user={{
                      name: post.creator_name,
                      avatar: post.creator_avatar,
                    }}
                  />
                  {this.isMod && (
                    <span className="mx-1 badge badge-light">
                      {i18n.t('mod')}
                    </span>
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
                <li className="list-inline-item">•</li>
                <li className="list-inline-item">
                  <span>
                    <MomentTime data={post} />
                  </span>
                </li>
                {post.body && (
                  <>
                    <li className="list-inline-item">•</li>
                    <li className="list-inline-item">
                      {/* Using a link with tippy doesn't work on touch devices unfortunately */}
                      <Link
                        className="text-muted"
                        data-tippy-content={md.render(previewLines(post.body))}
                        data-tippy-allowHtml={true}
                        to={`/post/${post.id}`}
                      >
                        <svg class="mr-1 icon icon-inline">
                          <use xlinkHref="#icon-book-open"></use>
                        </svg>
                      </Link>
                    </li>
                  </>
                )}
                <li className="list-inline-item">•</li>
                {this.state.upvotes !== this.state.score && (
                  <>
                    <span
                      class="unselectable pointer mr-2"
                      data-tippy-content={this.pointsTippy}
                    >
                      <li className="list-inline-item">
                        <span className="text-muted">
                          <svg class="small icon icon-inline mr-1">
                            <use xlinkHref="#icon-arrow-up"></use>
                          </svg>
                          {this.state.upvotes}
                        </span>
                      </li>
                      <li className="list-inline-item">
                        <span className="text-muted">
                          <svg class="small icon icon-inline mr-1">
                            <use xlinkHref="#icon-arrow-down"></use>
                          </svg>
                          {this.state.downvotes}
                        </span>
                      </li>
                    </span>
                    <li className="list-inline-item">•</li>
                  </>
                )}
                <li className="list-inline-item">
                  <Link
                    className="text-muted"
                    title={i18n.t('number_of_comments', {
                      count: post.number_of_comments,
                    })}
                    to={`/post/${post.id}`}
                  >
                    <svg class="mr-1 icon icon-inline">
                      <use xlinkHref="#icon-message-square"></use>
                    </svg>
                    {post.number_of_comments}
                  </Link>
                </li>
              </ul>
              {this.props.post.duplicates && (
                <ul class="list-inline mb-1 small text-muted">
                  <>
                    <li className="list-inline-item mr-2">
                      {i18n.t('cross_posted_to')}
                    </li>
                    {this.props.post.duplicates.map(post => (
                      <li className="list-inline-item mr-2">
                        <Link to={`/post/${post.id}`}>
                          {post.community_name}
                        </Link>
                      </li>
                    ))}
                  </>
                </ul>
              )}
              <ul class="list-inline mb-1 text-muted font-weight-bold">
                {UserService.Instance.user && (
                  <>
                    {this.props.showBody && (
                      <>
                        <li className="list-inline-item">
                          <button
                            class="btn btn-sm btn-link btn-animate text-muted"
                            onClick={linkEvent(this, this.handleSavePostClick)}
                            data-tippy-content={
                              post.saved ? i18n.t('unsave') : i18n.t('save')
                            }
                          >
                            <svg
                              class={`icon icon-inline ${
                                post.saved && 'text-warning'
                              }`}
                            >
                              <use xlinkHref="#icon-star"></use>
                            </svg>
                          </button>
                        </li>
                        <li className="list-inline-item">
                          <Link
                            class="btn btn-sm btn-link btn-animate text-muted"
                            to={`/create_post${this.crossPostParams}`}
                            title={i18n.t('cross_post')}
                          >
                            <svg class="icon icon-inline">
                              <use xlinkHref="#icon-copy"></use>
                            </svg>
                          </Link>
                        </li>
                      </>
                    )}
                    {this.myPost && this.props.showBody && (
                      <>
                        <li className="list-inline-item">
                          <button
                            class="btn btn-sm btn-link btn-animate text-muted"
                            onClick={linkEvent(this, this.handleEditClick)}
                            data-tippy-content={i18n.t('edit')}
                          >
                            <svg class="icon icon-inline">
                              <use xlinkHref="#icon-edit"></use>
                            </svg>
                          </button>
                        </li>
                        <li className="list-inline-item">
                          <button
                            class="btn btn-sm btn-link btn-animate text-muted"
                            onClick={linkEvent(this, this.handleDeleteClick)}
                            data-tippy-content={
                              !post.deleted
                                ? i18n.t('delete')
                                : i18n.t('restore')
                            }
                          >
                            <svg
                              class={`icon icon-inline ${
                                post.deleted && 'text-danger'
                              }`}
                            >
                              <use xlinkHref="#icon-trash"></use>
                            </svg>
                          </button>
                        </li>
                      </>
                    )}

                    {!this.state.showAdvanced && this.props.showBody ? (
                      <li className="list-inline-item">
                        <button
                          class="btn btn-sm btn-link btn-animate text-muted"
                          onClick={linkEvent(this, this.handleShowAdvanced)}
                          data-tippy-content={i18n.t('more')}
                        >
                          <svg class="icon icon-inline">
                            <use xlinkHref="#icon-more-vertical"></use>
                          </svg>
                        </button>
                      </li>
                    ) : (
                      <>
                        {this.props.showBody && post.body && (
                          <li className="list-inline-item">
                            <button
                              class="btn btn-sm btn-link btn-animate text-muted"
                              onClick={linkEvent(this, this.handleViewSource)}
                              data-tippy-content={i18n.t('view_source')}
                            >
                              <svg
                                class={`icon icon-inline ${
                                  this.state.viewSource && 'text-success'
                                }`}
                              >
                                <use xlinkHref="#icon-file-text"></use>
                              </svg>
                            </button>
                          </li>
                        )}
                        {this.canModOnSelf && (
                          <>
                            <li className="list-inline-item">
                              <button
                                class="btn btn-sm btn-link btn-animate text-muted"
                                onClick={linkEvent(this, this.handleModLock)}
                                data-tippy-content={
                                  post.locked
                                    ? i18n.t('unlock')
                                    : i18n.t('lock')
                                }
                              >
                                <svg
                                  class={`icon icon-inline ${
                                    post.locked && 'text-danger'
                                  }`}
                                >
                                  <use xlinkHref="#icon-lock"></use>
                                </svg>
                              </button>
                            </li>
                            <li className="list-inline-item">
                              <button
                                class="btn btn-sm btn-link btn-animate text-muted"
                                onClick={linkEvent(this, this.handleModSticky)}
                                data-tippy-content={
                                  post.stickied
                                    ? i18n.t('unsticky')
                                    : i18n.t('sticky')
                                }
                              >
                                <svg
                                  class={`icon icon-inline ${
                                    post.stickied && 'text-success'
                                  }`}
                                >
                                  <use xlinkHref="#icon-pin"></use>
                                </svg>
                              </button>
                            </li>
                          </>
                        )}
                        {/* Mods can ban from community, and appoint as mods to community */}
                        {(this.canMod || this.canAdmin) && (
                          <li className="list-inline-item">
                            {!post.removed ? (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleModRemoveShow
                                )}
                              >
                                {i18n.t('remove')}
                              </span>
                            ) : (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleModRemoveSubmit
                                )}
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
                        {(this.amCommunityCreator || this.canAdmin) &&
                          this.isMod && (
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
                                      this
                                        .handleCancelShowConfirmTransferCommunity
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
                                    onClick={linkEvent(
                                      this,
                                      this.handleModBanShow
                                    )}
                                  >
                                    {i18n.t('ban_from_site')}
                                  </span>
                                ) : (
                                  <span
                                    class="pointer"
                                    onClick={linkEvent(
                                      this,
                                      this.handleModBanSubmit
                                    )}
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
                                  onClick={linkEvent(
                                    this,
                                    this.handleTransferSite
                                  )}
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
                  </>
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
            </div>
          </div>
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
    setupTippy();
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
    setupTippy();
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
    let post = this.props.post;

    if (post.url) {
      params += `&url=${post.url}`;
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

  handleShowAdvanced(i: PostListing) {
    i.state.showAdvanced = !i.state.showAdvanced;
    i.setState(i.state);
    setupTippy();
  }

  get pointsTippy(): string {
    let points = i18n.t('number_of_points', {
      count: this.state.score,
    });

    let upvotes = i18n.t('number_of_upvotes', {
      count: this.state.upvotes,
    });

    let downvotes = i18n.t('number_of_downvotes', {
      count: this.state.downvotes,
    });

    return `${points} • ${upvotes} • ${downvotes}`;
  }
}
