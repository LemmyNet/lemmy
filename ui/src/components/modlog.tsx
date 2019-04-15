import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, GetModlogForm, GetModlogResponse, ModRemovePost, ModLockPost, ModRemoveComment, ModRemoveCommunity, ModBanFromCommunity, ModBan, ModAddCommunity, ModAdd } from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp, addTypeInfo } from '../utils';
import { MomentTime } from './moment-time';
import * as moment from 'moment';

interface ModlogState {
  removed_posts: Array<ModRemovePost>,
  locked_posts: Array<ModLockPost>,
  removed_comments: Array<ModRemoveComment>,
  removed_communities: Array<ModRemoveCommunity>,
  banned_from_community: Array<ModBanFromCommunity>,
  banned: Array<ModBan>,
  added_to_community: Array<ModAddCommunity>,
  added: Array<ModAdd>,
  loading: boolean;
}

export class Modlog extends Component<any, ModlogState> {
  private subscription: Subscription;
  private emptyState: ModlogState = {
    removed_posts: [],
    locked_posts: [],
    removed_comments: [],
    removed_communities: [],
    banned_from_community: [],
    banned: [],
    added_to_community: [],
    added: [],
    loading: true
  }

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    let modlogForm: GetModlogForm = {

    };
    WebSocketService.Instance.getModlog(modlogForm);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  combined() {
    let combined: Array<{type_: string, data: ModRemovePost | ModLockPost | ModRemoveCommunity}> = [];
    let removed_posts = addTypeInfo(this.state.removed_posts, "removed_posts");
    let locked_posts = addTypeInfo(this.state.locked_posts, "locked_posts");
    let removed_comments = addTypeInfo(this.state.removed_comments, "removed_comments");
    let removed_communities = addTypeInfo(this.state.removed_communities, "removed_communities");
    let banned_from_community = addTypeInfo(this.state.banned_from_community, "banned_from_community");
    let added_to_community = addTypeInfo(this.state.added_to_community, "added_to_community");

    combined.push(...removed_posts);
    combined.push(...locked_posts);
    combined.push(...removed_comments);
    combined.push(...removed_communities);
    combined.push(...banned_from_community);
    combined.push(...added_to_community);

    // Sort them by time
    combined.sort((a, b) => b.data.when_.localeCompare(a.data.when_));

    console.log(combined);

    return (
      <tbody>
        {combined.map(i =>
          <tr>
            <td><MomentTime data={i.data} /></td>
            <td><Link to={`/user/${i.data.mod_user_id}`}>{i.data.mod_user_name}</Link></td>
            <td>
              {i.type_ == 'removed_posts' && 
                <>
                  {(i.data as ModRemovePost).removed? 'Removed' : 'Restored'} 
                  <span> Post <Link to={`/post/${(i.data as ModRemovePost).post_id}`}>{(i.data as ModRemovePost).post_name}</Link></span>
                  <div>{(i.data as ModRemovePost).reason && ` reason: ${(i.data as ModRemovePost).reason}`}</div>
                </>
              }
              {i.type_ == 'locked_posts' && 
                <>
                  {(i.data as ModLockPost).locked? 'Locked' : 'Unlocked'} 
                  <span> Post <Link to={`/post/${(i.data as ModLockPost).post_id}`}>{(i.data as ModLockPost).post_name}</Link></span>
                </>
              }
              {i.type_ == 'removed_comments' && 
                <>
                  {(i.data as ModRemoveComment).removed? 'Removed' : 'Restored'} 
                  <span> Comment <Link to={`/post/${(i.data as ModRemoveComment).post_id}/comment/${(i.data as ModRemoveComment).comment_id}`}>{(i.data as ModRemoveComment).comment_content}</Link></span>
                  <div>{(i.data as ModRemoveComment).reason && ` reason: ${(i.data as ModRemoveComment).reason}`}</div>
                </>
              }
              {i.type_ == 'removed_communities' && 
                <>
                  {(i.data as ModRemoveCommunity).removed ? 'Removed' : 'Restored'} 
                  <span> Community <Link to={`/community/${i.data.community_id}`}>{i.data.community_name}</Link></span>
                  <div>{(i.data as ModRemoveCommunity).reason && ` reason: ${(i.data as ModRemoveCommunity).reason}`}</div>
                  <div>{(i.data as ModRemoveCommunity).expires && ` expires: ${moment.utc((i.data as ModRemoveCommunity).expires).fromNow()}`}</div>
                </>
              }
              {i.type_ == 'banned_from_community' && 
                <>
                  <span>{(i.data as ModBanFromCommunity).banned ? 'Banned ' : 'Unbanned '} </span>
                  <span><Link to={`/user/${(i.data as ModBanFromCommunity).other_user_id}`}>{(i.data as ModBanFromCommunity).other_user_name}</Link></span>
                  <div>{(i.data as ModBanFromCommunity).reason && ` reason: ${(i.data as ModBanFromCommunity).reason}`}</div>
                  <div>{(i.data as ModBanFromCommunity).expires && ` expires: ${moment.utc((i.data as ModBanFromCommunity).expires).fromNow()}`}</div>
                </>
              }
              {i.type_ == 'added_to_community' && 
                <>
                  <span>{(i.data as ModAddCommunity).removed ? 'Removed ' : 'Appointed '} </span>
                  <span><Link to={`/user/${(i.data as ModAddCommunity).other_user_id}`}>{(i.data as ModAddCommunity).other_user_name}</Link></span>
                  <span> as a mod to the community </span>
                  <span><Link to={`/community/${i.data.community_id}`}>{i.data.community_name}</Link></span>
                </>
              }
            </td>
          </tr>
                     )
        }

      </tbody>
    );

  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? 
        <h4 class=""><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h4> : 
        <div>
          <h4>Modlog</h4>
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
          </div>
        </div>
        }
      </div>
    );
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetModlog) {
      let res: GetModlogResponse = msg;
      this.state.loading = false;
      this.state.removed_posts = res.removed_posts;
      this.state.locked_posts = res.locked_posts;
      this.state.removed_comments = res.removed_comments;
      this.state.removed_communities = res.removed_communities;
      this.state.banned_from_community = res.banned_from_community;
      this.state.added_to_community = res.added_to_community;
    
      this.setState(this.state);
    } 
  }
}
