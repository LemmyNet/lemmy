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
  CommentSortType,
  SortType,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  mdToHtml,
  getUnixTime,
  canMod,
  isMod,
  pictshareAvatarThumbnail,
  showAvatars,
  setupTippy,
  colorList,
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
  showAdvanced: boolean;
  my_vote: number;
  score: number;
  upvotes: number;
  downvotes: number;
  borderColor: string;
}

interface CommentNodeProps {
  node: CommentNodeI;
  noIndent?: boolean;
  viewOnly?: boolean;
  locked?: boolean;
  markable?: boolean;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  // TODO is this necessary, can't I get it from the node itself?
  postCreatorId?: number;
  showCommunity?: boolean;
  sort?: CommentSortType;
  sortType?: SortType;
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
    showAdvanced: false,
    showConfirmTransferSite: false,
    showConfirmTransferCommunity: false,
    showConfirmAppointAsMod: false,
    showConfirmAppointAsAdmin: false,
    my_vote: this.props.node.comment.my_vote,
    score: this.props.node.comment.score,
    upvotes: this.props.node.comment.upvotes,
    downvotes: this.props.node.comment.downvotes,
    borderColor: this.props.node.comment.depth
      ? colorList[this.props.node.comment.depth % colorList.length]
      : colorList[0],
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
    this.handleCommentUpvote = this.handleCommentUpvote.bind(this);
    this.handleCommentDownvote = this.handleCommentDownvote.bind(this);
  }

  componentWillReceiveProps(nextProps: CommentNodeProps) {
    this.state.my_vote = nextProps.node.comment.my_vote;
    this.state.upvotes = nextProps.node.comment.upvotes;
    this.state.downvotes = nextProps.node.comment.downvotes;
    this.state.score = nextProps.node.comment.score;
    this.setState(this.state);
  }

  render() {
    let node = this.props.node;
    return (
      <div
        className={`comment ${
          node.comment.parent_id && !this.props.noIndent ? 'ml-2' : ''
        }`}
      >
        {!node.comment.parent_id && !this.props.noIndent && (
          <>
            <hr class="d-sm-none my-2" />
            <div class="d-none d-sm-block d-sm-none my-3" />
          </>
        )}
        <div
          id={`comment-${node.comment.id}`}
          className={`details comment-node mb-1 ${
            this.isCommentNew ? 'mark' : ''
          }`}
          style={
            !this.props.noIndent &&
            this.props.node.comment.parent_id &&
            `border-left: 2px ${this.state.borderColor} solid !important`
          }
        >
          <div
            class={`${!this.props.noIndent &&
              this.props.node.comment.parent_id &&
              'ml-2'}`}
          >
            <ul class="list-inline mb-1 text-muted small">
              <li className="list-inline-item">
                <Link
                  className="text-body font-weight-bold"
                  to={`/u/${node.comment.creator_name}`}
                >
                  {node.comment.creator_avatar && showAvatars() && (
                    <img
                      height="32"
                      width="32"
                      src={pictshareAvatarThumbnail(
                        node.comment.creator_avatar
                      )}
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
              {this.props.showCommunity && (
                <li className="list-inline-item">
                  <span> {i18n.t('to')} </span>
                  <Link to={`/c/${node.comment.community_name}`}>
                    {node.comment.community_name}
                  </Link>
                </li>
              )}
              <li className="list-inline-item">•</li>
              <li className="list-inline-item">
                <span
                  className={`unselectable pointer ${this.scoreColor}`}
                  onClick={linkEvent(node, this.handleCommentUpvote)}
                  data-tippy-content={this.pointsTippy}
                >
                  <svg class="icon icon-inline mr-1">
                    <use xlinkHref="#icon-zap"></use>
                  </svg>
                  {this.state.score}
                </span>
              </li>
              <li className="list-inline-item">•</li>
              <li className="list-inline-item">
                <span>
                  <MomentTime data={node.comment} />
                </span>
              </li>
              <li className="list-inline-item">
                <div
                  className="unselectable pointer text-monospace"
                  onClick={linkEvent(this, this.handleCommentCollapse)}
                >
                  {this.state.collapsed ? (
                    <svg class="icon icon-inline">
                      <use xlinkHref="#icon-plus-square"></use>
                    </svg>
                  ) : (
                    <svg class="icon icon-inline">
                      <use xlinkHref="#icon-minus-square"></use>
                    </svg>
                  )}
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
                    dangerouslySetInnerHTML={mdToHtml(
                      this.commentUnlessRemoved
                    )}
                  />
                )}
                <ul class="list-inline mb-0 text-muted font-weight-bold h5">
                  {this.props.markable && (
                    <li className="list-inline-item-action">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleMarkRead)}
                        data-tippy-content={
                          node.comment.read
                            ? i18n.t('mark_as_unread')
                            : i18n.t('mark_as_read')
                        }
                      >
                        <svg
                          class={`icon icon-inline ${node.comment.read &&
                            'text-success'}`}
                        >
                          <use xlinkHref="#icon-check"></use>
                        </svg>
                      </span>
                    </li>
                  )}
                  {UserService.Instance.user && !this.props.viewOnly && (
                    <>
                      <li className="list-inline-item-action">
                        <button
                          className={`vote-animate btn btn-link p-0 mb-1 ${
                            this.state.my_vote == 1 ? 'text-info' : 'text-muted'
                          }`}
                          onClick={linkEvent(node, this.handleCommentUpvote)}
                          data-tippy-content={i18n.t('upvote')}
                        >
                          <svg class="icon icon-inline">
                            <use xlinkHref="#icon-arrow-up"></use>
                          </svg>
                          {this.state.upvotes !== this.state.score && (
                            <span class="ml-1">{this.state.upvotes}</span>
                          )}
                        </button>
                      </li>
                      {WebSocketService.Instance.site.enable_downvotes && (
                        <li className="list-inline-item-action">
                          <button
                            className={`vote-animate btn btn-link p-0 mb-1 ${
                              this.state.my_vote == -1
                                ? 'text-danger'
                                : 'text-muted'
                            }`}
                            onClick={linkEvent(
                              node,
                              this.handleCommentDownvote
                            )}
                            data-tippy-content={i18n.t('downvote')}
                          >
                            <svg class="icon icon-inline">
                              <use xlinkHref="#icon-arrow-down"></use>
                            </svg>
                            {this.state.upvotes !== this.state.score && (
                              <span class="ml-1">{this.state.downvotes}</span>
                            )}
                          </button>
                        </li>
                      )}
                      <li className="list-inline-item-action">
                        <span
                          class="pointer"
                          onClick={linkEvent(this, this.handleReplyClick)}
                          data-tippy-content={i18n.t('reply')}
                        >
                          <svg class="icon icon-inline">
                            <use xlinkHref="#icon-reply1"></use>
                          </svg>
                        </span>
                      </li>
                      <li className="list-inline-item-action">
                        <Link
                          className="text-muted"
                          to={`/post/${node.comment.post_id}/comment/${node.comment.id}`}
                          title={i18n.t('link')}
                        >
                          <svg class="icon icon-inline">
                            <use xlinkHref="#icon-link"></use>
                          </svg>
                        </Link>
                      </li>
                      {!this.state.showAdvanced ? (
                        <li className="list-inline-item-action">
                          <span
                            className="unselectable pointer"
                            onClick={linkEvent(this, this.handleShowAdvanced)}
                            data-tippy-content={i18n.t('more')}
                          >
                            <svg class="icon icon-inline">
                              <use xlinkHref="#icon-more-vertical"></use>
                            </svg>
                          </span>
                        </li>
                      ) : (
                        <>
                          {!this.myComment && (
                            <li className="list-inline-item-action">
                              <Link
                                class="text-muted"
                                to={`/create_private_message?recipient_id=${node.comment.creator_id}`}
                                title={i18n.t('message').toLowerCase()}
                              >
                                <svg class="icon">
                                  <use xlinkHref="#icon-mail"></use>
                                </svg>
                              </Link>
                            </li>
                          )}
                          <li className="list-inline-item-action">
                            <span
                              class="pointer"
                              onClick={linkEvent(
                                this,
                                this.handleSaveCommentClick
                              )}
                              data-tippy-content={
                                node.comment.saved
                                  ? i18n.t('unsave')
                                  : i18n.t('save')
                              }
                            >
                              <svg
                                class={`icon icon-inline ${node.comment.saved &&
                                  'text-warning'}`}
                              >
                                <use xlinkHref="#icon-star"></use>
                              </svg>
                            </span>
                          </li>
                          <li className="list-inline-item-action">
                            <span
                              className="pointer"
                              onClick={linkEvent(this, this.handleViewSource)}
                              data-tippy-content={i18n.t('view_source')}
                            >
                              <svg
                                class={`icon icon-inline ${this.state
                                  .viewSource && 'text-success'}`}
                              >
                                <use xlinkHref="#icon-file-text"></use>
                              </svg>
                            </span>
                          </li>
                          {this.myComment && (
                            <>
                              <li className="list-inline-item-action">•</li>
                              <li className="list-inline-item-action">
                                <span
                                  class="pointer"
                                  onClick={linkEvent(
                                    this,
                                    this.handleEditClick
                                  )}
                                  data-tippy-content={i18n.t('edit')}
                                >
                                  <svg class="icon icon-inline">
                                    <use xlinkHref="#icon-edit"></use>
                                  </svg>
                                </span>
                              </li>
                              <li className="list-inline-item-action">
                                <span
                                  class="pointer"
                                  onClick={linkEvent(
                                    this,
                                    this.handleDeleteClick
                                  )}
                                  data-tippy-content={
                                    !node.comment.deleted
                                      ? i18n.t('delete')
                                      : i18n.t('restore')
                                  }
                                >
                                  <svg
                                    class={`icon icon-inline ${node.comment
                                      .deleted && 'text-danger'}`}
                                  >
                                    <use xlinkHref="#icon-trash"></use>
                                  </svg>
                                </span>
                              </li>
                            </>
                          )}
                          {/* Admins and mods can remove comments */}
                          {(this.canMod || this.canAdmin) && (
                            <>
                              <li className="list-inline-item-action">
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
                                <li className="list-inline-item-action">
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
                                <li className="list-inline-item-action">
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
                          {(this.amCommunityCreator || this.canAdmin) &&
                            this.isMod && (
                              <li className="list-inline-item-action">
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
                                        this
                                          .handleCancelShowConfirmTransferCommunity
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
                                <li className="list-inline-item-action">
                                  {!node.comment.banned ? (
                                    <span
                                      class="pointer"
                                      onClick={linkEvent(
                                        this,
                                        this.handleModBanShow
                                      )}
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
                                <li className="list-inline-item-action">
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
                                        onClick={linkEvent(
                                          this,
                                          this.handleAddAdmin
                                        )}
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
                            <li className="list-inline-item-action">
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
                                    onClick={linkEvent(
                                      this,
                                      this.handleTransferSite
                                    )}
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
                    </>
                  )}
                </ul>
              </div>
            )}
          </div>
        </div>
        {/* end of details */}
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
            sort={this.props.sort}
            sortType={this.props.sortType}
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
    let new_vote = this.state.my_vote == 1 ? 0 : 1;

    if (this.state.my_vote == 1) {
      this.state.score--;
      this.state.upvotes--;
    } else if (this.state.my_vote == -1) {
      this.state.downvotes--;
      this.state.upvotes++;
      this.state.score += 2;
    } else {
      this.state.upvotes++;
      this.state.score++;
    }

    this.state.my_vote = new_vote;

    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: this.state.my_vote,
    };

    WebSocketService.Instance.likeComment(form);
    this.setState(this.state);
    setupTippy();
  }

  handleCommentDownvote(i: CommentNodeI) {
    let new_vote = this.state.my_vote == -1 ? 0 : -1;

    if (this.state.my_vote == 1) {
      this.state.score -= 2;
      this.state.upvotes--;
      this.state.downvotes++;
    } else if (this.state.my_vote == -1) {
      this.state.downvotes--;
      this.state.score++;
    } else {
      this.state.downvotes++;
      this.state.score--;
    }

    this.state.my_vote = new_vote;

    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: this.state.my_vote,
    };

    WebSocketService.Instance.likeComment(form);
    this.setState(this.state);
    setupTippy();
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

  handleShowAdvanced(i: CommentNode) {
    i.state.showAdvanced = !i.state.showAdvanced;
    i.setState(i.state);
    setupTippy();
  }

  get scoreColor() {
    if (this.state.my_vote == 1) {
      return 'text-info';
    } else if (this.state.my_vote == -1) {
      return 'text-danger';
    } else {
      return 'text-muted';
    }
  }

  get pointsTippy(): string {
    let points = i18n.t('number_of_points', {
      count: this.state.score,
    });

    let upvotes = i18n.t('number_of_upvotes', {
      count: this.state.upvotes,
    });

    let downvotes = i18n.t('number_of_downvotes', {
      count: this.state.downvotes,
    });

    return `${points} • ${upvotes} • ${downvotes}`;
  }
}
