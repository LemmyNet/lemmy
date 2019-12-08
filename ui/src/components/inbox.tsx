import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Comment,
  SortType,
  GetRepliesForm,
  GetRepliesResponse,
  GetUserMentionsForm,
  GetUserMentionsResponse,
  UserMentionResponse,
  CommentResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';
import { CommentNodes } from './comment-nodes';
import { SortSelect } from './sort-select';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

enum UnreadOrAll {
  Unread,
  All,
}

enum UnreadType {
  Both,
  Replies,
  Mentions,
}

interface InboxState {
  unreadOrAll: UnreadOrAll;
  unreadType: UnreadType;
  replies: Array<Comment>;
  mentions: Array<Comment>;
  sort: SortType;
  page: number;
}

export class Inbox extends Component<any, InboxState> {
  private subscription: Subscription;
  private emptyState: InboxState = {
    unreadOrAll: UnreadOrAll.Unread,
    unreadType: UnreadType.Both,
    replies: [],
    mentions: [],
    sort: SortType.New,
    page: 1,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleSortChange = this.handleSortChange.bind(this);

    this.subscription = WebSocketService.Instance.subject
      .pipe(
        retryWhen(errors =>
          errors.pipe(
            delay(3000),
            take(10)
          )
        )
      )
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    this.refetch();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = `/u/${UserService.Instance.user.username} ${i18n.t(
      'inbox'
    )} - ${WebSocketService.Instance.site.name}`;
  }

  render() {
    let user = UserService.Instance.user;
    return (
      <div class="container">
        <div class="row">
          <div class="col-12">
            <h5 class="mb-0">
              <T
                class="d-inline"
                i18nKey="inbox_for"
                interpolation={{ user: user.username }}
              >
                #<Link to={`/u/${user.username}`}>#</Link>
              </T>
              <small>
                <a
                  href={`/feeds/inbox/${UserService.Instance.auth}.xml`}
                  target="_blank"
                >
                  <svg class="icon mx-2 text-muted small">
                    <use xlinkHref="#icon-rss">#</use>
                  </svg>
                </a>
              </small>
            </h5>
            {this.state.replies.length + this.state.mentions.length > 0 &&
              this.state.unreadOrAll == UnreadOrAll.Unread && (
                <ul class="list-inline mb-1 text-muted small font-weight-bold">
                  <li className="list-inline-item">
                    <span class="pointer" onClick={this.markAllAsRead}>
                      <T i18nKey="mark_all_as_read">#</T>
                    </span>
                  </li>
                </ul>
              )}
            {this.selects()}
            {this.state.unreadType == UnreadType.Both && this.both()}
            {this.state.unreadType == UnreadType.Replies && this.replies()}
            {this.state.unreadType == UnreadType.Mentions && this.mentions()}
            {this.paginator()}
          </div>
        </div>
      </div>
    );
  }

  selects() {
    return (
      <div className="mb-2">
        <select
          value={this.state.unreadOrAll}
          onChange={linkEvent(this, this.handleUnreadOrAllChange)}
          class="custom-select custom-select-sm w-auto mr-2"
        >
          <option disabled>
            <T i18nKey="type">#</T>
          </option>
          <option value={UnreadOrAll.Unread}>
            <T i18nKey="unread">#</T>
          </option>
          <option value={UnreadOrAll.All}>
            <T i18nKey="all">#</T>
          </option>
        </select>
        <select
          value={this.state.unreadType}
          onChange={linkEvent(this, this.handleUnreadTypeChange)}
          class="custom-select custom-select-sm w-auto mr-2"
        >
          <option disabled>
            <T i18nKey="type">#</T>
          </option>
          <option value={UnreadType.Both}>
            <T i18nKey="both">#</T>
          </option>
          <option value={UnreadType.Replies}>
            <T i18nKey="replies">#</T>
          </option>
          <option value={UnreadType.Mentions}>
            <T i18nKey="mentions">#</T>
          </option>
        </select>
        <SortSelect
          sort={this.state.sort}
          onChange={this.handleSortChange}
          hideHot
        />
      </div>
    );
  }

