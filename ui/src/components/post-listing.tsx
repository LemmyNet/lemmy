import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { WebSocketService, UserService } from '../services';
import { Post, CreatePostLikeForm, PostForm as PostFormI } from '../interfaces';
import { MomentTime } from './moment-time';
import { PostForm } from './post-form';
import { mdToHtml } from '../utils';

interface PostListingState {
  showEdit: boolean;
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
            <h4 className="d-inline"><a className="text-white" href={post.url}>{post.name}</a></h4>
            <small><a className="ml-2 text-muted font-italic" href={post.url}>{(new URL(post.url)).hostname}</a></small>
            { !this.state.iframeExpanded
              ? <span class="pointer ml-2 text-muted small" title="Expand here" onClick={linkEvent(this, this.handleIframeExpandClick)}>+</span>
              : 
              <span>
                <span class="pointer ml-2 text-muted small" onClick={linkEvent(this, this.handleIframeExpandClick)}>-</span>
                <div class="embed-responsive embed-responsive-1by1">
                  <iframe scrolling="yes" class="embed-responsive-item" src={post.url}></iframe>
                </div>
              </span>
            }
          </div> 
            : <h4 className="mb-0"><Link className="text-white" to={`/post/${post.id}`}>{post.name}</Link></h4>
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
          {this.myPost && 
            <ul class="list-inline mb-1 text-muted small font-weight-bold"> 
              <li className="list-inline-item">
                <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
              </li>
              <li className="list-inline-item">
                <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span>
              </li>
            </ul>
          }
          {this.props.showBody && this.props.post.body && <div className="md-div" dangerouslySetInnerHTML={mdToHtml(post.body)} />}
        </div>
      </div>
    )
  }

  private get myPost(): boolean {
    return this.props.editable && UserService.Instance.loggedIn && this.props.post.creator_id == UserService.Instance.user.id;
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
      auth: null
    };
    WebSocketService.Instance.editPost(deleteForm);
  }

  handleIframeExpandClick(i: PostListing) {
    i.state.iframeExpanded = !i.state.iframeExpanded;
    i.setState(i.state);
  }
}

