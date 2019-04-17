import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { WebSocketService, UserService } from '../services';
import { Post, CreatePostLikeForm, PostForm as PostFormI } from '../interfaces';
import { MomentTime } from './moment-time';
import { PostForm } from './post-form';
import { mdToHtml } from '../utils';

interface PostListingState {
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  iframeExpanded: boolean;
}

interface PostListingProps {
  post: Post;
  editable?: boolean;
  showCommunity?: boolean;
  showBody?: boolean;
  viewOnly?: boolean;
}

export class PostListing extends Component<PostListingProps, PostListingState> {

  private emptyState: PostListingState = {
    showEdit: false,
    showRemoveDialog: false,
    removeReason: null,
    iframeExpanded: false
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
        <div className={`float-left small text-center ${this.props.viewOnly && 'no-click'}`}>
          <div className={`pointer upvote ${post.my_vote == 1 ? 'text-info' : 'text-muted'}`} onClick={linkEvent(this, this.handlePostLike)}>▲</div>
          <div>{post.score}</div>
          <div className={`pointer downvote ${post.my_vote == -1 && 'text-danger'}`} onClick={linkEvent(this, this.handlePostDisLike)}>▼</div>
        </div>
        <div className="ml-4">
          {post.url 
            ? <div className="mb-0">
            <h4 className="d-inline"><a className="text-white" href={post.url} title={post.url}>{post.name}</a>
            {post.removed &&
              <small className="ml-2 text-muted font-italic">removed</small>
            }
            {post.locked &&
              <small className="ml-2 text-muted font-italic">locked</small>
            }
          </h4>
          <small><a className="ml-2 text-muted font-italic" href={post.url} title={post.url}>{(new URL(post.url)).hostname}</a></small>
          { !this.state.iframeExpanded
            ? <span class="badge badge-light pointer ml-2 text-muted small" title="Expand here" onClick={linkEvent(this, this.handleIframeExpandClick)}>+</span>
            : 
            <span>
              <span class="pointer ml-2 badge badge-light text-muted small" onClick={linkEvent(this, this.handleIframeExpandClick)}>-</span>
              <div class="embed-responsive embed-responsive-1by1">
                <iframe scrolling="yes" class="embed-responsive-item" src={post.url}></iframe>
              </div>
            </span>
          }
        </div> 
          : <h4 className="mb-0"><Link className="text-white" to={`/post/${post.id}`}>{post.name}</Link>
          {post.removed &&
            <small className="ml-2 text-muted font-italic">removed</small>
          }
          {post.locked &&
            <small className="ml-2 text-muted font-italic">locked</small>
          }
        </h4>
          }
        </div>
        <div className="details ml-4 mb-1">
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <span>by </span>
              <Link className="text-info" to={`/user/${post.creator_id}`}>{post.creator_name}</Link>
              {this.props.showCommunity && 
                <span>
                  <span> to </span>
                  <Link to={`/community/${post.community_id}`}>{post.community_name}</Link>
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
              <Link className="text-muted" to={`/post/${post.id}`}>{post.number_of_comments} Comments</Link>
            </li>
          </ul>
          {this.props.editable &&
            <ul class="list-inline mb-1 text-muted small font-weight-bold"> 
              {this.myPost && 
                <span>
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
                  </li>
                  <li className="list-inline-item mr-2">
                    <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span>
                  </li>
                </span>
              }
              {this.props.post.am_mod &&
                <span>
                  <li className="list-inline-item">
                    {!this.props.post.removed ? 
                    <span class="pointer" onClick={linkEvent(this, this.handleModRemoveShow)}>remove</span> :
                    <span class="pointer" onClick={linkEvent(this, this.handleModRemoveSubmit)}>restore</span>
                    }
                  </li>
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleModLock)}>{this.props.post.locked ? 'unlock' : 'lock'}</span>
                  </li>
                </span>
              }
            </ul>
          }
          {this.state.showRemoveDialog && 
            <form class="form-inline" onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
              <input type="text" class="form-control mr-2" placeholder="Reason" value={this.state.removeReason} onInput={linkEvent(this, this.handleModRemoveReasonChange)} />
              <button type="submit" class="btn btn-secondary">Remove Post</button>
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
      body: '',
      community_id: i.props.post.community_id,
      name: "deleted",
      url: '',
      edit_id: i.props.post.id,
      creator_id: i.props.post.creator_id,
      auth: null
    };
    WebSocketService.Instance.editPost(deleteForm);
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
      locked: !i.props.post.locked,
      auth: null,
    };
    WebSocketService.Instance.editPost(form);
  }

  handleIframeExpandClick(i: PostListing) {
    i.state.iframeExpanded = !i.state.iframeExpanded;
    i.setState(i.state);
  }
}

