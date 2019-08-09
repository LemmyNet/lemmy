import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Community as CommunityI, GetCommunityResponse, CommunityResponse,  CommunityUser, UserView, SortType, Post, GetPostsForm, ListingType, GetPostsResponse, CreatePostLikeResponse } from '../interfaces';
import { WebSocketService } from '../services';
import { PostListings } from './post-listings';
import { Sidebar } from './sidebar';
import { msgOp, routeSortTypeToEnum, fetchLimit } from '../utils';
import { T } from 'inferno-i18next';

interface State {
  community: CommunityI;
  communityId: number;
  communityName: string;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  loading: boolean;
  posts: Array<Post>;
  sort: SortType;
  page: number;
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
      deleted: null,
    },
    moderators: [],
    admins: [],
    communityId: Number(this.props.match.params.id),
    communityName: this.props.match.params.name,
    loading: true,
    posts: [],
    sort: this.getSortTypeFromProps(this.props),
    page: this.getPageFromProps(this.props),
  }

  getSortTypeFromProps(props: any): SortType {
    return (props.match.params.sort) ? 
      routeSortTypeToEnum(props.match.params.sort) : 
      SortType.Hot;
  }

  getPageFromProps(props: any): number {
    return (props.match.params.page) ? Number(props.match.params.page) : 1;
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

    if (this.state.communityId) {
      WebSocketService.Instance.getCommunity(this.state.communityId);
    } else if (this.state.communityName) {
      WebSocketService.Instance.getCommunityByName(this.state.communityName);
    }

  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  // Necessary for back button for some reason
  componentWillReceiveProps(nextProps: any) {
    if (nextProps.history.action == 'POP') {
      this.state = this.emptyState;
      this.state.sort = this.getSortTypeFromProps(nextProps);
      this.state.page = this.getPageFromProps(nextProps);
      this.fetchPosts();
    }
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? 
        <h5><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h5> : 
        <div class="row">
          <div class="col-12 col-md-8">
            <h5>{this.state.community.title}
            {this.state.community.removed &&
              <small className="ml-2 text-muted font-italic"><T i18nKey="removed">#</T></small>
            }
          </h5>
          {this.selects()}
          <PostListings posts={this.state.posts} />
          {this.paginator()}
          </div>
          <div class="col-12 col-md-4">
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

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.sort} onChange={linkEvent(this, this.handleSortChange)} class="custom-select custom-select-sm w-auto">
          <option disabled><T i18nKey="sort_type">#</T></option>
          <option value={SortType.Hot}><T i18nKey="hot">#</T></option>
          <option value={SortType.New}><T i18nKey="new">#</T></option>
          <option disabled>──────────</option>
          <option value={SortType.TopDay}><T i18nKey="top_day">#</T></option>
          <option value={SortType.TopWeek}><T i18nKey="week">#</T></option>
          <option value={SortType.TopMonth}><T i18nKey="month">#</T></option>
          <option value={SortType.TopYear}><T i18nKey="year">#</T></option>
          <option value={SortType.TopAll}><T i18nKey="all">#</T></option>
        </select>
      </div>
    )
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && 
          <button class="btn btn-sm btn-secondary mr-1" onClick={linkEvent(this, this.prevPage)}><T i18nKey="prev">#</T></button>
        }
        <button class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.nextPage)}><T i18nKey="next">#</T></button>
      </div>
    );
  }

  nextPage(i: Community) { 
    i.state.page++;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
  }

  prevPage(i: Community) { 
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
  }

  handleSortChange(i: Community, event: any) {
    i.state.sort = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
  }

  updateUrl() {
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(`/c/${this.state.community.name}/sort/${sortStr}/page/${this.state.page}`);
  }

  fetchPosts() {
    let getPostsForm: GetPostsForm = {
      page: this.state.page,
      limit: fetchLimit,
      sort: SortType[this.state.sort],
      type_: ListingType[ListingType.Community],
      community_id: this.state.community.id,
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
      let res: GetCommunityResponse = msg;
      this.state.community = res.community;
      this.state.moderators = res.moderators;
      this.state.admins = res.admins;
      document.title = `/c/${this.state.community.name} - ${WebSocketService.Instance.site.name}`;
      this.setState(this.state);
      this.fetchPosts();
    } else if (op == UserOperation.EditCommunity) {
      let res: CommunityResponse = msg;
      this.state.community = res.community;
      this.setState(this.state);
    } else if (op == UserOperation.FollowCommunity) {
      let res: CommunityResponse = msg;
      this.state.community.subscribed = res.community.subscribed;
      this.state.community.number_of_subscribers = res.community.number_of_subscribers;
      this.setState(this.state);
    } else if (op == UserOperation.GetPosts) {
      let res: GetPostsResponse = msg;
      this.state.posts = res.posts;
      this.state.loading = false;
      window.scrollTo(0,0);
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

