import { Component } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { CommunityForm } from './community-form';
import {
  Community,
  UserOperation,
  WebSocketJsonResponse,
  GetSiteResponse,
  Site,
} from 'lemmy-js-client';
import { toast, wsJsonToRes } from '../utils';
import { WebSocketService, UserService } from '../services';
import { i18n } from '../i18next';

interface CreateCommunityState {
  site: Site;
}

export class CreateCommunity extends Component<any, CreateCommunityState> {
  private subscription: Subscription;
  private emptyState: CreateCommunityState = {
    site: {
      id: undefined,
      name: undefined,
      creator_id: undefined,
      published: undefined,
      creator_name: undefined,
      number_of_users: undefined,
      number_of_posts: undefined,
      number_of_comments: undefined,
      number_of_communities: undefined,
      enable_downvotes: undefined,
      open_registration: undefined,
      enable_nsfw: undefined,
    },
  };
  constructor(props: any, context: any) {
    super(props, context);
    this.handleCommunityCreate = this.handleCommunityCreate.bind(this);
    this.state = this.emptyState;

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
    if (this.state.site.name) {
      return `${i18n.t('create_community')} - ${this.state.site.name}`;
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
            <h5>{i18n.t('create_community')}</h5>
            <CommunityForm
              onCreate={this.handleCommunityCreate}
              enableNsfw={this.state.site.enable_nsfw}
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
      // Toast errors are already handled by community-form
      return;
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }
}
