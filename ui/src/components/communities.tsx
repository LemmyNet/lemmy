import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community, Post as PostI, GetPostResponse, PostResponse, Comment, CommentForm as CommentFormI, CommentResponse, CommentLikeForm, CommentSortType, CreatePostLikeResponse, ListCommunitiesResponse, CommunityResponse, FollowCommunityForm } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp, hotRank,mdToHtml } from '../utils';

declare const Sortable: any;

interface CommunitiesState {
  communities: Array<Community>;
}

export class Communities extends Component<any, CommunitiesState> {
  private subscription: Subscription;
  private emptyState: CommunitiesState = {
    communities: []
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
    WebSocketService.Instance.listCommunities();

  }

  componentDidMount() {
    let table = document.querySelector('#community_table');
    Sortable.initTable(table);
  }

  render() {
    return (
      <div class="container-fluid">
        <h4>Communities</h4>
        <div class="table-responsive">
          <table id="community_table" class="table table-sm table-hover" data-sortable>
            <thead>
              <tr>
                <th>Name</th>
                <th>Title</th>
                <th>Category</th>
                <th class="text-right">Subscribers</th>
                <th class="text-right">Posts</th>
                <th class="text-right">Comments</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {this.state.communities.map(community =>
                <tr>
                  <td><Link to={`/community/${community.id}`}>{community.name}</Link></td>
                  <td>{community.title}</td>
                  <td>{community.category_name}</td>
                  <td class="text-right">{community.number_of_subscribers}</td>
                  <td class="text-right">{community.number_of_posts}</td>
                  <td class="text-right">{community.number_of_comments}</td>
                  <td class="text-right">
                    {community.subscribed 
                      ? <button class="btn btn-sm btn-secondary" onClick={linkEvent(community.id, this.handleUnsubscribe)}>Unsubscribe</button>
                      : <button class="btn btn-sm btn-secondary" onClick={linkEvent(community.id, this.handleSubscribe)}>Subscribe</button>
                    }
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  handleUnsubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: false
    };
    WebSocketService.Instance.followCommunity(form);
  }


  handleSubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: true
    };
    WebSocketService.Instance.followCommunity(form);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.communities = res.communities;
      this.state.communities.sort((a, b) => b.number_of_subscribers - a.number_of_subscribers);
      this.setState(this.state);
    } else if (op == UserOperation.FollowCommunity) {
      let res: CommunityResponse = msg;
      let found = this.state.communities.find(c => c.id == res.community.id);
      found.subscribed = res.community.subscribed;
      found.number_of_subscribers = res.community.number_of_subscribers;
      this.setState(this.state);
    }
  }
}
