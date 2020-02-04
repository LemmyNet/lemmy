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
  GetCommunityForm,
  ListingType,
  GetPostsResponse,
  PostResponse,
  AddModToCommunityResponse,
  BanFromCommunityResponse,
  WebSocketJsonResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { SortSelect } from './sort-select';
import { Sidebar } from './sidebar';
import { wsJsonToRes, routeSortTypeToEnum, fetchLimit, toast } from '../utils';
import { i18n } from '../i18next';

interface State {
  community: CommunityI;
  communityId: number;
  communityName: string;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  online: number;
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
      nsfw: false,
      deleted: null,
    },
    moderators: [],
    admins: [],
    communityId: Number(this.props.match.params.id),
    communityName: this.props.match.params.name,
    online: null,
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
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    let form: GetCommunityForm = {
      id: this.state.communityId ? this.state.communityId : null,
      name: this.state.communityName ? this.state.communityName : null,
    };
    WebSocketService.Instance.getCommunity(form);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
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
                    {i18n.t('removed')}
                  </small>
                )}
                {this.state.community.nsfw && (
                  <small className="ml-2 text-muted font-italic">
                    {i18n.t('nsfw')}
                  </small>
                )}
              </h5>
              {this.selects()}
              <PostListings posts={this.state.posts} removeDuplicates />
              {this.paginator()}
            </div>
            <div class="col-12 col-md-4">
              <Sidebar
                community={this.state.community}
                moderators={this.state.moderators}
                admins={this.state.admins}
                online={this.state.online}
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
            {i18n.t('prev')}
          </button>
        )}
        {this.state.posts.length == fetchLimit && (
          <button
            class="btn btn-sm btn-secondary"
            onClick={linkEvent(this, this.nextPage)}
          >
            {i18n.t('next')}
          </button>
        )}
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

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.context.router.history.push('/');
      return;
    } else if (res.op == UserOperation.GetCommunity) {
      let data = res.data as GetCommunityResponse;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.state.admins = data.admins;
      this.state.online = data.online;
      document.title = `/c/${this.state.community.name} - ${WebSocketService.Instance.site.name}`;
      this.setState(this.state);
      this.fetchPosts();
    } else if (res.op == UserOperation.EditCommunity) {
      let data = res.data as CommunityResponse;
      this.state.community = data.community;
      this.setState(this.state);
    } else if (res.op == UserOperation.FollowCommunity) {
      let data = res.data as CommunityResponse;
      this.state.community.subscribed = data.community.subscribed;
      this.state.community.number_of_subscribers =
        data.community.number_of_subscribers;
      this.setState(this.state);
    } else if (res.op == UserOperation.GetPosts) {
      let data = res.data as GetPostsResponse;
      this.state.posts = data.posts;
      this.state.loading = false;
      this.setState(this.state);
    } else if (res.op == UserOperation.EditPost) {
      let data = res.data as PostResponse;
      let found = this.state.posts.find(c => c.id == data.post.id);

      found.url = data.post.url;
      found.name = data.post.name;
      found.nsfw = data.post.nsfw;

      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePost) {
      let data = res.data as PostResponse;
      this.state.posts.unshift(data.post);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePostLike) {
      let data = res.data as PostResponse;
      let found = this.state.posts.find(c => c.id == data.post.id);

      found.score = data.post.score;
      found.upvotes = data.post.upvotes;
      found.downvotes = data.post.downvotes;
      if (data.post.my_vote !== null) {
        found.my_vote = data.post.my_vote;
        found.upvoteLoading = false;
        found.downvoteLoading = false;
      }

      this.setState(this.state);
    } else if (res.op == UserOperation.AddModToCommunity) {
      let data = res.data as AddModToCommunityResponse;
      this.state.moderators = data.moderators;
      this.setState(this.state);
    } else if (res.op == UserOperation.BanFromCommunity) {
      let data = res.data as BanFromCommunityResponse;

      this.state.posts
        .filter(p => p.creator_id == data.user.id)
        .forEach(p => (p.banned = data.banned));

      this.setState(this.state);
    }
  }
}
