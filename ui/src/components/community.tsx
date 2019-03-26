import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, CommunityResponse, Post } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

interface State {
  community: CommunityI;
  posts: Array<Post>;
}

export class Community extends Component<any, State> {

  private subscription: Subscription;
  private emptyState: State = {
    community: {
      id: null,
      name: null,
      published: null
    },
    posts: []
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;

    console.log(this.props.match.params.id);

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
      );

    let communityId = Number(this.props.match.params.id);
    WebSocketService.Instance.getCommunity(communityId);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            {this.state.community.name}
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
    } else if (op == UserOperation.GetCommunity) {
      let res: CommunityResponse = msg;
      this.state.community = res.community;
      this.setState(this.state);
    }  
  }
}
