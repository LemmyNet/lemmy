import { Component, linkEvent } from 'inferno';
import { WebSocketService, UserService } from '../services';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { i18n } from '../i18next';
import {
  UserOperation,
  Post,
  Comment,
  CommunityUser,
  SortType,
  UserDetailsResponse,
  UserView,
  WebSocketJsonResponse,
  CommentResponse,
  BanUserResponse,
  PostResponse,
} from 'lemmy-js-client';
import { UserDetailsView } from '../interfaces';
import {
  wsJsonToRes,
  toast,
  commentsToFlatNodes,
  setupTippy,
  editCommentRes,
  saveCommentRes,
  createCommentLikeRes,
  createPostLikeFindRes,
} from '../utils';
import { PostListing } from './post-listing';
import { CommentNodes } from './comment-nodes';

interface UserDetailsProps {
  username?: string;
  user_id?: number;
  page: number;
  limit: number;
  sort: SortType;
  enableDownvotes: boolean;
  enableNsfw: boolean;
  view: UserDetailsView;
  onPageChange(page: number): number | any;
  admins: Array<UserView>;
}

interface UserDetailsState {
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  comments: Array<Comment>;
  posts: Array<Post>;
  saved?: Array<Post>;
}

export class UserDetails extends Component<UserDetailsProps, UserDetailsState> {
  private subscription: Subscription;
  constructor(props: any, context: any) {
    super(props, context);

    this.state = {
      follows: [],
      moderates: [],
      comments: [],
      posts: [],
      saved: [],
    };

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    this.fetchUserData();
    setupTippy();
  }

  componentDidUpdate(lastProps: UserDetailsProps) {
    for (const key of Object.keys(lastProps)) {
      if (lastProps[key] !== this.props[key]) {
        this.fetchUserData();
        break;
      }
    }
  }

  fetchUserData() {
    WebSocketService.Instance.getUserDetails({
      user_id: this.props.user_id,
      username: this.props.username,
      sort: this.props.sort,
      saved_only: this.props.view === UserDetailsView.Saved,
      page: this.props.page,
      limit: this.props.limit,
    });
  }

  render() {
    return (
      <div>
        {this.viewSelector(this.props.view)}
        {this.paginator()}
      </div>
    );
  }

  viewSelector(view: UserDetailsView) {
    if (view === UserDetailsView.Overview || view === UserDetailsView.Saved) {
      return this.overview();
    }
    if (view === UserDetailsView.Comments) {
      return this.comments();
    }
    if (view === UserDetailsView.Posts) {
      return this.posts();
    }
  }

  overview() {
    const comments = this.state.comments.map((c: Comment) => {
      return { type: 'comments', data: c };
    });
    const posts = this.state.posts.map((p: Post) => {
      return { type: 'posts', data: p };
    });

    const combined: Array<{ type: string; data: Comment | Post }> = [
      ...comments,
      ...posts,
    ];

    // Sort it
    if (this.props.sort === SortType.New) {
      combined.sort((a, b) => b.data.published.localeCompare(a.data.published));
    } else {
      combined.sort((a, b) => b.data.score - a.data.score);
    }

    return (
      <div>
        {combined.map(i => (
          <>
            <div>
              {i.type === 'posts' ? (
                <PostListing
                  key={(i.data as Post).id}
                  post={i.data as Post}
                  admins={this.props.admins}
                  showCommunity
                  enableDownvotes={this.props.enableDownvotes}
                  enableNsfw={this.props.enableNsfw}
                />
              ) : (
                <CommentNodes
                  key={(i.data as Comment).id}
                  nodes={[{ comment: i.data as Comment }]}
                  admins={this.props.admins}
                  noBorder
                  noIndent
                  showCommunity
                  showContext
                  enableDownvotes={this.props.enableDownvotes}
                />
              )}
            </div>
            <hr class="my-3" />
          </>
        ))}
      </div>
    );
  }

  comments() {
    return (
      <div>
        <CommentNodes
          nodes={commentsToFlatNodes(this.state.comments)}
          admins={this.props.admins}
          noIndent
          showCommunity
          showContext
          enableDownvotes={this.props.enableDownvotes}
        />
      </div>
    );
  }

  posts() {
    return (
      <div>
        {this.state.posts.map(post => (
          <>
            <PostListing
              post={post}
              admins={this.props.admins}
              showCommunity
              enableDownvotes={this.props.enableDownvotes}
              enableNsfw={this.props.enableNsfw}
            />
            <hr class="my-3" />
          </>
        ))}
      </div>
    );
  }

  paginator() {
    return (
      <div class="my-2">
        {this.props.page > 1 && (
          <button
            class="btn btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            {i18n.t('prev')}
          </button>
        )}
        {this.state.comments.length + this.state.posts.length > 0 && (
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

  nextPage(i: UserDetails) {
    i.props.onPageChange(i.props.page + 1);
  }

  prevPage(i: UserDetails) {
    i.props.onPageChange(i.props.page - 1);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    const res = wsJsonToRes(msg);

    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      if (msg.error == 'couldnt_find_that_username_or_email') {
        this.context.router.history.push('/');
      }
      return;
    } else if (msg.reconnect) {
      this.fetchUserData();
    } else if (res.op == UserOperation.GetUserDetails) {
      const data = res.data as UserDetailsResponse;
      this.setState({
        comments: data.comments,
        follows: data.follows,
        moderates: data.moderates,
        posts: data.posts,
      });
    } else if (res.op == UserOperation.CreateCommentLike) {
      const data = res.data as CommentResponse;
      createCommentLikeRes(data, this.state.comments);
      this.setState({
        comments: this.state.comments,
      });
    } else if (
      res.op == UserOperation.EditComment ||
      res.op == UserOperation.DeleteComment ||
      res.op == UserOperation.RemoveComment
    ) {
      const data = res.data as CommentResponse;
      editCommentRes(data, this.state.comments);
      this.setState({
        comments: this.state.comments,
      });
    } else if (res.op == UserOperation.CreateComment) {
      const data = res.data as CommentResponse;
      if (
        UserService.Instance.user &&
        data.comment.creator_id == UserService.Instance.user.id
      ) {
        toast(i18n.t('reply_sent'));
      }
    } else if (res.op == UserOperation.SaveComment) {
      const data = res.data as CommentResponse;
      saveCommentRes(data, this.state.comments);
      this.setState({
        comments: this.state.comments,
      });
    } else if (res.op == UserOperation.CreatePostLike) {
      const data = res.data as PostResponse;
      createPostLikeFindRes(data, this.state.posts);
      this.setState({
        posts: this.state.posts,
      });
    } else if (res.op == UserOperation.BanUser) {
      const data = res.data as BanUserResponse;
      this.state.comments
        .filter(c => c.creator_id == data.user.id)
        .forEach(c => (c.banned = data.banned));
      this.state.posts
        .filter(c => c.creator_id == data.user.id)
        .forEach(c => (c.banned = data.banned));
      this.setState({
        posts: this.state.posts,
        comments: this.state.comments,
      });
    }
  }
}
