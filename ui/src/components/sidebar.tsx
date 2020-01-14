import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import {
  Community,
  CommunityUser,
  FollowCommunityForm,
  CommunityForm as CommunityFormI,
  UserView,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  mdToHtml,
  getUnixTime,
  pictshareAvatarThumbnail,
  showAvatars,
} from '../utils';
import { CommunityForm } from './community-form';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

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
    removeExpires: null,
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
          />
        )}
      </div>
    );
  }

  sidebar() {
    let community = this.props.community;
    return (
      <div>
        <div class="card border-secondary mb-3">
          <div class="card-body">
            <h5 className="mb-0">
              <span>{community.title}</span>
              {community.removed && (
                <small className="ml-2 text-muted font-italic">
                  <T i18nKey="removed">#</T>
                </small>
              )}
              {community.deleted && (
                <small className="ml-2 text-muted font-italic">
                  <T i18nKey="deleted">#</T>
                </small>
              )}
            </h5>
            <Link className="text-muted" to={`/c/${community.name}`}>
              /c/{community.name}
            </Link>
            <ul class="list-inline mb-1 text-muted small font-weight-bold">
              {this.canMod && (
                <>
                  <li className="list-inline-item">
                    <span
                      class="pointer"
                      onClick={linkEvent(this, this.handleEditClick)}
                    >
                      <T i18nKey="edit">#</T>
                    </span>
                  </li>
                  {this.amCreator && (
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleDeleteClick)}
                      >
                        {!community.deleted
                          ? i18n.t('delete')
                          : i18n.t('restore')}
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
                      <T i18nKey="remove">#</T>
                    </span>
                  ) : (
                    <span
                      class="pointer"
                      onClick={linkEvent(this, this.handleModRemoveSubmit)}
                    >
                      <T i18nKey="restore">#</T>
                    </span>
                  )}
                </li>
              )}
            </ul>
            {this.state.showRemoveDialog && (
              <form onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
                <div class="form-group row">
                  <label class="col-form-label">
                    <T i18nKey="reason">#</T>
                  </label>
                  <input
                    type="text"
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
                    <T i18nKey="remove_community">#</T>
                  </button>
                </div>
              </form>
            )}
            <ul class="my-1 list-inline">
              <li className="list-inline-item">
                <Link className="badge badge-secondary" to="/communities">
                  {community.category_name}
                </Link>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_subscribers"
                  interpolation={{ count: community.number_of_subscribers }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_posts"
                  interpolation={{ count: community.number_of_posts }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item badge badge-secondary">
                <T
                  i18nKey="number_of_comments"
                  interpolation={{ count: community.number_of_comments }}
                >
                  #
                </T>
              </li>
              <li className="list-inline-item">
                <Link
                  className="badge badge-secondary"
                  to={`/modlog/community/${this.props.community.id}`}
                >
                  <T i18nKey="modlog">#</T>
                </Link>
              </li>
            </ul>
            <ul class="list-inline small">
              <li class="list-inline-item">{i18n.t('mods')}: </li>
              {this.props.moderators.map(mod => (
                <li class="list-inline-item">
                  <Link class="text-info" to={`/u/${mod.user_name}`}>
                    {mod.avatar && showAvatars() && (
                      <img
                        height="32"
                        width="32"
                        src={pictshareAvatarThumbnail(mod.avatar)}
                        class="rounded-circle mr-1"
                      />
                    )}
                    <span>{mod.user_name}</span>
                  </Link>
                </li>
              ))}
            </ul>
            <Link
              class={`btn btn-sm btn-secondary btn-block mb-3 ${(community.deleted ||
                community.removed) &&
                'no-click'}`}
              to={`/create_post?community=${community.name}`}
            >
              <T i18nKey="create_a_post">#</T>
            </Link>
            <div>
              {community.subscribed ? (
                <button
                  class="btn btn-sm btn-secondary btn-block"
                  onClick={linkEvent(community.id, this.handleUnsubscribe)}
                >
                  <T i18nKey="unsubscribe">#</T>
                </button>
              ) : (
                <button
                  class="btn btn-sm btn-secondary btn-block"
                  onClick={linkEvent(community.id, this.handleSubscribe)}
                >
                  <T i18nKey="subscribe">#</T>
                </button>
              )}
            </div>
          </div>
        </div>
        {community.description && (
          <div class="card border-secondary">
            <div class="card-body">
              <div
                className="md-div"
                dangerouslySetInnerHTML={mdToHtml(community.description)}
              />
            </div>
          </div>
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

  handleDeleteClick(i: Sidebar) {
    event.preventDefault();
    let deleteForm: CommunityFormI = {
      name: i.props.community.name,
      title: i.props.community.title,
      category_id: i.props.community.category_id,
      edit_id: i.props.community.id,
      deleted: !i.props.community.deleted,
      nsfw: i.props.community.nsfw,
      auth: null,
    };
    WebSocketService.Instance.editCommunity(deleteForm);
  }

  handleUnsubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: false,
    };
    WebSocketService.Instance.followCommunity(form);
  }

  handleSubscribe(communityId: number) {
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
    let deleteForm: CommunityFormI = {
      name: i.props.community.name,
      title: i.props.community.title,
      category_id: i.props.community.category_id,
      edit_id: i.props.community.id,
      removed: !i.props.community.removed,
      reason: i.state.removeReason,
      expires: getUnixTime(i.state.removeExpires),
      nsfw: i.props.community.nsfw,
      auth: null,
    };
    WebSocketService.Instance.editCommunity(deleteForm);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }
}
