import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community, Post as PostI, GetPostResponse, PostResponse, Comment, CommentForm as CommentFormI, CommentResponse, CommentLikeForm, CreateCommentLikeResponse, CommentSortType, CreatePostLikeResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp, hotRank,mdToHtml } from '../utils';
import { MomentTime } from './moment-time';
import { PostListing } from './post-listing';
import * as autosize from 'autosize';

interface CommentNodeI {
  comment: Comment;
  children?: Array<CommentNodeI>;
};

interface State {
  post: PostI;
  comments: Array<Comment>;
  commentSort: CommentSortType;
}

export class Post extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    post: null,
    comments: [],
    commentSort: CommentSortType.Hot
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;

    let postId = Number(this.props.match.params.id);

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getPost(postId);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  render() {
    return (
      <div class="container">
        {this.state.post && 
          <div class="row">
            <div class="col-12 col-sm-8 col-lg-7 mb-3">
              <PostListing post={this.state.post} showBody showCommunity editable />
              <div className="mb-2" />
              <CommentForm postId={this.state.post.id} />
              {this.sortRadios()}
              {this.commentsTree()}
            </div>
            <div class="col-12 col-sm-4 col-lg-3 mb-3">
              {this.state.comments.length > 0 && this.newComments()}
            </div>
            <div class="col-12 col-sm-12 col-lg-2">
              {this.sidebar()}
            </div>
          </div>
        }
      </div>
    )
  }

  sortRadios() {
    return (
      <div class="btn-group btn-group-toggle mb-3">
        <label className={`btn btn-sm btn-secondary ${this.state.commentSort === CommentSortType.Hot && 'active'}`}>Hot
          <input type="radio" value={CommentSortType.Hot}
          checked={this.state.commentSort === CommentSortType.Hot} 
          onChange={linkEvent(this, this.handleCommentSortChange)}  />
        </label>
        <label className={`btn btn-sm btn-secondary ${this.state.commentSort === CommentSortType.Top && 'active'}`}>Top
          <input type="radio" value={CommentSortType.Top}
          checked={this.state.commentSort === CommentSortType.Top} 
          onChange={linkEvent(this, this.handleCommentSortChange)}  />
        </label>
        <label className={`btn btn-sm btn-secondary ${this.state.commentSort === CommentSortType.New && 'active'}`}>New
          <input type="radio" value={CommentSortType.New}
          checked={this.state.commentSort === CommentSortType.New} 
          onChange={linkEvent(this, this.handleCommentSortChange)}  />
        </label>
      </div>
    )
  }

  newComments() {
    return (
      <div class="sticky-top">
        <h5>New Comments</h5>
        {this.state.comments.map(comment => 
          <CommentNodes nodes={[{comment: comment}]} noIndent />
        )}
      </div>
    )
  }

  sidebar() {
    return ( 
      <div class="sticky-top">
        <h5>Sidebar</h5>
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p>
      </div>
    );
  }
  
  handleCommentSortChange(i: Post, event) {
    i.state.commentSort = Number(event.target.value);
    i.setState(i.state);
  }

  private buildCommentsTree(): Array<CommentNodeI> {
    let map = new Map<number, CommentNodeI>();
    for (let comment of this.state.comments) {
      let node: CommentNodeI = {
        comment: comment,
        children: []
      };
      map.set(comment.id, { ...node });
    }
    let tree: Array<CommentNodeI> = [];
    for (let comment of this.state.comments) {
      if( comment.parent_id ) {
        map.get(comment.parent_id).children.push(map.get(comment.id));
      } 
      else {
        tree.push(map.get(comment.id));
      }
    }

    this.sortTree(tree);

    return tree;
  }

  sortTree(tree: Array<CommentNodeI>) {

    if (this.state.commentSort == CommentSortType.Top) {
      tree.sort((a, b) => b.comment.score - a.comment.score);
    } else if (this.state.commentSort == CommentSortType.New) {
      tree.sort((a, b) => b.comment.published.localeCompare(a.comment.published));
    } else if (this.state.commentSort == CommentSortType.Hot) {
      tree.sort((a, b) => hotRank(b.comment) - hotRank(a.comment));
    }

    for (let node of tree) {
      this.sortTree(node.children);
    }

  }

  commentsTree() {
    let nodes = this.buildCommentsTree();
    return (
      <div className="">
        <CommentNodes nodes={nodes} />
      </div>
    );
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetPost) {
      let res: GetPostResponse = msg;
      this.state.post = res.post;
      this.state.comments = res.comments;
      this.setState(this.state);
    } else if (op == UserOperation.CreateComment) {
      let res: CommentResponse = msg;
      this.state.comments.unshift(res.comment);
      this.setState(this.state);
    } else if (op == UserOperation.EditComment) {
      let res: CommentResponse = msg;
      let found = this.state.comments.find(c => c.id == res.comment.id);
      found.content = res.comment.content;
      found.updated = res.comment.updated;
      this.setState(this.state);
    }
    else if (op == UserOperation.CreateCommentLike) {
      let res: CreateCommentLikeResponse = msg;
      let found: Comment = this.state.comments.find(c => c.id === res.comment.id);
      found.score = res.comment.score;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      if (res.comment.my_vote !== null) 
        found.my_vote = res.comment.my_vote;
      this.setState(this.state);
    } else if (op == UserOperation.CreatePostLike) {
      let res: CreatePostLikeResponse = msg;
      this.state.post.my_vote = res.post.my_vote;
      this.state.post.score = res.post.score;
      this.state.post.upvotes = res.post.upvotes;
      this.state.post.downvotes = res.post.downvotes;
      this.setState(this.state);
    } else if (op == UserOperation.EditPost) {
      let res: PostResponse = msg;
      this.state.post = res.post;
      this.setState(this.state);
    }

  }
}

