import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { UserService } from '../services';

export class Navbar extends Component<any, any> {

  constructor(props: any, context: any) {
    super(props, context);
    this.state = {isLoggedIn: UserService.Instance.loggedIn, expanded: false};

    // Subscribe to user changes
    UserService.Instance.sub.subscribe(user => {
      let loggedIn: boolean = user !== null;
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
      <nav class="navbar navbar-expand-sm navbar-light bg-light p-0 px-3 shadow">
        <a class="navbar-brand" href="#">
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
              <Link class="nav-link" to="/create_post">Create Post</Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/create_community">Create Forum</Link>
            </li>
          </ul>
          <ul class="navbar-nav ml-auto mr-2">
            <li class="nav-item">
              {this.state.isLoggedIn ? 
              <a role="button" class="nav-link pointer" onClick={ linkEvent(this, this.handleLogoutClick) }>Logout</a> :
              <Link class="nav-link" to="/login">Login</Link>
              }
            </li>
          </ul>
        </div>
      </nav>
    );
  }

  handleLogoutClick() {
    UserService.Instance.logout();
  }

  expandNavbar(i: Navbar) {
    i.state.expanded = !i.state.expanded;
    i.setState(i.state);
  }
}
