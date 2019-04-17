import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, Post, GetPostsForm, SortType, ListingType, GetPostsResponse, CreatePostLikeResponse, CommunityUser} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListing } from './post-listing';
import { msgOp, fetchLimit } from '../utils';

interface PostListingsProps {
  communityId?: number;
}

interface PostListingsState {
  community: CommunityI;
  moderators: Array<CommunityUser>;
  posts: Array<Post>;
  sortType: SortType;
  type_: ListingType;
  page: number;
  loading: boolean;
}

export class PostListings extends Component<PostListingsProps, PostListingsState> {

  private subscription: Subscription;
  private emptyState: PostListingsState = {
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
      published: null
    },
    moderators: [],
    posts: [],
    sortType: SortType.Hot,
    type_: this.props.communityId 
    ? ListingType.Community 
    : UserService.Instance.user
    ? ListingType.Subscribed 
    : ListingType.All,
    page: 1,
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

      this.refetch();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div>
        {this.state.loading ? 
        <h4><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h4> : 
        <div>
          {this.selects()}
          {this.state.posts.length > 0 
            ? this.state.posts.map(post => 
              <PostListing post={post} showCommunity={!this.props.communityId}/>) 
                : <div>No Listings. {!this.props.communityId && <span>Subscribe to some <Link to="/communities">forums</Link>.</span>}</div>
          }
          {this.paginator()}
        </div>
        }
      </div>
    )
  }

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.sortType} onChange={linkEvent(this, this.handleSortChange)} class="custom-select w-auto">
          <option disabled>Sort Type</option>
          <option value={SortType.Hot}>Hot</option>
          <option value={SortType.New}>New</option>
          <option disabled>──────────</option>
          <option value={SortType.TopDay}>Top Day</option>
          <option value={SortType.TopWeek}>Week</option>
          <option value={SortType.TopMonth}>Month</option>
          <option value={SortType.TopYear}>Year</option>
          <option value={SortType.TopAll}>All</option>
        </select>
        {!this.props.communityId && 
          UserService.Instance.user &&
            <select value={this.state.type_} onChange={linkEvent(this, this.handleTypeChange)} class="ml-2 custom-select w-auto">
              <option disabled>Type</option>
              <option value={ListingType.All}>All</option>
              <option value={ListingType.Subscribed}>Subscribed</option>
            </select>

        }
      </div>
    )
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && 
          <button class="btn btn-sm btn-secondary mr-1" onClick={linkEvent(this, this.prevPage)}>Prev</button>
        }
        <button class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.nextPage)}>Next</button>
      </div>
    );
  }

  nextPage(i: PostListings) { 
    i.state.page++;
    i.setState(i.state);
    i.refetch();
  }

  prevPage(i: PostListings) { 
    i.state.page--;
    i.setState(i.state);
    i.refetch();
  }

  handleSortChange(i: PostListings, event: any) {
    i.state.sortType = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.refetch();
  }

  refetch() {
    let getPostsForm: GetPostsForm = {
      community_id: this.state.community.id,
      page: this.state.page,
      limit: fetchLimit,
      sort: SortType[this.state.sortType],
      type_: ListingType[ListingType.Community]
    }
    WebSocketService.Instance.getPosts(getPostsForm);
  }

  handleTypeChange(i: PostListings, event: any) {
    i.state.type_ = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.refetch();
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetPosts) {
      let res: GetPostsResponse = msg;
      this.state.posts = res.posts;
      this.state.loading = false;
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


