import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
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
  PostResponse,
  Post,
  GetPostsForm,
  Comment,
  GetCommentsForm,
  GetCommentsResponse,
  CommentResponse,
  AddAdminResponse,
  BanUserResponse,
  WebSocketJsonResponse,
} from 'lemmy-js-client';
import { DataType } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { CommentNodes } from './comment-nodes';
import { SortSelect } from './sort-select';
import { ListingTypeSelect } from './listing-type-select';
import { DataTypeSelect } from './data-type-select';
import { SiteForm } from './site-form';
import { UserListing } from './user-listing';
import { CommunityLink } from './community-link';
import { BannerIconHeader } from './banner-icon-header';
import {
  wsJsonToRes,
  repoUrl,
  mdToHtml,
  fetchLimit,
  toast,
  getListingTypeFromProps,
  getPageFromProps,
  getSortTypeFromProps,
  getDataTypeFromProps,
  editCommentRes,
  saveCommentRes,
  createCommentLikeRes,
  createPostLikeFindRes,
  editPostFindRes,
  commentsToFlatNodes,
  setupTippy,
  favIconUrl,
  notifyPost,
} from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface MainState {
  subscribedCommunities: Array<CommunityUser>;
  trendingCommunities: Array<Community>;
  siteRes: GetSiteResponse;
  showEditSite: boolean;
  loading: boolean;
  posts: Array<Post>;
  comments: Array<Comment>;
  listingType: ListingType;
  dataType: DataType;
  sort: SortType;
  page: number;
}

interface MainProps {
  listingType: ListingType;
  dataType: DataType;
  sort: SortType;
  page: number;
}

interface UrlParams {
  listingType?: ListingType;
  dataType?: string;
  sort?: SortType;
  page?: number;
}

