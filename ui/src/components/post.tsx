import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Community,
  Post as PostI,
  GetPostResponse,
  PostResponse,
  Comment,
  MarkCommentAsReadForm,
  CommentResponse,
  CommentSortType,
  CommentViewType,
  CommunityUser,
  CommunityResponse,
  CommentNode as CommentNodeI,
  BanFromCommunityResponse,
  BanUserResponse,
  AddModToCommunityResponse,
  AddAdminResponse,
  SearchType,
  SortType,
  SearchForm,
  GetPostForm,
  SearchResponse,
  GetSiteResponse,
  GetCommunityResponse,
  WebSocketJsonResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  wsJsonToRes,
  toast,
  editCommentRes,
  saveCommentRes,
  createCommentLikeRes,
  createPostLikeRes,
  commentsToFlatNodes,
  setupTippy,
  favIconUrl,
} from '../utils';
import { PostListing } from './post-listing';
import { Sidebar } from './sidebar';
import { CommentForm } from './comment-form';
import { CommentNodes } from './comment-nodes';
import autosize from 'autosize';
import { i18n } from '../i18next';

interface PostState {
  post: PostI;
  comments: Array<Comment>;
  commentSort: CommentSortType;
  commentViewType: CommentViewType;
  community: Community;
  moderators: Array<CommunityUser>;
  online: number;
  scrolled?: boolean;
  scrolled_comment_id?: number;
  loading: boolean;
  crossPosts: Array<PostI>;
  siteRes: GetSiteResponse;
}

export class Post extends Component<any, PostState> {
  private subscription: Subscription;
  private emptyState: PostState = {
    post: null,
    comments: [],
    commentSort: CommentSortType.Hot,
    commentViewType: CommentViewType.Tree,
    community: null,
    moderators: [],
    online: null,
    scrolled: false,
    loading: true,
    crossPosts: [],
    siteRes: {
      admins: [],
      banned: [],
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
        icon: undefined,
        banner: undefined,
      },
      online: null,
      version: null,
      federated_instances: undefined,
    },
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    let postId = Number(this.props.match.params.id);
    if (this.props.match.params.comment_id) {
      this.state.scrolled_comment_id = this.props.match.params.comment_id;
    }

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    let form: GetPostForm = {
      id: postId,
    };
    WebSocketService.Instance.getPost(form);
    WebSocketService.Instance.getSite();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  componentDidUpdate(_lastProps: any, lastState: PostState, _snapshot: any) {
    if (
      this.state.scrolled_comment_id &&
      !this.state.scrolled &&
      lastState.comments.length > 0
    ) {
      var elmnt = document.getElementById(
        `comment-${this.state.scrolled_comment_id}`
      );
      elmnt.scrollIntoView();
      elmnt.classList.add('mark');
      this.state.scrolled = true;
      this.markScrolledAsRead(this.state.scrolled_comment_id);
    }

    // Necessary if you are on a post and you click another post (same route)
    if (_lastProps.location.pathname !== _lastProps.history.location.pathname) {
      // Couldnt get a refresh working. This does for now.
      location.reload();

      // let currentId = this.props.match.params.id;
      // WebSocketService.Instance.getPost(currentId);
      // this.context.router.history.push('/sponsors');
      // this.context.refresh();
      // this.context.router.history.push(_lastProps.location.pathname);
    }
  }

  markScrolledAsRead(commentId: number) {
    let found = this.state.comments.find(c => c.id == commentId);
    let parent = this.state.comments.find(c => found.parent_id == c.id);
    let parent_user_id = parent
      ? parent.creator_id
      : this.state.post.creator_id;

    if (
      UserService.Instance.user &&
      UserService.Instance.user.id == parent_user_id
    ) {
      let form: MarkCommentAsReadForm = {
        edit_id: found.id,
        read: true,
        auth: null,
      };
      WebSocketService.Instance.markCommentAsRead(form);
      UserService.Instance.unreadCountSub.next(
        UserService.Instance.unreadCountSub.value - 1
      );
    }
  }

