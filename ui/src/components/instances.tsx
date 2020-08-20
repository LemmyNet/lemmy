import { Component } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  WebSocketJsonResponse,
  GetSiteResponse,
} from 'lemmy-js-client';
import { WebSocketService } from '../services';
import { wsJsonToRes, toast } from '../utils';
import { i18n } from '../i18next';

interface InstancesState {
  loading: boolean;
  siteRes: GetSiteResponse;
}

export class Instances extends Component<any, InstancesState> {
  private subscription: Subscription;
  private emptyState: InstancesState = {
    loading: true,
    siteRes: undefined,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getSite();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  get documentTitle(): string {
    if (this.state.siteRes) {
      return `${i18n.t('instances')} - ${this.state.siteRes.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle} />
        {this.state.loading ? (
          <h5 class="">
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div>
            <h5>{i18n.t('linked_instances')}</h5>
            {this.state.siteRes &&
            this.state.siteRes.federated_instances.length ? (
              <ul>
                {this.state.siteRes.federated_instances.map(i => (
                  <li>
                    <a href={`https://${i}`} target="_blank" rel="noopener">
                      {i}
                    </a>
                  </li>
                ))}
              </ul>
            ) : (
              <div>{i18n.t('none_found')}</div>
            )}
          </div>
        )}
      </div>
    );
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.siteRes = data;
      this.state.loading = false;
      this.setState(this.state);
    }
  }
}
