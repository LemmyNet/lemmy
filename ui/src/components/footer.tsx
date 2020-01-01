import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { version } from '../version';
import { T } from 'inferno-i18next';

export class Footer extends Component<any, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <nav class="container navbar navbar-expand-md navbar-light navbar-bg p-0 px-3 mt-2">
        <div className="navbar-collapse">
          <ul class="navbar-nav ml-auto">
            <li class="nav-item">
              <span class="navbar-text">{version}</span>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/modlog">
                <T i18nKey="modlog">#</T>
              </Link>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={'/docs/index.html'}>
                <T i18nKey="docs">#</T>
              </a>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/sponsors">
                <T i18nKey="sponsors">#</T>
              </Link>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>
                <T i18nKey="code">#</T>
              </a>
            </li>
          </ul>
        </div>
      </nav>
    );
  }
}
