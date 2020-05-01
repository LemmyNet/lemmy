import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Community } from '../interfaces';
import { hostname } from '../utils';

interface CommunityOther {
  name: string;
  id?: number; // Necessary if its federated
  local?: boolean;
  actor_id?: string;
}

interface CommunityLinkProps {
  community: Community | CommunityOther;
  realLink?: boolean;
}

export class CommunityLink extends Component<CommunityLinkProps, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    let community = this.props.community;
    let name_: string, link: string;
    let local = community.local == null ? true : community.local;
    if (local) {
      name_ = community.name;
      link = `/c/${community.name}`;
    } else {
      name_ = `${community.name}@${hostname(community.actor_id)}`;
      link = !this.props.realLink
        ? `/community/${community.id}`
        : community.actor_id;
    }
    return <Link to={link}>{name_}</Link>;
  }
}
