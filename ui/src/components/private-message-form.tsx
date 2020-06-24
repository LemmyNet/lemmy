import { Component, linkEvent } from 'inferno';
import { Prompt } from 'inferno-router';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  PrivateMessageForm as PrivateMessageFormI,
  EditPrivateMessageForm,
  PrivateMessageFormParams,
  PrivateMessage,
  PrivateMessageResponse,
  UserView,
  UserOperation,
  UserDetailsResponse,
  GetUserDetailsForm,
  SortType,
  WebSocketJsonResponse,
} from '../interfaces';
import { WebSocketService } from '../services';
import {
  capitalizeFirstLetter,
  markdownHelpUrl,
  mdToHtml,
  wsJsonToRes,
  toast,
  randomStr,
  setupTribute,
  setupTippy,
} from '../utils';
import { UserListing } from './user-listing';
import Tribute from 'tributejs/src/Tribute.js';
import autosize from 'autosize';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface PrivateMessageFormProps {
  privateMessage?: PrivateMessage; // If a pm is given, that means this is an edit
  params?: PrivateMessageFormParams;
  onCancel?(): any;
  onCreate?(message: PrivateMessage): any;
  onEdit?(message: PrivateMessage): any;
}

interface PrivateMessageFormState {
  privateMessageForm: PrivateMessageFormI;
  recipient: UserView;
  loading: boolean;
  previewMode: boolean;
  showDisclaimer: boolean;
}

export class PrivateMessageForm extends Component<
  PrivateMessageFormProps,
  PrivateMessageFormState
