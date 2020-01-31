import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { WebSocketService, UserService } from '../services';
import {
  UserOperation,
  GetRepliesForm,
  GetRepliesResponse,
  GetUserMentionsForm,
  GetUserMentionsResponse,
  GetPrivateMessagesForm,
  PrivateMessagesResponse,
  SortType,
  GetSiteResponse,
  Comment,
  PrivateMessage,
  WebSocketJsonResponse,
} from '../interfaces';
import {
  wsJsonToRes,
  pictshareAvatarThumbnail,
  showAvatars,
  fetchLimit,
  isCommentType,
  toast,
} from '../utils';
import { version } from '../version';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface NavbarState {
  isLoggedIn: boolean;
  expanded: boolean;
  replies: Array<Comment>;
  mentions: Array<Comment>;
  messages: Array<PrivateMessage>;
  fetchCount: number;
  unreadCount: number;
  siteName: string;
}

export class Navbar extends Component<any, NavbarState> {
  private wsSub: Subscription;
  private userSub: Subscription;
  emptyState: NavbarState = {
    isLoggedIn: UserService.Instance.user !== undefined,
    unreadCount: 0,
    fetchCount: 0,
    replies: [],
    mentions: [],
    messages: [],
    expanded: false,
    siteName: undefined,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;

    this.keepFetchingUnreads();

    // Subscribe to user changes
    this.userSub = UserService.Instance.sub.subscribe(user => {
      this.state.isLoggedIn = user.user !== undefined;
      this.state.unreadCount = user.unreadCount;
      this.requestNotificationPermission();
      this.setState(this.state);
    });

    this.wsSub = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    if (this.state.isLoggedIn) {
      this.requestNotificationPermission();
    }

    WebSocketService.Instance.getSite();
  }

  render() {
    return this.navbar();
  }

  componentWillUnmount() {
    this.wsSub.unsubscribe();
    this.userSub.unsubscribe();
  }

  // TODO class active corresponding to current page
  navbar() {
    return (
      <nav class="container-fluid navbar navbar-expand-md navbar-light shadow p-0 px-3">
        <Link title={version} class="navbar-brand" to="/">
          {this.state.siteName}
        </Link>
        <button
          class="navbar-toggler"
          type="button"
          onClick={linkEvent(this, this.expandNavbar)}
        >
          <span class="navbar-toggler-icon"></span>
        </button>
        <div
          className={`${!this.state.expanded && 'collapse'} navbar-collapse`}
        >
          <ul class="navbar-nav mr-auto">
            <li class="nav-item">
              <Link class="nav-link" to="/communities">
                <T i18nKey="communities">#</T>
              </Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/search">
                <T i18nKey="search">#</T>
              </Link>
            </li>
            <li class="nav-item">
              <Link
                class="nav-link"
                to={{
                  pathname: '/create_post',
                  state: { prevPath: this.currentLocation },
                }}
              >
                <T i18nKey="create_post">#</T>
              </Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/create_community">
                <T i18nKey="create_community">#</T>
              </Link>
            </li>
            <li className="nav-item">
              <Link
                class="nav-link"
                to="/sponsors"
                title={i18n.t('donate_to_lemmy')}
              >
                <svg class="icon">
                  <use xlinkHref="#icon-coffee"></use>
                </svg>
              </Link>
            </li>
          </ul>
          <ul class="navbar-nav ml-auto">
            {this.state.isLoggedIn ? (
              <>
                <li className="nav-item mt-1">
                  <Link class="nav-link" to="/inbox">
                    <svg class="icon">
                      <use xlinkHref="#icon-mail"></use>
                    </svg>
                    {this.state.unreadCount > 0 && (
                      <span class="ml-1 badge badge-light">
                        {this.state.unreadCount}
                      </span>
                    )}
                  </Link>
                </li>
                <li className="nav-item">
                  <Link
                    class="nav-link"
                    to={`/u/${UserService.Instance.user.username}`}
                  >
                    <span>
                      {UserService.Instance.user.avatar && showAvatars() && (
                        <img
                          src={pictshareAvatarThumbnail(
                            UserService.Instance.user.avatar
                          )}
                          height="32"
                          width="32"
                          class="rounded-circle mr-2"
                        />
                      )}
                      {UserService.Instance.user.username}
                    </span>
                  </Link>
                </li>
              </>
            ) : (
              <Link class="nav-link" to="/login">
                <T i18nKey="login_sign_up">#</T>
              </Link>
            )}
          </ul>
        </div>
      </nav>
    );
  }

