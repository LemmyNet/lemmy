export interface LoginForm {
  username: string;
  password: string;
}
export interface RegisterForm {
  username: string;
  email?: string;
  password: string;
  password_verify: string;
}

export enum UserOperation {
  Login, Register
}
