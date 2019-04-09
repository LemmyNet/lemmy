import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Community, CommunityUser, FollowCommunityForm } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { mdToHtml } from '../utils';
import { CommunityForm } from './community-form';

interface SidebarProps {
  community: Community;
  moderators: Array<CommunityUser>;
}

interface SidebarState {
  showEdit: boolean;
}

export class Sidebar extends Component<SidebarProps, SidebarState> {

  private emptyState: SidebarState = {
    showEdit: false
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
        <h4 className="mb-0">{community.title}</h4>
        <Link className="text-muted" to={`/community/${community.id}`}>/f/{community.name}</Link>
        {this.amMod && 
            <ul class="list-inline mb-1 text-muted small font-weight-bold"> 
              <li className="list-inline-item">
                <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
              </li>
              {this.amCreator && 
                <li className="list-inline-item">
                {/* <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span> */}
              </li>
              }
            </ul>
          }
        <ul class="list-inline">
          <li className="list-inline-item"><Link className="badge badge-light" to="/communities">{community.category_name}</Link></li>
          <li className="list-inline-item badge badge-light">{community.number_of_subscribers} Subscribers</li>
          <li className="list-inline-item badge badge-light">{community.number_of_posts} Posts</li>
          <li className="list-inline-item badge badge-light">{community.number_of_comments} Comments</li>
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
          </div>
        }
        <hr />
        <h4>Moderators</h4>
        {this.props.moderators.map(mod =>
          <Link to={`/user/${mod.user_id}`}>{mod.user_name}</Link>
        )}
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
    return UserService.Instance.loggedIn && this.props.community.creator_id == UserService.Instance.user.id;
  }

  private get amMod(): boolean {
    console.log(this.props.moderators);
    console.log(this.props);
    return UserService.Instance.loggedIn && 
      this.props.moderators.map(m => m.user_id).includes(UserService.Instance.user.id);
  }
}
