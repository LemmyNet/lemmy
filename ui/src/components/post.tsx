import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Community,
  Post as PostI,
  GetPostResponse,
  PostResponse,
  Comment,
  CommentForm as CommentFormI,
  CommentResponse,
  CommentSortType,
  CommunityUser,
  CommunityResponse,
  CommentNode as CommentNodeI,
  BanFromCommunityResponse,
  BanUserResponse,
  AddModToCommunityResponse,
  AddAdminResponse,
  UserView,
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
import { wsJsonToRes, hotRank, toast } from '../utils';
import { PostListing } from './post-listing';
import { PostListings } from './post-listings';
import { Sidebar } from './sidebar';
import { CommentForm } from './comment-form';
import { CommentNodes } from './comment-nodes';
import autosize from 'autosize';
import { i18n } from '../i18next';

interface PostState {
  post: PostI;
  comments: Array<Comment>;
  commentSort: CommentSortType;
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  online: number;
  scrolled?: boolean;
  scrolled_comment_id?: number;
  loading: boolean;
  crossPosts: Array<PostI>;
}

export class Post extends Component<any, PostState> {
  private subscription: Subscription;
  private emptyState: PostState = {
    post: null,
    comments: [],
    commentSort: CommentSortType.Hot,
    community: null,
    moderators: [],
    admins: [],
    online: null,
    scrolled: false,
    loading: true,
    crossPosts: [],
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
      let form: CommentFormI = {
        content: found.content,
        edit_id: found.id,
        creator_id: found.creator_id,
        post_id: found.post_id,
        parent_id: found.parent_id,
        read: true,
        auth: null,
      };
      WebSocketService.Instance.editComment(form);
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
            <div class="col-12 col-md-8 mb-3">
              <PostListing
                post={this.state.post}
                showBody
                showCommunity
                moderators={this.state.moderators}
                admins={this.state.admins}
              />
              {this.state.crossPosts.length > 0 && (
                <>
                  <div class="my-1 text-muted small font-weight-bold">
                    {i18n.t('cross_posts')}
                  </div>
                  <PostListings showCommunity posts={this.state.crossPosts} />
                </>
              )}
              <div className="mb-2" />
              <CommentForm
                postId={this.state.post.id}
                disabled={this.state.post.locked}
              />
              {this.state.comments.length > 0 && this.sortRadios()}
              {this.commentsTree()}
            </div>
            <div class="col-12 col-sm-12 col-md-4">
              {this.state.comments.length > 0 && this.newComments()}
              {this.sidebar()}
            </div>
          </div>
        )}
      </div>
    );
  }

  sortRadios() {
    return (
      <div class="btn-group btn-group-toggle mb-3">
        <label
          className={`btn btn-sm btn-secondary pointer ${this.state
            .commentSort === CommentSortType.Hot && 'active'}`}
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
          className={`btn btn-sm btn-secondary pointer ${this.state
            .commentSort === CommentSortType.Top && 'active'}`}
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
          className={`btn btn-sm btn-secondary pointer ${this.state
            .commentSort === CommentSortType.New && 'active'}`}
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
          className={`btn btn-sm btn-secondary pointer ${this.state
            .commentSort === CommentSortType.Old && 'active'}`}
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
    );
  }

  newComments() {
    return (
      <div class="d-none d-md-block new-comments mb-3 card border-secondary">
        <div class="card-body small">
          <h6>{i18n.t('recent_comments')}</h6>
          {this.state.comments.map(comment => (
            <CommentNodes
              nodes={[{ comment: comment }]}
              noIndent
              locked={this.state.post.locked}
              moderators={this.state.moderators}
              admins={this.state.admins}
              postCreatorId={this.state.post.creator_id}
            />
          ))}
        </div>
      </div>
    );
  }

  sidebar() {
    return (
      <div class="mb-3">
        <Sidebar
          community={this.state.community}
          moderators={this.state.moderators}
          admins={this.state.admins}
          online={this.state.online}
        />
      </div>
    );
  }

  handleCommentSortChange(i: Post, event: any) {
    i.state.commentSort = Number(event.target.value);
    i.setState(i.state);
  }

  private buildCommentsTree(): Array<CommentNodeI> {
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
      if (comment.parent_id) {
        map.get(comment.parent_id).children.push(map.get(comment.id));
      } else {
        tree.push(map.get(comment.id));
      }
    }

    this.sortTree(tree);

    return tree;
  }

  sortTree(tree: Array<CommentNodeI>) {
    // First, put removed and deleted comments at the bottom, then do your other sorts
    if (this.state.commentSort == CommentSortType.Top) {
      tree.sort(
        (a, b) =>
          +a.comment.removed - +b.comment.removed ||
          +a.comment.deleted - +b.comment.deleted ||
          b.comment.score - a.comment.score
      );
    } else if (this.state.commentSort == CommentSortType.New) {
      tree.sort(
        (a, b) =>
          +a.comment.removed - +b.comment.removed ||
          +a.comment.deleted - +b.comment.deleted ||
          b.comment.published.localeCompare(a.comment.published)
      );
    } else if (this.state.commentSort == CommentSortType.Old) {
      tree.sort(
        (a, b) =>
          +a.comment.removed - +b.comment.removed ||
          +a.comment.deleted - +b.comment.deleted ||
          a.comment.published.localeCompare(b.comment.published)
      );
    } else if (this.state.commentSort == CommentSortType.Hot) {
      tree.sort(
        (a, b) =>
          +a.comment.removed - +b.comment.removed ||
          +a.comment.deleted - +b.comment.deleted ||
          hotRank(b.comment) - hotRank(a.comment)
      );
    }

    for (let node of tree) {
      this.sortTree(node.children);
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
          admins={this.state.admins}
          postCreatorId={this.state.post.creator_id}
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
    } else if (res.op == UserOperation.GetPost) {
      let data = res.data as GetPostResponse;
      this.state.post = data.post;
      this.state.comments = data.comments;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.state.admins = data.admins;
      this.state.online = data.online;
      this.state.loading = false;
      document.title = `${this.state.post.name} - ${WebSocketService.Instance.site.name}`;

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
    } else if (res.op == UserOperation.CreateComment) {
      let data = res.data as CommentResponse;

      // Necessary since it might be a user reply
      if (data.recipient_ids.length == 0) {
        this.state.comments.unshift(data.comment);
        this.setState(this.state);
      }
    } else if (res.op == UserOperation.EditComment) {
      let data = res.data as CommentResponse;
      let found = this.state.comments.find(c => c.id == data.comment.id);
      found.content = data.comment.content;
      found.updated = data.comment.updated;
      found.removed = data.comment.removed;
      found.deleted = data.comment.deleted;
      found.upvotes = data.comment.upvotes;
      found.downvotes = data.comment.downvotes;
      found.score = data.comment.score;
      found.read = data.comment.read;

      this.setState(this.state);
    } else if (res.op == UserOperation.SaveComment) {
      let data = res.data as CommentResponse;
      let found = this.state.comments.find(c => c.id == data.comment.id);
      found.saved = data.comment.saved;
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateCommentLike) {
      let data = res.data as CommentResponse;
      let found: Comment = this.state.comments.find(
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
      let data = res.data as PostResponse;
      this.state.post.score = data.post.score;
      this.state.post.upvotes = data.post.upvotes;
      this.state.post.downvotes = data.post.downvotes;
      if (data.post.my_vote !== null) {
        this.state.post.my_vote = data.post.my_vote;
        this.state.post.upvoteLoading = false;
        this.state.post.downvoteLoading = false;
      }

      this.setState(this.state);
    } else if (res.op == UserOperation.EditPost) {
      let data = res.data as PostResponse;
      this.state.post = data.post;
      this.setState(this.state);
    } else if (res.op == UserOperation.SavePost) {
      let data = res.data as PostResponse;
      this.state.post = data.post;
      this.setState(this.state);
    } else if (res.op == UserOperation.EditCommunity) {
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
      this.state.admins = data.admins;
      this.setState(this.state);
    } else if (res.op == UserOperation.Search) {
      let data = res.data as SearchResponse;
      this.state.crossPosts = data.posts.filter(
        p => p.id != this.state.post.id
      );
      this.setState(this.state);
    } else if (res.op == UserOperation.TransferSite) {
      let data = res.data as GetSiteResponse;

      this.state.admins = data.admins;
      this.setState(this.state);
    } else if (res.op == UserOperation.TransferCommunity) {
      let data = res.data as GetCommunityResponse;
      this.state.community = data.community;
      this.state.moderators = data.moderators;
      this.state.admins = data.admins;
      this.setState(this.state);
    }
  }
}
