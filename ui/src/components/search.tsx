import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Post,
  Comment,
  Community,
  UserView,
  SortType,
  SearchForm,
  SearchResponse,
  SearchType,
  CreatePostLikeResponse,
  CommentResponse,
  WebSocketJsonResponse,
} from '../interfaces';
import { WebSocketService } from '../services';
import {
  wsJsonToRes,
  fetchLimit,
  routeSearchTypeToEnum,
  routeSortTypeToEnum,
  pictshareAvatarThumbnail,
  showAvatars,
  toast,
} from '../utils';
import { PostListing } from './post-listing';
import { SortSelect } from './sort-select';
import { CommentNodes } from './comment-nodes';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface SearchState {
  q: string;
  type_: SearchType;
  sort: SortType;
  page: number;
  searchResponse: SearchResponse;
  loading: boolean;
}

export class Search extends Component<any, SearchState> {
  private subscription: Subscription;
  private emptyState: SearchState = {
    q: this.getSearchQueryFromProps(this.props),
    type_: this.getSearchTypeFromProps(this.props),
    sort: this.getSortTypeFromProps(this.props),
    page: this.getPageFromProps(this.props),
    searchResponse: {
      type_: null,
      posts: [],
      comments: [],
      communities: [],
      users: [],
    },
    loading: false,
  };

  getSearchQueryFromProps(props: any): string {
    return props.match.params.q ? props.match.params.q : '';
  }

  getSearchTypeFromProps(props: any): SearchType {
    return props.match.params.type
      ? routeSearchTypeToEnum(props.match.params.type)
      : SearchType.All;
  }

