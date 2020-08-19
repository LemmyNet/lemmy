import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  LoginResponse,
  PasswordChangeForm,
  WebSocketJsonResponse,
  GetSiteResponse,
  Site,
} from 'lemmy-js-client';
import { WebSocketService, UserService } from '../services';
import { wsJsonToRes, capitalizeFirstLetter, toast } from '../utils';
import { i18n } from '../i18next';

interface State {
  passwordChangeForm: PasswordChangeForm;
  loading: boolean;
  site: Site;
}

export class PasswordChange extends Component<any, State> {
  private subscription: Subscription;

  emptyState: State = {
    passwordChangeForm: {
      token: this.props.match.params.token,
      password: undefined,
      password_verify: undefined,
    },
    loading: false,
    site: undefined,
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
    if (this.state.site) {
      return `${i18n.t('password_change')} - ${this.state.site.name}`;
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
            <h5>{i18n.t('password_change')}</h5>
            {this.passwordChangeForm()}
          </div>
        </div>
      </div>
    );
  }

  passwordChangeForm() {
    return (
      <form onSubmit={linkEvent(this, this.handlePasswordChangeSubmit)}>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">
            {i18n.t('new_password')}
          </label>
          <div class="col-sm-10">
            <input
              type="password"
              value={this.state.passwordChangeForm.password}
              onInput={linkEvent(this, this.handlePasswordChange)}
              class="form-control"
              required
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">
            {i18n.t('verify_password')}
          </label>
          <div class="col-sm-10">
            <input
              type="password"
              value={this.state.passwordChangeForm.password_verify}
              onInput={linkEvent(this, this.handleVerifyPasswordChange)}
              class="form-control"
              required
            />
          </div>
        </div>
        <div class="form-group row">
          <div class="col-sm-10">
            <button type="submit" class="btn btn-secondary">
              {this.state.loading ? (
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
    );
  }

  handlePasswordChange(i: PasswordChange, event: any) {
    i.state.passwordChangeForm.password = event.target.value;
    i.setState(i.state);
  }

  handleVerifyPasswordChange(i: PasswordChange, event: any) {
    i.state.passwordChangeForm.password_verify = event.target.value;
    i.setState(i.state);
  }

  handlePasswordChangeSubmit(i: PasswordChange, event: any) {
    event.preventDefault();
    i.state.loading = true;
    i.setState(i.state);

    WebSocketService.Instance.passwordChange(i.state.passwordChangeForm);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (res.op == UserOperation.PasswordChange) {
      let data = res.data as LoginResponse;
      this.state = this.emptyState;
      this.setState(this.state);
      UserService.Instance.login(data);
      this.props.history.push('/');
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }
}
