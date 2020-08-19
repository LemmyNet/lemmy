import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import {
  Community,
  CommunityUser,
  FollowCommunityForm,
  DeleteCommunityForm,
  RemoveCommunityForm,
  UserView,
  AddModToCommunityForm,
} from 'lemmy-js-client';
import { WebSocketService, UserService } from '../services';
import { mdToHtml, getUnixTime } from '../utils';
import { CommunityForm } from './community-form';
import { UserListing } from './user-listing';
import { CommunityLink } from './community-link';
import { BannerIconHeader } from './banner-icon-header';
import { i18n } from '../i18next';

interface SidebarProps {
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  online: number;
  enableNsfw: boolean;
  showIcon?: boolean;
}

interface SidebarState {
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  removeExpires: string;
  showConfirmLeaveModTeam: boolean;
}

export class Sidebar extends Component<SidebarProps, SidebarState> {
  private emptyState: SidebarState = {
    showEdit: false,
    showRemoveDialog: false,
    removeReason: null,
    removeExpires: null,
    showConfirmLeaveModTeam: false,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.handleEditCommunity = this.handleEditCommunity.bind(this);
    this.handleEditCancel = this.handleEditCancel.bind(this);
  }

  render() {
    return (
      <div>
        {!this.state.showEdit ? (
          this.sidebar()
        ) : (
          <CommunityForm
            community={this.props.community}
            onEdit={this.handleEditCommunity}
            onCancel={this.handleEditCancel}
            enableNsfw={this.props.enableNsfw}
          />
        )}
      </div>
    );
  }

  sidebar() {
    return (
      <div>
        <div class="card bg-transparent border-secondary mb-3">
          <div class="card-header bg-transparent border-secondary">
            {this.communityTitle()}
            {this.adminButtons()}
          </div>
          <div class="card-body">{this.subscribes()}</div>
        </div>
        <div class="card bg-transparent border-secondary mb-3">
          <div class="card-body">
            {this.description()}
            {this.badges()}
            {this.mods()}
          </div>
        </div>
      </div>
    );
  }

  communityTitle() {
    let community = this.props.community;
    return (
      <div>
        <h5 className="mb-0">
          {this.props.showIcon && (
            <BannerIconHeader icon={community.icon} banner={community.banner} />
          )}
          <span>{community.title}</span>
          {community.removed && (
            <small className="ml-2 text-muted font-italic">
              {i18n.t('removed')}
            </small>
          )}
          {community.deleted && (
            <small className="ml-2 text-muted font-italic">
              {i18n.t('deleted')}
            </small>
          )}
          {community.nsfw && (
            <small className="ml-2 text-muted font-italic">
              {i18n.t('nsfw')}
            </small>
          )}
        </h5>
        <CommunityLink
          community={community}
          realLink
          useApubName
          muted
          hideAvatar
        />
      </div>
    );
  }

