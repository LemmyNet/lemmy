import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { i18n } from '../i18next';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { WebSocketService } from '../services';
import { repoUrl, wsJsonToRes } from '../utils';
import {
  UserOperation,
  WebSocketJsonResponse,
  GetSiteResponse,
} from '../interfaces';

interface FooterState {
  version: string;
}

export class Footer extends Component<any, FooterState> {
  private wsSub: Subscription;
  emptyState: FooterState = {
    version: null,
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
  }

  componentWillUnmount() {
    this.wsSub.unsubscribe();
  }

  render() {
    return (
      <nav class="container navbar navbar-expand-md navbar-light navbar-bg p-0 px-3 mt-2">
        <div className="navbar-collapse">
          <ul class="navbar-nav ml-auto">
            <li class="nav-item">
              <span class="navbar-text">{this.state.version}</span>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/modlog">
                {i18n.t('modlog')}
              </Link>
            </li>
            <li class="nav-item">
              <Link class="nav-link" to="/instances">
                {i18n.t('instances')}
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
  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);

    if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.setState({ version: data.version });
    }
  }
}
