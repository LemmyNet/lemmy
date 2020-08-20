import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  SiteResponse,
  GetSiteResponse,
  SiteConfigForm,
  GetSiteConfigResponse,
  WebSocketJsonResponse,
} from 'lemmy-js-client';
import { WebSocketService } from '../services';
import { wsJsonToRes, capitalizeFirstLetter, toast, randomStr } from '../utils';
import autosize from 'autosize';
import { SiteForm } from './site-form';
import { UserListing } from './user-listing';
import { i18n } from '../i18next';

interface AdminSettingsState {
  siteRes: GetSiteResponse;
  siteConfigRes: GetSiteConfigResponse;
  siteConfigForm: SiteConfigForm;
  loading: boolean;
  siteConfigLoading: boolean;
}

export class AdminSettings extends Component<any, AdminSettingsState> {
  private siteConfigTextAreaId = `site-config-${randomStr()}`;
  private subscription: Subscription;
  private emptyState: AdminSettingsState = {
    siteRes: {
      site: {
        id: null,
        name: null,
        creator_id: null,
        creator_name: null,
        published: null,
        number_of_users: null,
        number_of_posts: null,
        number_of_comments: null,
        number_of_communities: null,
        enable_downvotes: null,
        open_registration: null,
        enable_nsfw: null,
      },
      admins: [],
      banned: [],
      online: null,
      version: null,
      federated_instances: null,
    },
    siteConfigForm: {
      config_hjson: null,
      auth: null,
    },
    siteConfigRes: {
      config_hjson: null,
    },
    loading: true,
    siteConfigLoading: null,
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
    WebSocketService.Instance.getSiteConfig();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  get documentTitle(): string {
    if (this.state.siteRes.site.name) {
      return `${i18n.t('admin_settings')} - ${this.state.siteRes.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle} />
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div class="row">
            <div class="col-12 col-md-6">
              {this.state.siteRes.site.id && (
                <SiteForm site={this.state.siteRes.site} />
              )}
              {this.admins()}
              {this.bannedUsers()}
            </div>
            <div class="col-12 col-md-6">{this.adminSettings()}</div>
          </div>
        )}
      </div>
    );
  }

  admins() {
    return (
      <>
        <h5>{capitalizeFirstLetter(i18n.t('admins'))}</h5>
        <ul class="list-unstyled">
          {this.state.siteRes.admins.map(admin => (
            <li class="list-inline-item">
              <UserListing
                user={{
                  name: admin.name,
                  preferred_username: admin.preferred_username,
                  avatar: admin.avatar,
                  id: admin.id,
                  local: admin.local,
                  actor_id: admin.actor_id,
                }}
              />
            </li>
          ))}
        </ul>
      </>
    );
  }

  bannedUsers() {
    return (
      <>
        <h5>{i18n.t('banned_users')}</h5>
        <ul class="list-unstyled">
          {this.state.siteRes.banned.map(banned => (
            <li class="list-inline-item">
              <UserListing
                user={{
                  name: banned.name,
                  preferred_username: banned.preferred_username,
                  avatar: banned.avatar,
                  id: banned.id,
                  local: banned.local,
                  actor_id: banned.actor_id,
                }}
              />
            </li>
          ))}
        </ul>
      </>
    );
  }

  adminSettings() {
    return (
      <div>
        <h5>{i18n.t('admin_settings')}</h5>
        <form onSubmit={linkEvent(this, this.handleSiteConfigSubmit)}>
          <div class="form-group row">
            <label
              class="col-12 col-form-label"
              htmlFor={this.siteConfigTextAreaId}
            >
              {i18n.t('site_config')}
            </label>
            <div class="col-12">
              <textarea
                id={this.siteConfigTextAreaId}
                value={this.state.siteConfigForm.config_hjson}
                onInput={linkEvent(this, this.handleSiteConfigHjsonChange)}
                class="form-control text-monospace"
                rows={3}
              />
            </div>
          </div>
          <div class="form-group row">
            <div class="col-12">
              <button type="submit" class="btn btn-secondary mr-2">
                {this.state.siteConfigLoading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : (
                  capitalizeFirstLetter(i18n.t('save'))
                )}
              </button>
            </div>
          </div>
        </form>
      </div>
    );
  }

  handleSiteConfigSubmit(i: AdminSettings, event: any) {
    event.preventDefault();
    i.state.siteConfigLoading = true;
    WebSocketService.Instance.saveSiteConfig(i.state.siteConfigForm);
    i.setState(i.state);
  }

  handleSiteConfigHjsonChange(i: AdminSettings, event: any) {
    i.state.siteConfigForm.config_hjson = event.target.value;
    i.setState(i.state);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.context.router.history.push('/');
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (msg.reconnect) {
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;

      // This means it hasn't been set up yet
      if (!data.site) {
        this.context.router.history.push('/setup');
      }
      this.state.siteRes = data;
      this.setState(this.state);
    } else if (res.op == UserOperation.EditSite) {
      let data = res.data as SiteResponse;
      this.state.siteRes.site = data.site;
      this.setState(this.state);
      toast(i18n.t('site_saved'));
    } else if (res.op == UserOperation.GetSiteConfig) {
      let data = res.data as GetSiteConfigResponse;
      this.state.siteConfigRes = data;
      this.state.loading = false;
      this.state.siteConfigForm.config_hjson = this.state.siteConfigRes.config_hjson;
      this.setState(this.state);
      var textarea: any = document.getElementById(this.siteConfigTextAreaId);
      autosize(textarea);
    } else if (res.op == UserOperation.SaveSiteConfig) {
      let data = res.data as GetSiteConfigResponse;
      this.state.siteConfigRes = data;
      this.state.siteConfigForm.config_hjson = this.state.siteConfigRes.config_hjson;
      this.state.siteConfigLoading = false;
      toast(i18n.t('site_saved'));
      this.setState(this.state);
    }
  }
}
