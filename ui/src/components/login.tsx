import { Component, linkEvent } from 'inferno';

import { LoginForm, RegisterForm } from '../interfaces';
import { WebSocketService } from '../services';

interface State {
  loginForm: LoginForm;
  registerForm: RegisterForm;
}

let emptyState: State = {
  loginForm: {
    username: null,
    password: null
  },
  registerForm: {
    username: null,
    password: null,
    password_verify: null
  }
}

export class Login extends Component<any, State> {

  constructor(props, context) {
    super(props, context);

    this.state = emptyState;

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
          <h3>Login</h3>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Email or Username</label>
            <div class="col-sm-10">
              <input type="text" class="form-control" value={this.state.loginForm.username} onInput={linkEvent(this, this.handleLoginUsernameChange)} required minLength={3} />
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
              <button type="submit" class="btn btn-secondary">Login</button>
            </div>
          </div>
        </form>
        Forgot your password or deleted your account? Reset your password. TODO
      </div>
    );
  }
  registerForm() {
    return (
      <form onSubmit={linkEvent(this, this.handleRegisterSubmit)}>
        <h3>Sign Up</h3>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Username</label>
          <div class="col-sm-10">
            <input type="text" class="form-control" value={this.state.registerForm.username} onInput={linkEvent(this, this.handleRegisterUsernameChange)} required minLength={3} />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-sm-2 col-form-label">Email</label>
          <div class="col-sm-10">
            <input type="email" class="form-control" value={this.state.registerForm.email} onInput={linkEvent(this, this.handleRegisterEmailChange)} minLength={3} />
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
            <button type="submit" class="btn btn-secondary">Sign Up</button>
          </div>
        </div>
      </form>
    );
  }

  handleLoginSubmit(i: Login, event) {
    console.log(i.state);
    event.preventDefault();
    WebSocketService.Instance.login(i.state.loginForm);
  }

  handleLoginUsernameChange(i: Login, event) {
    i.state.loginForm.username = event.target.value;
  }

  handleLoginPasswordChange(i: Login, event) {
    i.state.loginForm.password = event.target.value;
  }

  handleRegisterSubmit(i: Login, event) {
    console.log(i.state);
    event.preventDefault();
    WebSocketService.Instance.register(i.state.registerForm);
  }

  handleRegisterUsernameChange(i: Login, event) {
    i.state.registerForm.username = event.target.value;
  }

  handleRegisterEmailChange(i: Login, event) {
    i.state.registerForm.email = event.target.value;
  }

  handleRegisterPasswordChange(i: Login, event) {
    i.state.registerForm.password = event.target.value;
  }
  
  handleRegisterPasswordVerifyChange(i: Login, event) {
    i.state.registerForm.password_verify = event.target.value;
  }
}
