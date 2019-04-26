import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community, ListCommunitiesResponse, CommunityResponse, FollowCommunityForm, ListCommunitiesForm, SortType } from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp } from '../utils';

declare const Sortable: any;

interface CommunitiesState {
  communities: Array<Community>;
  loading: boolean;
}

export class Communities extends Component<any, CommunitiesState> {
  private subscription: Subscription;
  private emptyState: CommunitiesState = {
    communities: [],
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

    let listCommunitiesForm: ListCommunitiesForm = {
      sort: SortType[SortType.TopAll],
      limit: 100,
    }

    WebSocketService.Instance.listCommunities(listCommunitiesForm);

  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = "Forums - Lemmy";
    let table = document.querySelector('#community_table');
    Sortable.initTable(table);
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? 
        <h5 class=""><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h5> : 
        <div>
          <h5>Forums</h5>
          <div class="table-responsive">
            <table id="community_table" class="table table-sm table-hover">
              <thead class="pointer">
                <tr>
                  <th>Name</th>
                  <th>Title</th>
                  <th>Category</th>
                  <th class="text-right d-none d-md-table-cell">Subscribers</th>
                  <th class="text-right d-none d-md-table-cell">Posts</th>
                  <th class="text-right d-none d-md-table-cell">Comments</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {this.state.communities.map(community =>
                  <tr>
                    <td><Link to={`/f/${community.name}`}>{community.name}</Link></td>
                    <td>{community.title}</td>
                    <td>{community.category_name}</td>
                    <td class="text-right d-none d-md-table-cell">{community.number_of_subscribers}</td>
                    <td class="text-right d-none d-md-table-cell">{community.number_of_posts}</td>
                    <td class="text-right d-none d-md-table-cell">{community.number_of_comments}</td>
                    <td class="text-right">
                      {community.subscribed ? 
                      <span class="pointer btn-link" onClick={linkEvent(community.id, this.handleUnsubscribe)}>Unsubscribe</span> : 
                      <span class="pointer btn-link" onClick={linkEvent(community.id, this.handleSubscribe)}>Subscribe</span>
                      }
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
        }
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
      this.state.loading = false;
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
