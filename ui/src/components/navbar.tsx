import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { WebSocketService, UserService } from '../services';
import { UserOperation, GetRepliesForm, GetRepliesResponse, SortType, GetSiteResponse, Comment} from '../interfaces';
import { msgOp } from '../utils';
import { version } from '../version';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface NavbarState {
  isLoggedIn: boolean;
  expanded: boolean;
  expandUserDropdown: boolean;
  replies: Array<Comment>,
  fetchCount: number,
  unreadCount: number;
  siteName: string;
}

export class Navbar extends Component<any, NavbarState> {
  private wsSub: Subscription;
  private userSub: Subscription;
  emptyState: NavbarState = {
    isLoggedIn: (UserService.Instance.user !== undefined),
    unreadCount: 0,
    fetchCount: 0,
    replies: [],
    expanded: false,
    expandUserDropdown: false,
    siteName: undefined
  }

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.handleOverviewClick = this.handleOverviewClick.bind(this);

    this.keepFetchingReplies();

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
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    if (this.state.isLoggedIn) {
      this.requestNotificationPermission();
    }

    WebSocketService.Instance.getSite();
  }

  render() {
    return (
      <div>{this.navbar()}</div>
    )
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
        <button class="navbar-toggler" type="button" onClick={linkEvent(this, this.expandNavbar)}>
          <span class="navbar-toggler-icon"></span>
        </button>
        <div className={`${!this.state.expanded && 'collapse'} navbar-collapse`}>
          <ul class="navbar-nav mr-auto">
            <li class="nav-item">
              <Link class="nav-link" to="/communities"><T i18nKey="communities">#</T></Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/search"><T i18nKey="search">#</T></Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to={{pathname: '/create_post', state: { prevPath: this.currentLocation }}}><T i18nKey="create_post">#</T></Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/create_community"><T i18nKey="create_community">#</T></Link>
            </li>
          </ul>
          <ul class="navbar-nav ml-auto mr-2">
            {this.state.isLoggedIn ? 
            <>
              {
                <li className="nav-item">
                  <Link class="nav-link" to="/inbox">
                    <svg class="icon"><use xlinkHref="#icon-mail"></use></svg>
                    {this.state.unreadCount> 0 && <span class="ml-1 badge badge-light">{this.state.unreadCount}</span>}
                  </Link>
                </li>
              }
              <li className={`nav-item dropdown ${this.state.expandUserDropdown && 'show'}`}>
                <a class="pointer nav-link dropdown-toggle" onClick={linkEvent(this, this.expandUserDropdown)} role="button">
                  {UserService.Instance.user.username}
                </a>
                <div className={`dropdown-menu dropdown-menu-right ${this.state.expandUserDropdown && 'show'}`}>
                  <a role="button" class="dropdown-item pointer" onClick={linkEvent(this, this.handleOverviewClick)}><T i18nKey="overview">#</T></a>
                  <a role="button" class="dropdown-item pointer" onClick={ linkEvent(this, this.handleLogoutClick) }><T i18nKey="logout">#</T></a>
                </div>
              </li> 
            </>
              : 
              <Link class="nav-link" to="/login"><T i18nKey="login_sign_up">#</T></Link>
            }
          </ul>
        </div>
      </nav>
    );
  }

  expandUserDropdown(i: Navbar) {
    i.state.expandUserDropdown = !i.state.expandUserDropdown;
    i.setState(i.state);
  }

  handleLogoutClick(i: Navbar) {
    i.state.expandUserDropdown = false;
    UserService.Instance.logout();
    i.context.router.history.push('/');
  }

  handleOverviewClick(i: Navbar) {
    i.state.expandUserDropdown = false;
    i.setState(i.state);
    let userPage = `/u/${UserService.Instance.user.username}`;
    i.context.router.history.push(userPage);
  }

  expandNavbar(i: Navbar) {
    i.state.expanded = !i.state.expanded;
    i.setState(i.state);
  }

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      if (msg.error == "not_logged_in") {
        UserService.Instance.logout();
        location.reload();
      }
      return;
    } else if (op == UserOperation.GetReplies) {
      let res: GetRepliesResponse = msg;
      let unreadReplies = res.replies.filter(r => !r.read);
      if (unreadReplies.length > 0 && this.state.fetchCount > 1 && 
          (JSON.stringify(this.state.replies) !== JSON.stringify(unreadReplies))) {
        this.notify(unreadReplies);
      }

      this.state.replies = unreadReplies;
      this.sendRepliesCount(res);
    } else if (op == UserOperation.GetSite) {
      let res: GetSiteResponse = msg;

      if (res.site) {
        this.state.siteName = res.site.name;
        WebSocketService.Instance.site = res.site;
        this.setState(this.state);
      }
    } 
  }

  keepFetchingReplies() {
    this.fetchReplies();
    setInterval(() => this.fetchReplies(), 15000);
  }

  fetchReplies() {
    if (this.state.isLoggedIn) {
      let repliesForm: GetRepliesForm = {
        sort: SortType[SortType.New],
        unread_only: true,
        page: 1,
        limit: 9999,
      };
      if (this.currentLocation !=='/inbox') { 
        WebSocketService.Instance.getReplies(repliesForm);
        this.state.fetchCount++;
      }
    }
  }

  get currentLocation() {
    return this.context.router.history.location.pathname;
  }

  sendRepliesCount(res: GetRepliesResponse) {
    UserService.Instance.sub.next({user: UserService.Instance.user, unreadCount: res.replies.filter(r => !r.read).length});
  }

  requestNotificationPermission() {
    if (UserService.Instance.user) {
    document.addEventListener('DOMContentLoaded', function () {
      if (!Notification) {
        alert(i18n.t('notifications_error')); 
        return;
      }

      if (Notification.permission !== 'granted')
        Notification.requestPermission();
    });
    }
  }

  notify(replies: Array<Comment>) {
    let recentReply = replies[0];
    if (Notification.permission !== 'granted')
      Notification.requestPermission();
    else {
      var notification = new Notification(`${replies.length} ${i18n.t('unread_messages')}`, {
        icon: `${window.location.protocol}//${window.location.host}/static/assets/apple-touch-icon.png`,
        body: `${recentReply.creator_name}: ${recentReply.content}`
      });

      notification.onclick = () => {
        this.context.router.history.push(`/post/${recentReply.post_id}/comment/${recentReply.id}`);
      };

    }
  }
}