> {
  private id = `message-form-${randomStr()}`;
  private tribute: Tribute;
  private subscription: Subscription;
  private emptyState: PrivateMessageFormState = {
    privateMessageForm: {
      content: null,
      recipient_id: null,
    },
    recipient: null,
    loading: false,
    previewMode: false,
    showDisclaimer: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.tribute = setupTribute();
    this.state = this.emptyState;

    if (this.props.privateMessage) {
      this.state.privateMessageForm = {
        content: this.props.privateMessage.content,
        recipient_id: this.props.privateMessage.recipient_id,
      };
    }

    if (this.props.params) {
      this.state.privateMessageForm.recipient_id = this.props.params.recipient_id;
      let form: GetUserDetailsForm = {
        user_id: this.state.privateMessageForm.recipient_id,
        sort: SortType[SortType.New],
        saved_only: false,
      };
      WebSocketService.Instance.getUserDetails(form);
    }

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );
  }

  componentDidMount() {
    var textarea: any = document.getElementById(this.id);
    autosize(textarea);
    this.tribute.attach(textarea);
    textarea.addEventListener('tribute-replaced', () => {
      this.state.privateMessageForm.content = textarea.value;
      this.setState(this.state);
      autosize.update(textarea);
    });
    setupTippy();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div>
        <Prompt
          when={!this.state.loading && this.state.privateMessageForm.content}
          message={i18n.t('block_leaving')}
        />
        <form onSubmit={linkEvent(this, this.handlePrivateMessageSubmit)}>
          {!this.props.privateMessage && (
            <div class="form-group row">
              <label class="col-sm-2 col-form-label">
                {capitalizeFirstLetter(i18n.t('to'))}
              </label>

              {this.state.recipient && (
                <div class="col-sm-10 form-control-plaintext">
                  <UserListing
                    user={{
                      name: this.state.recipient.name,
                      avatar: this.state.recipient.avatar,
                      id: this.state.recipient.id,
                      local: this.state.recipient.local,
                      actor_id: this.state.recipient.actor_id,
                    }}
                  />
                </div>
              )}
            </div>
          )}
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">{i18n.t('message')}</label>
            <div class="col-sm-10">
              <textarea
                id={this.id}
                value={this.state.privateMessageForm.content}
                onInput={linkEvent(this, this.handleContentChange)}
                className={`form-control ${this.state.previewMode && 'd-none'}`}
                rows={4}
                maxLength={10000}
              />
              {this.state.previewMode && (
                <div
                  className="card card-body md-div"
                  dangerouslySetInnerHTML={mdToHtml(
                    this.state.privateMessageForm.content
                  )}
                />
              )}
            </div>
          </div>

          {this.state.showDisclaimer && (
            <div class="form-group row">
              <div class="offset-sm-2 col-sm-10">
                <div class="alert alert-danger" role="alert">
                  <T i18nKey="private_message_disclaimer">
                    #
                    <a
                      class="alert-link"
                      target="_blank"
                      href="https://about.riot.im/"
                    >
                      #
                    </a>
                  </T>
                </div>
              </div>
            </div>
          )}
          <div class="form-group row">
            <div class="offset-sm-2 col-sm-10">
              <button
                type="submit"
                class="btn btn-secondary mr-2"
                disabled={this.state.loading}
              >
                {this.state.loading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : this.props.privateMessage ? (
                  capitalizeFirstLetter(i18n.t('save'))
                ) : (
                  capitalizeFirstLetter(i18n.t('send_message'))
                )}
              </button>
              {this.state.privateMessageForm.content && (
                <button
                  className={`btn btn-secondary mr-2 ${
                    this.state.previewMode && 'active'
                  }`}
                  onClick={linkEvent(this, this.handlePreviewToggle)}
                >
                  {i18n.t('preview')}
                </button>
              )}
              {this.props.privateMessage && (
                <button
                  type="button"
                  class="btn btn-secondary"
                  onClick={linkEvent(this, this.handleCancel)}
                >
                  {i18n.t('cancel')}
                </button>
              )}
              <ul class="d-inline-block float-right list-inline mb-1 text-muted font-weight-bold">
                <li class="list-inline-item">
                  <span
                    onClick={linkEvent(this, this.handleShowDisclaimer)}
                    class="pointer"
                    data-tippy-content={i18n.t('disclaimer')}
                  >
                    <svg class={`icon icon-inline`}>
                      <use xlinkHref="#icon-alert-triangle"></use>
                    </svg>
                  </span>
                </li>
                <li class="list-inline-item">
                  <a
                    href={markdownHelpUrl}
                    target="_blank"
                    class="text-muted"
                    title={i18n.t('formatting_help')}
                  >
                    <svg class="icon icon-inline">
                      <use xlinkHref="#icon-help-circle"></use>
                    </svg>
                  </a>
                </li>
              </ul>
            </div>
          </div>
        </form>
      </div>
    );
  }

  handlePrivateMessageSubmit(i: PrivateMessageForm, event: any) {
    event.preventDefault();
    if (i.props.privateMessage) {
      let editForm: EditPrivateMessageForm = {
        edit_id: i.props.privateMessage.id,
        content: i.state.privateMessageForm.content,
      };
      WebSocketService.Instance.editPrivateMessage(editForm);
    } else {
      WebSocketService.Instance.createPrivateMessage(
        i.state.privateMessageForm
      );
    }
    i.state.loading = true;
    i.setState(i.state);
  }

  handleRecipientChange(i: PrivateMessageForm, event: any) {
    i.state.recipient = event.target.value;
    i.setState(i.state);
  }

  handleContentChange(i: PrivateMessageForm, event: any) {
    i.state.privateMessageForm.content = event.target.value;
    i.setState(i.state);
  }

  handleCancel(i: PrivateMessageForm) {
    i.props.onCancel();
  }

  handlePreviewToggle(i: PrivateMessageForm, event: any) {
    event.preventDefault();
    i.state.previewMode = !i.state.previewMode;
    i.setState(i.state);
  }

  handleShowDisclaimer(i: PrivateMessageForm) {
    i.state.showDisclaimer = !i.state.showDisclaimer;
    i.setState(i.state);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (res.op == UserOperation.EditPrivateMessage) {
      let data = res.data as PrivateMessageResponse;
      this.state.loading = false;
      this.props.onEdit(data.message);
    } else if (res.op == UserOperation.GetUserDetails) {
      let data = res.data as UserDetailsResponse;
      this.state.recipient = data.user;
      this.state.privateMessageForm.recipient_id = data.user.id;
      this.setState(this.state);
    } else if (res.op == UserOperation.CreatePrivateMessage) {
      let data = res.data as PrivateMessageResponse;
      this.state.loading = false;
      this.props.onCreate(data.message);
      this.setState(this.state);
    }
  }
}
