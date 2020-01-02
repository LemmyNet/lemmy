import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  CommunityUser,
  GetFollowedCommunitiesResponse,
  ListCommunitiesForm,
  ListCommunitiesResponse,
  Community,
  SortType,
  GetSiteResponse,
  ListingType,
  SiteResponse,
  GetPostsResponse,
  CreatePostLikeResponse,
  Post,
  GetPostsForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { SortSelect } from './sort-select';
import { ListingTypeSelect } from './listing-type-select';
import { SiteForm } from './site-form';
import {
  msgOp,
  repoUrl,
  mdToHtml,
  fetchLimit,
  routeSortTypeToEnum,
  routeListingTypeToEnum,
  postRefetchSeconds,
  pictshareAvatarThumbnail,
  showAvatars,
} from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface MainState {
  subscribedCommunities: Array<CommunityUser>;
  trendingCommunities: Array<Community>;
  site: GetSiteResponse;
  showEditSite: boolean;
  loading: boolean;
  posts: Array<Post>;
  type_: ListingType;
  sort: SortType;
  page: number;
}

export class Main extends Component<any, MainState> {
  private subscription: Subscription;
  private postFetcher: any;
  private emptyState: MainState = {
    subscribedCommunities: [],
    trendingCommunities: [],
    site: {
      op: null,
      site: {
        id: null,
        name: null,
        creator_id: null,
        creator_name: null,
        published: null,
        number_of_users: null,
        number_of_posts: null,
        number_of_comments: null,
        number_of_communities: null,
        enable_downvotes: null,
        open_registration: null,
        enable_nsfw: null,
      },
      admins: [],
      banned: [],
      online: null,
    },
    showEditSite: false,
    loading: true,
    posts: [],
    type_: this.getListingTypeFromProps(this.props),
    sort: this.getSortTypeFromProps(this.props),
    page: this.getPageFromProps(this.props),
  };

  getListingTypeFromProps(props: any): ListingType {
    return props.match.params.type
      ? routeListingTypeToEnum(props.match.params.type)
      : UserService.Instance.user
      ? UserService.Instance.user.default_listing_type
      : ListingType.All;
  }

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
    this.handleEditCancel = this.handleEditCancel.bind(this);
    this.handleSortChange = this.handleSortChange.bind(this);
    this.handleTypeChange = this.handleTypeChange.bind(this);

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

    WebSocketService.Instance.getSite();

    if (UserService.Instance.user) {
      WebSocketService.Instance.getFollowedCommunities();
    }

    let listCommunitiesForm: ListCommunitiesForm = {
      sort: SortType[SortType.Hot],
      limit: 6,
    };

    WebSocketService.Instance.listCommunities(listCommunitiesForm);

