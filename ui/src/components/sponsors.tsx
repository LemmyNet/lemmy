import { Component } from 'inferno';
import { WebSocketService } from '../services';

let general = 
  [
  "Nathan J. Goode",
];
// let highlighted = [];
// let silver = [];
// let gold = [];
// let latinum = [];

export class Sponsors extends Component<any, any> {

  constructor(props: any, context: any) {
    super(props, context);

  }

  componentDidMount() {
    document.title = `Sponsors - ${WebSocketService.Instance.site.name}`;
  }

  render() {
    return (
      <div class="container text-center">
        {this.topMessage()}
        <hr />
        {this.sponsors()}
        <hr />
        {this.bitcoin()}
      </div>
    )
  }

  topMessage() {
    return (
      <div>
        <h5>Sponsors of Lemmy</h5>
        <p>
          Lemmy is free, <a href="https://github.com/dessalines/lemmy">open-source</a> software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project. Thank you to the following people:
        </p>
        <a class="btn btn-secondary" href="https://www.patreon.com/dessalines">Support on Patreon</a>
      </div>
    )
  }
  sponsors() {
    return (
      <div class="container">
        <h5>Sponsors</h5>
        <p>General Sponsors are those that pledged $10 to $39 to Lemmy.</p>
        <div class="row card-columns">
          {general.map(s => 
            <div class="card col-12 col-md-2">
              <div>{s}</div>
            </div>
          )}
        </div>
      </div>
    )
  }

  bitcoin() {
    return (
      <div>
      <h5>Crypto</h5>
      <div class="table-responsive">
        <table class="table table-hover text-center">
          <tbody>
          <tr>
            <td>Bitcoin</td>
            <td><code>1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK</code></td>
          </tr>
          <tr>
            <td>Ethereum</td>
            <td><code>0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01</code></td>
          </tr>
          </tbody>
        </table>
      </div>
    </div>
    )
  }
}

