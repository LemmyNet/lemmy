import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, CommunityUser, GetFollowedCommunitiesResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { msgOp } from '../utils';

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
          <div class="col-12 col-md-9">
            <PostListings />
          </div>
          <div class="col-12 col-md-3">
            <h4>A Landing message</h4>
            {UserService.Instance.loggedIn &&
              <div>
                {this.state.loading ? 
                <h4 class="mt-3"><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h4> : 
                <div>
                  <hr />
                  <h4>Subscribed forums</h4>
                  <ul class="list-unstyled"> 
                    {this.state.subscribedCommunities.map(community =>
                      <li><Link to={`/community/${community.community_id}`}>{community.community_name}</Link></li>
                    )}
                  </ul>
                </div>
                }
              </div>
            }
          </div>
        </div>
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

