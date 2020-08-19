import { Component } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { WebSocketService } from '../services';
import {
  GetSiteResponse,
  Site,
  WebSocketJsonResponse,
  UserOperation,
} from 'lemmy-js-client';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';
import { repoUrl, wsJsonToRes, toast } from '../utils';

interface SilverUser {
  name: string;
  link?: string;
}

let general = [
  'Brendan',
  'mexicanhalloween',
  'William Moore',
  'Rachel Schmitz',
  'comradeda',
  'ybaumy',
  'dude in phx',
  'twilight loki',
  'Andrew Plaza',
  'Jonathan Cremin',
  'Arthur Nieuwland',
  'Ernest Wiśniewski',
  'HN',
  'Forrest Weghorst',
  'Andre Vallestero',
  'NotTooHighToHack',
];
let highlighted = ['DQW', 'DiscountFuneral', 'Oskenso Kashi', 'Alex Benishek'];
let silver: Array<SilverUser> = [
  {
    name: 'Redjoker',
    link: 'https://iww.org',
  },
];
// let gold = [];
// let latinum = [];

interface SponsorsState {
  site: Site;
}

export class Sponsors extends Component<any, SponsorsState> {
  private subscription: Subscription;
  private emptyState: SponsorsState = {
    site: undefined,
  };
  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getSite();
  }

  componentDidMount() {
    window.scrollTo(0, 0);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  get documentTitle(): string {
    if (this.state.site) {
      return `${i18n.t('sponsors')} - ${this.state.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  render() {
    return (
      <div class="container text-center">
        <Helmet title={this.documentTitle} />
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
        <p>{i18n.t('silver_sponsors')}</p>
        <div class="row justify-content-md-center card-columns">
          {silver.map(s => (
            <div class="card col-12 col-md-2">
              <div>
                {s.link ? (
                  <a href={s.link} target="_blank" rel="noopener">
                    💎 {s.name}
                  </a>
                ) : (
                  <div>💎 {s.name}</div>
                )}
              </div>
            </div>
          ))}
        </div>
        <p>{i18n.t('general_sponsors')}</p>
        <div class="row justify-content-md-center card-columns">
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

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }
}
