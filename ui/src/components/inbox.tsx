import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Comment, SortType, GetRepliesForm, GetRepliesResponse, CommentResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';
import { CommentNodes } from './comment-nodes';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

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
    document.title = `/u/${UserService.Instance.user.username} ${i18n.t('inbox')} - ${WebSocketService.Instance.site.name}`;
  }

  render() {
    let user = UserService.Instance.user;
    return (
      <div class="container">
        <div class="row">
          <div class="col-12">
            <h5 class="mb-0">
              <span><T i18nKey="inbox_for" interpolation={{user: user.username}}>#<Link to={`/u/${user.username}`}>#</Link></T></span>
            </h5>
            {this.state.replies.length > 0 && this.state.unreadType == UnreadType.Unread &&
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                <li className="list-inline-item">
                  <span class="pointer" onClick={this.markAllAsRead}><T i18nKey="mark_all_as_read">#</T></span>
                </li>
              </ul>
            }
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
        <select value={this.state.unreadType} onChange={linkEvent(this, this.handleUnreadTypeChange)} class="custom-select custom-select-sm w-auto">
          <option disabled><T i18nKey="type">#</T></option>
          <option value={UnreadType.Unread}><T i18nKey="unread">#</T></option>
          <option value={UnreadType.All}><T i18nKey="all">#</T></option>
        </select>
        <select value={this.state.sort} onChange={linkEvent(this, this.handleSortChange)} class="custom-select custom-select-sm w-auto ml-2">
          <option disabled><T i18nKey="sort_type">#</T></option>
          <option value={SortType.New}><T i18nKey="new">#</T></option>
          <option value={SortType.TopDay}><T i18nKey="top_day">#</T></option>
          <option value={SortType.TopWeek}><T i18nKey="week">#</T></option>
          <option value={SortType.TopMonth}><T i18nKey="month">#</T></option>
          <option value={SortType.TopYear}><T i18nKey="year">#</T></option>
          <option value={SortType.TopAll}><T i18nKey="all">#</T></option>
        </select>
      </div>
    )

  }

  replies() {
    return (
      <div>
        {this.state.replies.map(reply => 
          <CommentNodes nodes={[{comment: reply}]} noIndent markable />
        )}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && 
          <button class="btn btn-sm btn-secondary mr-1" onClick={linkEvent(this, this.prevPage)}><T i18nKey="prev">#</T></button>
        }
        <button class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.nextPage)}><T i18nKey="next">#</T></button>
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

  markAllAsRead() {
    WebSocketService.Instance.markAllAsRead();
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      return;
    } else if (op == UserOperation.GetReplies || op == UserOperation.MarkAllAsRead) {
      let res: GetRepliesResponse = msg;
      this.state.replies = res.replies;
      this.sendRepliesCount();
      window.scrollTo(0,0);
      this.setState(this.state);
    } else if (op == UserOperation.EditComment) {
      let res: CommentResponse = msg;

      let found = this.state.replies.find(c => c.id == res.comment.id);
      found.content = res.comment.content;
      found.updated = res.comment.updated;
      found.removed = res.comment.removed;
      found.deleted = res.comment.deleted;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      found.score = res.comment.score;

      // If youre in the unread view, just remove it from the list
      if (this.state.unreadType == UnreadType.Unread && res.comment.read) {
        this.state.replies = this.state.replies.filter(r => r.id !== res.comment.id);
      } else {
        let found = this.state.replies.find(c => c.id == res.comment.id);
        found.read = res.comment.read;
      }
      this.sendRepliesCount();

      this.setState(this.state);
    } else if (op == UserOperation.CreateComment) {
      // let res: CommentResponse = msg;
      alert(i18n.t('reply_sent'));
      // this.state.replies.unshift(res.comment); // TODO do this right
      // this.setState(this.state);
    } else if (op == UserOperation.SaveComment) {
      let res: CommentResponse = msg;
      let found = this.state.replies.find(c => c.id == res.comment.id);
      found.saved = res.comment.saved;
      this.setState(this.state);
    } else if (op == UserOperation.CreateCommentLike) {
      let res: CommentResponse = msg;
      let found: Comment = this.state.replies.find(c => c.id === res.comment.id);
      found.score = res.comment.score;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      if (res.comment.my_vote !== null) 
        found.my_vote = res.comment.my_vote;
      this.setState(this.state);
    }
  }

  sendRepliesCount() {
    UserService.Instance.sub.next({user: UserService.Instance.user, unreadCount: this.state.replies.filter(r => !r.read).length});
  }
}

