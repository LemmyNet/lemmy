import { Component, linkEvent, createRef, RefObject } from 'inferno';
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
  CommentResponse,
  PrivateMessage,
  UserView,
  PrivateMessageResponse,
  WebSocketJsonResponse,
} from '../interfaces';
import {
  wsJsonToRes,
  pictrsAvatarThumbnail,
  showAvatars,
  fetchLimit,
  isCommentType,
  toast,
  messageToastify,
  md,
  setTheme,
} from '../utils';
import { i18n, i18nextSetup } from '../i18next';

interface NavbarState {
  isLoggedIn: boolean;
  expanded: boolean;
  replies: Array<Comment>;
  mentions: Array<Comment>;
  messages: Array<PrivateMessage>;
  unreadCount: number;
  siteName: string;
  version: string;
  admins: Array<UserView>;
  searchParam: string;
  toggleSearch: boolean;
  siteLoading: boolean;
}

export class Navbar extends Component<any, NavbarState> {
  private wsSub: Subscription;
  private userSub: Subscription;
  private unreadCountSub: Subscription;
  private searchTextField: RefObject<HTMLInputElement>;
  emptyState: NavbarState = {
    isLoggedIn: false,
    unreadCount: 0,
    replies: [],
    mentions: [],
    messages: [],
    expanded: false,
    siteName: undefined,
    version: undefined,
    admins: [],
    searchParam: '',
    toggleSearch: false,
    siteLoading: true,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;

    this.wsSub = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getSite();

    this.searchTextField = createRef();
  }

  componentDidMount() {
    // Subscribe to jwt changes
    this.userSub = UserService.Instance.jwtSub.subscribe(res => {
      // A login
      if (res !== undefined) {
        this.requestNotificationPermission();
      } else {
        this.state.isLoggedIn = false;
      }
      WebSocketService.Instance.getSite();
      this.setState(this.state);
    });

    // Subscribe to unread count changes
    this.unreadCountSub = UserService.Instance.unreadCountSub.subscribe(res => {
      this.setState({ unreadCount: res });
    });
  }

  handleSearchParam(i: Navbar, event: any) {
    i.state.searchParam = event.target.value;
    i.setState(i.state);
  }

  updateUrl() {
    const searchParam = this.state.searchParam;
    this.setState({ searchParam: '' });
    this.setState({ toggleSearch: false });
    if (searchParam === '') {
      this.context.router.history.push(`/search/`);
    } else {
      this.context.router.history.push(
        `/search/q/${searchParam}/type/all/sort/topall/page/1`
      );
    }
  }

  handleSearchSubmit(i: Navbar, event: any) {
    event.preventDefault();
    i.updateUrl();
  }

  handleSearchBtn(i: Navbar, event: any) {
    event.preventDefault();
    i.setState({ toggleSearch: true });

    i.searchTextField.current.focus();
    const offsetWidth = i.searchTextField.current.offsetWidth;
    if (i.state.searchParam && offsetWidth > 100) {
      i.updateUrl();
    }
  }

  handleSearchBlur(i: Navbar, event: any) {
    if (!(event.relatedTarget && event.relatedTarget.name !== 'search-btn')) {
      i.state.toggleSearch = false;
      i.setState(i.state);
    }
  }

  render() {
    return this.navbar();
  }

  componentWillUnmount() {
    this.wsSub.unsubscribe();
    this.userSub.unsubscribe();
    this.unreadCountSub.unsubscribe();
  }

