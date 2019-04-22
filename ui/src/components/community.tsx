import { Component } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, GetCommunityResponse, CommunityResponse,  CommunityUser, UserView } from '../interfaces';
import { WebSocketService } from '../services';
import { PostListings } from './post-listings';
import { Sidebar } from './sidebar';
import { msgOp } from '../utils';

interface State {
  community: CommunityI;
  communityId: number;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  loading: boolean;
}

export class Community extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    community: {
      id: null,
      name: null,
      title: null,
      category_id: null,
      category_name: null,
      creator_id: null,
      creator_name: null,
      number_of_subscribers: null,
      number_of_posts: null,
      number_of_comments: null,
      published: null,
      removed: null,
    },
    moderators: [],
    admins: [],
    communityId: Number(this.props.match.params.id),
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

    WebSocketService.Instance.getCommunity(this.state.communityId);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? 
        <h5><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h5> : 
        <div class="row">
          <div class="col-12 col-md-9">
            <h5>{this.state.community.title}
            {this.state.community.removed &&
              <small className="ml-2 text-muted font-italic">removed</small>
            }
          </h5>
            <PostListings communityId={this.state.communityId} />
          </div>
          <div class="col-12 col-md-3">
            <Sidebar 
              community={this.state.community} 
              moderators={this.state.moderators} 
              admins={this.state.admins}
            />
          </div>
        </div>
        }
      </div>
    )
  }


  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetCommunity) {
      let res: GetCommunityResponse = msg;
      this.state.community = res.community;
      this.state.moderators = res.moderators;
      this.state.admins = res.admins;
      this.state.loading = false;
      document.title = `/f/${this.state.community.name} - Lemmy`;
      this.setState(this.state);
    } else if (op == UserOperation.EditCommunity) {
      let res: CommunityResponse = msg;
      this.state.community = res.community;
      this.setState(this.state);
    } else if (op == UserOperation.FollowCommunity) {
      let res: CommunityResponse = msg;
      this.state.community.subscribed = res.community.subscribed;
      this.state.community.number_of_subscribers = res.community.number_of_subscribers;
      this.setState(this.state);
    }
  }
}

