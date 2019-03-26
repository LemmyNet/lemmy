import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { UserService } from '../services';

export class Navbar extends Component<any, any> {

  constructor(props, context) {
    super(props, context);
    this.state = {isLoggedIn: UserService.Instance.loggedIn};

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
        <a class="navbar-brand" href="#">rrf</a>
        <button class="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarNav" aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
          <span class="navbar-toggler-icon"></span>
        </button>
        <div class="collapse navbar-collapse">
          <ul class="navbar-nav mr-auto">
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>github</a>
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

  handleLogoutClick(i: Navbar, event) {
    UserService.Instance.logout();
    // i.props.history.push('/');
  }
}