interface CommentNodesState {
}

interface CommentNodesProps {
  nodes: Array<CommentNodeI>;
  noIndent?: boolean;
}

export class CommentNodes extends Component<CommentNodesProps, CommentNodesState> {

  constructor(props, context) {
    super(props, context);
  }

  render() {
    return (
      <div className="comments">
        {this.props.nodes.map(node =>
          <CommentNode node={node} noIndent={this.props.noIndent} />
        )}
      </div>
    )
  }
}


interface CommentNodeState {
  showReply: boolean;
  showEdit: boolean;
}

interface CommentNodeProps {
  node: CommentNodeI;
  noIndent?: boolean;
}

export class CommentNode extends Component<CommentNodeProps, CommentNodeState> {

  private emptyState: CommentNodeState = {
    showReply: false,
    showEdit: false
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
    this.handleCommentLike = this.handleCommentLike.bind(this);
    this.handleCommentDisLike = this.handleCommentDisLike.bind(this);
  }

  render() {
    let node = this.props.node;
    return (
      <div className={`comment ${node.comment.parent_id  && !this.props.noIndent ? 'ml-4' : ''}`}>
        <div className="float-left small text-center">
          <div className={`pointer upvote ${node.comment.my_vote == 1 ? 'text-info' : 'text-muted'}`} onClick={linkEvent(node, this.handleCommentLike)}>▲</div>
          <div>{node.comment.score}</div>
          <div className={`pointer downvote ${node.comment.my_vote == -1 && 'text-danger'}`} onClick={linkEvent(node, this.handleCommentDisLike)}>▼</div>
        </div>
        <div className="details ml-4">
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <Link to={`/user/${node.comment.creator_id}`}>{node.comment.creator_name}</Link>
            </li>
            <li className="list-inline-item">
              <span>(
                <span className="text-info">+{node.comment.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{node.comment.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <span><MomentTime data={node.comment} /></span>
            </li>
          </ul>
          {this.state.showEdit && <CommentForm node={node} edit onReplyCancel={this.handleReplyCancel} />}
          {!this.state.showEdit &&
            <div>
              <div className="md-div" dangerouslySetInnerHTML={mdToHtml(node.comment.content)} />
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                <li className="list-inline-item">
                  <span class="pointer" onClick={linkEvent(this, this.handleReplyClick)}>reply</span>
                </li>
                {this.myComment && 
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
                  </li>
                }
                {this.myComment &&
                  <li className="list-inline-item">
                    <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span>
                  </li>
                }
                <li className="list-inline-item">
                  <a className="text-muted" href="test">link</a>
                </li>
              </ul>
            </div>
          }
        </div>
        {this.state.showReply && <CommentForm node={node} onReplyCancel={this.handleReplyCancel} />}
        {this.props.node.children && <CommentNodes nodes={this.props.node.children} />}
      </div>
    )
  }

  private get myComment(): boolean {
    return UserService.Instance.loggedIn && this.props.node.comment.creator_id == UserService.Instance.user.id;
  }

  handleReplyClick(i: CommentNode, event) {
    i.state.showReply = true;
    i.setState(i.state);
  }

  handleEditClick(i: CommentNode, event) {
    i.state.showEdit = true;
    i.setState(i.state);
  }

  handleDeleteClick(i: CommentNode, event) {
    let deleteForm: CommentFormI = {
      content: "*deleted*",
      edit_id: i.props.node.comment.id,
      post_id: i.props.node.comment.post_id,
      parent_id: i.props.node.comment.parent_id,
      auth: null
    };
    WebSocketService.Instance.editComment(deleteForm);
  }

  handleReplyCancel(): any {
    this.state.showReply = false;
    this.state.showEdit = false;
    this.setState(this.state);
  }


  handleCommentLike(i: CommentNodeI, event) {

    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: (i.comment.my_vote == 1) ? 0 : 1
    };
    WebSocketService.Instance.likeComment(form);
  }

  handleCommentDisLike(i: CommentNodeI, event) {
    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: (i.comment.my_vote == -1) ? 0 : -1
    };
    WebSocketService.Instance.likeComment(form);
  }
}

interface CommentFormProps {
  postId?: number;
  node?: CommentNodeI;
  onReplyCancel?();
  edit?: boolean;
}

interface CommentFormState {
  commentForm: CommentFormI;
  buttonTitle: string;
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {

  private emptyState: CommentFormState = {
    commentForm: {
      auth: null,
      content: null,
      post_id: this.props.node ? this.props.node.comment.post_id : this.props.postId
    },
    buttonTitle: !this.props.node ? "Post" : this.props.edit ? "Edit" : "Reply"
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;

    if (this.props.node) {
      if (this.props.edit) {
        this.state.commentForm.edit_id = this.props.node.comment.id;
        this.state.commentForm.parent_id = this.props.node.comment.parent_id;
        this.state.commentForm.content = this.props.node.comment.content;
      } else {
        // A reply gets a new parent id
        this.state.commentForm.parent_id = this.props.node.comment.id;
      }
    }  
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  render() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleCommentSubmit)}>
          <div class="form-group row">
            <div class="col-sm-12">
              <textarea class="form-control" value={this.state.commentForm.content} onInput={linkEvent(this, this.handleCommentContentChange)} placeholder="Comment here" required />
            </div>
          </div>
          <div class="row">
            <div class="col-sm-12">
              <button type="submit" class="btn btn-sm btn-secondary mr-2">{this.state.buttonTitle}</button>
              {this.props.node && <button type="button" class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.handleReplyCancel)}>Cancel</button>}
            </div>
          </div>
        </form>
      </div>
    );
  }

  handleCommentSubmit(i: CommentForm, event) {
    if (i.props.edit) {
      WebSocketService.Instance.editComment(i.state.commentForm);
    } else {
      WebSocketService.Instance.createComment(i.state.commentForm);
    }

    i.state.commentForm.content = undefined;
    i.setState(i.state);
    event.target.reset();
    if (i.props.node) {
      i.props.onReplyCancel();
    }
  }

  handleCommentContentChange(i: CommentForm, event) {
    i.state.commentForm.content = event.target.value;
    i.setState(i.state);
  }

  handleReplyCancel(i: CommentForm, event) {
    i.props.onReplyCancel();
  }
}
