import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Comment, SortType, GetRepliesForm, GetRepliesResponse, CommentResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';
import { CommentNodes } from './comment-nodes';

enum UnreadType {
  Unread, All
}

interface InboxState {
  unreadType: UnreadType;
  replies: Array<Comment>;
  sort: SortType;
  page: number;
}

export class Inbox extends Component<any, InboxState> {

  private subscription: Subscription;
  private emptyState: InboxState = {
    unreadType: UnreadType.Unread,
    replies: [],
    sort: SortType.New,
    page: 1,
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    this.refetch();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = `/u/${UserService.Instance.user.username} Inbox - Lemmy`;
  }

  render() {
    let user = UserService.Instance.user;
    return (
      <div class="container">
        <div class="row">
          <div class="col-12">
            <h5>Inbox for <Link to={`/user/${user.id}`}>{user.username}</Link></h5>
            {this.selects()}
            {this.replies()}
            {this.paginator()}
          </div>
        </div>
      </div>
    )
  }

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.unreadType} onChange={linkEvent(this, this.handleUnreadTypeChange)} class="custom-select w-auto">
          <option disabled>Type</option>
          <option value={UnreadType.Unread}>Unread</option>
          <option value={UnreadType.All}>All</option>
        </select>
        <select value={this.state.sort} onChange={linkEvent(this, this.handleSortChange)} class="custom-select w-auto ml-2">
          <option disabled>Sort Type</option>
          <option value={SortType.New}>New</option>
          <option value={SortType.TopDay}>Top Day</option>
          <option value={SortType.TopWeek}>Week</option>
          <option value={SortType.TopMonth}>Month</option>
          <option value={SortType.TopYear}>Year</option>
          <option value={SortType.TopAll}>All</option>
        </select>
      </div>
    )

  }

  replies() {
    return (
      <div>
        {this.state.replies.map(reply => 
          <CommentNodes nodes={[{comment: reply}]} noIndent viewOnly markable />
        )}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && 
          <button class="btn btn-sm btn-secondary mr-1" onClick={linkEvent(this, this.prevPage)}>Prev</button>
        }
        <button class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.nextPage)}>Next</button>
      </div>
    );
  }

  nextPage(i: Inbox) { 
    i.state.page++;
    i.setState(i.state);
    i.refetch();
  }

  prevPage(i: Inbox) { 
    i.state.page--;
    i.setState(i.state);
    i.refetch();
  }

  handleUnreadTypeChange(i: Inbox, event: any) {
    i.state.unreadType = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.refetch();
  }

  refetch() {
    let form: GetRepliesForm = {
      sort: SortType[this.state.sort],
      unread_only: (this.state.unreadType == UnreadType.Unread),
      page: this.state.page,
      limit: 9999,
    };
    WebSocketService.Instance.getReplies(form);
  }

  handleSortChange(i: Inbox, event: any) {
    i.state.sort = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.refetch();
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetReplies) {
      let res: GetRepliesResponse = msg;
      this.state.replies = res.replies;
      this.sendRepliesCount();
      this.setState(this.state);
    } else if (op == UserOperation.EditComment) {
      let res: CommentResponse = msg;

      // If youre in the unread view, just remove it from the list
      if (this.state.unreadType == UnreadType.Unread && res.comment.read) {
        this.state.replies = this.state.replies.filter(r => r.id !== res.comment.id);
      } else {
        let found = this.state.replies.find(c => c.id == res.comment.id);
        found.read = res.comment.read;
      }

      this.sendRepliesCount();
      this.setState(this.state);
    }
  }

  sendRepliesCount() {
    UserService.Instance.sub.next({user: UserService.Instance.user, unreadCount: this.state.replies.filter(r => !r.read).length});
  }
}

