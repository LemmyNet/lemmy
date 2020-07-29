import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { UserView } from '../interfaces';
import {
  pictrsAvatarThumbnail,
  showAvatars,
  hostname,
  isCakeDay,
} from '../utils';
import { CakeDay } from './cake-day';

interface UserOther {
  name: string;
  id?: number; // Necessary if its federated
  avatar?: string;
  local?: boolean;
  actor_id?: string;
  published?: string;
}

interface UserListingProps {
  user: UserView | UserOther;
  realLink?: boolean;
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
      name_ = `${user.name}@${hostname(user.actor_id)}`;
      link = !this.props.realLink ? `/user/${user.id}` : user.actor_id;
    }

    return (
      <>
        <Link className="text-info" to={link}>
          {user.avatar && showAvatars() && (
            <img
              style="width: 2rem; height: 2rem;"
              src={pictrsAvatarThumbnail(user.avatar)}
              class="rounded-circle mr-2"
            />
          )}
          <span>{name_}</span>
        </Link>

        {isCakeDay(user.published) && <CakeDay creatorName={name_} />}
      </>
    );
  }
}
