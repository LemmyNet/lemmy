import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community, Post as PostI, PostResponse, Comment, CommentForm as CommentFormI, CommentResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

interface CommentNodeI {
  comment: Comment;
  children?: Array<CommentNodeI>;
  showReply?: boolean;
};

interface State {
  post: PostI;
  comments: Array<Comment>;
}

export class Post extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    post: {
      name: null,
      attributed_to: null,
      community_id: null,
      id: null,
      published: null,
    },
    comments: []
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;

    this.state.post.id = Number(this.props.match.params.id);

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getPost(this.state.post.id);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-sm-8 col-lg-7 mb-3">
            {this.postHeader()}
            <CommentForm postId={this.state.post.id} />
            {this.commentsTree()}
          </div>
          <div class="col-12 col-sm-4 col-lg-3 mb-3">
            {this.newComments()}
          </div>
          <div class="col-12 col-sm-12 col-lg-2">
            {this.sidebar()}
          </div>
        </div>
      </div>
    )
  }

  postHeader() {
    let title = this.state.post.url 
      ? <h5><a href={this.state.post.url}>{this.state.post.name}</a></h5> 
      : <h5>{this.state.post.name}</h5>;
    return (
      <div>
        {title}
        via {this.state.post.attributed_to} X hours ago
          {this.state.post.body}
        </div>
    )
  }

  newComments() {
    return (
      <div class="sticky-top">
        <h4>New Comments</h4>
        {this.state.comments.map(comment => 
          <CommentNodes nodes={[{comment: comment}]} />
        )}
      </div>
    )
  }

  sidebar() {
    return ( 
      <div class="sticky-top">
        <h4>Sidebar</h4>
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p>
      </div>
    );
  }

  // buildCommentsTree(): Array<CommentNodeI> {
  buildCommentsTree(): any {
    let tree: Array<CommentNodeI> = this.createCommentsTree(this.state.comments);
    console.log(tree); // TODO this is redoing every time and it shouldn't
    return tree;
  }

  private createCommentsTree(comments: Array<Comment>): Array<CommentNodeI> {
    let hashTable = {};
    for (let comment of comments) {
      let node: CommentNodeI = {
        comment: comment
      };
      hashTable[comment.id] = { ...node, children : [] };
    }
    let tree: Array<CommentNodeI> = [];
    for (let comment of comments) {
      if( comment.parent_id ) hashTable[comment.parent_id].children.push(hashTable[comment.id]);
      else tree.push(hashTable[comment.id]);
    }
    return tree;
  }

  commentsTree() {
    let nodes = this.buildCommentsTree();
    return (
      <div className="sticky-top">
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
      let res: PostResponse = msg;
      this.state.post = res.post;
      this.state.comments = res.comments.reverse();
      this.setState(this.state);
    } else if (op == UserOperation.CreateComment) {
      let res: CommentResponse = msg;
      this.state.comments.unshift(res.comment);
      this.setState(this.state);
    }

  }
}

interface CommentNodesState {
}

interface CommentNodesProps {
  nodes: Array<CommentNodeI>;
}

export class CommentNodes extends Component<CommentNodesProps, CommentNodesState> {

  constructor(props, context) {
    super(props, context);
    this.handleReplyClick = this.handleReplyClick.bind(this);
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
  }

  render() {
    return (
      <div className="comments">
        {this.props.nodes.map(node =>
          <div className="comment ml-2">
            <div className="float-left small font-weight-light">
              <div className="pointer">▲</div>
              <div className="pointer">▼</div>
            </div>
            <div className="details ml-4">
            <ul class="list-inline mb-0 text-muted small">
              <li className="list-inline-item">
                <a href={node.comment.attributed_to}>{node.comment.attributed_to}</a>
              </li>
              <li className="list-inline-item">
                <span>(
                  <span className="text-info"> 1300</span>
                  <span> | </span>
                  <span className="text-danger">-29</span>
                  <span> ) points</span>
                </span>
              </li>
              <li className="list-inline-item">
                <span>X hours ago</span>
              </li>
            </ul>
            <p className="mb-0">{node.comment.content}</p>
            <ul class="list-inline mb-1 text-muted small font-weight-bold">
              <li className="list-inline-item">
                <span class="pointer" onClick={linkEvent(node, this.handleReplyClick)}>reply</span>
              </li>
              <li className="list-inline-item">
                <a className="text-muted" href="test">link</a>
              </li>
            </ul>
          </div>
          {node.showReply && <CommentForm node={node} onReplyCancel={this.handleReplyCancel} />}
          {node.children && <CommentNodes nodes={node.children}/>}
          </div>
        )}
      </div>
    )
  }

  handleReplyClick(i: CommentNodeI, event) {
    i.showReply = true;
    this.setState(this.state);
  }

  handleReplyCancel(i: CommentNodeI): any {
    i.showReply = false;
    this.setState(this.state);
  }
}

interface CommentFormProps {
  postId?: number;
  node?: CommentNodeI;
  onReplyCancel?(node: CommentNodeI);
}

interface CommentFormState {
  commentForm: CommentFormI;
  topReply: boolean;
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {

  private emptyState: CommentFormState = {
    commentForm: {
      auth: null,
      content: null,
      post_id: null,
      parent_id: null
    },
    topReply: true
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;
    if (this.props.node) {
      this.state.topReply = false;
      this.state.commentForm.post_id = this.props.node.comment.post_id;
      this.state.commentForm.parent_id = this.props.node.comment.id;
    } else {
      this.state.commentForm.post_id = this.props.postId;
    }

    console.log(this.state);

    this.handleReplyCancel = this.handleReplyCancel.bind(this);
  }

  render() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleCreateCommentSubmit)}>
          <div class="form-group row">
            <div class="col-sm-12">
              <textarea class="form-control" value={this.state.commentForm.content} onInput={linkEvent(this, this.handleCommentContentChange)} placeholder="Comment here" required />
            </div>
          </div>
          <div class="row">
            <div class="col-sm-12">
              <button type="submit" class="btn btn-secondary mr-2">Post</button>
              {!this.state.topReply && <button type="button" class="btn btn-secondary" onClick={this.handleReplyCancel}>Cancel</button>}
            </div>
          </div>
        </form>
      </div>
    );
  }

  handleCreateCommentSubmit(i: CommentForm, event) {
    WebSocketService.Instance.createComment(i.state.commentForm);
    i.state.commentForm.content = undefined;
    i.setState(i.state);
    event.target.reset();
  }

  handleCommentContentChange(i: CommentForm, event) {
    // TODO don't use setState, it triggers a re-render
    i.state.commentForm.content = event.target.value;
  }

  handleReplyCancel(event) {
    this.props.onReplyCancel(this.props.node);
  }
}
