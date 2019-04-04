import { render, Component } from 'inferno';
import { HashRouter, Route, Switch } from 'inferno-router';

import { Navbar } from './components/navbar';
import { Home } from './components/home';
import { Login } from './components/login';
import { CreatePost } from './components/create-post';
import { CreateCommunity } from './components/create-community';
import { Post } from './components/post';
import { Community } from './components/community';
import { Communities } from './components/communities';

import './main.css';

import { WebSocketService, UserService } from './services';

const container = document.getElementById('app');

class Index extends Component<any, any> {

  constructor(props, context) {
    super(props, context);
    WebSocketService.Instance;
    UserService.Instance;
  }

  render() {
    return (
      <HashRouter>
        <Navbar />
        <div class="mt-3 p-0">
          <Switch>
            <Route exact path="/" component={Home} />
            <Route path={`/login`} component={Login} />
            <Route path={`/create_post`} component={CreatePost} />
            <Route path={`/create_community`} component={CreateCommunity} />
            <Route path={`/communities`} component={Communities} />
            <Route path={`/post/:id`} component={Post} />
            <Route path={`/community/:id`} component={Community} />
          </Switch>
        </div>
      </HashRouter>
    );
  }
}

render(<Index />, container);
