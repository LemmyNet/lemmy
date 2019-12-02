import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Community as CommunityI,
  GetCommunityResponse,
  CommunityResponse,
  CommunityUser,
  UserView,
  SortType,
  Post,
  GetPostsForm,
  ListingType,
  GetPostsResponse,
  CreatePostLikeResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { SortSelect } from './sort-select';
import { Sidebar } from './sidebar';
import {
  msgOp,
  routeSortTypeToEnum,
  fetchLimit,
  postRefetchSeconds,
} from '../utils';
import { T } from 'inferno-i18next';
import { i18n } from '../i18next';

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
  private postFetcher: any;
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
      nsfw: false,
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
  };

  getSortTypeFromProps(props: any): SortType {
    return props.match.params.sort
      ? routeSortTypeToEnum(props.match.params.sort)
      : UserService.Instance.user
      ? UserService.Instance.user.default_sort_type
      : SortType.Hot;
  }

  getPageFromProps(props: any): number {
    return props.match.params.page ? Number(props.match.params.page) : 1;
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleSortChange = this.handleSortChange.bind(this);

    this.subscription = WebSocketService.Instance.subject
      .pipe(
        retryWhen(errors =>
          errors.pipe(
            delay(3000),
            take(10)
          )
        )
      )
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
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
    clearInterval(this.postFetcher);
  }

  // Necessary for back button for some reason
  componentWillReceiveProps(nextProps: any) {
    if (
      nextProps.history.action == 'POP' ||
      nextProps.history.action == 'PUSH'
    ) {
      this.state.sort = this.getSortTypeFromProps(nextProps);
      this.state.page = this.getPageFromProps(nextProps);
      this.setState(this.state);
      this.fetchPosts();
    }
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div class="row">
            <div class="col-12 col-md-8">
              <h5>
                {this.state.community.title}
                {this.state.community.removed && (
                  <small className="ml-2 text-muted font-italic">
                    <T i18nKey="removed">#</T>
                  </small>
                )}
                {this.state.community.nsfw && (
                  <small className="ml-2 text-muted font-italic">
                    <T i18nKey="nsfw">#</T>
                  </small>
                )}
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
        )}
      </div>
    );
  }

  selects() {
    return (
      <div class="mb-2">
        <SortSelect sort={this.state.sort} onChange={this.handleSortChange} />
        <a
          href={`/feeds/c/${this.state.communityName}.xml?sort=${
            SortType[this.state.sort]
          }`}
          target="_blank"
        >
          <svg class="icon mx-2 text-muted small">
            <use xlinkHref="#icon-rss">#</use>
          </svg>
        </a>
      </div>
    );
  }

  paginator() {
    return (
      <div class="my-2">
        {this.state.page > 1 && (
          <button
            class="btn btn-sm btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            <T i18nKey="prev">#</T>
          </button>
        )}
        <button
          class="btn btn-sm btn-secondary"
          onClick={linkEvent(this, this.nextPage)}
        >
          <T i18nKey="next">#</T>
        </button>
      </div>
    );
  }

  nextPage(i: Community) {
    i.state.page++;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
    window.scrollTo(0, 0);
  }

  prevPage(i: Community) {
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
    window.scrollTo(0, 0);
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.state.loading = true;
    this.setState(this.state);
    this.updateUrl();
    this.fetchPosts();
    window.scrollTo(0, 0);
  }

  updateUrl() {
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(
      `/c/${this.state.community.name}/sort/${sortStr}/page/${this.state.page}`
    );
  }

  keepFetchingPosts() {
    this.fetchPosts();
    this.postFetcher = setInterval(() => this.fetchPosts(), postRefetchSeconds);
  }

  fetchPosts() {
    let getPostsForm: GetPostsForm = {
      page: this.state.page,
      limit: fetchLimit,
      sort: SortType[this.state.sort],
      type_: ListingType[ListingType.Community],
      community_id: this.state.community.id,
    };
    WebSocketService.Instance.getPosts(getPostsForm);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.context.router.history.push('/');
      return;
    } else if (op == UserOperation.GetCommunity) {
      let res: GetCommunityResponse = msg;
      this.state.community = res.community;
      this.state.moderators = res.moderators;
      this.state.admins = res.admins;
      document.title = `/c/${this.state.community.name} - ${WebSocketService.Instance.site.name}`;
      this.setState(this.state);
      this.keepFetchingPosts();
    } else if (op == UserOperation.EditCommunity) {
      let res: CommunityResponse = msg;
      this.state.community = res.community;
      this.setState(this.state);
    } else if (op == UserOperation.FollowCommunity) {
      let res: CommunityResponse = msg;
      this.state.community.subscribed = res.community.subscribed;
      this.state.community.number_of_subscribers =
        res.community.number_of_subscribers;
      this.setState(this.state);
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
