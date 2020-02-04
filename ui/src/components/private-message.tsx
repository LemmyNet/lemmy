import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import {
  PrivateMessage as PrivateMessageI,
  EditPrivateMessageForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  mdToHtml,
  pictshareAvatarThumbnail,
  showAvatars,
  toast,
} from '../utils';
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
      <div class="mb-2">
        <div>
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              {this.mine ? i18n.t('to') : i18n.t('from')}
            </li>
            <li className="list-inline-item">
              <Link
                className="text-info"
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
                      src={pictshareAvatarThumbnail(
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
                {this.state.collapsed ? '[+]' : '[-]'}
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
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                {!this.mine && (
                  <>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleMarkRead)}
                      >
                        {message.read
                          ? i18n.t('mark_as_unread')
                          : i18n.t('mark_as_read')}
                      </span>
                    </li>
                    <li className="list-inline-item">
                      <span
                        class="pointer"
                        onClick={linkEvent(this, this.handleReplyClick)}
                      >
                        {i18n.t('reply')}
                      </span>
                    </li>
                  </>
                )}
                {this.mine && (
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
                        {!message.deleted
                          ? i18n.t('delete')
                          : i18n.t('restore')}
                      </span>
                    </li>
                  </>
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