  both() {
    let combined: Array<{
      type_: string;
      data: Comment;
    }> = [];
    let replies = this.state.replies.map(e => {
      return { type_: 'replies', data: e };
    });
    let mentions = this.state.mentions.map(e => {
      return { type_: 'mentions', data: e };
    });

    combined.push(...replies);
    combined.push(...mentions);

    // Sort it
    if (this.state.sort == SortType.New) {
      combined.sort((a, b) => b.data.published.localeCompare(a.data.published));
    } else {
      combined.sort((a, b) => b.data.score - a.data.score);
    }

    return (
      <div>
        {combined.map(i => (
          <CommentNodes nodes={[{ comment: i.data }]} noIndent markable />
        ))}
      </div>
    );
  }

  replies() {
    return (
      <div>
        {this.state.replies.map(reply => (
          <CommentNodes nodes={[{ comment: reply }]} noIndent markable />
        ))}
      </div>
    );
  }

  mentions() {
    return (
      <div>
        {this.state.mentions.map(mention => (
          <CommentNodes nodes={[{ comment: mention }]} noIndent markable />
        ))}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && (
          <button
            class="btn btn-sm btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            <T i18nKey="prev">#</T>
          </button>
        )}
        <button
          class="btn btn-sm btn-secondary"
          onClick={linkEvent(this, this.nextPage)}
        >
          <T i18nKey="next">#</T>
        </button>
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

  handleUnreadOrAllChange(i: Inbox, event: any) {
    i.state.unreadOrAll = Number(event.target.value);
    i.state.page = 1;
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
    let repliesForm: GetRepliesForm = {
      sort: SortType[this.state.sort],
      unread_only: this.state.unreadOrAll == UnreadOrAll.Unread,
      page: this.state.page,
      limit: 9999,
    };
    WebSocketService.Instance.getReplies(repliesForm);

    let userMentionsForm: GetUserMentionsForm = {
      sort: SortType[this.state.sort],
      unread_only: this.state.unreadOrAll == UnreadOrAll.Unread,
      page: this.state.page,
      limit: 9999,
    };
    WebSocketService.Instance.getUserMentions(userMentionsForm);
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.setState(this.state);
    this.refetch();
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
    } else if (op == UserOperation.GetReplies) {
      let res: GetRepliesResponse = msg;
      this.state.replies = res.replies;
      this.sendUnreadCount();
      window.scrollTo(0, 0);
      this.setState(this.state);
    } else if (op == UserOperation.GetUserMentions) {
      let res: GetUserMentionsResponse = msg;
      this.state.mentions = res.mentions;
      this.sendUnreadCount();
      window.scrollTo(0, 0);
      this.setState(this.state);
    } else if (op == UserOperation.MarkAllAsRead) {
      this.state.replies = [];
      this.state.mentions = [];
      window.scrollTo(0, 0);
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
      if (this.state.unreadOrAll == UnreadOrAll.Unread && res.comment.read) {
        this.state.replies = this.state.replies.filter(
          r => r.id !== res.comment.id
        );
      } else {
        let found = this.state.replies.find(c => c.id == res.comment.id);
        found.read = res.comment.read;
      }
      this.sendUnreadCount();
      this.setState(this.state);
    } else if (op == UserOperation.EditUserMention) {
      let res: UserMentionResponse = msg;

      let found = this.state.mentions.find(c => c.id == res.mention.id);
      found.content = res.mention.content;
      found.updated = res.mention.updated;
      found.removed = res.mention.removed;
      found.deleted = res.mention.deleted;
      found.upvotes = res.mention.upvotes;
      found.downvotes = res.mention.downvotes;
      found.score = res.mention.score;

      // If youre in the unread view, just remove it from the list
      if (this.state.unreadOrAll == UnreadOrAll.Unread && res.mention.read) {
        this.state.mentions = this.state.mentions.filter(
          r => r.id !== res.mention.id
        );
      } else {
        let found = this.state.mentions.find(c => c.id == res.mention.id);
        found.read = res.mention.read;
      }
      this.sendUnreadCount();
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
      let found: Comment = this.state.replies.find(
        c => c.id === res.comment.id
      );
      found.score = res.comment.score;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      if (res.comment.my_vote !== null) found.my_vote = res.comment.my_vote;
      this.setState(this.state);
    }
  }

  sendUnreadCount() {
    let count =
      this.state.replies.filter(r => !r.read).length +
      this.state.mentions.filter(r => !r.read).length;
    UserService.Instance.sub.next({
      user: UserService.Instance.user,
      unreadCount: count,
    });
  }
}
