import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  LoginForm,
  RegisterForm,
  LoginResponse,
  UserOperation,
  PasswordResetForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp, validEmail } from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface State {
  loginForm: LoginForm;
  registerForm: RegisterForm;
  loginLoading: boolean;
  registerLoading: boolean;
}

export class Login extends Component<any, State> {
  private subscription: Subscription;

  emptyState: State = {
    loginForm: {
      username_or_email: undefined,
      password: undefined,
    },
    registerForm: {
      username: undefined,
      password: undefined,
      password_verify: undefined,
      admin: false,
      show_nsfw: false,
    },
    loginLoading: false,
    registerLoading: false,
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
    document.title = `${i18n.t('login')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">{this.loginForm()}</div>
          <div class="col-12 col-lg-6">{this.registerForm()}</div>
        </div>
      </div>
    );
  }

  loginForm() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleLoginSubmit)}>
          <h5>Login</h5>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">
              <T i18nKey="email_or_username">#</T>
            </label>
            <div class="col-sm-10">
              <input
                type="text"
                class="form-control"
                value={this.state.loginForm.username_or_email}
                onInput={linkEvent(this, this.handleLoginUsernameChange)}
                required
                minLength={3}
              />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">
              <T i18nKey="password">#</T>
            </label>
            <div class="col-sm-10">
              <input
                type="password"
                value={this.state.loginForm.password}
                onInput={linkEvent(this, this.handleLoginPasswordChange)}
                class="form-control"
                required
              />
              <button
                disabled={!validEmail(this.state.loginForm.username_or_email)}
                onClick={linkEvent(this, this.handlePasswordReset)}
                className="btn p-0 btn-link d-inline-block float-right text-muted small font-weight-bold"
              >
                <T i18nKey="forgot_password">#</T>
              </button>
            </div>
          </div>
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary">
                {this.state.loginLoading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : (
                  i18n.t('login')
                )}
              </button>
            </div>
          </div>
        </form>
      </div>
    );
  }
  registerForm() {
    return (
      <form onSubmit={linkEvent(this, this.handleRegisterSubmit)}>
        <h5>
          <T i18nKey="sign_up">#</T>
        </h5>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">
            <T i18nKey="username">#</T>
          </label>
          <div class="col-sm-10">
            <input
              type="text"
              class="form-control"
              value={this.state.registerForm.username}
              onInput={linkEvent(this, this.handleRegisterUsernameChange)}
              required
              minLength={3}
              maxLength={20}
              pattern="[a-zA-Z0-9_]+"
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">
            <T i18nKey="email">#</T>
          </label>
          <div class="col-sm-10">
            <input
              type="email"
              class="form-control"
              placeholder={i18n.t('optional')}
              value={this.state.registerForm.email}
              onInput={linkEvent(this, this.handleRegisterEmailChange)}
              minLength={3}
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">
            <T i18nKey="password">#</T>
          </label>
          <div class="col-sm-10">
            <input
              type="password"
              value={this.state.registerForm.password}
              onInput={linkEvent(this, this.handleRegisterPasswordChange)}
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
              value={this.state.registerForm.password_verify}
              onInput={linkEvent(this, this.handleRegisterPasswordVerifyChange)}
              class="form-control"
              required
            />
          </div>
        </div>
        {WebSocketService.Instance.site.enable_nsfw && (
          <div class="form-group row">
            <div class="col-sm-10">
              <div class="form-check">
                <input
                  class="form-check-input"
                  type="checkbox"
                  checked={this.state.registerForm.show_nsfw}
                  onChange={linkEvent(this, this.handleRegisterShowNsfwChange)}
                />
                <label class="form-check-label">
                  <T i18nKey="show_nsfw">#</T>
                </label>
              </div>
            </div>
          </div>
        )}
        <div class="form-group row">
          <div class="col-sm-10">
            <button type="submit" class="btn btn-secondary">
              {this.state.registerLoading ? (
                <svg class="icon icon-spinner spin">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              ) : (
                i18n.t('sign_up')
              )}
            </button>
          </div>
        </div>
      </form>
    );
  }

  handleLoginSubmit(i: Login, event: any) {
    event.preventDefault();
    i.state.loginLoading = true;
    i.setState(i.state);
    WebSocketService.Instance.login(i.state.loginForm);
  }

  handleLoginUsernameChange(i: Login, event: any) {
    i.state.loginForm.username_or_email = event.target.value;
    i.setState(i.state);
  }

  handleLoginPasswordChange(i: Login, event: any) {
    i.state.loginForm.password = event.target.value;
    i.setState(i.state);
  }

  handleRegisterSubmit(i: Login, event: any) {
    event.preventDefault();
    i.state.registerLoading = true;
    i.setState(i.state);

    WebSocketService.Instance.register(i.state.registerForm);
  }

  handleRegisterUsernameChange(i: Login, event: any) {
    i.state.registerForm.username = event.target.value;
    i.setState(i.state);
  }

  handleRegisterEmailChange(i: Login, event: any) {
    i.state.registerForm.email = event.target.value;
    if (i.state.registerForm.email == '') {
      i.state.registerForm.email = undefined;
    }
    i.setState(i.state);
  }

  handleRegisterPasswordChange(i: Login, event: any) {
    i.state.registerForm.password = event.target.value;
    i.setState(i.state);
  }

  handleRegisterPasswordVerifyChange(i: Login, event: any) {
    i.state.registerForm.password_verify = event.target.value;
    i.setState(i.state);
  }

  handleRegisterShowNsfwChange(i: Login, event: any) {
    i.state.registerForm.show_nsfw = event.target.checked;
    i.setState(i.state);
  }

  handlePasswordReset(i: Login) {
    event.preventDefault();
    let resetForm: PasswordResetForm = {
      email: i.state.loginForm.username_or_email,
    };
    WebSocketService.Instance.passwordReset(resetForm);
  }

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.state = this.emptyState;
      this.setState(this.state);
      return;
    } else {
      if (op == UserOperation.Login) {
        this.state = this.emptyState;
        this.setState(this.state);
        let res: LoginResponse = msg;
        UserService.Instance.login(res);
        this.props.history.push('/');
      } else if (op == UserOperation.Register) {
        this.state = this.emptyState;
        this.setState(this.state);
        let res: LoginResponse = msg;
        UserService.Instance.login(res);
        this.props.history.push('/communities');
      } else if (op == UserOperation.PasswordReset) {
        alert(i18n.t('reset_password_mail_sent'));
      }
    }
  }
}
