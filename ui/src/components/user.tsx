import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, Post, Comment, CommunityUser, GetUserDetailsForm, SortType, UserDetailsResponse, UserView } from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp } from '../utils';
import { PostListing } from './post-listing';
import { CommentNodes } from './comment-nodes';
import { MomentTime } from './moment-time';

enum View {
  Overview, Comments, Posts, Saved
}

interface UserState {
  user: UserView;
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  comments: Array<Comment>;
  posts: Array<Post>;
  saved?: Array<Post>;
  view: View;
  sort: SortType;
}

export class User extends Component<any, UserState> {

  private subscription: Subscription;
  private emptyState: UserState = {
    user: {
      id: null,
      name: null,
      fedi_name: null,
      published: null,
      number_of_posts: null,
      post_score: null,
      number_of_comments: null,
      comment_score: null,
    },
    follows: [],
    moderates: [],
    comments: [],
    posts: [],
    view: View.Overview,
    sort: SortType.New
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    let userId = Number(this.props.match.params.id);

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    let form: GetUserDetailsForm = {
      user_id: userId,
      sort: SortType[this.state.sort],
      limit: 999
    };
    WebSocketService.Instance.getUserDetails(form);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-9">
            <h4>/u/{this.state.user.name}</h4>
            {this.selects()}
            {this.state.view == View.Overview &&
              this.overview()
            }
            {this.state.view == View.Comments &&
              this.comments()
            }
            {this.state.view == View.Posts &&
              this.posts()
            }
          </div>
          <div class="col-12 col-lg-3">
            {this.userInfo()}
            {this.moderates()}
            {this.follows()}
          </div>
        </div>
      </div>
    )
  }

  selects() {
    return (
      <div className="mb-2">
        <select value={this.state.view} onChange={linkEvent(this, this.handleViewChange)} class="custom-select w-auto">
          <option disabled>View</option>
          <option value={View.Overview}>Overview</option>
          <option value={View.Comments}>Comments</option>
          <option value={View.Posts}>Posts</option>
          {/* <option value={View.Saved}>Saved</option> */}
        </select>
        <select value={this.state.sort} onChange={linkEvent(this, this.handleSortChange)} class="custom-select w-auto ml-2">
          <option disabled>Sort Type</option>
          <option value={SortType.New}>New</option>
          <option value={SortType.TopDay}>Top Day</option>
          <option value={SortType.TopWeek}>Week</option>
          <option value={SortType.TopMonth}>Month</option>
          <option value={SortType.TopYear}>Year</option>
          <option value={SortType.TopAll}>All</option>
        </select>
      </div>
    )

  }

  overview() {
    let combined: Array<any> = [];
    combined.push(...this.state.comments);
    combined.push(...this.state.posts);

    // Sort it
    if (this.state.sort == SortType.New) {
      combined.sort((a, b) => b.published.localeCompare(a.published));
    } else {
      combined.sort((a, b) => b.score - a.score);
    }

    return (
      <div>
        {combined.map(i =>
          <div>
            {i.community_id 
              ? <PostListing post={i} showCommunity viewOnly />
              : <CommentNodes nodes={[{comment: i}]} noIndent viewOnly />
            }
          </div>
                     )
        }
      </div>
    )
  }

  comments() {
    return (
      <div>
        {this.state.comments.map(comment => 
          <CommentNodes nodes={[{comment: comment}]} noIndent viewOnly />
        )}
      </div>
    );
  }

  posts() {
    return (
      <div>
        {this.state.posts.map(post => 
          <PostListing post={post} showCommunity viewOnly />
        )}
      </div>
    );
  }

  userInfo() {
    let user = this.state.user;
    return (
      <div>
        <h4>{user.name}</h4>
        <div>Joined <MomentTime data={user} /></div>
        <table class="table table-bordered table-sm mt-2">
          <tr>
            <td>{user.post_score} points</td>
            <td>{user.number_of_posts} posts</td>
          </tr>
          <tr>
            <td>{user.comment_score} points</td>
            <td>{user.number_of_comments} comments</td>
          </tr>
        </table>
        <hr />
      </div>
    )
  }

  moderates() {
    return (
      <div>
        {this.state.moderates.length > 0 &&
          <div>
            <h4>Moderates</h4>
            <ul class="list-unstyled"> 
              {this.state.moderates.map(community =>
                <li><Link to={`/community/${community.community_id}`}>{community.community_name}</Link></li>
              )}
            </ul>
          </div>
        }
      </div>
    )
  }

  follows() {
    return (
      <div>
        {this.state.follows.length > 0 &&
          <div>
            <hr />
            <h4>Subscribed</h4>
            <ul class="list-unstyled"> 
              {this.state.follows.map(community =>
                <li><Link to={`/community/${community.community_id}`}>{community.community_name}</Link></li>
              )}
            </ul>
          </div>
        }
      </div>
    )
  }

  handleSortChange(i: User, event: any) {
    i.state.sort = Number(event.target.value);
    i.setState(i.state);

    let form: GetUserDetailsForm = {
      user_id: i.state.user.id,
      sort: SortType[i.state.sort],
      limit: 999
    };
    WebSocketService.Instance.getUserDetails(form);
  }

  handleViewChange(i: User, event: any) {
    i.state.view = Number(event.target.value);
    i.setState(i.state);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetUserDetails) {
      let res: UserDetailsResponse = msg;
      this.state.user = res.user;
      this.state.comments = res.comments;
      this.state.follows = res.follows;
      this.state.moderates = res.moderates;
      this.state.posts = res.posts;
      this.setState(this.state);
    } 
  }
}