  get documentTitle(): string {
    if (this.state.post) {
      return `${this.state.post.name} - ${this.state.siteRes.site.name}`;
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
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div class="row">
            <div class="col-12 col-md-8 mb-3">
              <PostListing
                post={this.state.post}
                showBody
                showCommunity
                moderators={this.state.moderators}
                admins={this.state.siteRes.admins}
                enableDownvotes={this.state.siteRes.site.enable_downvotes}
                enableNsfw={this.state.siteRes.site.enable_nsfw}
              />
              <div className="mb-2" />
              <CommentForm
                postId={this.state.post.id}
                disabled={this.state.post.locked}
              />
              {this.state.comments.length > 0 && this.sortRadios()}
              {this.state.commentViewType == CommentViewType.Tree &&
                this.commentsTree()}
              {this.state.commentViewType == CommentViewType.Chat &&
                this.commentsFlat()}
            </div>
            <div class="col-12 col-sm-12 col-md-4">{this.sidebar()}</div>
          </div>
        )}
      </div>
    );
  }

  sortRadios() {
    return (
      <>
        <div class="btn-group btn-group-toggle flex-wrap mr-3 mb-2">
          <label
            className={`btn btn-outline-secondary pointer ${
              this.state.commentSort === CommentSortType.Hot && 'active'
            }`}
          >
            {i18n.t('hot')}
            <input
              type="radio"
              value={CommentSortType.Hot}
              checked={this.state.commentSort === CommentSortType.Hot}
              onChange={linkEvent(this, this.handleCommentSortChange)}
            />
          </label>
          <label
            className={`btn btn-outline-secondary pointer ${
              this.state.commentSort === CommentSortType.Top && 'active'
            }`}
          >
            {i18n.t('top')}
            <input
              type="radio"
              value={CommentSortType.Top}
              checked={this.state.commentSort === CommentSortType.Top}
              onChange={linkEvent(this, this.handleCommentSortChange)}
            />
          </label>
          <label
            className={`btn btn-outline-secondary pointer ${
              this.state.commentSort === CommentSortType.New && 'active'
            }`}
          >
            {i18n.t('new')}
            <input
              type="radio"
              value={CommentSortType.New}
              checked={this.state.commentSort === CommentSortType.New}
              onChange={linkEvent(this, this.handleCommentSortChange)}
            />
          </label>
          <label
            className={`btn btn-outline-secondary pointer ${
              this.state.commentSort === CommentSortType.Old && 'active'
            }`}
          >
            {i18n.t('old')}
            <input
              type="radio"
              value={CommentSortType.Old}
              checked={this.state.commentSort === CommentSortType.Old}
              onChange={linkEvent(this, this.handleCommentSortChange)}
            />
          </label>
        </div>
        <div class="btn-group btn-group-toggle flex-wrap mb-2">
          <label
            className={`btn btn-outline-secondary pointer ${
              this.state.commentViewType === CommentViewType.Chat && 'active'
            }`}
          >
            {i18n.t('chat')}
            <input
              type="radio"
              value={CommentViewType.Chat}
              checked={this.state.commentViewType === CommentViewType.Chat}
              onChange={linkEvent(this, this.handleCommentViewTypeChange)}
            />
          </label>
        </div>
      </>
    );
  }

  commentsFlat() {
    return (
      <div>
        <CommentNodes
          nodes={commentsToFlatNodes(this.state.comments)}
          noIndent
          locked={this.state.post.locked}
          moderators={this.state.moderators}
          admins={this.state.siteRes.admins}
          postCreatorId={this.state.post.creator_id}
          showContext
          enableDownvotes={this.state.siteRes.site.enable_downvotes}
          sort={this.state.commentSort}
        />
      </div>
    );
  }

  sidebar() {
    return (
      <div class="mb-3">
        <Sidebar
          community={this.state.community}
          moderators={this.state.moderators}
          admins={this.state.siteRes.admins}
          online={this.state.online}
          enableNsfw={this.state.siteRes.site.enable_nsfw}
          showIcon
        />
      </div>
    );
  }

  handleCommentSortChange(i: Post, event: any) {
    i.state.commentSort = Number(event.target.value);
    i.state.commentViewType = CommentViewType.Tree;
    i.setState(i.state);
  }

  handleCommentViewTypeChange(i: Post, event: any) {
    i.state.commentViewType = Number(event.target.value);
    i.state.commentSort = CommentSortType.New;
    i.setState(i.state);
  }

  buildCommentsTree(): Array<CommentNodeI> {
    let map = new Map<number, CommentNodeI>();
    for (let comment of this.state.comments) {
      let node: CommentNodeI = {
        comment: comment,
        children: [],
      };
      map.set(comment.id, { ...node });
    }
    let tree: Array<CommentNodeI> = [];
    for (let comment of this.state.comments) {
      let child = map.get(comment.id);
      if (comment.parent_id) {
        let parent_ = map.get(comment.parent_id);
        parent_.children.push(child);
      } else {
        tree.push(child);
      }

      this.setDepth(child);
    }

    return tree;
  }

  setDepth(node: CommentNodeI, i: number = 0): void {
    for (let child of node.children) {
      child.comment.depth = i;
      this.setDepth(child, i + 1);
    }
  }

  commentsTree() {
    let nodes = this.buildCommentsTree();
    return (
      <div>
        <CommentNodes
          nodes={nodes}
          locked={this.state.post.locked}
          moderators={this.state.moderators}
          admins={this.state.siteRes.admins}
          postCreatorId={this.state.post.creator_id}
          sort={this.state.commentSort}
          enableDownvotes={this.state.siteRes.site.enable_downvotes}
        />
      </div>
    );
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (msg.reconnect) {
      WebSocketService.Instance.getPost({
        id: Number(this.props.match.params.id),
      });
    } else if (res.op == UserOperation.GetPost) {
      let data = res.data as GetPostResponse;
      this.state.post = data.post;
      this.state.comments = data.comments;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.state.online = data.online;
      this.state.loading = false;

      // Get cross-posts
      if (this.state.post.url) {
        let form: SearchForm = {
          q: this.state.post.url,
          type_: SearchType[SearchType.Url],
          sort: SortType[SortType.TopAll],
          page: 1,
          limit: 6,
        };
        WebSocketService.Instance.search(form);
      }

      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      // Necessary since it might be a user reply
      if (data.recipient_ids.length == 0) {
        this.state.comments.unshift(data.comment);
        this.setState(this.state);
      }
    } else if (
      res.op == UserOperation.EditComment ||
      res.op == UserOperation.DeleteComment ||
      res.op == UserOperation.RemoveComment
    ) {
      let data = res.data as CommentResponse;
      editCommentRes(data, this.state.comments);
      this.setState(this.state);
    } else if (res.op == UserOperation.SaveComment) {
      let data = res.data as CommentResponse;
      saveCommentRes(data, this.state.comments);
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.CreateCommentLike) {
      let data = res.data as CommentResponse;
      createCommentLikeRes(data, this.state.comments);
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePostLike) {
      let data = res.data as PostResponse;
      createPostLikeRes(data, this.state.post);
      this.setState(this.state);
    } else if (
      res.op == UserOperation.EditPost ||
      res.op == UserOperation.DeletePost ||
      res.op == UserOperation.RemovePost ||
      res.op == UserOperation.LockPost ||
      res.op == UserOperation.StickyPost
    ) {
      let data = res.data as PostResponse;
      this.state.post = data.post;
      this.setState(this.state);
      setupTippy();
    } else if (res.op == UserOperation.SavePost) {
      let data = res.data as PostResponse;
      this.state.post = data.post;
      this.setState(this.state);
      setupTippy();
    } else if (
      res.op == UserOperation.EditCommunity ||
      res.op == UserOperation.DeleteCommunity ||
      res.op == UserOperation.RemoveCommunity
    ) {
      let data = res.data as CommunityResponse;
      this.state.community = data.community;
      this.state.post.community_id = data.community.id;
      this.state.post.community_name = data.community.name;
      this.setState(this.state);
    } else if (res.op == UserOperation.FollowCommunity) {
      let data = res.data as CommunityResponse;
      this.state.community.subscribed = data.community.subscribed;
      this.state.community.number_of_subscribers =
        data.community.number_of_subscribers;
      this.setState(this.state);
    } else if (res.op == UserOperation.BanFromCommunity) {
      let data = res.data as BanFromCommunityResponse;
      this.state.comments
        .filter(c => c.creator_id == data.user.id)
        .forEach(c => (c.banned_from_community = data.banned));
      if (this.state.post.creator_id == data.user.id) {
        this.state.post.banned_from_community = data.banned;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.AddModToCommunity) {
      let data = res.data as AddModToCommunityResponse;
      this.state.moderators = data.moderators;
      this.setState(this.state);
    } else if (res.op == UserOperation.BanUser) {
      let data = res.data as BanUserResponse;
      this.state.comments
        .filter(c => c.creator_id == data.user.id)
        .forEach(c => (c.banned = data.banned));
      if (this.state.post.creator_id == data.user.id) {
        this.state.post.banned = data.banned;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.AddAdmin) {
      let data = res.data as AddAdminResponse;
      this.state.siteRes.admins = data.admins;
      this.setState(this.state);
    } else if (res.op == UserOperation.Search) {
      let data = res.data as SearchResponse;
      this.state.crossPosts = data.posts.filter(
        p => p.id != Number(this.props.match.params.id)
      );
      if (this.state.crossPosts.length) {
        this.state.post.duplicates = this.state.crossPosts;
      }
      this.setState(this.state);
    } else if (
      res.op == UserOperation.TransferSite ||
      res.op == UserOperation.GetSite
    ) {
      let data = res.data as GetSiteResponse;
      this.state.siteRes = data;
      this.setState(this.state);
    } else if (res.op == UserOperation.TransferCommunity) {
      let data = res.data as GetCommunityResponse;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.setState(this.state);
    }
  }
}
