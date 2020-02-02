import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import {
  CommentNode as CommentNodeI,
  CommentLikeForm,
  CommentForm as CommentFormI,
  EditUserMentionForm,
  SaveCommentForm,
  BanFromCommunityForm,
  BanUserForm,
  CommunityUser,
  UserView,
  AddModToCommunityForm,
  AddAdminForm,
  TransferCommunityForm,
  TransferSiteForm,
  BanType,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  mdToHtml,
  getUnixTime,
  canMod,
  isMod,
  pictshareAvatarThumbnail,
  showAvatars,
} from '../utils';
import moment from 'moment';
import { MomentTime } from './moment-time';
import { CommentForm } from './comment-form';
import { CommentNodes } from './comment-nodes';
import { i18n } from '../i18next';

interface CommentNodeState {
  showReply: boolean;
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  showBanDialog: boolean;
  banReason: string;
  banExpires: string;
  banType: BanType;
  showConfirmTransferSite: boolean;
  showConfirmTransferCommunity: boolean;
  showConfirmAppointAsMod: boolean;
  showConfirmAppointAsAdmin: boolean;
  collapsed: boolean;
  viewSource: boolean;
  upvoteLoading: boolean;
  downvoteLoading: boolean;
}

interface CommentNodeProps {
  node: CommentNodeI;
  noIndent?: boolean;
  viewOnly?: boolean;
  locked?: boolean;
  markable?: boolean;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  postCreatorId?: number;
}

export class CommentNode extends Component<CommentNodeProps, CommentNodeState> {
  private emptyState: CommentNodeState = {
    showReply: false,
    showEdit: false,
    showRemoveDialog: false,
    removeReason: null,
    showBanDialog: false,
    banReason: null,
    banExpires: null,
    banType: BanType.Community,
    collapsed: false,
    viewSource: false,
    showConfirmTransferSite: false,
    showConfirmTransferCommunity: false,
    showConfirmAppointAsMod: false,
    showConfirmAppointAsAdmin: false,
    upvoteLoading: this.props.node.comment.upvoteLoading,
    downvoteLoading: this.props.node.comment.downvoteLoading,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
    this.handleCommentUpvote = this.handleCommentUpvote.bind(this);
    this.handleCommentDownvote = this.handleCommentDownvote.bind(this);
  }

  componentWillReceiveProps(nextProps: CommentNodeProps) {
    if (
      nextProps.node.comment.upvoteLoading !== this.state.upvoteLoading ||
      nextProps.node.comment.downvoteLoading !== this.state.downvoteLoading
    ) {
      this.setState({
        upvoteLoading: false,
        downvoteLoading: false,
      });
    }
  }