  expandNavbar(i: Navbar) {
    i.state.expanded = !i.state.expanded;
    i.setState(i.state);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);
    if (msg.error) {
      if (msg.error == 'not_logged_in') {
        UserService.Instance.logout();
        location.reload();
      }
      return;
    } else if (res.op == UserOperation.GetReplies) {
      let data = res.data as GetRepliesResponse;
      let unreadReplies = data.replies.filter(r => !r.read);
      if (
        unreadReplies.length > 0 &&
        this.state.fetchCount > 1 &&
        JSON.stringify(this.state.replies) !== JSON.stringify(unreadReplies)
      ) {
        this.notify(unreadReplies);
      }

      this.state.replies = unreadReplies;
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.GetUserMentions) {
      let data = res.data as GetUserMentionsResponse;
      let unreadMentions = data.mentions.filter(r => !r.read);
      if (
        unreadMentions.length > 0 &&
        this.state.fetchCount > 1 &&
        JSON.stringify(this.state.mentions) !== JSON.stringify(unreadMentions)
      ) {
        this.notify(unreadMentions);
      }

      this.state.mentions = unreadMentions;
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.GetPrivateMessages) {
      let data = res.data as PrivateMessagesResponse;
      let unreadMessages = data.messages.filter(r => !r.read);
      if (
        unreadMessages.length > 0 &&
        this.state.fetchCount > 1 &&
        JSON.stringify(this.state.messages) !== JSON.stringify(unreadMessages)
      ) {
        this.notify(unreadMessages);
      }

      this.state.messages = unreadMessages;
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;

      if (data.site) {
        this.state.siteName = data.site.name;
        WebSocketService.Instance.site = data.site;
        this.setState(this.state);
      }
    }
  }

  keepFetchingUnreads() {
    this.fetchUnreads();
    setInterval(() => this.fetchUnreads(), 15000);
  }

  fetchUnreads() {
    if (this.state.isLoggedIn) {
      let repliesForm: GetRepliesForm = {
        sort: SortType[SortType.New],
        unread_only: true,
        page: 1,
        limit: fetchLimit,
      };

      let userMentionsForm: GetUserMentionsForm = {
        sort: SortType[SortType.New],
        unread_only: true,
        page: 1,
        limit: fetchLimit,
      };

      let privateMessagesForm: GetPrivateMessagesForm = {
        unread_only: true,
        page: 1,
        limit: fetchLimit,
      };

      if (this.currentLocation !== '/inbox') {
        WebSocketService.Instance.getReplies(repliesForm);
        WebSocketService.Instance.getUserMentions(userMentionsForm);
        WebSocketService.Instance.getPrivateMessages(privateMessagesForm);
        this.state.fetchCount++;
      }
    }
  }

  get currentLocation() {
    return this.context.router.history.location.pathname;
  }

  sendUnreadCount() {
    UserService.Instance.sub.next({
      user: UserService.Instance.user,
      unreadCount: this.unreadCount,
    });
  }

  get unreadCount() {
    return (
      this.state.replies.filter(r => !r.read).length +
      this.state.mentions.filter(r => !r.read).length +
      this.state.messages.filter(r => !r.read).length
    );
  }

  requestNotificationPermission() {
    if (UserService.Instance.user) {
      document.addEventListener('DOMContentLoaded', function() {
        if (!Notification) {
          toast(i18n.t('notifications_error'), 'danger');
          return;
        }

        if (Notification.permission !== 'granted')
          Notification.requestPermission();
      });
    }
  }

  notify(replies: Array<Comment | PrivateMessage>) {
    let recentReply = replies[0];
    if (Notification.permission !== 'granted') Notification.requestPermission();
    else {
      var notification = new Notification(
        `${replies.length} ${i18n.t('unread_messages')}`,
        {
          icon: recentReply.creator_avatar
            ? recentReply.creator_avatar
            : `${window.location.protocol}//${window.location.host}/static/assets/apple-touch-icon.png`,
          body: `${recentReply.creator_name}: ${recentReply.content}`,
        }
      );

      notification.onclick = () => {
        this.context.router.history.push(
          isCommentType(recentReply)
            ? `/post/${recentReply.post_id}/comment/${recentReply.id}`
            : `/inbox`
        );
      };
    }
  }
}
