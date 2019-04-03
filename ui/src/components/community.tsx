import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, CommunityResponse, Post, GetPostsForm, ListingSortType, ListingType, GetPostsResponse, CreatePostLikeForm, CreatePostLikeResponse} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { MomentTime } from './moment-time';
import { PostListing } from './post-listing';
import { Sidebar } from './sidebar';
import { msgOp, mdToHtml } from '../utils';

interface State {
  community: CommunityI;
  posts: Array<Post>;
  sortType: ListingSortType;
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
      published: null
    },
    posts: [],
    sortType: ListingSortType.Hot,
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

    let communityId = Number(this.props.match.params.id);
    WebSocketService.Instance.getCommunity(communityId);

    let getPostsForm: GetPostsForm = {
      community_id: communityId,
      limit: 10,
      sort: ListingSortType[ListingSortType.Hot],
      type_: ListingType[ListingType.Community]
    }
    WebSocketService.Instance.getPosts(getPostsForm);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-sm-10 col-lg-9">
            <h4>/f/{this.state.community.name}</h4>
            <div>{this.selects()}</div>
            {this.state.posts.length > 0 
              ? this.state.posts.map(post => 
                <PostListing post={post} />) 
              : <div>no listings</div>
            }
          </div>
          <div class="col-12 col-sm-2 col-lg-3">
            <Sidebar community={this.state.community} />
          </div>
        </div>
      </div>
    )
  }

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.sortType} onChange={linkEvent(this, this.handleSortChange)} class="custom-select w-auto">
          <option disabled>Sort Type</option>
          <option value={ListingSortType.Hot}>Hot</option>
          <option value={ListingSortType.New}>New</option>
          <option disabled>──────────</option>
          <option value={ListingSortType.TopDay}>Top Day</option>
          <option value={ListingSortType.TopWeek}>Week</option>
          <option value={ListingSortType.TopMonth}>Month</option>
          <option value={ListingSortType.TopYear}>Year</option>
          <option value={ListingSortType.TopAll}>All</option>
        </select>
      </div>
    )

  }

  handleSortChange(i: Community, event) {
    i.state.sortType = Number(event.target.value);
    i.setState(i.state);

    let getPostsForm: GetPostsForm = {
      community_id: i.state.community.id,
      limit: 10,
      sort: ListingSortType[i.state.sortType],
      type_: ListingType[ListingType.Community]
    }
    WebSocketService.Instance.getPosts(getPostsForm);
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
    }  else if (op == UserOperation.GetPosts) {
      let res: GetPostsResponse = msg;
      this.state.posts = res.posts;
      this.setState(this.state);
    } else if (op == UserOperation.CreatePostLike) {
      let res: CreatePostLikeResponse = msg;
      let found = this.state.posts.find(c => c.id == res.post.id);
      found.my_vote = res.post.my_vote;
      found.score = res.post.score;
      found.upvotes = res.post.upvotes;
      found.downvotes = res.post.downvotes;
      this.setState(this.state);
    }
  }
}


