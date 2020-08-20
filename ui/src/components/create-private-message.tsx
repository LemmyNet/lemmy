import { Component } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { PrivateMessageForm } from './private-message-form';
import { WebSocketService, UserService } from '../services';
import {
  UserOperation,
  WebSocketJsonResponse,
  GetSiteResponse,
  Site,
  PrivateMessageFormParams,
} from 'lemmy-js-client';
import { toast, wsJsonToRes } from '../utils';
import { i18n } from '../i18next';

interface CreatePrivateMessageState {
  site: Site;
}

export class CreatePrivateMessage extends Component<
  any,
  CreatePrivateMessageState
> {
  private subscription: Subscription;
  private emptyState: CreatePrivateMessageState = {
    site: undefined,
  };
  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.handlePrivateMessageCreate = this.handlePrivateMessageCreate.bind(
      this
    );

    if (!UserService.Instance.user) {
      toast(i18n.t('not_logged_in'), 'danger');
      this.context.router.history.push(`/login`);
    }

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
    if (this.state.site) {
      return `${i18n.t('create_private_message')} - ${this.state.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle} />
        <div class="row">
          <div class="col-12 col-lg-6 offset-lg-3 mb-4">
            <h5>{i18n.t('create_private_message')}</h5>
            <PrivateMessageForm
              onCreate={this.handlePrivateMessageCreate}
              params={this.params}
            />
          </div>
        </div>
      </div>
    );
  }

  get params(): PrivateMessageFormParams {
    let urlParams = new URLSearchParams(this.props.location.search);
    let params: PrivateMessageFormParams = {
      recipient_id: Number(urlParams.get('recipient_id')),
    };

    return params;
  }

  handlePrivateMessageCreate() {
    toast(i18n.t('message_sent'));

    // Navigate to the front
    this.props.history.push(`/`);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }
}