  getSortTypeFromProps(props: any): SortType {
    return props.match.params.sort
      ? routeSortTypeToEnum(props.match.params.sort)
      : SortType.TopAll;
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

    if (this.state.q) {
      this.search();
    }
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
      this.state = this.emptyState;
      this.state.q = this.getSearchQueryFromProps(nextProps);
      this.state.type_ = this.getSearchTypeFromProps(nextProps);
      this.state.sort = this.getSortTypeFromProps(nextProps);
      this.state.page = this.getPageFromProps(nextProps);
      this.setState(this.state);
      this.search();
    }
  }

  componentDidMount() {
    document.title = `${i18n.t('search')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  render() {
    return (
      <div class="container">
        <h5>
          { i18n.t('search') }
        </h5>
        {this.selects()}
        {this.searchForm()}
        {this.state.type_ == SearchType.All && this.all()}
        {this.state.type_ == SearchType.Comments && this.comments()}
        {this.state.type_ == SearchType.Posts && this.posts()}
        {this.state.type_ == SearchType.Communities && this.communities()}
        {this.state.type_ == SearchType.Users && this.users()}
        {this.noResults()}
        {this.paginator()}
      </div>
    );
  }

  searchForm() {
    return (
      <form
        class="form-inline"
        onSubmit={linkEvent(this, this.handleSearchSubmit)}
      >
        <input
          type="text"
          class="form-control mr-2"
          value={this.state.q}
          placeholder={`${i18n.t('search')}...`}
          onInput={linkEvent(this, this.handleQChange)}
          required
          minLength={3}
        />
        <button type="submit" class="btn btn-secondary mr-2">
          {this.state.loading ? (
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          ) : (
            <span>
              { i18n.t('search') }
            </span>
          )}
        </button>
      </form>
    );
  }

  selects() {
    return (
      <div className="mb-2">
        <select
          value={this.state.type_}
          onChange={linkEvent(this, this.handleTypeChange)}
          class="custom-select custom-select-sm w-auto"
        >
          <option disabled>
            { i18n.t('type') }
          </option>
          <option value={SearchType.All}>
            { i18n.t('all') }
          </option>
          <option value={SearchType.Comments}>
            { i18n.t('comments') }
          </option>
          <option value={SearchType.Posts}>
            { i18n.t('posts') }
          </option>
          <option value={SearchType.Communities}>
            { i18n.t('communities') }
          </option>
          <option value={SearchType.Users}>
            { i18n.t('users') }
          </option>
        </select>
        <span class="ml-2">
          <SortSelect
            sort={this.state.sort}
            onChange={this.handleSortChange}
            hideHot
          />
        </span>
      </div>
    );
  }

  all() {
    let combined: Array<{
      type_: string;
      data: Comment | Post | Community | UserView;
    }> = [];
    let comments = this.state.searchResponse.comments.map(e => {
      return { type_: 'comments', data: e };
    });
    let posts = this.state.searchResponse.posts.map(e => {
      return { type_: 'posts', data: e };
    });
    let communities = this.state.searchResponse.communities.map(e => {
      return { type_: 'communities', data: e };
    });
    let users = this.state.searchResponse.users.map(e => {
      return { type_: 'users', data: e };
    });

    combined.push(...comments);
    combined.push(...posts);
    combined.push(...communities);
    combined.push(...users);

    // Sort it
    if (this.state.sort == SortType.New) {
      combined.sort((a, b) => b.data.published.localeCompare(a.data.published));
    } else {
      combined.sort(
        (a, b) =>
          ((b.data as Comment | Post).score |
            (b.data as Community).number_of_subscribers |
            (b.data as UserView).comment_score) -
          ((a.data as Comment | Post).score |
            (a.data as Community).number_of_subscribers |
            (a.data as UserView).comment_score)
      );
    }

    return (
      <div>
        {combined.map(i => (
          <div class="row">
            <div class="col-12">
              {i.type_ == 'posts' && (
                <PostListing post={i.data as Post} showCommunity />
              )}
              {i.type_ == 'comments' && (
                <CommentNodes
                  nodes={[{ comment: i.data as Comment }]}
                  locked
                  noIndent
                />
              )}
              {i.type_ == 'communities' && (
                <div>
                  <span>
                    <Link to={`/c/${(i.data as Community).name}`}>{`/c/${
                      (i.data as Community).name
                    }`}</Link>
                  </span>
                  <span>{` - ${(i.data as Community).title} - ${
                    (i.data as Community).number_of_subscribers
                  } subscribers`}</span>
                </div>
              )}
              {i.type_ == 'users' && (
                <div>
                  <span>
                    <Link
                      className="text-info"
                      to={`/u/${(i.data as UserView).name}`}
                    >
                      {(i.data as UserView).avatar && showAvatars() && (
                        <img
                          height="32"
                          width="32"
                          src={pictshareAvatarThumbnail(
                            (i.data as UserView).avatar
                          )}
                          class="rounded-circle mr-1"
                        />
                      )}
                      <span>{`/u/${(i.data as UserView).name}`}</span>
                    </Link>
                  </span>
                  <span>{` - ${
                    (i.data as UserView).comment_score
                  } comment karma`}</span>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    );
  }

  comments() {
    return (
      <>
        {this.state.searchResponse.comments.map(comment => (
          <div class="row">
            <div class="col-12">
              <CommentNodes nodes={[{ comment: comment }]} locked noIndent />
            </div>
          </div>
        ))}
      </>
    );
  }

  posts() {
    return (
      <>
        {this.state.searchResponse.posts.map(post => (
          <div class="row">
            <div class="col-12">
              <PostListing post={post} showCommunity />
            </div>
          </div>
        ))}
      </>
    );
  }

  // Todo possibly create UserListing and CommunityListing
  communities() {
    return (
      <>
        {this.state.searchResponse.communities.map(community => (
          <div class="row">
            <div class="col-12">
              <span>
                <Link
                  to={`/c/${community.name}`}
                >{`/c/${community.name}`}</Link>
              </span>
              <span>{` - ${community.title} - ${community.number_of_subscribers} subscribers`}</span>
            </div>
          </div>
        ))}
      </>
    );
  }

  users() {
    return (
      <>
        {this.state.searchResponse.users.map(user => (
          <div class="row">
            <div class="col-12">
              <span>
                <Link
                  className="text-info"
                  to={`/u/${user.name}`}
                >{`/u/${user.name}`}</Link>
              </span>
              <span>{` - ${user.comment_score} comment karma`}</span>
            </div>
          </div>
        ))}
      </>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && (
          <button
            class="btn btn-sm btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            { i18n.t('prev') }
          </button>
        )}
        <button
          class="btn btn-sm btn-secondary"
          onClick={linkEvent(this, this.nextPage)}
        >
          { i18n.t('next') }
        </button>
      </div>
    );
  }

  noResults() {
    let res = this.state.searchResponse;
    return (
      <div>
        {res &&
          res.posts.length == 0 &&
          res.comments.length == 0 &&
          res.communities.length == 0 &&
          res.users.length == 0 && (
            <span>
              { i18n.t('no_results') }
            </span>
          )}
      </div>
    );
  }

  nextPage(i: Search) {
    i.state.page++;
    i.setState(i.state);
    i.updateUrl();
    i.search();
  }

  prevPage(i: Search) {
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.search();
  }

  search() {
    let form: SearchForm = {
      q: this.state.q,
      type_: SearchType[this.state.type_],
      sort: SortType[this.state.sort],
      page: this.state.page,
      limit: fetchLimit,
    };

    if (this.state.q != '') {
      WebSocketService.Instance.search(form);
    }
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.setState(this.state);
    this.updateUrl();
  }

  handleTypeChange(i: Search, event: any) {
    i.state.type_ = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.updateUrl();
  }

  handleSearchSubmit(i: Search, event: any) {
    event.preventDefault();
    i.state.loading = true;
    i.search();
    i.setState(i.state);
    i.updateUrl();
  }

  handleQChange(i: Search, event: any) {
    i.state.q = event.target.value;
    i.setState(i.state);
  }

  updateUrl() {
    let typeStr = SearchType[this.state.type_].toLowerCase();
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(
      `/search/q/${this.state.q}/type/${typeStr}/sort/${sortStr}/page/${this.state.page}`
    );
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.Search) {
      let data = res.data as SearchResponse;
      this.state.searchResponse = data;
      this.state.loading = false;
      document.title = `${i18n.t('search')} - ${this.state.q} - ${
        WebSocketService.Instance.site.name
      }`;
      window.scrollTo(0, 0);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateCommentLike) {
      let data = res.data as CommentResponse;
      let found: Comment = this.state.searchResponse.comments.find(
        c => c.id === data.comment.id
      );
      found.score = data.comment.score;
      found.upvotes = data.comment.upvotes;
      found.downvotes = data.comment.downvotes;
      if (data.comment.my_vote !== null) {
        found.my_vote = data.comment.my_vote;
        found.upvoteLoading = false;
        found.downvoteLoading = false;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePostLike) {
      let data = res.data as CreatePostLikeResponse;
      let found = this.state.searchResponse.posts.find(
        c => c.id == data.post.id
      );
      found.my_vote = data.post.my_vote;
      found.score = data.post.score;
      found.upvotes = data.post.upvotes;
      found.downvotes = data.post.downvotes;
      this.setState(this.state);
    }
  }
}
