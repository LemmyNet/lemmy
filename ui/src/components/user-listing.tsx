import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { UserView } from '../interfaces';
import { pictshareAvatarThumbnail, showAvatars, hostname } from '../utils';

interface UserOther {
  name: string;
  id?: number; // Necessary if its federated
  avatar?: string;
  local?: boolean;
  actor_id?: string;
}

interface UserListingProps {
  user: UserView | UserOther;
}

export class UserListing extends Component<UserListingProps, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    let user = this.props.user;
    let local = user.local == null ? true : user.local;
    let name_: string, link: string;

    if (local) {
      name_ = user.name;
      link = `/u/${user.name}`;
    } else {
      name_ = `${hostname(user.actor_id)}/${user.name}`;
      link = `/user/${user.id}`;
    }

    return (
      <Link className="text-body font-weight-bold" to={link}>
        {user.avatar && showAvatars() && (
          <img
            height="32"
            width="32"
            src={pictshareAvatarThumbnail(user.avatar)}
            class="rounded-circle mr-2"
          />
        )}
        <span>{name_}</span>
      </Link>
    );
  }
}