export class Main extends Component<any, MainState> {
  private subscription: Subscription;
  private emptyState: MainState = {
    subscribedCommunities: [],
    trendingCommunities: [],
    siteRes: {
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
        icon: null,
        banner: null,
        creator_preferred_username: null,
      },
      admins: [],
      banned: [],
      online: null,
      version: null,
      federated_instances: null,
    },
    showEditSite: false,
    loading: true,
    posts: [],
    comments: [],
    listingType: getListingTypeFromProps(this.props),
    dataType: getDataTypeFromProps(this.props),
    sort: getSortTypeFromProps(this.props),
    page: getPageFromProps(this.props),
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleEditCancel = this.handleEditCancel.bind(this);
    this.handleSortChange = this.handleSortChange.bind(this);
    this.handleListingTypeChange = this.handleListingTypeChange.bind(this);
    this.handleDataTypeChange = this.handleDataTypeChange.bind(this);

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
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
      sort: SortType.Hot,
      limit: 6,
    };

    WebSocketService.Instance.listCommunities(listCommunitiesForm);

    this.fetchData();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  static getDerivedStateFromProps(props: any): MainProps {
    return {
      listingType: getListingTypeFromProps(props),
      dataType: getDataTypeFromProps(props),
      sort: getSortTypeFromProps(props),
      page: getPageFromProps(props),
    };
  }

  componentDidUpdate(_: any, lastState: MainState) {
    if (
      lastState.listingType !== this.state.listingType ||
      lastState.dataType !== this.state.dataType ||
      lastState.sort !== this.state.sort ||
      lastState.page !== this.state.page
    ) {
      this.setState({ loading: true });
      this.fetchData();
    }
  }

  get documentTitle(): string {
    if (this.state.siteRes.site.name) {
      return `${this.state.siteRes.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  get favIcon(): string {
    return this.state.siteRes.site.icon
      ? this.state.siteRes.site.icon
      : favIconUrl;
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle}>
          <link
            id="favicon"
            rel="icon"
            type="image/x-icon"
            href={this.favIcon}
          />
        </Helmet>
        <div class="row">
          <main role="main" class="col-12 col-md-8">
            {this.posts()}
          </main>
          <aside class="col-12 col-md-4">{this.mySidebar()}</aside>
        </div>
      </div>
    );
  }

  mySidebar() {
    return (
      <div>
        {!this.state.loading && (
          <div>
            <div class="card bg-transparent border-secondary mb-3">
              <div class="card-header bg-transparent border-secondary">
                <div class="mb-2">
                  {this.siteName()}
                  {this.adminButtons()}
                </div>
                <BannerIconHeader banner={this.state.siteRes.site.banner} />
              </div>
              <div class="card-body">
                {this.trendingCommunities()}
                {this.createCommunityButton()}
                {/*
                {this.subscribedCommunities()}
                */}
              </div>
            </div>

            <div class="card bg-transparent border-secondary mb-3">
              <div class="card-body">{this.sidebar()}</div>
            </div>

            <div class="card bg-transparent border-secondary">
              <div class="card-body">{this.landing()}</div>
            </div>
          </div>
        )}
      </div>
    );
  }

  createCommunityButton() {
    return (
      <Link class="btn btn-secondary btn-block" to="/create_community">
        {i18n.t('create_a_community')}
      </Link>
    );
  }

  trendingCommunities() {
    return (
      <div>
        <h5>
          <T i18nKey="trending_communities">
            #
            <Link class="text-body" to="/communities">
              #
            </Link>
          </T>
        </h5>
        <ul class="list-inline">
          {this.state.trendingCommunities.map(community => (
            <li class="list-inline-item">
              <CommunityLink community={community} />
            </li>
          ))}
        </ul>
      </div>
    );
  }

  subscribedCommunities() {
    return (
      UserService.Instance.user &&
      this.state.subscribedCommunities.length > 0 && (
        <div>
          <h5>
            <T i18nKey="subscribed_to_communities">
              #
              <Link class="text-body" to="/communities">
                #
              </Link>
            </T>
          </h5>
          <ul class="list-inline">
            {this.state.subscribedCommunities.map(community => (
              <li class="list-inline-item">
                <CommunityLink
                  community={{
                    name: community.community_name,
                    id: community.community_id,
                    local: community.community_local,
                    actor_id: community.community_actor_id,
                    icon: community.community_icon,
                  }}
                />
              </li>
            ))}
          </ul>
        </div>
      )
    );
  }

  sidebar() {
    return (
      <div>
        {!this.state.showEditSite ? (
          this.siteInfo()
        ) : (
          <SiteForm
            site={this.state.siteRes.site}
            onCancel={this.handleEditCancel}
          />
        )}
      </div>
    );
  }

  updateUrl(paramUpdates: UrlParams) {
    const listingTypeStr = paramUpdates.listingType || this.state.listingType;
    const dataTypeStr = paramUpdates.dataType || DataType[this.state.dataType];
    const sortStr = paramUpdates.sort || this.state.sort;
    const page = paramUpdates.page || this.state.page;
    this.props.history.push(
      `/home/data_type/${dataTypeStr}/listing_type/${listingTypeStr}/sort/${sortStr}/page/${page}`
    );
  }

  siteInfo() {
    return (
      <div>
        {this.state.siteRes.site.description && this.siteDescription()}
        {this.badges()}
        {this.admins()}
      </div>
    );
  }

  siteName() {
    return <h5 class="mb-0">{`${this.state.siteRes.site.name}`}</h5>;
  }

  admins() {
    return (
      <ul class="mt-1 list-inline small mb-0">
        <li class="list-inline-item">{i18n.t('admins')}:</li>
        {this.state.siteRes.admins.map(admin => (
          <li class="list-inline-item">
            <UserListing
              user={{
                name: admin.name,
                preferred_username: admin.preferred_username,
                avatar: admin.avatar,
                local: admin.local,
                actor_id: admin.actor_id,
                id: admin.id,
              }}
            />
          </li>
        ))}
      </ul>
    );
  }

  badges() {
    return (
      <ul class="my-2 list-inline">
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_online', { count: this.state.siteRes.online })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_users', {
            count: this.state.siteRes.site.number_of_users,
          })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_communities', {
            count: this.state.siteRes.site.number_of_communities,
          })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_posts', {
            count: this.state.siteRes.site.number_of_posts,
          })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_comments', {
            count: this.state.siteRes.site.number_of_comments,
          })}
        </li>
        <li className="list-inline-item">
          <Link className="badge badge-light" to="/modlog">
            {i18n.t('modlog')}
          </Link>
        </li>
      </ul>
    );
  }

  adminButtons() {
    return (
      this.canAdmin && (
        <ul class="list-inline mb-1 text-muted font-weight-bold">
          <li className="list-inline-item-action">
            <span
              class="pointer"
              onClick={linkEvent(this, this.handleEditClick)}
              data-tippy-content={i18n.t('edit')}
            >
              <svg class="icon icon-inline">
                <use xlinkHref="#icon-edit"></use>
              </svg>
            </span>
          </li>
        </ul>
      )
    );
  }

  siteDescription() {
    return (
      <div
        className="md-div"
        dangerouslySetInnerHTML={mdToHtml(this.state.siteRes.site.description)}
      />
    );
  }

  landing() {
    return (
      <>
        <h5>
          {i18n.t('powered_by')}
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
            <br class="big"></br>
            <code>#</code>
            <br></br>
            <b>#</b>
            <br class="big"></br>
            <a href={repoUrl}>#</a>
            <br class="big"></br>
            <a href="https://www.rust-lang.org">#</a>
            <a href="https://actix.rs/">#</a>
            <a href="https://infernojs.org">#</a>
            <a href="https://www.typescriptlang.org/">#</a>
            <br class="big"></br>
            <a href="https://github.com/LemmyNet/lemmy/graphs/contributors?type=a">
              #
            </a>
          </T>
        </p>
      </>
    );
  }

  posts() {
    return (
      <div class="main-content-wrapper">
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div>
            {this.selects()}
            {this.listings()}
            {this.paginator()}
          </div>
        )}
      </div>
    );
  }

  listings() {
    return this.state.dataType == DataType.Post ? (
      <PostListings
        posts={this.state.posts}
        showCommunity
        removeDuplicates
        sort={this.state.sort}
        enableDownvotes={this.state.siteRes.site.enable_downvotes}
        enableNsfw={this.state.siteRes.site.enable_nsfw}
      />
    ) : (
      <CommentNodes
        nodes={commentsToFlatNodes(this.state.comments)}
        noIndent
        showCommunity
        sortType={this.state.sort}
        showContext
        enableDownvotes={this.state.siteRes.site.enable_downvotes}
      />
    );
  }

  selects() {
    return (
      <div className="mb-3">
        <span class="mr-3">
          <DataTypeSelect
            type_={this.state.dataType}
            onChange={this.handleDataTypeChange}
          />
        </span>
        <span class="mr-3">
          <ListingTypeSelect
            type_={this.state.listingType}
            showLocal={
              this.state.siteRes.federated_instances &&
              this.state.siteRes.federated_instances.length > 0
            }
            onChange={this.handleListingTypeChange}
          />
        </span>
        <span class="mr-2">
          <SortSelect sort={this.state.sort} onChange={this.handleSortChange} />
        </span>
        {this.state.listingType == ListingType.All && (
          <a
            href={`/feeds/all.xml?sort=${this.state.sort}`}
            target="_blank"
            rel="noopener"
            title="RSS"
          >
            <svg class="icon text-muted small">
              <use xlinkHref="#icon-rss">#</use>
            </svg>
          </a>
        )}
        {UserService.Instance.user &&
          this.state.listingType == ListingType.Subscribed && (
            <a
              href={`/feeds/front/${UserService.Instance.auth}.xml?sort=${this.state.sort}`}
              target="_blank"
              title="RSS"
              rel="noopener"
            >
              <svg class="icon text-muted small">
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
            class="btn btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            {i18n.t('prev')}
          </button>
        )}
        {this.state.posts.length > 0 && (
          <button
            class="btn btn-secondary"
            onClick={linkEvent(this, this.nextPage)}
          >
            {i18n.t('next')}
          </button>
        )}
      </div>
    );
  }

  get canAdmin(): boolean {
    return (
      UserService.Instance.user &&
      this.state.siteRes.admins
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
    i.updateUrl({ page: i.state.page + 1 });
    window.scrollTo(0, 0);
  }

  prevPage(i: Main) {
    i.updateUrl({ page: i.state.page - 1 });
    window.scrollTo(0, 0);
  }

  handleSortChange(val: SortType) {
    this.updateUrl({ sort: val, page: 1 });
    window.scrollTo(0, 0);
  }

  handleListingTypeChange(val: ListingType) {
    this.updateUrl({ listingType: val, page: 1 });
    window.scrollTo(0, 0);
  }

  handleDataTypeChange(val: DataType) {
    this.updateUrl({ dataType: DataType[val], page: 1 });
    window.scrollTo(0, 0);
  }

  fetchData() {
    if (this.state.dataType == DataType.Post) {
      let getPostsForm: GetPostsForm = {
        page: this.state.page,
        limit: fetchLimit,
        sort: this.state.sort,
        type_: this.state.listingType,
      };
      WebSocketService.Instance.getPosts(getPostsForm);
    } else {
      let getCommentsForm: GetCommentsForm = {
        page: this.state.page,
        limit: fetchLimit,
        sort: this.state.sort,
        type_: this.state.listingType,
      };
      WebSocketService.Instance.getComments(getCommentsForm);
    }
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (msg.reconnect) {
      this.fetchData();
    } else if (res.op == UserOperation.GetFollowedCommunities) {
      let data = res.data as GetFollowedCommunitiesResponse;
      this.state.subscribedCommunities = data.communities;
      this.setState(this.state);
    } else if (res.op == UserOperation.ListCommunities) {
      let data = res.data as ListCommunitiesResponse;
      this.state.trendingCommunities = data.communities;
      this.setState(this.state);
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;

      // This means it hasn't been set up yet
      if (!data.site) {
        this.context.router.history.push('/setup');
      }
      this.state.siteRes.admins = data.admins;
      this.state.siteRes.site = data.site;
      this.state.siteRes.banned = data.banned;
      this.state.siteRes.online = data.online;
      this.setState(this.state);
    } else if (res.op == UserOperation.EditSite) {
      let data = res.data as SiteResponse;
      this.state.siteRes.site = data.site;
      this.state.showEditSite = false;
      this.setState(this.state);
      toast(i18n.t('site_saved'));
    } else if (res.op == UserOperation.GetPosts) {
      let data = res.data as GetPostsResponse;
      this.state.posts = data.posts;
      this.state.loading = false;
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.CreatePost) {
      let data = res.data as PostResponse;

      // If you're on subscribed, only push it if you're subscribed.
      if (this.state.listingType == ListingType.Subscribed) {
        if (
          this.state.subscribedCommunities
            .map(c => c.community_id)
            .includes(data.post.community_id)
        ) {
          this.state.posts.unshift(data.post);
          notifyPost(data.post, this.context.router);
        }
      } else {
        // NSFW posts
        let nsfw = data.post.nsfw || data.post.community_nsfw;

        // Don't push the post if its nsfw, and don't have that setting on
        if (
          !nsfw ||
          (nsfw &&
            UserService.Instance.user &&
            UserService.Instance.user.show_nsfw)
        ) {
          this.state.posts.unshift(data.post);
          notifyPost(data.post, this.context.router);
        }
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.EditPost) {
      let data = res.data as PostResponse;
      editPostFindRes(data, this.state.posts);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePostLike) {
      let data = res.data as PostResponse;
      createPostLikeFindRes(data, this.state.posts);
      this.setState(this.state);
    } else if (res.op == UserOperation.AddAdmin) {
      let data = res.data as AddAdminResponse;
      this.state.siteRes.admins = data.admins;
      this.setState(this.state);
    } else if (res.op == UserOperation.BanUser) {
      let data = res.data as BanUserResponse;
      let found = this.state.siteRes.banned.find(u => (u.id = data.user.id));

      // Remove the banned if its found in the list, and the action is an unban
      if (found && !data.banned) {
        this.state.siteRes.banned = this.state.siteRes.banned.filter(
          i => i.id !== data.user.id
        );
      } else {
        this.state.siteRes.banned.push(data.user);
      }

      this.state.posts
        .filter(p => p.creator_id == data.user.id)
        .forEach(p => (p.banned = data.banned));

      this.setState(this.state);
    } else if (res.op == UserOperation.GetComments) {
      let data = res.data as GetCommentsResponse;
      this.state.comments = data.comments;
      this.state.loading = false;
      this.setState(this.state);
    } else if (
      res.op == UserOperation.EditComment ||
      res.op == UserOperation.DeleteComment ||
      res.op == UserOperation.RemoveComment
    ) {
      let data = res.data as CommentResponse;
      editCommentRes(data, this.state.comments);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      // Necessary since it might be a user reply
      if (data.recipient_ids.length == 0) {
        // If you're on subscribed, only push it if you're subscribed.
        if (this.state.listingType == ListingType.Subscribed) {
          if (
            this.state.subscribedCommunities
              .map(c => c.community_id)
              .includes(data.comment.community_id)
          ) {
            this.state.comments.unshift(data.comment);
          }
        } else {
          this.state.comments.unshift(data.comment);
        }
        this.setState(this.state);
      }
    } else if (res.op == UserOperation.SaveComment) {
      let data = res.data as CommentResponse;
      saveCommentRes(data, this.state.comments);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateCommentLike) {
      let data = res.data as CommentResponse;
      createCommentLikeRes(data, this.state.comments);
      this.setState(this.state);
    }
  }
}
