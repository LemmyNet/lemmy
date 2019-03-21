import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';

export class Navbar extends Component<any, any> {

  constructor(props, context) {
    super(props, context);
  }

  render() {
    return (
      <div class="sticky-top">{this.navbar()}</div>
    )
  }

  // TODO class active corresponding to current page
  navbar() {
    return (
      <nav class="navbar navbar-light bg-light p-0 px-3 shadow">
        <a class="navbar-brand mx-1" href="#">
          rrf
        </a>
        <ul class="navbar-nav mr-auto">
          <li class="nav-item">
            <a class="nav-item nav-link" href={repoUrl}>github</a>
          </li>
        </ul>
        <ul class="navbar-nav ml-auto mr-2">
          <li class="nav-item">
            <Link class="nav-item nav-link" to="/login">Login</Link>
          </li>
        </ul>
      </nav>
    );
  }

}
