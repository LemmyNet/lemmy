import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  GetModlogForm,
  GetModlogResponse,
  ModRemovePost,
  ModLockPost,
  ModStickyPost,
  ModRemoveComment,
  ModRemoveCommunity,
  ModBanFromCommunity,
  ModBan,
  ModAddCommunity,
  ModAdd,
} from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp, addTypeInfo, fetchLimit } from '../utils';
import { MomentTime } from './moment-time';
import * as moment from 'moment';
import { i18n } from '../i18next';

interface ModlogState {
  combined: Array<{
    type_: string;
    data:
      | ModRemovePost
      | ModLockPost
      | ModStickyPost
      | ModRemoveCommunity
      | ModAdd
      | ModBan;
  }>;
  communityId?: number;
  communityName?: string;
  page: number;
  loading: boolean;
}

export class Modlog extends Component<any, ModlogState> {
  private subscription: Subscription;
  private emptyState: ModlogState = {
    combined: [],
    page: 1,
    loading: true,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.state.communityId = this.props.match.params.community_id
      ? Number(this.props.match.params.community_id)
      : undefined;
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

    this.refetch();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = `Modlog - ${WebSocketService.Instance.site.name}`;
  }

  setCombined(res: GetModlogResponse) {
    let removed_posts = addTypeInfo(res.removed_posts, 'removed_posts');
    let locked_posts = addTypeInfo(res.locked_posts, 'locked_posts');
    let stickied_posts = addTypeInfo(res.stickied_posts, 'stickied_posts');
    let removed_comments = addTypeInfo(
      res.removed_comments,
      'removed_comments'
    );
    let removed_communities = addTypeInfo(
      res.removed_communities,
      'removed_communities'
    );
    let banned_from_community = addTypeInfo(
      res.banned_from_community,
      'banned_from_community'
    );
    let added_to_community = addTypeInfo(
      res.added_to_community,
      'added_to_community'
    );
    let added = addTypeInfo(res.added, 'added');
    let banned = addTypeInfo(res.banned, 'banned');
    this.state.combined = [];

    this.state.combined.push(...removed_posts);
    this.state.combined.push(...locked_posts);
    this.state.combined.push(...stickied_posts);
    this.state.combined.push(...removed_comments);
    this.state.combined.push(...removed_communities);
    this.state.combined.push(...banned_from_community);
    this.state.combined.push(...added_to_community);
    this.state.combined.push(...added);
    this.state.combined.push(...banned);

    if (this.state.communityId && this.state.combined.length > 0) {
      this.state.communityName = (this.state.combined[0]
        .data as ModRemovePost).community_name;
    }

    // Sort them by time
    this.state.combined.sort((a, b) =>
      b.data.when_.localeCompare(a.data.when_)
    );

    this.setState(this.state);
  }

