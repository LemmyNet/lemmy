import { Component } from 'inferno';
import { WebSocketService } from '../services';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

let general = ['Andre Vallestero', 'riccardo', 'NotTooHighToHack'];
let highlighted = ['Alex Benishek'];
// let silver = [];
// let gold = [];
// let latinum = [];

export class Sponsors extends Component<any, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  componentDidMount() {
    document.title = `${i18n.t('sponsors')} - ${
      WebSocketService.Instance.site.name
    }`;
    window.scrollTo(0, 0);
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
    );
  }

  topMessage() {
    return (
      <div>
        <h5>
          <T i18nKey="donate_to_lemmy">#</T>
        </h5>
        <p>
          <T i18nKey="sponsor_message">
            #<a href="https://github.com/dessalines/lemmy">#</a>
          </T>
        </p>
        <a class="btn btn-secondary" href="https://www.patreon.com/dessalines">
          <T i18nKey="support_on_patreon">#</T>
        </a>
      </div>
    );
  }
  sponsors() {
    return (
      <div class="container">
        <h5>
          <T i18nKey="sponsors">#</T>
        </h5>
        <p>
          <T i18nKey="general_sponsors">#</T>
        </p>
        <div class="row card-columns">
          {highlighted.map(s => (
            <div class="card bg-primary col-12 col-md-2 font-weight-bold">
              <div>{s}</div>
            </div>
          ))}
          {general.map(s => (
            <div class="card col-12 col-md-2">
              <div>{s}</div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  bitcoin() {
    return (
      <div>
        <h5>
          <T i18nKey="crypto">#</T>
        </h5>
        <div class="table-responsive">
          <table class="table table-hover text-center">
            <tbody>
              <tr>
                <td>
                  <T i18nKey="bitcoin">#</T>
                </td>
                <td>
                  <code>1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK</code>
                </td>
              </tr>
              <tr>
                <td>
                  <T i18nKey="ethereum">#</T>
                </td>
                <td>
                  <code>0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01</code>
                </td>
              </tr>
              <tr>
                <td>
                  <T i18nKey="monero">#</T>
                </td>
                <td>
                  <code>
                    41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV
                  </code>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    );
  }
}
