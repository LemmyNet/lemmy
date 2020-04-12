import { Component } from 'inferno';
import { WebSocketService } from '../services';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';
import { repoUrl } from '../utils';

let general = [
  'Nathan J. Goode',
  'Andre Vallestero',
  'riccardo',
  'NotTooHighToHack',
];
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
        <h5>{i18n.t('donate_to_lemmy')}</h5>
        <p>
          <T i18nKey="sponsor_message">
            #<a href={repoUrl}>#</a>
          </T>
        </p>
        <a class="btn btn-secondary" href="https://liberapay.com/Lemmy/">
          {i18n.t('support_on_liberapay')}
        </a>
        <a
          class="btn btn-secondary ml-2"
          href="https://www.patreon.com/dessalines"
        >
          {i18n.t('support_on_patreon')}
        </a>
        <a
          class="btn btn-secondary ml-2"
          href="https://opencollective.com/lemmy"
        >
          {i18n.t('support_on_open_collective')}
        </a>
      </div>
    );
  }
  sponsors() {
    return (
      <div class="container">
        <h5>{i18n.t('sponsors')}</h5>
        <p>{i18n.t('general_sponsors')}</p>
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
        <h5>{i18n.t('crypto')}</h5>
        <div class="table-responsive">
          <table class="table table-hover text-center">
            <tbody>
              <tr>
                <td>{i18n.t('bitcoin')}</td>
                <td>
                  <code>1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK</code>
                </td>
              </tr>
              <tr>
                <td>{i18n.t('ethereum')}</td>
                <td>
                  <code>0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01</code>
                </td>
              </tr>
              <tr>
                <td>{i18n.t('monero')}</td>
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