  combined() {
    return (
      <tbody>
        {this.state.combined.map(i => (
          <tr>
            <td>
              <MomentTime data={i.data} />
            </td>
            <td>
              <Link to={`/u/${i.data.mod_user_name}`}>
                {i.data.mod_user_name}
              </Link>
            </td>
            <td>
              {i.type_ == 'removed_posts' && (
                <>
                  {(i.data as ModRemovePost).removed ? 'Removed' : 'Restored'}
                  <span>
                    {' '}
                    Post{' '}
                    <Link to={`/post/${(i.data as ModRemovePost).post_id}`}>
                      {(i.data as ModRemovePost).post_name}
                    </Link>
                  </span>
                  <div>
                    {(i.data as ModRemovePost).reason &&
                      ` reason: ${(i.data as ModRemovePost).reason}`}
                  </div>
                </>
              )}
              {i.type_ == 'locked_posts' && (
                <>
                  {(i.data as ModLockPost).locked ? 'Locked' : 'Unlocked'}
                  <span>
                    {' '}
                    Post{' '}
                    <Link to={`/post/${(i.data as ModLockPost).post_id}`}>
                      {(i.data as ModLockPost).post_name}
                    </Link>
                  </span>
                </>
              )}
              {i.type_ == 'stickied_posts' && (
                <>
                  {(i.data as ModStickyPost).stickied
                    ? 'Stickied'
                    : 'Unstickied'}
                  <span>
                    {' '}
                    Post{' '}
                    <Link to={`/post/${(i.data as ModStickyPost).post_id}`}>
                      {(i.data as ModStickyPost).post_name}
                    </Link>
                  </span>
                </>
              )}
              {i.type_ == 'removed_comments' && (
                <>
                  {(i.data as ModRemoveComment).removed
                    ? 'Removed'
                    : 'Restored'}
                  <span>
                    {' '}
                    Comment{' '}
                    <Link
                      to={`/post/${
                        (i.data as ModRemoveComment).post_id
                      }/comment/${(i.data as ModRemoveComment).comment_id}`}
                    >
                      {(i.data as ModRemoveComment).comment_content}
                    </Link>
                  </span>
                  <span>
                    {' '}
                    by{' '}
                    <Link
                      to={`/u/${
                        (i.data as ModRemoveComment).comment_user_name
                      }`}
                    >
                      {(i.data as ModRemoveComment).comment_user_name}
                    </Link>
                  </span>
                  <div>
                    {(i.data as ModRemoveComment).reason &&
                      ` reason: ${(i.data as ModRemoveComment).reason}`}
                  </div>
                </>
              )}
              {i.type_ == 'removed_communities' && (
                <>
                  {(i.data as ModRemoveCommunity).removed
                    ? 'Removed'
                    : 'Restored'}
                  <span>
                    {' '}
                    Community{' '}
                    <Link
                      to={`/c/${(i.data as ModRemoveCommunity).community_name}`}
                    >
                      {(i.data as ModRemoveCommunity).community_name}
                    </Link>
                  </span>
                  <div>
                    {(i.data as ModRemoveCommunity).reason &&
                      ` reason: ${(i.data as ModRemoveCommunity).reason}`}
                  </div>
                  <div>
                    {(i.data as ModRemoveCommunity).expires &&
                      ` expires: ${moment
                        .utc((i.data as ModRemoveCommunity).expires)
                        .fromNow()}`}
                  </div>
                </>
              )}
              {i.type_ == 'banned_from_community' && (
                <>
                  <span>
                    {(i.data as ModBanFromCommunity).banned
                      ? 'Banned '
                      : 'Unbanned '}{' '}
                  </span>
                  <span>
                    <Link
                      to={`/u/${
                        (i.data as ModBanFromCommunity).other_user_name
                      }`}
                    >
                      {(i.data as ModBanFromCommunity).other_user_name}
                    </Link>
                  </span>
                  <span> from the community </span>
                  <span>
                    <Link
                      to={`/c/${
                        (i.data as ModBanFromCommunity).community_name
                      }`}
                    >
                      {(i.data as ModBanFromCommunity).community_name}
                    </Link>
                  </span>
                  <div>
                    {(i.data as ModBanFromCommunity).reason &&
                      ` reason: ${(i.data as ModBanFromCommunity).reason}`}
                  </div>
                  <div>
                    {(i.data as ModBanFromCommunity).expires &&
                      ` expires: ${moment
                        .utc((i.data as ModBanFromCommunity).expires)
                        .fromNow()}`}
                  </div>
                </>
              )}
              {i.type_ == 'added_to_community' && (
                <>
                  <span>
                    {(i.data as ModAddCommunity).removed
                      ? 'Removed '
                      : 'Appointed '}{' '}
                  </span>
                  <span>
                    <Link
                      to={`/u/${(i.data as ModAddCommunity).other_user_name}`}
                    >
                      {(i.data as ModAddCommunity).other_user_name}
                    </Link>
                  </span>
                  <span> as a mod to the community </span>
                  <span>
                    <Link
                      to={`/c/${(i.data as ModAddCommunity).community_name}`}
                    >
                      {(i.data as ModAddCommunity).community_name}
                    </Link>
                  </span>
                </>
              )}
              {i.type_ == 'banned' && (
                <>
                  <span>
                    {(i.data as ModBan).banned ? 'Banned ' : 'Unbanned '}{' '}
                  </span>
                  <span>
                    <Link to={`/u/${(i.data as ModBan).other_user_name}`}>
                      {(i.data as ModBan).other_user_name}
                    </Link>
                  </span>
                  <div>
                    {(i.data as ModBan).reason &&
                      ` reason: ${(i.data as ModBan).reason}`}
                  </div>
                  <div>
                    {(i.data as ModBan).expires &&
                      ` expires: ${moment
                        .utc((i.data as ModBan).expires)
                        .fromNow()}`}
                  </div>
                </>
              )}
              {i.type_ == 'added' && (
                <>
                  <span>
                    {(i.data as ModAdd).removed ? 'Removed ' : 'Appointed '}{' '}
                  </span>
                  <span>
                    <Link to={`/u/${(i.data as ModAdd).other_user_name}`}>
                      {(i.data as ModAdd).other_user_name}
                    </Link>
                  </span>
                  <span> as an admin </span>
                </>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    );
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? (
          <h5 class="">
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div>
            <h5>
              {this.state.communityName && (
                <Link
                  className="text-white"
                  to={`/c/${this.state.communityName}`}
                >
                  /c/{this.state.communityName}{' '}
                </Link>
              )}
              <span>Modlog</span>
            </h5>
            <div class="table-responsive">
              <table id="modlog_table" class="table table-sm table-hover">
                <thead class="pointer">
                  <tr>
                    <th>Time</th>
                    <th>Mod</th>
                    <th>Action</th>
                  </tr>
                </thead>
                {this.combined()}
              </table>
              {this.paginator()}
            </div>
          </div>
        )}
      </div>
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
            Prev
          </button>
        )}
        <button
          class="btn btn-sm btn-secondary"
          onClick={linkEvent(this, this.nextPage)}
        >
          Next
        </button>
      </div>
    );
  }

  nextPage(i: Modlog) {
    i.state.page++;
    i.setState(i.state);
    i.refetch();
  }

  prevPage(i: Modlog) {
    i.state.page--;
    i.setState(i.state);
    i.refetch();
  }

  refetch() {
    let modlogForm: GetModlogForm = {
      community_id: this.state.communityId,
      page: this.state.page,
      limit: fetchLimit,
    };
    WebSocketService.Instance.getModlog(modlogForm);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      return;
    } else if (op == UserOperation.GetModlog) {
      let res: GetModlogResponse = msg;
      this.state.loading = false;
      window.scrollTo(0, 0);
      this.setCombined(res);
    }
  }
}
