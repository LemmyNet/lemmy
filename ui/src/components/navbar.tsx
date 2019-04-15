import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { UserService } from '../services';
import { version } from '../version';

interface NavbarState {
  isLoggedIn: boolean;
  expanded: boolean;
  expandUserDropdown: boolean;
}

export class Navbar extends Component<any, NavbarState> {

  emptyState: NavbarState = {
    isLoggedIn: UserService.Instance.loggedIn,
    expanded: false,
    expandUserDropdown: false
  }

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.handleOverviewClick = this.handleOverviewClick.bind(this);

    // Subscribe to user changes
    UserService.Instance.sub.subscribe(user => {
      let loggedIn: boolean = user !== undefined;
      this.setState({isLoggedIn: loggedIn});
    });
  }

  render() {
    return (
      <div>{this.navbar()}</div>
    )
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
              <a class="nav-link" href={repoUrl}>About</a>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/communities">Forums</Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/modlog">Modlog</Link>
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
            <li className={`nav-item dropdown ${this.state.expandUserDropdown && 'show'}`}>
              <a class="pointer nav-link dropdown-toggle" onClick={linkEvent(this, this.expandUserDropdown)} role="button">
                {UserService.Instance.user.username}
              </a>
              <div className={`dropdown-menu dropdown-menu-right ${this.state.expandUserDropdown && 'show'}`}>
                <a role="button" class="dropdown-item pointer" onClick={linkEvent(this, this.handleOverviewClick)}>Overview</a>
                <a role="button" class="dropdown-item pointer" onClick={ linkEvent(this, this.handleLogoutClick) }>Logout</a>
              </div>
            </li> : 
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
}