  render() {
    let node = this.props.node;
    return (
      <div
        className={`comment ${
          node.comment.parent_id && !this.props.noIndent ? 'ml-4' : ''
        }`}
      >
        {!this.state.collapsed && (
          <div
            className={`vote-bar mr-2 float-left small text-center ${this.props
              .viewOnly && 'no-click'}`}
          >
            <button
              className={`btn btn-link p-0 ${
                node.comment.my_vote == 1 ? 'text-info' : 'text-muted'
              }`}
              onClick={linkEvent(node, this.handleCommentUpvote)}
            >
              {this.state.upvoteLoading ? (
                <svg class="icon icon-spinner spin upvote">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              ) : (
                <svg class="icon upvote">
                  <use xlinkHref="#icon-arrow-up"></use>
                </svg>
              )}
            </button>
            <div class={`font-weight-bold text-muted`}>
              {node.comment.score}
            </div>
            {WebSocketService.Instance.site.enable_downvotes && (
              <button
                className={`btn btn-link p-0 ${
                  node.comment.my_vote == -1 ? 'text-danger' : 'text-muted'
                }`}
                onClick={linkEvent(node, this.handleCommentDownvote)}
              >
                {this.state.downvoteLoading ? (
                  <svg class="icon icon-spinner spin downvote">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : (
                  <svg class="icon downvote">
                    <use xlinkHref="#icon-arrow-down"></use>
                  </svg>
                )}
              </button>
            )}
          </div>
        )}
        <div
          id={`comment-${node.comment.id}`}
          className={`details comment-node ml-4 ${
            this.isCommentNew ? 'mark' : ''
          }`}
        >
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <Link
                className="text-info"
                to={`/u/${node.comment.creator_name}`}
              >
                {node.comment.creator_avatar && showAvatars() && (
                  <img
                    height="32"
                    width="32"
                    src={pictshareAvatarThumbnail(node.comment.creator_avatar)}
                    class="rounded-circle mr-1"
                  />
                )}
                <span>{node.comment.creator_name}</span>
              </Link>
            </li>
            {this.isMod && (
              <li className="list-inline-item badge badge-light">
                {i18n.t('mod')}
              </li>
            )}
            {this.isAdmin && (
              <li className="list-inline-item badge badge-light">
                {i18n.t('admin')}
              </li>
            )}
            {this.isPostCreator && (
              <li className="list-inline-item badge badge-light">
                {i18n.t('creator')}
              </li>
            )}
            {(node.comment.banned_from_community || node.comment.banned) && (
              <li className="list-inline-item badge badge-danger">
                {i18n.t('banned')}
              </li>
            )}
            <li className="list-inline-item">
              <span>
                (<span className="text-info">+{node.comment.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{node.comment.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <span>
                <MomentTime data={node.comment} />
              </span>
            </li>
            <li className="list-inline-item">
              <div
                className="pointer text-monospace"
                onClick={linkEvent(this, this.handleCommentCollapse)}
              >
                {this.state.collapsed ? '[+]' : '[-]'}
              </div>
            </li>
          </ul>
          {this.state.showEdit && (
            <CommentForm
              node={node}
              edit
              onReplyCancel={this.handleReplyCancel}
              disabled={this.props.locked}
            />
          )}
          {!this.state.showEdit && !this.state.collapsed && (
            <div>
              {this.state.viewSource ? (
                <pre>{this.commentUnlessRemoved}</pre>
              ) : (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(this.commentUnlessRemoved)}
                />
              )}
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                {this.props.markable && (
                  <li className="list-inline-item">
                    <span
                      class="pointer"
                      onClick={linkEvent(this, this.handleMarkRead)}
                    >
                      {node.comment.read
                        ? i18n.t('mark_as_unread')
                        : i18n.t('mark_as_read')}
                    </span>
                  </li>
                )}
                {UserService.Instance.user && !this.props.viewOnly && (
                  <>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleReplyClick)}
                      >
                        {i18n.t('reply')}
                      </span>
                    </li>
                    <li className="list-inline-item mr-2">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleSaveCommentClick)}
                      >
                        {node.comment.saved ? i18n.t('unsave') : i18n.t('save')}
                      </span>
                    </li>
                    {this.myComment && (
                      <>
                        <li className="list-inline-item">
                          <span
                            class="pointer"
                            onClick={linkEvent(this, this.handleEditClick)}
                          >
                            {i18n.t('edit')}
                          </span>
                        </li>
                        <li className="list-inline-item">
                          <span
                            class="pointer"
                            onClick={linkEvent(this, this.handleDeleteClick)}
                          >
                            {!node.comment.deleted
                              ? i18n.t('delete')
                              : i18n.t('restore')}
                          </span>
                        </li>
                      </>
                    )}
                    {!this.myComment && (
                      <li className="list-inline-item">
                        <Link
                          class="text-muted"
                          to={`/create_private_message?recipient_id=${node.comment.creator_id}`}
                        >
                          {i18n.t('message').toLowerCase()}
                        </Link>
                      </li>
                    )}
                    <li className="list-inline-item">•</li>
                    <li className="list-inline-item">
                      <span
                        className="pointer"
                        onClick={linkEvent(this, this.handleViewSource)}
                      >
                        {i18n.t('view_source')}
                      </span>
                    </li>
                    <li className="list-inline-item">
                      <Link
                        className="text-muted"
                        to={`/post/${node.comment.post_id}/comment/${node.comment.id}`}
                      >
                        {i18n.t('link')}
                      </Link>
                    </li>
                    {/* Admins and mods can remove comments */}
                    {(this.canMod || this.canAdmin) && (
                      <>
                        <li className="list-inline-item">•</li>
                        <li className="list-inline-item">
                          {!node.comment.removed ? (
                            <span
                              class="pointer"
                              onClick={linkEvent(
                                this,
                                this.handleModRemoveShow
                              )}
                            >
                              {i18n.t('remove')}
                            </span>
                          ) : (
                            <span
                              class="pointer"
                              onClick={linkEvent(
                                this,
                                this.handleModRemoveSubmit
                              )}
                            >
                              {i18n.t('restore')}
                            </span>
                          )}
                        </li>
                      </>
                    )}
                    {/* Mods can ban from community, and appoint as mods to community */}
                    {this.canMod && (
                      <>
                        {!this.isMod && (
                          <li className="list-inline-item">
                            {!node.comment.banned_from_community ? (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleModBanFromCommunityShow
                                )}
                              >
                                {i18n.t('ban')}
                              </span>
                            ) : (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleModBanFromCommunitySubmit
                                )}
                              >
                                {i18n.t('unban')}
                              </span>
                            )}
                          </li>
                        )}
                        {!node.comment.banned_from_community && (
                          <li className="list-inline-item">
                            {!this.state.showConfirmAppointAsMod ? (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleShowConfirmAppointAsMod
                                )}
                              >
                                {this.isMod
                                  ? i18n.t('remove_as_mod')
                                  : i18n.t('appoint_as_mod')}
                              </span>
                            ) : (
                              <>
                                <span class="d-inline-block mr-1">
                                  {i18n.t('are_you_sure')}
                                </span>
                                <span
                                  class="pointer d-inline-block mr-1"
                                  onClick={linkEvent(
                                    this,
                                    this.handleAddModToCommunity
                                  )}
                                >
                                  {i18n.t('yes')}
                                </span>
                                <span
                                  class="pointer d-inline-block"
                                  onClick={linkEvent(
                                    this,
                                    this.handleCancelConfirmAppointAsMod
                                  )}
                                >
                                  {i18n.t('no')}
                                </span>
                              </>
                            )}
                          </li>
                        )}
                      </>
                    )}
                    {/* Community creators and admins can transfer community to another mod */}
                    {(this.amCommunityCreator || this.canAdmin) && this.isMod && (
                      <li className="list-inline-item">
                        {!this.state.showConfirmTransferCommunity ? (
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleShowConfirmTransferCommunity
                            )}
                          >
                            {i18n.t('transfer_community')}
                          </span>
                        ) : (
                          <>
                            <span class="d-inline-block mr-1">
                              {i18n.t('are_you_sure')}
                            </span>
                            <span
                              class="pointer d-inline-block mr-1"
                              onClick={linkEvent(
                                this,
                                this.handleTransferCommunity
                              )}
                            >
                              {i18n.t('yes')}
                            </span>
                            <span
                              class="pointer d-inline-block"
                              onClick={linkEvent(
                                this,
                                this.handleCancelShowConfirmTransferCommunity
                              )}
                            >
                              {i18n.t('no')}
                            </span>
                          </>
                        )}
                      </li>
                    )}
                    {/* Admins can ban from all, and appoint other admins */}
                    {this.canAdmin && (
                      <>
                        {!this.isAdmin && (
                          <li className="list-inline-item">
                            {!node.comment.banned ? (
                              <span
                                class="pointer"
                                onClick={linkEvent(this, this.handleModBanShow)}
                              >
                                {i18n.t('ban_from_site')}
                              </span>
                            ) : (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleModBanSubmit
                                )}
                              >
                                {i18n.t('unban_from_site')}
                              </span>
                            )}
                          </li>
                        )}
                        {!node.comment.banned && (
                          <li className="list-inline-item">
                            {!this.state.showConfirmAppointAsAdmin ? (
                              <span
                                class="pointer"
                                onClick={linkEvent(
                                  this,
                                  this.handleShowConfirmAppointAsAdmin
                                )}
                              >
                                {this.isAdmin
                                  ? i18n.t('remove_as_admin')
                                  : i18n.t('appoint_as_admin')}
                              </span>
                            ) : (
                              <>
                                <span class="d-inline-block mr-1">
                                  {i18n.t('are_you_sure')}
                                </span>
                                <span
                                  class="pointer d-inline-block mr-1"
                                  onClick={linkEvent(this, this.handleAddAdmin)}
                                >
                                  {i18n.t('yes')}
                                </span>
                                <span
                                  class="pointer d-inline-block"
                                  onClick={linkEvent(
                                    this,
                                    this.handleCancelConfirmAppointAsAdmin
                                  )}
                                >
                                  {i18n.t('no')}
                                </span>
                              </>
                            )}
                          </li>
                        )}
                      </>
                    )}
                    {/* Site Creator can transfer to another admin */}
                    {this.amSiteCreator && this.isAdmin && (
                      <li className="list-inline-item">
                        {!this.state.showConfirmTransferSite ? (
                          <span
                            class="pointer"
                            onClick={linkEvent(
                              this,
                              this.handleShowConfirmTransferSite
                            )}
                          >
                            {i18n.t('transfer_site')}
                          </span>
                        ) : (
                          <>
                            <span class="d-inline-block mr-1">
                              {i18n.t('are_you_sure')}
                            </span>
                            <span
                              class="pointer d-inline-block mr-1"
                              onClick={linkEvent(this, this.handleTransferSite)}
                            >
                              {i18n.t('yes')}
                            </span>
                            <span
                              class="pointer d-inline-block"
                              onClick={linkEvent(
                                this,
                                this.handleCancelShowConfirmTransferSite
                              )}
                            >
                              {i18n.t('no')}
                            </span>
                          </>
                        )}
                      </li>
                    )}
                  </>
                )}
              </ul>
            </div>
          )}
        </div>
        {this.state.showRemoveDialog && (
          <form
            class="form-inline"
            onSubmit={linkEvent(this, this.handleModRemoveSubmit)}
          >
            <input
              type="text"
              class="form-control mr-2"
              placeholder={i18n.t('reason')}
              value={this.state.removeReason}
              onInput={linkEvent(this, this.handleModRemoveReasonChange)}
            />
            <button type="submit" class="btn btn-secondary">
              {i18n.t('remove_comment')}
            </button>
          </form>
        )}
        {this.state.showBanDialog && (
          <form onSubmit={linkEvent(this, this.handleModBanBothSubmit)}>
            <div class="form-group row">
              <label class="col-form-label">{i18n.t('reason')}</label>
              <input
                type="text"
                class="form-control mr-2"
                placeholder={i18n.t('reason')}
                value={this.state.banReason}
                onInput={linkEvent(this, this.handleModBanReasonChange)}
              />
            </div>
            {/* TODO hold off on expires until later */}
            {/* <div class="form-group row"> */}
            {/*   <label class="col-form-label">Expires</label> */}
            {/*   <input type="date" class="form-control mr-2" placeholder={i18n.t('expires')} value={this.state.banExpires} onInput={linkEvent(this, this.handleModBanExpiresChange)} /> */}
            {/* </div> */}
            <div class="form-group row">
              <button type="submit" class="btn btn-secondary">
                {i18n.t('ban')} {node.comment.creator_name}
              </button>
            </div>
          </form>
        )}
        {this.state.showReply && (
          <CommentForm
            node={node}
            onReplyCancel={this.handleReplyCancel}
            disabled={this.props.locked}
          />
        )}
        {node.children && !this.state.collapsed && (
          <CommentNodes
            nodes={node.children}
            locked={this.props.locked}
            moderators={this.props.moderators}
            admins={this.props.admins}
            postCreatorId={this.props.postCreatorId}
          />
        )}
        {/* A collapsed clearfix */}
        {this.state.collapsed && <div class="row col-12"></div>}
      </div>
    );
  }

  get myComment(): boolean {
    return (
      UserService.Instance.user &&
      this.props.node.comment.creator_id == UserService.Instance.user.id
    );
  }

  get isMod(): boolean {
    return (
      this.props.moderators &&
      isMod(
        this.props.moderators.map(m => m.user_id),
        this.props.node.comment.creator_id
      )
    );
  }

  get isAdmin(): boolean {
    return (
      this.props.admins &&
      isMod(
        this.props.admins.map(a => a.id),
        this.props.node.comment.creator_id
      )
    );
  }

  get isPostCreator(): boolean {
    return this.props.node.comment.creator_id == this.props.postCreatorId;
  }

  get canMod(): boolean {
    if (this.props.admins && this.props.moderators) {
      let adminsThenMods = this.props.admins
        .map(a => a.id)
        .concat(this.props.moderators.map(m => m.user_id));

      return canMod(
        UserService.Instance.user,
        adminsThenMods,
        this.props.node.comment.creator_id
      );
    } else {
      return false;
    }
  }

  get canAdmin(): boolean {
    return (
      this.props.admins &&
      canMod(
        UserService.Instance.user,
        this.props.admins.map(a => a.id),
        this.props.node.comment.creator_id
      )
    );
  }

  get amCommunityCreator(): boolean {
    return (
      this.props.moderators &&
      UserService.Instance.user &&
      this.props.node.comment.creator_id != UserService.Instance.user.id &&
      UserService.Instance.user.id == this.props.moderators[0].user_id
    );
  }

  get amSiteCreator(): boolean {
    return (
      this.props.admins &&
      UserService.Instance.user &&
      this.props.node.comment.creator_id != UserService.Instance.user.id &&
      UserService.Instance.user.id == this.props.admins[0].id
    );
  }

  get commentUnlessRemoved(): string {
    let node = this.props.node;
    return node.comment.removed
      ? `*${i18n.t('removed')}*`
      : node.comment.deleted
      ? `*${i18n.t('deleted')}*`
      : node.comment.content;
  }

  handleReplyClick(i: CommentNode) {
    i.state.showReply = true;
    i.setState(i.state);
  }

  handleEditClick(i: CommentNode) {
    i.state.showEdit = true;
    i.setState(i.state);
  }

  handleDeleteClick(i: CommentNode) {
    let deleteForm: CommentFormI = {
      content: i.props.node.comment.content,
      edit_id: i.props.node.comment.id,
      creator_id: i.props.node.comment.creator_id,
      post_id: i.props.node.comment.post_id,
      parent_id: i.props.node.comment.parent_id,
      deleted: !i.props.node.comment.deleted,
      auth: null,
    };
    WebSocketService.Instance.editComment(deleteForm);
  }

  handleSaveCommentClick(i: CommentNode) {
    let saved =
      i.props.node.comment.saved == undefined
        ? true
        : !i.props.node.comment.saved;
    let form: SaveCommentForm = {
      comment_id: i.props.node.comment.id,
      save: saved,
    };

    WebSocketService.Instance.saveComment(form);
  }

  handleReplyCancel() {
    this.state.showReply = false;
    this.state.showEdit = false;
    this.setState(this.state);
  }

  handleCommentUpvote(i: CommentNodeI) {
    if (UserService.Instance.user) {
      this.setState({
        upvoteLoading: true,
      });
    }
    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: i.comment.my_vote == 1 ? 0 : 1,
    };
    WebSocketService.Instance.likeComment(form);
  }

  handleCommentDownvote(i: CommentNodeI) {
    if (UserService.Instance.user) {
      this.setState({
        downvoteLoading: true,
      });
    }
    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: i.comment.my_vote == -1 ? 0 : -1,
    };
    WebSocketService.Instance.likeComment(form);
  }

  handleModRemoveShow(i: CommentNode) {
    i.state.showRemoveDialog = true;
    i.setState(i.state);
  }

  handleModRemoveReasonChange(i: CommentNode, event: any) {
    i.state.removeReason = event.target.value;
    i.setState(i.state);
  }

  handleModRemoveSubmit(i: CommentNode) {
    event.preventDefault();
    let form: CommentFormI = {
      content: i.props.node.comment.content,
      edit_id: i.props.node.comment.id,
      creator_id: i.props.node.comment.creator_id,
      post_id: i.props.node.comment.post_id,
      parent_id: i.props.node.comment.parent_id,
      removed: !i.props.node.comment.removed,
      reason: i.state.removeReason,
      auth: null,
    };
    WebSocketService.Instance.editComment(form);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }

  handleMarkRead(i: CommentNode) {
    // if it has a user_mention_id field, then its a mention
    if (i.props.node.comment.user_mention_id) {
      let form: EditUserMentionForm = {
        user_mention_id: i.props.node.comment.user_mention_id,
        read: !i.props.node.comment.read,
      };
      WebSocketService.Instance.editUserMention(form);
    } else {
      let form: CommentFormI = {
        content: i.props.node.comment.content,
        edit_id: i.props.node.comment.id,
        creator_id: i.props.node.comment.creator_id,
        post_id: i.props.node.comment.post_id,
        parent_id: i.props.node.comment.parent_id,
        read: !i.props.node.comment.read,
        auth: null,
      };
      WebSocketService.Instance.editComment(form);
    }
  }

  handleModBanFromCommunityShow(i: CommentNode) {
    i.state.showBanDialog = !i.state.showBanDialog;
    i.state.banType = BanType.Community;
    i.setState(i.state);
  }

  handleModBanShow(i: CommentNode) {
    i.state.showBanDialog = !i.state.showBanDialog;
    i.state.banType = BanType.Site;
    i.setState(i.state);
  }

  handleModBanReasonChange(i: CommentNode, event: any) {
    i.state.banReason = event.target.value;
    i.setState(i.state);
  }

  handleModBanExpiresChange(i: CommentNode, event: any) {
    i.state.banExpires = event.target.value;
    i.setState(i.state);
  }

  handleModBanFromCommunitySubmit(i: CommentNode) {
    i.state.banType = BanType.Community;
    i.setState(i.state);
    i.handleModBanBothSubmit(i);
  }

  handleModBanSubmit(i: CommentNode) {
    i.state.banType = BanType.Site;
    i.setState(i.state);
    i.handleModBanBothSubmit(i);
  }

  handleModBanBothSubmit(i: CommentNode) {
    event.preventDefault();

    if (i.state.banType == BanType.Community) {
      let form: BanFromCommunityForm = {
        user_id: i.props.node.comment.creator_id,
        community_id: i.props.node.comment.community_id,
        ban: !i.props.node.comment.banned_from_community,
        reason: i.state.banReason,
        expires: getUnixTime(i.state.banExpires),
      };
      WebSocketService.Instance.banFromCommunity(form);
    } else {
      let form: BanUserForm = {
        user_id: i.props.node.comment.creator_id,
        ban: !i.props.node.comment.banned,
        reason: i.state.banReason,
        expires: getUnixTime(i.state.banExpires),
      };
      WebSocketService.Instance.banUser(form);
    }

    i.state.showBanDialog = false;
    i.setState(i.state);
  }

  handleShowConfirmAppointAsMod(i: CommentNode) {
    i.state.showConfirmAppointAsMod = true;
    i.setState(i.state);
  }

  handleCancelConfirmAppointAsMod(i: CommentNode) {
    i.state.showConfirmAppointAsMod = false;
    i.setState(i.state);
  }

  handleAddModToCommunity(i: CommentNode) {
    let form: AddModToCommunityForm = {
      user_id: i.props.node.comment.creator_id,
      community_id: i.props.node.comment.community_id,
      added: !i.isMod,
    };
    WebSocketService.Instance.addModToCommunity(form);
    i.state.showConfirmAppointAsMod = false;
    i.setState(i.state);
  }

  handleShowConfirmAppointAsAdmin(i: CommentNode) {
    i.state.showConfirmAppointAsAdmin = true;
    i.setState(i.state);
  }

  handleCancelConfirmAppointAsAdmin(i: CommentNode) {
    i.state.showConfirmAppointAsAdmin = false;
    i.setState(i.state);
  }

  handleAddAdmin(i: CommentNode) {
    let form: AddAdminForm = {
      user_id: i.props.node.comment.creator_id,
      added: !i.isAdmin,
    };
    WebSocketService.Instance.addAdmin(form);
    i.state.showConfirmAppointAsAdmin = false;
    i.setState(i.state);
  }

  handleShowConfirmTransferCommunity(i: CommentNode) {
    i.state.showConfirmTransferCommunity = true;
    i.setState(i.state);
  }

  handleCancelShowConfirmTransferCommunity(i: CommentNode) {
    i.state.showConfirmTransferCommunity = false;
    i.setState(i.state);
  }

  handleTransferCommunity(i: CommentNode) {
    let form: TransferCommunityForm = {
      community_id: i.props.node.comment.community_id,
      user_id: i.props.node.comment.creator_id,
    };
    WebSocketService.Instance.transferCommunity(form);
    i.state.showConfirmTransferCommunity = false;
    i.setState(i.state);
  }

  handleShowConfirmTransferSite(i: CommentNode) {
    i.state.showConfirmTransferSite = true;
    i.setState(i.state);
  }

  handleCancelShowConfirmTransferSite(i: CommentNode) {
    i.state.showConfirmTransferSite = false;
    i.setState(i.state);
  }

  handleTransferSite(i: CommentNode) {
    let form: TransferSiteForm = {
      user_id: i.props.node.comment.creator_id,
    };
    WebSocketService.Instance.transferSite(form);
    i.state.showConfirmTransferSite = false;
    i.setState(i.state);
  }

  get isCommentNew(): boolean {
    let now = moment.utc().subtract(10, 'minutes');
    let then = moment.utc(this.props.node.comment.published);
    return now.isBefore(then);
  }

  handleCommentCollapse(i: CommentNode) {
    i.state.collapsed = !i.state.collapsed;
    i.setState(i.state);
  }

  handleViewSource(i: CommentNode) {
    i.state.viewSource = !i.state.viewSource;
    i.setState(i.state);
  }
}
