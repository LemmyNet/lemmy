import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Community, CommunityUser } from '../interfaces';
import { mdToHtml } from '../utils';

interface SidebarProps {
  community: Community;
  moderators: Array<CommunityUser>;
}

interface SidebarState {
}

export class Sidebar extends Component<SidebarProps, SidebarState> {

  constructor(props, context) {
    super(props, context);
  }


  render() {
    let community = this.props.community;
    return (
      <div>
        <h4>{community.title}</h4>
        <ul class="list-inline">
          <li className="list-inline-item"><Link className="badge badge-light" to="/communities">{community.category_name}</Link></li>
          <li className="list-inline-item badge badge-light">{community.number_of_subscribers} Subscribers</li>
          <li className="list-inline-item badge badge-light">{community.number_of_posts} Posts</li>
          <li className="list-inline-item badge badge-light">{community.number_of_comments} Comments</li>
        </ul>
        <div><button type="button" class="btn btn-secondary mb-2">Subscribe</button></div>
        {community.description && 
          <div>
            <hr />
            <div className="md-div" dangerouslySetInnerHTML={mdToHtml(community.description)} />
          </div>
        }
        <hr />
        <h5>Moderators</h5>
        {this.props.moderators.map(mod =>
          <Link to={`/user/${mod.user_id}`}>{mod.user_name}</Link>
        )}
      </div>
    );
  }
}