    this.keepFetchingPosts();
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
      this.state.type_ = this.getListingTypeFromProps(nextProps);
      this.state.sort = this.getSortTypeFromProps(nextProps);
      this.state.page = this.getPageFromProps(nextProps);
      this.setState(this.state);
      this.fetchPosts();
    }
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-md-8">{this.posts()}</div>
          <div class="col-12 col-md-4">{this.my_sidebar()}</div>
        </div>
      </div>
    );
  }

  my_sidebar() {
    return (
      <div>
        {!this.state.loading && (
          <div>
            <div class="card border-secondary mb-3">
              <div class="card-body">
                {this.trendingCommunities()}
                {UserService.Instance.user &&
                  this.state.subscribedCommunities.length > 0 && (
                    <div>
                      <h5>
                        <T i18nKey="subscribed_to_communities">
                          #
                          <Link class="text-white" to="/communities">
                            #
                          </Link>
                        </T>
                      </h5>
                      <ul class="list-inline">
                        {this.state.subscribedCommunities.map(community => (
                          <li class="list-inline-item">
                            <Link to={`/c/${community.community_name}`}>
                              {community.community_name}
                            </Link>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                <Link
                  class="btn btn-sm btn-secondary btn-block"
                  to="/create_community"
                >
                  <T i18nKey="create_a_community">#</T>
                </Link>
              </div>
            </div>
            {this.sidebar()}
            {this.landing()}
          </div>
        )}
      </div>
    );
  }

  trendingCommunities() {
    return (
      <div>
        <h5>
          <T i18nKey="trending_communities">
            #
            <Link class="text-white" to="/communities">
              #
            </Link>
          </T>
        </h5>
        <ul class="list-inline">
          {this.state.trendingCommunities.map(community => (
            <li class="list-inline-item">
              <Link to={`/c/${community.name}`}>{community.name}</Link>
            </li>
          ))}
        </ul>
      </div>
    );
  }

  sidebar() {
    return (
      <div>
        {!this.state.showEditSite ? (
          this.siteInfo()
        ) : (
          <SiteForm
            site={this.state.site.site}
            onCancel={this.handleEditCancel}
          />
        )}
      </div>
    );
  }

  updateUrl() {
    let typeStr = ListingType[this.state.type_].toLowerCase();
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(
      `/home/type/${typeStr}/sort/${sortStr}/page/${this.state.page}`
    );
  }

  siteInfo() {
    return (
      <div>
        <div class="card border-secondary mb-3">
          <div class="card-body">
            <h5 class="mb-0">{`${this.state.site.site.name}`}</h5>
            {this.canAdmin && (
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                <li className="list-inline-item">
                  <span
                    class="pointer"
                    onClick={linkEvent(this, this.handleEditClick)}
                  >
                    <T i18nKey="edit">#</T>
                  </span>
                </li>
              </ul>
            )}
            <ul class="my-2 list-inline">
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_online"
                  interpolation={{ count: this.state.site.online }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_users"
                  interpolation={{
                    count: this.state.site.site.number_of_users,
                  }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_communities"
                  interpolation={{
                    count: this.state.site.site.number_of_communities,
                  }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_posts"
                  interpolation={{
                    count: this.state.site.site.number_of_posts,
                  }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_comments"
                  interpolation={{
                    count: this.state.site.site.number_of_comments,
                  }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item">
                <Link className="badge badge-secondary" to="/modlog">
                  <T i18nKey="modlog">#</T>
                </Link>
              </li>
            </ul>
            <ul class="mt-1 list-inline small mb-0">
              <li class="list-inline-item">
                <T i18nKey="admins" class="d-inline">
                  #
                </T>
                :
              </li>
              {this.state.site.admins.map(admin => (
                <li class="list-inline-item">
                  <Link class="text-info" to={`/u/${admin.name}`}>
                    {admin.avatar && showAvatars() && (
                      <img
                        height="32"
                        width="32"
                        src={pictshareAvatarThumbnail(admin.avatar)}
                        class="rounded-circle mr-1"
                      />
                    )}
                    <span>{admin.name}</span>
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>
        {this.state.site.site.description && (
          <div class="card border-secondary mb-3">
            <div class="card-body">
              <div
                className="md-div"
                dangerouslySetInnerHTML={mdToHtml(
                  this.state.site.site.description
                )}
              />
            </div>
          </div>
        )}
      </div>
    );
  }

  landing() {
    return (
      <div class="card border-secondary">
        <div class="card-body">
          <h5>
            <T i18nKey="powered_by" class="d-inline">
              #
            </T>
            <svg class="icon mx-2">
              <use xlinkHref="#icon-mouse">#</use>
            </svg>
            <a href={repoUrl}>
              Lemmy<sup>beta</sup>
            </a>
          </h5>
          <p class="mb-0">
            <T i18nKey="landing_0">
              #
              <a href="https://en.wikipedia.org/wiki/Social_network_aggregation">
                #
              </a>
              <a href="https://en.wikipedia.org/wiki/Fediverse">#</a>
              <br></br>
              <code>#</code>
              <br></br>
              <b>#</b>
              <br></br>
              <a href={repoUrl}>#</a>
              <br></br>
              <a href="https://www.rust-lang.org">#</a>
              <a href="https://actix.rs/">#</a>
              <a href="https://infernojs.org">#</a>
              <a href="https://www.typescriptlang.org/">#</a>
            </T>
          </p>
        </div>
      </div>
    );
  }

  posts() {
    return (
      <div>
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div>
            {this.selects()}
            <PostListings posts={this.state.posts} showCommunity />
            {this.paginator()}
          </div>
        )}
      </div>
    );
  }

  selects() {
    return (
      <div className="mb-3">
        <ListingTypeSelect
          type_={this.state.type_}
          onChange={this.handleTypeChange}
        />
        <span class="mx-2">
          <SortSelect sort={this.state.sort} onChange={this.handleSortChange} />
        </span>
        {this.state.type_ == ListingType.All && (
          <a
            href={`/feeds/all.xml?sort=${SortType[this.state.sort]}`}
            target="_blank"
          >
            <svg class="icon mx-1 text-muted small">
              <use xlinkHref="#icon-rss">#</use>
            </svg>
          </a>
        )}
        {UserService.Instance.user &&
          this.state.type_ == ListingType.Subscribed && (
            <a
              href={`/feeds/front/${UserService.Instance.auth}.xml?sort=${
                SortType[this.state.sort]
              }`}
              target="_blank"
            >
              <svg class="icon mx-1 text-muted small">
                <use xlinkHref="#icon-rss">#</use>
              </svg>
            </a>
          )}
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

  get canAdmin(): boolean {
    return (
      UserService.Instance.user &&
      this.state.site.admins
        .map(a => a.id)
        .includes(UserService.Instance.user.id)
    );
  }

  handleEditClick(i: Main) {
    i.state.showEditSite = true;
    i.setState(i.state);
  }

  handleEditCancel() {
    this.state.showEditSite = false;
    this.setState(this.state);
  }

  nextPage(i: Main) {
    i.state.page++;
    i.state.loading = true;
    i.setState(i.state);
    i.updateUrl();
    i.fetchPosts();
    window.scrollTo(0, 0);
  }

  prevPage(i: Main) {
    i.state.page--;
    i.state.loading = true;
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

  handleTypeChange(val: ListingType) {
    this.state.type_ = val;
    this.state.page = 1;
    this.state.loading = true;
    this.setState(this.state);
    this.updateUrl();
    this.fetchPosts();
    window.scrollTo(0, 0);
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
      type_: ListingType[this.state.type_],
    };
    WebSocketService.Instance.getPosts(getPostsForm);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      return;
    } else if (op == UserOperation.GetFollowedCommunities) {
      let res: GetFollowedCommunitiesResponse = msg;
      this.state.subscribedCommunities = res.communities;
      this.setState(this.state);
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.trendingCommunities = res.communities;
      this.setState(this.state);
    } else if (op == UserOperation.GetSite) {
      let res: GetSiteResponse = msg;

      // This means it hasn't been set up yet
      if (!res.site) {
        this.context.router.history.push('/setup');
      }
      this.state.site.admins = res.admins;
      this.state.site.site = res.site;
      this.state.site.banned = res.banned;
      this.state.site.online = res.online;
      this.setState(this.state);
      document.title = `${WebSocketService.Instance.site.name}`;
    } else if (op == UserOperation.EditSite) {
      let res: SiteResponse = msg;
      this.state.site.site = res.site;
      this.state.showEditSite = false;
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
