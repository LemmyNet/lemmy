import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
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
  WebSocketJsonResponse,
  PrivateMessage as PrivateMessageI,
  GetPrivateMessagesForm,
  PrivateMessagesResponse,
  PrivateMessageResponse,
  GetSiteResponse,
  Site,
} from 'lemmy-js-client';
import { WebSocketService, UserService } from '../services';
import {
  wsJsonToRes,
  fetchLimit,
  isCommentType,
  toast,
  editCommentRes,
  saveCommentRes,
  createCommentLikeRes,
  commentsToFlatNodes,
  setupTippy,
} from '../utils';
import { CommentNodes } from './comment-nodes';
import { PrivateMessage } from './private-message';
import { SortSelect } from './sort-select';
import { i18n } from '../i18next';

enum UnreadOrAll {
  Unread,
  All,
}

enum MessageType {
  All,
  Replies,
  Mentions,
  Messages,
}

type ReplyType = Comment | PrivateMessageI;

interface InboxState {
  unreadOrAll: UnreadOrAll;
  messageType: MessageType;
  replies: Array<Comment>;
  mentions: Array<Comment>;
  messages: Array<PrivateMessageI>;
  sort: SortType;
  page: number;
  site: Site;
}

export class Inbox extends Component<any, InboxState> {
  private subscription: Subscription;
  private emptyState: InboxState = {
    unreadOrAll: UnreadOrAll.Unread,
    messageType: MessageType.All,
    replies: [],
    mentions: [],
    messages: [],
    sort: SortType.New,
    page: 1,
    site: {
      id: undefined,
      name: undefined,
      creator_id: undefined,
      published: undefined,
      creator_name: undefined,
      number_of_users: undefined,
      number_of_posts: undefined,
      number_of_comments: undefined,
      number_of_communities: undefined,
      enable_downvotes: undefined,
      open_registration: undefined,
      enable_nsfw: undefined,
    },
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleSortChange = this.handleSortChange.bind(this);

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    this.refetch();
    WebSocketService.Instance.getSite();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  get documentTitle(): string {
    if (this.state.site.name) {
      return `@${UserService.Instance.user.name} ${i18n.t('inbox')} - ${
        this.state.site.name
      }`;
    } else {
      return 'Lemmy';
    }
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle} />
        <div class="row">
          <div class="col-12">
            <h5 class="mb-1">
              {i18n.t('inbox')}
              <small>
                <a
                  href={`/feeds/inbox/${UserService.Instance.auth}.xml`}
                  target="_blank"
                  title="RSS"
                  rel="noopener"
                >
                  <svg class="icon ml-2 text-muted small">
                    <use xlinkHref="#icon-rss">#</use>
                  </svg>
                </a>
              </small>
            </h5>
            {this.state.replies.length +
              this.state.mentions.length +
              this.state.messages.length >
              0 &&
              this.state.unreadOrAll == UnreadOrAll.Unread && (
                <ul class="list-inline mb-1 text-muted small font-weight-bold">
                  <li className="list-inline-item">
                    <span
                      class="pointer"
                      onClick={linkEvent(this, this.markAllAsRead)}
                    >
                      {i18n.t('mark_all_as_read')}
                    </span>
                  </li>
                </ul>
              )}
            {this.selects()}
            {this.state.messageType == MessageType.All && this.all()}
            {this.state.messageType == MessageType.Replies && this.replies()}
            {this.state.messageType == MessageType.Mentions && this.mentions()}
            {this.state.messageType == MessageType.Messages && this.messages()}
            {this.paginator()}
          </div>
        </div>
      </div>
    );
  }

  unreadOrAllRadios() {
    return (
      <div class="btn-group btn-group-toggle flex-wrap mb-2">
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.unreadOrAll == UnreadOrAll.Unread && 'active'}
          `}
        >
          <input
            type="radio"
            value={UnreadOrAll.Unread}
            checked={this.state.unreadOrAll == UnreadOrAll.Unread}
            onChange={linkEvent(this, this.handleUnreadOrAllChange)}
          />
          {i18n.t('unread')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.unreadOrAll == UnreadOrAll.All && 'active'}
          `}
        >
          <input
            type="radio"
            value={UnreadOrAll.All}
            checked={this.state.unreadOrAll == UnreadOrAll.All}
            onChange={linkEvent(this, this.handleUnreadOrAllChange)}
          />
          {i18n.t('all')}
        </label>
      </div>
    );
  }

  messageTypeRadios() {
    return (
      <div class="btn-group btn-group-toggle flex-wrap mb-2">
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.messageType == MessageType.All && 'active'}
          `}
        >
          <input
            type="radio"
            value={MessageType.All}
            checked={this.state.messageType == MessageType.All}
            onChange={linkEvent(this, this.handleMessageTypeChange)}
          />
          {i18n.t('all')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.messageType == MessageType.Replies && 'active'}
          `}
        >
          <input
            type="radio"
            value={MessageType.Replies}
            checked={this.state.messageType == MessageType.Replies}
            onChange={linkEvent(this, this.handleMessageTypeChange)}
          />
          {i18n.t('replies')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.messageType == MessageType.Mentions && 'active'}
          `}
        >
          <input
            type="radio"
            value={MessageType.Mentions}
            checked={this.state.messageType == MessageType.Mentions}
            onChange={linkEvent(this, this.handleMessageTypeChange)}
          />
          {i18n.t('mentions')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.messageType == MessageType.Messages && 'active'}
          `}
        >
          <input
            type="radio"
            value={MessageType.Messages}
            checked={this.state.messageType == MessageType.Messages}
            onChange={linkEvent(this, this.handleMessageTypeChange)}
          />
          {i18n.t('messages')}
        </label>
      </div>
    );
  }

  selects() {
    return (
      <div className="mb-2">
        <span class="mr-3">{this.unreadOrAllRadios()}</span>
        <span class="mr-3">{this.messageTypeRadios()}</span>
        <SortSelect
          sort={this.state.sort}
          onChange={this.handleSortChange}
          hideHot
        />
      </div>
    );
  }

  combined(): Array<ReplyType> {
    return [
      ...this.state.replies,
      ...this.state.mentions,
      ...this.state.messages,
    ].sort((a, b) => b.published.localeCompare(a.published));
  }

  all() {
    return (
      <div>
        {this.combined().map(i =>
          isCommentType(i) ? (
            <CommentNodes
              key={i.id}
              nodes={[{ comment: i }]}
              noIndent
              markable
              showCommunity
              showContext
              enableDownvotes={this.state.site.enable_downvotes}
            />
          ) : (
            <PrivateMessage key={i.id} privateMessage={i} />
          )
        )}
      </div>
    );
  }

  replies() {
    return (
      <div>
        <CommentNodes
          nodes={commentsToFlatNodes(this.state.replies)}
          noIndent
          markable
          showCommunity
          showContext
          enableDownvotes={this.state.site.enable_downvotes}
        />
      </div>
    );
  }

  mentions() {
    return (
      <div>
        {this.state.mentions.map(mention => (
          <CommentNodes
            key={mention.id}
            nodes={[{ comment: mention }]}
            noIndent
            markable
            showCommunity
            showContext
            enableDownvotes={this.state.site.enable_downvotes}
          />
        ))}
      </div>
    );
  }

  messages() {
    return (
      <div>
        {this.state.messages.map(message => (
          <PrivateMessage key={message.id} privateMessage={message} />
        ))}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && (
          <button
            class="btn btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            {i18n.t('prev')}
          </button>
        )}
        {this.unreadCount() > 0 && (
          <button
            class="btn btn-secondary"
            onClick={linkEvent(this, this.nextPage)}
          >
            {i18n.t('next')}
          </button>
        )}
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

  handleMessageTypeChange(i: Inbox, event: any) {
    i.state.messageType = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.refetch();
  }

  refetch() {
    let repliesForm: GetRepliesForm = {
      sort: this.state.sort,
      unread_only: this.state.unreadOrAll == UnreadOrAll.Unread,
      page: this.state.page,
      limit: fetchLimit,
    };
    WebSocketService.Instance.getReplies(repliesForm);

    let userMentionsForm: GetUserMentionsForm = {
      sort: this.state.sort,
      unread_only: this.state.unreadOrAll == UnreadOrAll.Unread,
      page: this.state.page,
      limit: fetchLimit,
    };
    WebSocketService.Instance.getUserMentions(userMentionsForm);

    let privateMessagesForm: GetPrivateMessagesForm = {
      unread_only: this.state.unreadOrAll == UnreadOrAll.Unread,
      page: this.state.page,
      limit: fetchLimit,
    };
    WebSocketService.Instance.getPrivateMessages(privateMessagesForm);
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.setState(this.state);
    this.refetch();
  }

  markAllAsRead(i: Inbox) {
    WebSocketService.Instance.markAllAsRead();
    i.state.replies = [];
    i.state.mentions = [];
    i.state.messages = [];
    i.sendUnreadCount();
    window.scrollTo(0, 0);
    i.setState(i.state);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (msg.reconnect) {
      this.refetch();
    } else if (res.op == UserOperation.GetReplies) {
      let data = res.data as GetRepliesResponse;
      this.state.replies = data.replies;
      this.sendUnreadCount();
      window.scrollTo(0, 0);
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.GetUserMentions) {
      let data = res.data as GetUserMentionsResponse;
      this.state.mentions = data.mentions;
      this.sendUnreadCount();
      window.scrollTo(0, 0);
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.GetPrivateMessages) {
      let data = res.data as PrivateMessagesResponse;
      this.state.messages = data.messages;
      this.sendUnreadCount();
      window.scrollTo(0, 0);
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.EditPrivateMessage) {
      let data = res.data as PrivateMessageResponse;
      let found: PrivateMessageI = this.state.messages.find(
        m => m.id === data.message.id
      );
      if (found) {
        found.content = data.message.content;
        found.updated = data.message.updated;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.DeletePrivateMessage) {
      let data = res.data as PrivateMessageResponse;
      let found: PrivateMessageI = this.state.messages.find(
        m => m.id === data.message.id
      );
      if (found) {
        found.deleted = data.message.deleted;
        found.updated = data.message.updated;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.MarkPrivateMessageAsRead) {
      let data = res.data as PrivateMessageResponse;
      let found: PrivateMessageI = this.state.messages.find(
        m => m.id === data.message.id
      );

      if (found) {
        found.updated = data.message.updated;

        // If youre in the unread view, just remove it from the list
        if (this.state.unreadOrAll == UnreadOrAll.Unread && data.message.read) {
          this.state.messages = this.state.messages.filter(
            r => r.id !== data.message.id
          );
        } else {
          let found = this.state.messages.find(c => c.id == data.message.id);
          found.read = data.message.read;
        }
      }
      this.sendUnreadCount();
      this.setState(this.state);
    } else if (res.op == UserOperation.MarkAllAsRead) {
      // Moved to be instant
    } else if (
      res.op == UserOperation.EditComment ||
      res.op == UserOperation.DeleteComment ||
      res.op == UserOperation.RemoveComment
    ) {
      let data = res.data as CommentResponse;
      editCommentRes(data, this.state.replies);
      this.setState(this.state);
    } else if (res.op == UserOperation.MarkCommentAsRead) {
      let data = res.data as CommentResponse;

      // If youre in the unread view, just remove it from the list
      if (this.state.unreadOrAll == UnreadOrAll.Unread && data.comment.read) {
        this.state.replies = this.state.replies.filter(
          r => r.id !== data.comment.id
        );
      } else {
        let found = this.state.replies.find(c => c.id == data.comment.id);
        found.read = data.comment.read;
      }
      this.sendUnreadCount();
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.MarkUserMentionAsRead) {
      let data = res.data as UserMentionResponse;

      let found = this.state.mentions.find(c => c.id == data.mention.id);
      found.content = data.mention.content;
      found.updated = data.mention.updated;
      found.removed = data.mention.removed;
      found.deleted = data.mention.deleted;
      found.upvotes = data.mention.upvotes;
      found.downvotes = data.mention.downvotes;
      found.score = data.mention.score;

      // If youre in the unread view, just remove it from the list
      if (this.state.unreadOrAll == UnreadOrAll.Unread && data.mention.read) {
        this.state.mentions = this.state.mentions.filter(
          r => r.id !== data.mention.id
        );
      } else {
        let found = this.state.mentions.find(c => c.id == data.mention.id);
        found.read = data.mention.read;
      }
      this.sendUnreadCount();
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      if (data.recipient_ids.includes(UserService.Instance.user.id)) {
        this.state.replies.unshift(data.comment);
        this.setState(this.state);
      } else if (data.comment.creator_id == UserService.Instance.user.id) {
        toast(i18n.t('reply_sent'));
      }
    } else if (res.op == UserOperation.CreatePrivateMessage) {
      let data = res.data as PrivateMessageResponse;
      if (data.message.recipient_id == UserService.Instance.user.id) {
        this.state.messages.unshift(data.message);
        this.setState(this.state);
      }
    } else if (res.op == UserOperation.SaveComment) {
      let data = res.data as CommentResponse;
      saveCommentRes(data, this.state.replies);
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.CreateCommentLike) {
      let data = res.data as CommentResponse;
      createCommentLikeRes(data, this.state.replies);
      this.setState(this.state);
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }

  sendUnreadCount() {
    UserService.Instance.unreadCountSub.next(this.unreadCount());
  }

  unreadCount(): number {
    return (
      this.state.replies.filter(r => !r.read).length +
      this.state.mentions.filter(r => !r.read).length +
      this.state.messages.filter(
        r =>
          UserService.Instance.user &&
          !r.read &&
          r.creator_id !== UserService.Instance.user.id
      ).length
    );
  }
}
