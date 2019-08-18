import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { WebSocketService, UserService } from '../services';
import { Post, CreatePostLikeForm, PostForm as PostFormI, SavePostForm, CommunityUser, UserView } from '../interfaces';
import { MomentTime } from './moment-time';
import { PostForm } from './post-form';
import { mdToHtml, canMod, isMod, isImage } from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface PostListingState {
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  imageExpanded: boolean;
}

interface PostListingProps {
  post: Post;
  editable?: boolean;
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
    imageExpanded: false
  }

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
      <div>
        {!this.state.showEdit 
          ? this.listing()
          : <PostForm post={this.props.post} onEdit={this.handleEditPost} onCancel={this.handleEditCancel}/>
        }
      </div>
    )
  }

  listing() {
    let post = this.props.post;
    return (
      <div class="listing">
        <div className={`vote-bar mr-1 float-left small text-center ${this.props.viewOnly && 'no-click'}`}>
          <div className={`pointer ${post.my_vote == 1 ? 'text-info' : 'text-muted'}`} onClick={linkEvent(this, this.handlePostLike)}>
            <svg class="pointer icon upvote"><use xlinkHref="#icon-arrow-up"></use></svg>
          </div>
          <div class={`font-weight-bold text-muted`}>{post.score}</div>
          <div className={`pointer ${post.my_vote == -1 ? 'text-danger' : 'text-muted'}`} onClick={linkEvent(this, this.handlePostDisLike)}>
            <svg class="pointer icon downvote"><use xlinkHref="#icon-arrow-down"></use></svg>
          </div>
        </div>
        {post.url && isImage(post.url) &&
          <span title={i18n.t('expand_here')} class="pointer" onClick={linkEvent(this, this.handleImageExpandClick)}><img class="mx-2 mt-1 float-left img-fluid thumbnail rounded" src={post.url} /></span>
        }
        <div className="ml-4">
          <div>
            <h6 className="mb-0 d-inline">
              {post.url ? 
              <a className="text-white" href={post.url} target="_blank" title={post.url}>{post.name}</a> : 
              <Link className="text-white" to={`/post/${post.id}`} title={i18n.t('comments')}>{post.name}</Link>
              }
            </h6>
            {post.url && 
              <small>
                <a className="ml-2 text-muted font-italic" href={post.url} target="_blank" title={post.url}>{(new URL(post.url)).hostname}</a>
              </small>
            }
            { post.url && isImage(post.url) && 
              <>
                { !this.state.imageExpanded
                  ? <span class="text-monospace pointer ml-2 text-muted small" title={i18n.t('expand_here')} onClick={linkEvent(this, this.handleImageExpandClick)}>[+]</span>
                  : 
                  <span>
                    <span class="text-monospace pointer ml-2 text-muted small" onClick={linkEvent(this, this.handleImageExpandClick)}>[-]</span>
                    <div>
                      <span class="pointer" onClick={linkEvent(this, this.handleImageExpandClick)}><img class="img-fluid" src={post.url} /></span>
                    </div>
                  </span>
                }
              </>
            }
            {post.removed &&
              <small className="ml-2 text-muted font-italic"><T i18nKey="removed">#</T></small>
            }
            {post.deleted &&
              <small className="ml-2 text-muted font-italic"><T i18nKey="deleted">#</T></small>
            }
            {post.locked &&
              <small className="ml-2 text-muted font-italic"><T i18nKey="locked">#</T></small>
            }
            {post.nsfw &&
              <small className="ml-2 text-muted font-italic"><T i18nKey="nsfw">#</T></small>
            }
          </div>
        </div>
        <div className="details ml-4 mb-1">
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <span>by </span>
              <Link className="text-info" to={`/u/${post.creator_name}`}>{post.creator_name}</Link>
              {this.isMod && 
                <span className="mx-1 badge badge-light"><T i18nKey="mod">#</T></span>
              }
              {this.isAdmin && 
                <span className="mx-1 badge badge-light"><T i18nKey="admin">#</T></span>
              }
              {this.props.showCommunity && 
                <span>
                  <span> to </span>
                  <Link to={`/c/${post.community_name}`}>{post.community_name}</Link>
                </span>
              }
            </li>
            <li className="list-inline-item">
              <span><MomentTime data={post} /></span>
            </li>
            <li className="list-inline-item">
              <span>(
                <span className="text-info">+{post.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{post.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <Link className="text-muted" to={`/post/${post.id}`}><T i18nKey="number_of_comments" interpolation={{count: post.number_of_comments}}>#</T></Link>
            </li>
          </ul>
          {UserService.Instance.user && this.props.editable &&
            <ul class="list-inline mb-1 text-muted small font-weight-bold"> 
              <li className="list-inline-item mr-2">
                <span class="pointer" onClick={linkEvent(this, this.handleSavePostClick)}>{post.saved ? i18n.t('unsave') : i18n.t('save')}</span>
              </li>
              {this.myPost && 
                <>
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}><T i18nKey="edit">#</T></span>
                  </li>
                  <li className="list-inline-item mr-2">
                    <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>
                      {!post.deleted ? i18n.t('delete') : i18n.t('restore')}
                    </span>
                  </li>
                </>
              }
              {this.canMod &&
                <span>
                  <li className="list-inline-item">
                    {!this.props.post.removed ? 
                    <span class="pointer" onClick={linkEvent(this, this.handleModRemoveShow)}><T i18nKey="remove">#</T></span> :
                    <span class="pointer" onClick={linkEvent(this, this.handleModRemoveSubmit)}><T i18nKey="restore">#</T></span>
                    }
                  </li>
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleModLock)}>{this.props.post.locked ? i18n.t('unlock') : i18n.t('lock')}</span>
                  </li>
                </span>
              }
            </ul>
          }
          {this.state.showRemoveDialog && 
            <form class="form-inline" onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
              <input type="text" class="form-control mr-2" placeholder={i18n.t('reason')} value={this.state.removeReason} onInput={linkEvent(this, this.handleModRemoveReasonChange)} />
              <button type="submit" class="btn btn-secondary"><T i18nKey="remove_post">#</T></button>
            </form>
          }
          {this.props.showBody && this.props.post.body && <div className="md-div" dangerouslySetInnerHTML={mdToHtml(post.body)} />}
        </div>
      </div>
    )
  }

  private get myPost(): boolean {
    return UserService.Instance.user && this.props.post.creator_id == UserService.Instance.user.id;
  }

  get isMod(): boolean {
    return this.props.moderators && isMod(this.props.moderators.map(m => m.user_id), this.props.post.creator_id);
  }

  get isAdmin(): boolean {
    return this.props.admins && isMod(this.props.admins.map(a => a.id), this.props.post.creator_id);
  }

  get canMod(): boolean {

    if (this.props.editable) {
      let adminsThenMods = this.props.admins.map(a => a.id)
      .concat(this.props.moderators.map(m => m.user_id));

      return canMod(UserService.Instance.user, adminsThenMods, this.props.post.creator_id);

    } else return false;
  }

  handlePostLike(i: PostListing) {

    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: (i.props.post.my_vote == 1) ? 0 : 1
    };
    WebSocketService.Instance.likePost(form);
  }

  handlePostDisLike(i: PostListing) {
    let form: CreatePostLikeForm = {
      post_id: i.props.post.id,
      score: (i.props.post.my_vote == -1) ? 0 : -1
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
      auth: null
    };
    WebSocketService.Instance.editPost(deleteForm);
  }

  handleSavePostClick(i: PostListing) {
    let saved = (i.props.post.saved == undefined) ? true : !i.props.post.saved;
    let form: SavePostForm = {
      post_id: i.props.post.id,
      save: saved
    };

    WebSocketService.Instance.savePost(form);
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

  handleImageExpandClick(i: PostListing) {
    i.state.imageExpanded = !i.state.imageExpanded;
    i.setState(i.state);
  }
}