  // TODO class active corresponding to current page
  navbar() {
    return (
      <nav class="navbar navbar-expand-lg navbar-light shadow-sm p-0 px-3">
        <div class="container">
          {!this.state.siteLoading ? (
            <Link title={this.state.version} class="navbar-brand" to="/">
              {this.state.siteName}
            </Link>
          ) : (
            <div class="navbar-item">
              <svg class="icon icon-spinner spin">
                <use xlinkHref="#icon-spinner"></use>
              </svg>
            </div>
          )}
          {this.state.isLoggedIn && (
            <Link
              class="ml-auto p-0 navbar-toggler nav-link border-0"
              to="/inbox"
              title={i18n.t('inbox')}
            >
              <svg class="icon">
                <use xlinkHref="#icon-bell"></use>
              </svg>
              {this.state.unreadCount > 0 && (
                <span class="ml-1 badge badge-light">
                  {this.state.unreadCount}
                </span>
              )}
            </Link>
          )}
          <button
            class="navbar-toggler border-0"
            type="button"
            aria-label="menu"
            onClick={linkEvent(this, this.expandNavbar)}
            data-tippy-content={i18n.t('expand_here')}
          >
            <span class="navbar-toggler-icon"></span>
          </button>
          {!this.state.siteLoading && (
            <div
              className={`${
                !this.state.expanded && 'collapse'
              } navbar-collapse`}
            >
              <ul class="navbar-nav my-2 mr-auto">
                <li class="nav-item">
                  <Link
                    class="nav-link"
                    to="/communities"
                    title={i18n.t('communities')}
                  >
                    {i18n.t('communities')}
                  </Link>
                </li>
                <li class="nav-item">
                  <Link
                    class="nav-link"
                    to={{
                      pathname: '/create_post',
                      state: { prevPath: this.currentLocation },
                    }}
                    title={i18n.t('create_post')}
                  >
                    {i18n.t('create_post')}
                  </Link>
                </li>
                <li class="nav-item">
                  <Link
                    class="nav-link"
                    to="/create_community"
                    title={i18n.t('create_community')}
                  >
                    {i18n.t('create_community')}
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
              {!this.context.router.history.location.pathname.match(
                /^\/search/
              ) && (
                <form
                  class="form-inline"
                  onSubmit={linkEvent(this, this.handleSearchSubmit)}
                >
                  <input
                    class={`form-control mr-0 search-input ${
                      this.state.toggleSearch ? 'show-input' : 'hide-input'
                    }`}
                    onInput={linkEvent(this, this.handleSearchParam)}
                    value={this.state.searchParam}
                    ref={this.searchTextField}
                    type="text"
                    placeholder={i18n.t('search')}
                    onBlur={linkEvent(this, this.handleSearchBlur)}
                  ></input>
                  <button
                    name="search-btn"
                    onClick={linkEvent(this, this.handleSearchBtn)}
                    class="btn btn-link"
                    style="color: var(--gray)"
                  >
                    <svg class="icon">
                      <use xlinkHref="#icon-search"></use>
                    </svg>
                  </button>
                </form>
              )}
              <ul class="navbar-nav my-2">
                {this.canAdmin && (
                  <li className="nav-item">
                    <Link
                      class="nav-link"
                      to={`/admin`}
                      title={i18n.t('admin_settings')}
                    >
                      <svg class="icon">
                        <use xlinkHref="#icon-settings"></use>
                      </svg>
                    </Link>
                  </li>
                )}
              </ul>
              {this.state.isLoggedIn ? (
                <>
                  <ul class="navbar-nav my-2">
                    <li className="nav-item">
                      <Link
                        class="nav-link"
                        to="/inbox"
                        title={i18n.t('inbox')}
                      >
                        <svg class="icon">
                          <use xlinkHref="#icon-bell"></use>
                        </svg>
                        {this.state.unreadCount > 0 && (
                          <span class="ml-1 badge badge-light">
                            {this.state.unreadCount}
                          </span>
                        )}
                      </Link>
                    </li>
                  </ul>
                  <ul class="navbar-nav">
                    <li className="nav-item">
                      <Link
                        class="nav-link"
                        to={`/u/${UserService.Instance.user.name}`}
                        title={i18n.t('settings')}
                      >
                        <span>
                          {UserService.Instance.user.avatar &&
                            showAvatars() && (
                              <img
                                src={pictrsAvatarThumbnail(
                                  UserService.Instance.user.avatar
                                )}
                                height="32"
                                width="32"
                                class="rounded-circle mr-2"
                              />
                            )}
                          {UserService.Instance.user.name}
                        </span>
                      </Link>
                    </li>
                  </ul>
                </>
              ) : (
                <ul class="navbar-nav my-2">
                  <li className="nav-item">
                    <Link
                      class="btn btn-success"
                      to="/login"
                      title={i18n.t('login_sign_up')}
                    >
                      {i18n.t('login_sign_up')}
                    </Link>
                  </li>
                </ul>
              )}
            </div>
          )}
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
    } else if (msg.reconnect) {
      this.fetchUnreads();
    } else if (res.op == UserOperation.GetReplies) {
      let data = res.data as GetRepliesResponse;
      let unreadReplies = data.replies.filter(r => !r.read);

      this.state.replies = unreadReplies;
      this.state.unreadCount = this.calculateUnreadCount();
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.GetUserMentions) {
      let data = res.data as GetUserMentionsResponse;
      let unreadMentions = data.mentions.filter(r => !r.read);

      this.state.mentions = unreadMentions;
      this.state.unreadCount = this.calculateUnreadCount();
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.GetPrivateMessages) {
      let data = res.data as PrivateMessagesResponse;
      let unreadMessages = data.messages.filter(r => !r.read);

      this.state.messages = unreadMessages;
      this.state.unreadCount = this.calculateUnreadCount();
      this.setState(this.state);
      this.sendUnreadCount();
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      if (this.state.isLoggedIn) {
        if (data.recipient_ids.includes(UserService.Instance.user.id)) {
          this.state.replies.push(data.comment);
          this.state.unreadCount++;
          this.setState(this.state);
          this.sendUnreadCount();
          this.notify(data.comment);
        }
      }
    } else if (res.op == UserOperation.CreatePrivateMessage) {
      let data = res.data as PrivateMessageResponse;

      if (this.state.isLoggedIn) {
        if (data.message.recipient_id == UserService.Instance.user.id) {
          this.state.messages.push(data.message);
          this.state.unreadCount++;
          this.setState(this.state);
          this.sendUnreadCount();
          this.notify(data.message);
        }
      }
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;

      if (data.site && !this.state.siteName) {
        this.state.siteName = data.site.name;
        this.state.version = data.version;
        this.state.admins = data.admins;
      }

      // The login
      if (data.my_user) {
        UserService.Instance.user = data.my_user;
        // On the first load, check the unreads
        if (this.state.isLoggedIn == false) {
          this.requestNotificationPermission();
          this.fetchUnreads();
          setTheme(data.my_user.theme, true);
        }
        this.state.isLoggedIn = true;
      }

      i18nextSetup();

      this.state.siteLoading = false;
      this.setState(this.state);
    }
  }

