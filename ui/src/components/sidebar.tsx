import { Component, linkEvent } from 'inferno';
import { Community } from '../interfaces';
import { mdToHtml } from '../utils';

interface SidebarProps {
  community: Community;
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
          <li className="list-inline-item badge badge-light">{community.category_name}</li>
          <li className="list-inline-item badge badge-light">{community.number_of_subscribers} Subscribers</li>
          <li className="list-inline-item badge badge-light">{community.number_of_posts} Posts</li>
          <li className="list-inline-item badge badge-light">{community.number_of_comments} Comments</li>
        </ul>
        <div><button type="button" class="btn btn-secondary mb-2">Subscribe</button></div>
        <hr />
        {community.description && <div className="md-div" dangerouslySetInnerHTML={mdToHtml(community.description)} />}
      </div>
    );
  }
}
