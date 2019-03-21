import { render, Component } from 'inferno';
import { HashRouter, Route, Switch } from 'inferno-router';

import { Navbar } from './components/navbar';
import { Home } from './components/home';
import { Login } from './components/login';

import './main.css';

import { WebSocketService } from './services';

const container = document.getElementById('app');

class Index extends Component<any, any> {

  constructor(props, context) {
    super(props, context);
    WebSocketService.Instance;
  }

  render() {
    return (
      <HashRouter>
        <Navbar />
        <div class="mt-3 p-0">
          <Switch>
            <Route exact path="/" component={Home} />
            <Route path={`/login`} component={Login} />
            {/*
            <Route path={`/search/:type_/:q/:page`} component={Search} />
            <Route path={`/submit`} component={Submit} />
            <Route path={`/user/:id`} component={Login} />
            <Route path={`/community/:id`} component={Login} /> 
            */}
          </Switch>
        </div>
      </HashRouter>
    );
  }
}

render(<Index />, container);
