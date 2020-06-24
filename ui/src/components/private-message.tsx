import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import {
  PrivateMessage as PrivateMessageI,
  EditPrivateMessageForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { mdToHtml, pictrsAvatarThumbnail, showAvatars, toast } from '../utils';
import { MomentTime } from './moment-time';
import { PrivateMessageForm } from './private-message-form';
import { i18n } from '../i18next';

interface PrivateMessageState {
  showReply: boolean;
  showEdit: boolean;
  collapsed: boolean;
  viewSource: boolean;
}

interface PrivateMessageProps {
  privateMessage: PrivateMessageI;
}

export class PrivateMessage extends Component<
  PrivateMessageProps,
  PrivateMessageState
> {
  private emptyState: PrivateMessageState = {
    showReply: false,
    showEdit: false,
    collapsed: false,
    viewSource: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
    this.handlePrivateMessageCreate = this.handlePrivateMessageCreate.bind(
      this
    );
    this.handlePrivateMessageEdit = this.handlePrivateMessageEdit.bind(this);
  }

  get mine(): boolean {
    return UserService.Instance.user.id == this.props.privateMessage.creator_id;
  }

  render() {
    let message = this.props.privateMessage;
    return (
      <div class="border-top border-light">
        <div>
          <ul class="list-inline mb-0 text-muted small">
            {/* TODO refactor this */}
            <li className="list-inline-item">
              {this.mine ? i18n.t('to') : i18n.t('from')}
            </li>
            <li className="list-inline-item">
              <Link
                className="text-body font-weight-bold"
                to={
                  this.mine
                    ? `/u/${message.recipient_name}`
                    : `/u/${message.creator_name}`
                }
              >
                {(this.mine
                  ? message.recipient_avatar
                  : message.creator_avatar) &&
                  showAvatars() && (
                    <img
                      height="32"
                      width="32"
                      src={pictrsAvatarThumbnail(
                        this.mine
                          ? message.recipient_avatar
                          : message.creator_avatar
                      )}
                      class="rounded-circle mr-1"
                    />
                  )}
                <span>
                  {this.mine ? message.recipient_name : message.creator_name}
                </span>
              </Link>
            </li>
            <li className="list-inline-item">
              <span>
                <MomentTime data={message} />
              </span>
            </li>
            <li className="list-inline-item">
              <div
                className="pointer text-monospace"
                onClick={linkEvent(this, this.handleMessageCollapse)}
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
            <PrivateMessageForm
              privateMessage={message}
              onEdit={this.handlePrivateMessageEdit}
              onCancel={this.handleReplyCancel}
            />
          )}
          {!this.state.showEdit && !this.state.collapsed && (
            <div>
              {this.state.viewSource ? (
                <pre>{this.messageUnlessRemoved}</pre>
              ) : (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(this.messageUnlessRemoved)}
                />
              )}
              <ul class="list-inline mb-0 text-muted font-weight-bold">
                {!this.mine && (
                  <>
                    <li className="list-inline-item">
                      <button
                        class="btn btn-link btn-sm btn-animate text-muted"
                        onClick={linkEvent(this, this.handleMarkRead)}
                        data-tippy-content={
                          message.read
                            ? i18n.t('mark_as_unread')
                            : i18n.t('mark_as_read')
                        }
                      >
                        <svg
                          class={`icon icon-inline ${
                            message.read && 'text-success'
                          }`}
                        >
                          <use xlinkHref="#icon-check"></use>
                        </svg>
                      </button>
                    </li>
                    <li className="list-inline-item">
                      <button
                        class="btn btn-link btn-sm btn-animate text-muted"
                        onClick={linkEvent(this, this.handleReplyClick)}
                        data-tippy-content={i18n.t('reply')}
                      >
                        <svg class="icon icon-inline">
                          <use xlinkHref="#icon-reply1"></use>
                        </svg>
                      </button>
                    </li>
                  </>
                )}
                {this.mine && (
                  <>
                    <li className="list-inline-item">
                      <button
                        class="btn btn-link btn-sm btn-animate text-muted"
                        onClick={linkEvent(this, this.handleEditClick)}
                        data-tippy-content={i18n.t('edit')}
                      >
                        <svg class="icon icon-inline">
                          <use xlinkHref="#icon-edit"></use>
                        </svg>
                      </button>
                    </li>
                    <li className="list-inline-item">
                      <button
                        class="btn btn-link btn-sm btn-animate text-muted"
                        onClick={linkEvent(this, this.handleDeleteClick)}
                        data-tippy-content={
                          !message.deleted
                            ? i18n.t('delete')
                            : i18n.t('restore')
                        }
                      >
                        <svg
                          class={`icon icon-inline ${
                            message.deleted && 'text-danger'
                          }`}
                        >
                          <use xlinkHref="#icon-trash"></use>
                        </svg>
                      </button>
                    </li>
                  </>
                )}
                <li className="list-inline-item">
                  <button
                    class="btn btn-link btn-sm btn-animate text-muted"
                    onClick={linkEvent(this, this.handleViewSource)}
                    data-tippy-content={i18n.t('view_source')}
                  >
                    <svg
                      class={`icon icon-inline ${
                        this.state.viewSource && 'text-success'
                      }`}
                    >
                      <use xlinkHref="#icon-file-text"></use>
                    </svg>
                  </button>
                </li>
              </ul>
            </div>
          )}
        </div>
        {this.state.showReply && (
          <PrivateMessageForm
            params={{
              recipient_id: this.props.privateMessage.creator_id,
            }}
            onCreate={this.handlePrivateMessageCreate}
          />
        )}
        {/* A collapsed clearfix */}
        {this.state.collapsed && <div class="row col-12"></div>}
      </div>
    );
  }

  get messageUnlessRemoved(): string {
    let message = this.props.privateMessage;
    return message.deleted ? `*${i18n.t('deleted')}*` : message.content;
  }

  handleReplyClick(i: PrivateMessage) {
    i.state.showReply = true;
    i.setState(i.state);
  }

  handleEditClick(i: PrivateMessage) {
    i.state.showEdit = true;
    i.setState(i.state);
  }

  handleDeleteClick(i: PrivateMessage) {
    let form: EditPrivateMessageForm = {
      edit_id: i.props.privateMessage.id,
      deleted: !i.props.privateMessage.deleted,
    };
    WebSocketService.Instance.editPrivateMessage(form);
  }

  handleReplyCancel() {
    this.state.showReply = false;
    this.state.showEdit = false;
    this.setState(this.state);
  }

  handleMarkRead(i: PrivateMessage) {
    let form: EditPrivateMessageForm = {
      edit_id: i.props.privateMessage.id,
      read: !i.props.privateMessage.read,
    };
    WebSocketService.Instance.editPrivateMessage(form);
  }

  handleMessageCollapse(i: PrivateMessage) {
    i.state.collapsed = !i.state.collapsed;
    i.setState(i.state);
  }

  handleViewSource(i: PrivateMessage) {
    i.state.viewSource = !i.state.viewSource;
    i.setState(i.state);
  }

  handlePrivateMessageEdit() {
    this.state.showEdit = false;
    this.setState(this.state);
  }

  handlePrivateMessageCreate() {
    this.state.showReply = false;
    this.setState(this.state);
    toast(i18n.t('message_sent'));
  }
}
