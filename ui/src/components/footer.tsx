import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { repoUrl } from '../utils';
import { version } from '../version';
import { i18n } from '../i18next';

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
                {i18n.t('modlog')}
              </Link>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={'/docs/index.html'}>
                {i18n.t('docs')}
              </a>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/sponsors">
                {i18n.t('donate')}
              </Link>
            </li>
            <li class="nav-item">
              <a class="nav-link" href={repoUrl}>
                {i18n.t('code')}
              </a>
            </li>
          </ul>
        </div>
      </nav>
    );
  }
}
