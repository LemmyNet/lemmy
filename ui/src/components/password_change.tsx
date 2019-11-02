import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  LoginResponse,
  PasswordChangeForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp, capitalizeFirstLetter } from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface State {
  passwordChangeForm: PasswordChangeForm;
  loading: boolean;
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
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
      .pipe(
        retryWhen(errors =>
          errors.pipe(
            delay(3000),
            take(10)
          )
        )
      )
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = `${i18n.t('password_change')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 offset-lg-3 mb-4">
            <h5>
              <T i18nKey="password_change">#</T>
            </h5>
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
            <T i18nKey="new_password">#</T>
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
            <T i18nKey="verify_password">#</T>
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

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else {
      if (op == UserOperation.PasswordChange) {
        this.state = this.emptyState;
        this.setState(this.state);
        let res: LoginResponse = msg;
        UserService.Instance.login(res);
        this.props.history.push('/');
      }
    }
  }
}
