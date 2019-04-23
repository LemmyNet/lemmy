import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { WebSocketService, UserService } from '../services';
import { UserOperation, GetRepliesForm, GetRepliesResponse, SortType } from '../interfaces';
import { msgOp } from '../utils';
import { version } from '../version';

interface NavbarState {
  isLoggedIn: boolean;
  expanded: boolean;
  expandUserDropdown: boolean;
  unreadCount: number;
}

export class Navbar extends Component<any, NavbarState> {
  private wsSub: Subscription;
  private userSub: Subscription;
  emptyState: NavbarState = {
    isLoggedIn: (UserService.Instance.user !== undefined),
    unreadCount: 0,
    expanded: false,
    expandUserDropdown: false
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
      this.setState(this.state);
    });

    this.wsSub = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );
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
  // TODO toggle css collapse
  navbar() {
    return (
      <nav class="container navbar navbar-expand-md navbar-light navbar-bg p-0 px-3">
        <a title={version} class="navbar-brand" href="#">
          <svg class="icon mr-2"><use xlinkHref="#icon-mouse"></use></svg>
          Lemmy
        </a>
        <button class="navbar-toggler" type="button" onClick={linkEvent(this, this.expandNavbar)}>
          <span class="navbar-toggler-icon"></span>
        </button>
        <div className={`${!this.state.expanded && 'collapse'} navbar-collapse`}>
          <ul class="navbar-nav mr-auto">
            <li class="nav-item">
              <Link class="nav-link" to="/communities">Forums</Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/search">Search</Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/create_post">Create Post</Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/create_community">Create Forum</Link>
            </li>
          </ul>
          <ul class="navbar-nav ml-auto mr-2">
            {this.state.isLoggedIn ? 
            <>
              {
                <li className="nav-item">
                  <Link class="inbox nav-link" to="/inbox">
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
                  <a role="button" class="dropdown-item pointer" onClick={linkEvent(this, this.handleOverviewClick)}>Overview</a>
                  <a role="button" class="dropdown-item pointer" onClick={ linkEvent(this, this.handleLogoutClick) }>Logout</a>
                </div>
              </li> 
            </>
              : 
              <Link class="nav-link" to="/login">Login / Sign up</Link>
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
    let userPage = `/user/${UserService.Instance.user.id}`;
    i.context.router.history.push(userPage);
  }

  expandNavbar(i: Navbar) {
    i.state.expanded = !i.state.expanded;
    i.setState(i.state);
  }

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      return;
    } else if (op == UserOperation.GetReplies) {
      let res: GetRepliesResponse = msg;
      this.sendRepliesCount(res);
    } 
  }

  keepFetchingReplies() {
    this.fetchReplies();
    setInterval(() => this.fetchReplies(), 30000);
  }

  fetchReplies() {
    if (this.state.isLoggedIn) {
      let repliesForm: GetRepliesForm = {
        sort: SortType[SortType.New],
        unread_only: true,
        page: 1,
        limit: 9999,
      };
      WebSocketService.Instance.getReplies(repliesForm);
    }
  }

  sendRepliesCount(res: GetRepliesResponse) {
    UserService.Instance.sub.next({user: UserService.Instance.user, unreadCount: res.replies.filter(r => !r.read).length});
  }
}

