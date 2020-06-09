import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { UserView } from '../interfaces';
import { pictrsAvatarThumbnail, showAvatars } from '../utils';

interface UserOther {
  name: string;
  avatar?: string;
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
    return (
      <Link className="text-body font-weight-bold" to={`/u/${user.name}`}>
        {user.avatar && showAvatars() && (
          <img
            height="32"
            width="32"
            src={pictrsAvatarThumbnail(user.avatar)}
            class="rounded-circle mr-2"
          />
        )}
        <span>{user.name}</span>
      </Link>
    );
  }
}
