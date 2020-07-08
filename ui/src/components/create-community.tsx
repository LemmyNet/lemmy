import { Component } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { CommunityForm } from './community-form';
import {
  Community,
  UserOperation,
  WebSocketJsonResponse,
  GetSiteResponse,
} from '../interfaces';
import { toast, wsJsonToRes } from '../utils';
import { WebSocketService } from '../services';
import { i18n } from '../i18next';

interface CreateCommunityState {
  enableNsfw: boolean;
}

export class CreateCommunity extends Component<any, CreateCommunityState> {
  private subscription: Subscription;
  private emptyState: CreateCommunityState = {
    enableNsfw: null,
  };
  constructor(props: any, context: any) {
    super(props, context);
    this.handleCommunityCreate = this.handleCommunityCreate.bind(this);
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

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 offset-lg-3 mb-4">
            <h5>{i18n.t('create_community')}</h5>
            <CommunityForm
              onCreate={this.handleCommunityCreate}
              enableNsfw={this.state.enableNsfw}
            />
          </div>
        </div>
      </div>
    );
  }

  handleCommunityCreate(community: Community) {
    this.props.history.push(`/c/${community.name}`);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.enableNsfw = data.site.enable_nsfw;
      this.setState(this.state);
      document.title = `${i18n.t('create_community')} - ${data.site.name}`;
    }
  }
}
