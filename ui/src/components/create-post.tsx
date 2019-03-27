import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { PostForm, Post, PostResponse, UserOperation, Community, ListCommunitiesResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

interface State {
  postForm: PostForm;
  communities: Array<Community>;
}


export class CreatePost extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    postForm: {
      name: null,
      auth: null,
      community_id: null
    },
    communities: []
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.listCommunities();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            {this.postForm()}
          </div>
        </div>
      </div>
    )
  }

  postForm() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handlePostSubmit)}>
          <h3>Create a Post</h3>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">URL</label>
            <div class="col-sm-10">
              <input type="url" class="form-control" value={this.state.postForm.url} onInput={linkEvent(this, this.handlePostUrlChange)} />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Title</label>
            <div class="col-sm-10">
              <textarea value={this.state.postForm.name} onInput={linkEvent(this, this.handlePostNameChange)} class="form-control" required rows="3" />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Body</label>
            <div class="col-sm-10">
              <textarea value={this.state.postForm.body} onInput={linkEvent(this, this.handlePostBodyChange)} class="form-control" rows="6" />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Forum</label>
            <div class="col-sm-10">
              <select class="form-control" value={this.state.postForm.community_id} onInput={linkEvent(this, this.handlePostCommunityChange)}>
                {this.state.communities.map(community =>
                  <option value={community.id}>{community.name}</option>
                )}
              </select>
            </div>
          </div>
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary">Create Post</button>
            </div>
          </div>
        </form>
      </div>
    );
  }

  handlePostSubmit(i: CreatePost, event) {
    event.preventDefault();
    WebSocketService.Instance.createPost(i.state.postForm);
  }

  handlePostUrlChange(i: CreatePost, event) {
    i.state.postForm.url = event.target.value;
  }

  handlePostNameChange(i: CreatePost, event) {
    i.state.postForm.name = event.target.value;
  }

  handlePostBodyChange(i: CreatePost, event) {
    i.state.postForm.body = event.target.value;
  }

  handlePostCommunityChange(i: CreatePost, event) {
    i.state.postForm.community_id = Number(event.target.value);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.communities = res.communities;
      this.state.postForm.community_id = res.communities[0].id; // TODO set it to the default community
      this.setState(this.state);
    } else if (op == UserOperation.CreatePost) {
      let res: PostResponse = msg;
      this.props.history.push(`/post/${res.post.id}`);
    }
  }

}
