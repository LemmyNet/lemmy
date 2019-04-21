import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Community, CommunityUser, FollowCommunityForm, CommunityForm as CommunityFormI, UserView } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { mdToHtml, getUnixTime } from '../utils';
import { CommunityForm } from './community-form';

interface SidebarProps {
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
}

interface SidebarState {
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  removeExpires: string;
}

export class Sidebar extends Component<SidebarProps, SidebarState> {

  private emptyState: SidebarState = {
    showEdit: false,
    showRemoveDialog: false,
    removeReason: null,
    removeExpires: null
  }

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.handleEditCommunity = this.handleEditCommunity.bind(this);
    this.handleEditCancel = this.handleEditCancel.bind(this);
  }

  render() {
    return (
      <div>
        {!this.state.showEdit 
          ? this.sidebar()
          : <CommunityForm community={this.props.community} onEdit={this.handleEditCommunity} onCancel={this.handleEditCancel}/>
        }
      </div>
    )
  }

  sidebar() {
    let community = this.props.community;
    return (
      <div>
        <h5 className="mb-0">{community.title}
        {community.removed &&
          <small className="ml-2 text-muted font-italic">removed</small>
        }
      </h5>
      <Link className="text-muted" to={`/community/${community.id}`}>/f/{community.name}</Link>
      <ul class="list-inline mb-1 text-muted small font-weight-bold"> 
        {this.canMod && 
          <>
            <li className="list-inline-item">
              <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
            </li>
            {this.amCreator && 
              <li className="list-inline-item">
                {/* <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span> */}
              </li>
            }
          </>
        }
        {this.canAdmin &&
          <li className="list-inline-item">
            {!this.props.community.removed ? 
            <span class="pointer" onClick={linkEvent(this, this.handleModRemoveShow)}>remove</span> :
            <span class="pointer" onClick={linkEvent(this, this.handleModRemoveSubmit)}>restore</span>
            }
          </li>

        }
      </ul>
      {this.state.showRemoveDialog && 
        <form onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
          <div class="form-group row">
            <label class="col-form-label">Reason</label>
            <input type="text" class="form-control mr-2" placeholder="Optional" value={this.state.removeReason} onInput={linkEvent(this, this.handleModRemoveReasonChange)} />
          </div>
          <div class="form-group row">
            <label class="col-form-label">Expires</label>
            <input type="date" class="form-control mr-2" placeholder="Expires" value={this.state.removeExpires} onInput={linkEvent(this, this.handleModRemoveExpiresChange)} />
          </div>
          <div class="form-group row">
            <button type="submit" class="btn btn-secondary">Remove Community</button>
          </div>
        </form>
      }
      <ul class="my-1 list-inline">
        <li className="list-inline-item"><Link className="badge badge-light" to="/communities">{community.category_name}</Link></li>
        <li className="list-inline-item badge badge-light">{community.number_of_subscribers} Subscribers</li>
        <li className="list-inline-item badge badge-light">{community.number_of_posts} Posts</li>
        <li className="list-inline-item badge badge-light">{community.number_of_comments} Comments</li>
        <li className="list-inline-item"><Link className="badge badge-light" to={`/modlog/community/${this.props.community.id}`}>Modlog</Link></li>
      </ul>
      <ul class="list-inline small"> 
        <li class="list-inline-item">mods: </li>
        {this.props.moderators.map(mod =>
          <li class="list-inline-item"><Link class="text-info" to={`/user/${mod.user_id}`}>{mod.user_name}</Link></li>
        )}
      </ul>
      <div>
        {community.subscribed 
          ? <button class="btn btn-sm btn-secondary" onClick={linkEvent(community.id, this.handleUnsubscribe)}>Unsubscribe</button>
          : <button class="btn btn-sm btn-secondary" onClick={linkEvent(community.id, this.handleSubscribe)}>Subscribe</button>
        }
      </div>
      {community.description && 
        <div>
          <hr />
          <div className="md-div" dangerouslySetInnerHTML={mdToHtml(community.description)} />
          <hr />
        </div>
      }
    </div>
    );
  }

  handleEditClick(i: Sidebar) {
    i.state.showEdit = true;
    i.setState(i.state);
  }

  handleEditCommunity() {
    this.state.showEdit = false;
    this.setState(this.state);
  }

  handleEditCancel() {
    this.state.showEdit = false;
    this.setState(this.state);
  }

  // TODO no deleting communities yet
  // handleDeleteClick(i: Sidebar, event) {
  // }

  handleUnsubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: false
    };
    WebSocketService.Instance.followCommunity(form);
  }

  handleSubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: true
    };
    WebSocketService.Instance.followCommunity(form);
  }

  private get amCreator(): boolean {
    return this.props.community.creator_id == UserService.Instance.user.id;
  }

  get canMod(): boolean {
    return UserService.Instance.user && this.props.moderators.map(m => m.user_id).includes(UserService.Instance.user.id);
  }

  get canAdmin(): boolean {
    return UserService.Instance.user && this.props.admins.map(a => a.id).includes(UserService.Instance.user.id);
  }

  handleDeleteClick() {
  }

  handleModRemoveShow(i: Sidebar) {
    i.state.showRemoveDialog = true;
    i.setState(i.state);
  }

  handleModRemoveReasonChange(i: Sidebar, event: any) {
    i.state.removeReason = event.target.value;
    i.setState(i.state);
  }

  handleModRemoveExpiresChange(i: Sidebar, event: any) {
    console.log(event.target.value);
    i.state.removeExpires = event.target.value;
    i.setState(i.state);
  }

  handleModRemoveSubmit(i: Sidebar) {
    event.preventDefault();
    let deleteForm: CommunityFormI = {
      name: i.props.community.name,
      title: i.props.community.title,
      category_id: i.props.community.category_id,
      edit_id: i.props.community.id,
      removed: !i.props.community.removed,
      reason: i.state.removeReason,
      expires: getUnixTime(i.state.removeExpires),
      auth: null,
    };
    WebSocketService.Instance.editCommunity(deleteForm);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }



}