  badges() {
    let community = this.props.community;
    return (
      <ul class="my-1 list-inline">
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_online', { count: this.props.online })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_subscribers', {
            count: community.number_of_subscribers,
          })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_posts', {
            count: community.number_of_posts,
          })}
        </li>
        <li className="list-inline-item badge badge-light">
          {i18n.t('number_of_comments', {
            count: community.number_of_comments,
          })}
        </li>
        <li className="list-inline-item">
          <Link className="badge badge-light" to="/communities">
            {community.category_name}
          </Link>
        </li>
        <li className="list-inline-item">
          <Link
            className="badge badge-light"
            to={`/modlog/community/${this.props.community.id}`}
          >
            {i18n.t('modlog')}
          </Link>
        </li>
        <li className="list-inline-item badge badge-light">
          <CommunityLink community={community} realLink />
        </li>
      </ul>
    );
  }

  mods() {
    return (
      <ul class="list-inline small">
        <li class="list-inline-item">{i18n.t('mods')}: </li>
        {this.props.moderators.map(mod => (
          <li class="list-inline-item">
            <UserListing
              user={{
                name: mod.user_name,
                preferred_username: mod.user_preferred_username,
                avatar: mod.avatar,
                id: mod.user_id,
                local: mod.user_local,
                actor_id: mod.user_actor_id,
              }}
            />
          </li>
        ))}
      </ul>
    );
  }

  subscribes() {
    let community = this.props.community;
    return (
      <div class="d-flex flex-wrap">
        <Link
          class={`btn btn-secondary flex-fill mr-2 mb-2 ${
            community.deleted || community.removed ? 'no-click' : ''
          }`}
          to={`/create_post?community=${community.name}`}
        >
          {i18n.t('create_a_post')}
        </Link>
        {community.subscribed ? (
          <a
            class="btn btn-secondary flex-fill mb-2"
            href="#"
            onClick={linkEvent(community.id, this.handleUnsubscribe)}
          >
            {i18n.t('unsubscribe')}
          </a>
        ) : (
          <a
            class="btn btn-secondary flex-fill mb-2"
            href="#"
            onClick={linkEvent(community.id, this.handleSubscribe)}
          >
            {i18n.t('subscribe')}
          </a>
        )}
      </div>
    );
  }

  description() {
    let community = this.props.community;
    return (
      community.description && (
        <div
          className="md-div"
          dangerouslySetInnerHTML={mdToHtml(community.description)}
        />
      )
    );
  }

  adminButtons() {
    let community = this.props.community;
    return (
      <>
        <ul class="list-inline mb-1 text-muted font-weight-bold">
          {this.canMod && (
            <>
              <li className="list-inline-item-action">
                <span
                  class="pointer"
                  onClick={linkEvent(this, this.handleEditClick)}
                  data-tippy-content={i18n.t('edit')}
                >
                  <svg class="icon icon-inline">
                    <use xlinkHref="#icon-edit"></use>
                  </svg>
                </span>
              </li>
              {!this.amCreator &&
                (!this.state.showConfirmLeaveModTeam ? (
                  <li className="list-inline-item-action">
                    <span
                      class="pointer"
                      onClick={linkEvent(
                        this,
                        this.handleShowConfirmLeaveModTeamClick
                      )}
                    >
                      {i18n.t('leave_mod_team')}
                    </span>
                  </li>
                ) : (
                  <>
                    <li className="list-inline-item-action">
                      {i18n.t('are_you_sure')}
                    </li>
                    <li className="list-inline-item-action">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleLeaveModTeamClick)}
                      >
                        {i18n.t('yes')}
                      </span>
                    </li>
                    <li className="list-inline-item-action">
                      <span
                        class="pointer"
                        onClick={linkEvent(
                          this,
                          this.handleCancelLeaveModTeamClick
                        )}
                      >
                        {i18n.t('no')}
                      </span>
                    </li>
                  </>
                ))}
              {this.amCreator && (
                <li className="list-inline-item-action">
                  <span
                    class="pointer"
                    onClick={linkEvent(this, this.handleDeleteClick)}
                    data-tippy-content={
                      !community.deleted ? i18n.t('delete') : i18n.t('restore')
                    }
                  >
                    <svg
                      class={`icon icon-inline ${
                        community.deleted && 'text-danger'
                      }`}
                    >
                      <use xlinkHref="#icon-trash"></use>
                    </svg>
                  </span>
                </li>
              )}
            </>
          )}
          {this.canAdmin && (
            <li className="list-inline-item">
              {!this.props.community.removed ? (
                <span
                  class="pointer"
                  onClick={linkEvent(this, this.handleModRemoveShow)}
                >
                  {i18n.t('remove')}
                </span>
              ) : (
                <span
                  class="pointer"
                  onClick={linkEvent(this, this.handleModRemoveSubmit)}
                >
                  {i18n.t('restore')}
                </span>
              )}
            </li>
          )}
        </ul>
        {this.state.showRemoveDialog && (
          <form onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
            <div class="form-group row">
              <label class="col-form-label" htmlFor="remove-reason">
                {i18n.t('reason')}
              </label>
              <input
                type="text"
                id="remove-reason"
                class="form-control mr-2"
                placeholder={i18n.t('optional')}
                value={this.state.removeReason}
                onInput={linkEvent(this, this.handleModRemoveReasonChange)}
              />
            </div>
            {/* TODO hold off on expires for now */}
            {/* <div class="form-group row"> */}
            {/*   <label class="col-form-label">Expires</label> */}
            {/*   <input type="date" class="form-control mr-2" placeholder={i18n.t('expires')} value={this.state.removeExpires} onInput={linkEvent(this, this.handleModRemoveExpiresChange)} /> */}
            {/* </div> */}
            <div class="form-group row">
              <button type="submit" class="btn btn-secondary">
                {i18n.t('remove_community')}
              </button>
            </div>
          </form>
        )}
      </>
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

  handleDeleteClick(i: Sidebar) {
    event.preventDefault();
    let deleteForm: DeleteCommunityForm = {
      edit_id: i.props.community.id,
      deleted: !i.props.community.deleted,
    };
    WebSocketService.Instance.deleteCommunity(deleteForm);
  }

  handleShowConfirmLeaveModTeamClick(i: Sidebar) {
    i.state.showConfirmLeaveModTeam = true;
    i.setState(i.state);
  }

  handleLeaveModTeamClick(i: Sidebar) {
    let form: AddModToCommunityForm = {
      user_id: UserService.Instance.user.id,
      community_id: i.props.community.id,
      added: false,
    };
    WebSocketService.Instance.addModToCommunity(form);
    i.state.showConfirmLeaveModTeam = false;
    i.setState(i.state);
  }

  handleCancelLeaveModTeamClick(i: Sidebar) {
    i.state.showConfirmLeaveModTeam = false;
    i.setState(i.state);
  }

  handleUnsubscribe(communityId: number) {
    event.preventDefault();
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: false,
    };
    WebSocketService.Instance.followCommunity(form);
  }

  handleSubscribe(communityId: number) {
    event.preventDefault();
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: true,
    };
    WebSocketService.Instance.followCommunity(form);
  }

  private get amCreator(): boolean {
    return this.props.community.creator_id == UserService.Instance.user.id;
  }

  get canMod(): boolean {
    return (
      UserService.Instance.user &&
      this.props.moderators
        .map(m => m.user_id)
        .includes(UserService.Instance.user.id)
    );
  }

  get canAdmin(): boolean {
    return (
      UserService.Instance.user &&
      this.props.admins.map(a => a.id).includes(UserService.Instance.user.id)
    );
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
    let removeForm: RemoveCommunityForm = {
      edit_id: i.props.community.id,
      removed: !i.props.community.removed,
      reason: i.state.removeReason,
      expires: getUnixTime(i.state.removeExpires),
    };
    WebSocketService.Instance.removeCommunity(removeForm);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }
}
