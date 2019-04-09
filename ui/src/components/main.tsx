import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, CommunityUser, GetFollowedCommunitiesResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { msgOp, repoUrl } from '../utils';

interface State {
  subscribedCommunities: Array<CommunityUser>;
  loading: boolean;
}

export class Main extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    subscribedCommunities: [],
    loading: true
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    if (UserService.Instance.loggedIn) {
      WebSocketService.Instance.getFollowedCommunities();
    }
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-md-8">
            <PostListings />
          </div>
          <div class="col-12 col-md-4">
            {UserService.Instance.loggedIn ?
              <div>
                {this.state.loading ? 
                <h4><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h4> : 
                <div>
                  <h4>Subscribed forums</h4>
                  <ul class="list-unstyled"> 
                    {this.state.subscribedCommunities.map(community =>
                      <li><Link to={`/community/${community.community_id}`}>{community.community_name}</Link></li>
                    )}
                  </ul>
                </div>
                }
              </div> :
            this.landing()
            }
          </div>
        </div>
      </div>
    )
  }

  landing() {
    return (
      <div>
        <h4>Welcome to 
          <svg class="icon mx-2"><use xlinkHref="#icon-mouse"></use></svg>
          <a href={repoUrl}>Lemmy<sup>Beta</sup></a>
        </h4>
        <p>Lemmy is a <a href="https://en.wikipedia.org/wiki/Link_aggregation">link aggregator</a> / reddit alternative, intended to work in the <a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>.</p>
        <p>Its self-hostable, has live-updating comment threads, and is tiny (<code>~80kB</code>). Federation into the ActivityPub network is on the roadmap.</p>
        <p>This is a <b>very early beta version</b>, and a lot of features are currently broken or missing.</p>
        <p>Suggest new features or report bugs <a href={repoUrl}>here.</a></p>
        <p>Made with <a href="https://www.rust-lang.org">Rust</a>, <a href="https://actix.rs/">Actix</a>, <a href="https://www.infernojs.org">Inferno</a>, <a href="https://www.typescriptlang.org/">Typescript</a>.</p>
      </div>
    )
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetFollowedCommunities) {
      let res: GetFollowedCommunitiesResponse = msg;
      this.state.subscribedCommunities = res.communities;
      this.state.loading = false;
      this.setState(this.state);
    }
  }
}