  fetchUnreads() {
    console.log('Fetching unreads...');
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
    }
  }

  get currentLocation() {
    return this.context.router.history.location.pathname;
  }

  sendUnreadCount() {
    UserService.Instance.unreadCountSub.next(this.state.unreadCount);
  }

  calculateUnreadCount(): number {
    return (
      this.state.replies.filter(r => !r.read).length +
      this.state.mentions.filter(r => !r.read).length +
      this.state.messages.filter(r => !r.read).length
    );
  }

  get canAdmin(): boolean {
    return (
      UserService.Instance.user &&
      this.state.admins.map(a => a.id).includes(UserService.Instance.user.id)
    );
  }

  requestNotificationPermission() {
    if (UserService.Instance.user) {
      document.addEventListener('DOMContentLoaded', function () {
        if (!Notification) {
          toast(i18n.t('notifications_error'), 'danger');
          return;
        }

        if (Notification.permission !== 'granted')
          Notification.requestPermission();
      });
    }
  }

  notify(reply: Comment | PrivateMessage) {
    let creator_name = reply.creator_name;
    let creator_avatar = reply.creator_avatar
      ? reply.creator_avatar
      : `${window.location.protocol}//${window.location.host}/static/assets/apple-touch-icon.png`;
    let link = isCommentType(reply)
      ? `/post/${reply.post_id}/comment/${reply.id}`
      : `/inbox`;
    let htmlBody = md.render(reply.content);
    let body = reply.content; // Unfortunately the notifications API can't do html

    messageToastify(
      creator_name,
      creator_avatar,
      htmlBody,
      link,
      this.context.router
    );

    if (Notification.permission !== 'granted') Notification.requestPermission();
    else {
      var notification = new Notification(reply.creator_name, {
        icon: creator_avatar,
        body: body,
      });

      notification.onclick = () => {
        event.preventDefault();
        this.context.router.history.push(link);
      };
    }
  }
}
