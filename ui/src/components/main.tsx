import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, GetCommunityResponse, CommunityResponse, Post, GetPostsForm, ListingSortType, ListingType, GetPostsResponse, CreatePostLikeForm, CreatePostLikeResponse, CommunityUser, GetFollowedCommunitiesResponse } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { MomentTime } from './moment-time';
import { PostListings } from './post-listings';
import { Sidebar } from './sidebar';
import { msgOp, mdToHtml } from '../utils';

interface State {
  subscribedCommunities: Array<CommunityUser>;
}

export class Main extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    subscribedCommunities: []
  }

  constructor(props, context) {
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
          <div class="col-12 col-lg-9">
            <PostListings />
          </div>
          <div class="col-12 col-lg-3">
            <h4>A Landing message</h4>
            {UserService.Instance.loggedIn &&
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
      this.setState(this.state);
    }
  }
}

