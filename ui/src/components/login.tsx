import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { LoginForm, RegisterForm, LoginResponse, UserOperation } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

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
      password: undefined
    },
    registerForm: {
      username: undefined,
      password: undefined,
      password_verify: undefined,
      admin: false,
    },
    loginLoading: false,
    registerLoading: false,
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log("complete")
    );
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = "Login - Lemmy";
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            {this.loginForm()}
          </div>
          <div class="col-12 col-lg-6">
            {this.registerForm()}
          </div>
        </div>
      </div>
    )
  }

  loginForm() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleLoginSubmit)}>
          <h5>Login</h5>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Email or Username</label>
            <div class="col-sm-10">
              <input type="text" class="form-control" value={this.state.loginForm.username_or_email} onInput={linkEvent(this, this.handleLoginUsernameChange)} required minLength={3} />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Password</label>
            <div class="col-sm-10">
              <input type="password" value={this.state.loginForm.password} onInput={linkEvent(this, this.handleLoginPasswordChange)} class="form-control" required />
            </div>
          </div>
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary">{this.state.loginLoading ? 
              <svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg> : 'Login'}</button>
            </div>
          </div>
        </form>
        {/* Forgot your password or deleted your account? Reset your password. TODO */}
      </div>
    );
  }
  registerForm() {
    return (
      <form onSubmit={linkEvent(this, this.handleRegisterSubmit)}>
        <h5>Sign Up</h5>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Username</label>
          <div class="col-sm-10">
            <input type="text" class="form-control" value={this.state.registerForm.username} onInput={linkEvent(this, this.handleRegisterUsernameChange)} required minLength={3} maxLength={20} pattern="[a-zA-Z0-9_]+" />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Email</label>
          <div class="col-sm-10">
            <input type="email" class="form-control" placeholder="Optional" value={this.state.registerForm.email} onInput={linkEvent(this, this.handleRegisterEmailChange)} minLength={3} />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Password</label>
          <div class="col-sm-10">
            <input type="password" value={this.state.registerForm.password} onInput={linkEvent(this, this.handleRegisterPasswordChange)} class="form-control" required />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Verify Password</label>
          <div class="col-sm-10">
            <input type="password" value={this.state.registerForm.password_verify} onInput={linkEvent(this, this.handleRegisterPasswordVerifyChange)} class="form-control" required />
          </div>
        </div>
        <div class="form-group row">
          <div class="col-sm-10">
            <button type="submit" class="btn btn-secondary">{this.state.registerLoading ? 
            <svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg> : 'Sign Up'}</button>

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

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
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
      }
    }
  }

}
