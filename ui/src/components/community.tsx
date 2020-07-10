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
  DataType,
  GetPostsResponse,
  PostResponse,
  AddModToCommunityResponse,
  BanFromCommunityResponse,
  Comment,
  GetCommentsForm,
  GetCommentsResponse,
  CommentResponse,
  WebSocketJsonResponse,
  GetSiteResponse,
  Site,
} from '../interfaces';
import { WebSocketService } from '../services';
import { PostListings } from './post-listings';
import { CommentNodes } from './comment-nodes';
import { SortSelect } from './sort-select';
import { DataTypeSelect } from './data-type-select';
import { Sidebar } from './sidebar';
import {
  wsJsonToRes,
  fetchLimit,
  toast,
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
} from '../utils';
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
  comments: Array<Comment>;
  dataType: DataType;
  sort: SortType;
  page: number;
  site: Site;
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
      local: null,
      actor_id: null,
      last_refreshed_at: null,
      creator_actor_id: null,
      creator_local: null,
    },
    moderators: [],
    admins: [],
    communityId: Number(this.props.match.params.id),
    communityName: this.props.match.params.name,
    online: null,
    loading: true,
    posts: [],
    comments: [],
    dataType: getDataTypeFromProps(this.props),
    sort: getSortTypeFromProps(this.props),
    page: getPageFromProps(this.props),
    site: {
      id: undefined,
      name: undefined,
      creator_id: undefined,
      published: undefined,
      creator_name: undefined,
      number_of_users: undefined,
      number_of_posts: undefined,
      number_of_comments: undefined,
      number_of_communities: undefined,
      enable_downvotes: undefined,
      open_registration: undefined,
      enable_nsfw: undefined,
    },
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleSortChange = this.handleSortChange.bind(this);
    this.handleDataTypeChange = this.handleDataTypeChange.bind(this);

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
    WebSocketService.Instance.getSite();
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
      this.state.dataType = getDataTypeFromProps(nextProps);
      this.state.sort = getSortTypeFromProps(nextProps);
      this.state.page = getPageFromProps(nextProps);
      this.setState(this.state);
      this.fetchData();
    }
  }

  render() {
    return (
      <div class="container">
        {this.selects()}
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
              {this.listings()}
              {this.paginator()}
            </div>
            <div class="col-12 col-md-4">
              <Sidebar
                community={this.state.community}
                moderators={this.state.moderators}
                admins={this.state.admins}
                online={this.state.online}
                enableNsfw={this.state.site.enable_nsfw}
              />
            </div>
          </div>
        )}
      </div>
    );
  }

  listings() {
    return this.state.dataType == DataType.Post ? (
      <PostListings
        posts={this.state.posts}
        removeDuplicates
        sort={this.state.sort}
        enableDownvotes={this.state.site.enable_downvotes}
        enableNsfw={this.state.site.enable_nsfw}
      />
    ) : (
      <CommentNodes
        nodes={commentsToFlatNodes(this.state.comments)}
        noIndent
        sortType={this.state.sort}
        showContext
        enableDownvotes={this.state.site.enable_downvotes}
      />
    );
  }

  selects() {
    return (
      <div class="mb-3">
        <span class="mr-3">
          <DataTypeSelect
            type_={this.state.dataType}
            onChange={this.handleDataTypeChange}
          />
        </span>
        <span class="mr-2">
          <SortSelect sort={this.state.sort} onChange={this.handleSortChange} />
        </span>
        <a
          href={`/feeds/c/${this.state.communityName}.xml?sort=${
            SortType[this.state.sort]
          }`}
          target="_blank"
          title="RSS"
          rel="noopener"
        >
          <svg class="icon text-muted small">
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
        {this.state.posts.length > 0 && (
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
    i.fetchData();
    window.scrollTo(0, 0);
  }

  prevPage(i: Community) {
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.fetchData();
    window.scrollTo(0, 0);
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.state.loading = true;
    this.setState(this.state);
    this.updateUrl();
    this.fetchData();
    window.scrollTo(0, 0);
  }

  handleDataTypeChange(val: DataType) {
    this.state.dataType = val;
    this.state.page = 1;
    this.state.loading = true;
    this.setState(this.state);
    this.updateUrl();
    this.fetchData();
    window.scrollTo(0, 0);
  }

  updateUrl() {
    let dataTypeStr = DataType[this.state.dataType].toLowerCase();
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(
      `/c/${this.state.community.name}/data_type/${dataTypeStr}/sort/${sortStr}/page/${this.state.page}`
    );
  }

  fetchData() {
    if (this.state.dataType == DataType.Post) {
      let getPostsForm: GetPostsForm = {
        page: this.state.page,
        limit: fetchLimit,
        sort: SortType[this.state.sort],
        type_: ListingType[ListingType.Community],
        community_id: this.state.community.id,
      };
      WebSocketService.Instance.getPosts(getPostsForm);
    } else {
      let getCommentsForm: GetCommentsForm = {
        page: this.state.page,
        limit: fetchLimit,
        sort: SortType[this.state.sort],
        type_: ListingType[ListingType.Community],
        community_id: this.state.community.id,
      };
      WebSocketService.Instance.getComments(getCommentsForm);
    }
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.context.router.history.push('/');
      return;
    } else if (msg.reconnect) {
      this.fetchData();
    } else if (res.op == UserOperation.GetCommunity) {
      let data = res.data as GetCommunityResponse;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.state.admins = data.admins;
      this.state.online = data.online;
      document.title = `/c/${this.state.community.name} - ${this.state.site.name}`;
      this.setState(this.state);
      this.fetchData();
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
      setupTippy();
    } else if (res.op == UserOperation.EditPost) {
      let data = res.data as PostResponse;
      editPostFindRes(data, this.state.posts);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePost) {
      let data = res.data as PostResponse;
      this.state.posts.unshift(data.post);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePostLike) {
      let data = res.data as PostResponse;
      createPostLikeFindRes(data, this.state.posts);
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
    } else if (res.op == UserOperation.GetComments) {
      let data = res.data as GetCommentsResponse;
      this.state.comments = data.comments;
      this.state.loading = false;
      this.setState(this.state);
    } else if (res.op == UserOperation.EditComment) {
      let data = res.data as CommentResponse;
      editCommentRes(data, this.state.comments);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      // Necessary since it might be a user reply
      if (data.recipient_ids.length == 0) {
        this.state.comments.unshift(data.comment);
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
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.site = data.site;
      this.setState(this.state);
    }
  }
}
