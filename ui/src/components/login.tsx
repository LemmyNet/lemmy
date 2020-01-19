import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  LoginForm,
  RegisterForm,
  LoginResponse,
  UserOperation,
  PasswordResetForm,
  GetSiteResponse,
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
  enable_nsfw: boolean;
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
    enable_nsfw: undefined,
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

    WebSocketService.Instance.getSite();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
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
          <h2>{ i18n.t('login') }</h2>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label" for="login-email-or-username">
                { i18n.t('email_or_username') }
            </label>
            <div class="col-sm-10">
              <input
                type="text"
                class="form-control"
                id="login-email-or-username"
                value={this.state.loginForm.username_or_email}
                onInput={linkEvent(this, this.handleLoginUsernameChange)}
                required
                minLength={3}
              />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label" for="login-password">
                { i18n.t('password') }
            </label>
            <div class="col-sm-10">
              <input
                type="password"
                id="login-password"
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
                { i18n.t('forgot_password') }
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
        <h2>
          { i18n.t('sign_up') }
        </h2>

        <div class="form-group row">
          <label class="col-sm-2 col-form-label" for="register-username">
            { i18n.t('username') }
          </label>

          <div class="col-sm-10">
            <input
              type="text"
              id="register-username"
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
          <label class="col-sm-2 col-form-label" for="register-email">
            { i18n.t('email') }
          </label>
          <div class="col-sm-10">
            <input
              type="email"
              id="register-email"
              class="form-control"
              placeholder={i18n.t('optional')}
              value={this.state.registerForm.email}
              onInput={linkEvent(this, this.handleRegisterEmailChange)}
              minLength={3}
            />
          </div>
        </div>

        <div class="form-group row">
          <label class="col-sm-2 col-form-label" for="register-password">
            { i18n.t('password') }
          </label>
          <div class="col-sm-10">
            <input
              type="password"
              id="register-password"
              value={this.state.registerForm.password}
              onInput={linkEvent(this, this.handleRegisterPasswordChange)}
              class="form-control"
              required
            />
          </div>
        </div>

        <div class="form-group row">
          <label class="col-sm-2 col-form-label" for="register-verify-password">
            { i18n.t('verify_password') }
          </label>
          <div class="col-sm-10">
            <input
              type="password"
              id="register-verify-password"
              value={this.state.registerForm.password_verify}
              onInput={linkEvent(this, this.handleRegisterPasswordVerifyChange)}
              class="form-control"
              required
            />
          </div>
        </div>

        { this.state.enable_nsfw && (
          <div class="form-group row">
            <div class="col-sm-10">
              <div class="form-check">
                <input
                  class="form-check-input"
                  id="register-show-nsfw"
                  type="checkbox"
                  checked={this.state.registerForm.show_nsfw}
                  onChange={linkEvent(this, this.handleRegisterShowNsfwChange)}
                />
                <label class="form-check-label" for="register-show-nsfw">
                    { i18n.t('show_nsfw') }
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
      } else if (op == UserOperation.GetSite) {
        let res: GetSiteResponse = msg;
        this.state.enable_nsfw = res.site.enable_nsfw;
        this.setState(this.state);
        document.title = `${i18n.t('login')} - ${
          WebSocketService.Instance.site.name
        }`;
      }
    }
  }
}
