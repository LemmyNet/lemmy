import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community, Post as PostI, PostResponse, Comment, CommentForm } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

interface State {
  post: PostI;
  commentForm: CommentForm;
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
    commentForm: {
      auth: null,
      content: null,
      post_id: null
    },
    comments: []
  }

  constructor(props, context) {
    super(props, context);

    let postId = Number(this.props.match.params.id);

    this.state = this.emptyState;
    this.state.commentForm.post_id = postId;

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

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            {this.state.post.name}
            {this.commentForm()}
            {this.comments()}
          </div>
        </div>
      </div>
    )
  }

  comments() {
    return (
      <div>
        <h3>Comments</h3>
        {this.state.comments.map(comment => 
          <div>{comment.content}</div>
        )}
      </div>
    )
  }
  
  
  commentForm() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleCreateCommentSubmit)}>
          <h3>Create Comment</h3>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Name</label>
            <div class="col-sm-10">
              <textarea class="form-control" value={this.state.commentForm.content} onInput={linkEvent(this, this.handleCommentContentChange)} required minLength={3} />
            </div>
          </div>
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary">Create</button>
            </div>
          </div>
        </form>
      </div>
    );
  }
  
  handleCreateCommentSubmit(i: Post, event) {
    event.preventDefault();
    WebSocketService.Instance.createComment(i.state.commentForm);
  }

  handleCommentContentChange(i: Post, event) {
    i.state.commentForm.content = event.target.value;
    i.setState(i.state);
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
      this.state.comments = res.comments;
      this.setState(this.state);
    }
  }
}
