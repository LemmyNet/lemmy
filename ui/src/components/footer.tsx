import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { version } from '../version';

export class Footer extends Component<any, any> {


  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <nav title={version} class="container navbar navbar-expand-md navbar-light navbar-bg p-0 px-3 my-2">
        <div className="navbar-collapse">
          <ul class="navbar-nav ml-auto">
            <li class="nav-item">
              <Link class="nav-link" to="/modlog">Modlog</Link>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>Contribute</a>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>Code</a>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>About</a>
            </li>
          </ul>
        </div>
      </nav>
    );
  }
}

