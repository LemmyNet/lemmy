import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import { Prompt } from 'inferno-router';
import {
  CommentNode as CommentNodeI,
  CommentForm as CommentFormI,
  WebSocketJsonResponse,
  UserOperation,
  CommentResponse,
} from '../interfaces';
import {
  capitalizeFirstLetter,
  mdToHtml,
  randomStr,
  markdownHelpUrl,
  toast,
  setupTribute,
  wsJsonToRes,
} from '../utils';
import { WebSocketService, UserService } from '../services';
import autosize from 'autosize';
import Tribute from 'tributejs/src/Tribute.js';
import { i18n } from '../i18next';

interface CommentFormProps {
  postId?: number;
  node?: CommentNodeI;
  onReplyCancel?(): any;
  edit?: boolean;
  disabled?: boolean;
}

interface CommentFormState {
  commentForm: CommentFormI;
  buttonTitle: string;
  previewMode: boolean;
  loading: boolean;
  imageLoading: boolean;
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {
  private id = `comment-textarea-${randomStr()}`;
  private formId = `comment-form-${randomStr()}`;
  private tribute: Tribute;
  private subscription: Subscription;
  private emptyState: CommentFormState = {
    commentForm: {
      auth: null,
      content: null,
      post_id: this.props.node
        ? this.props.node.comment.post_id
        : this.props.postId,
      creator_id: UserService.Instance.user
        ? UserService.Instance.user.id
        : null,
    },
    buttonTitle: !this.props.node
      ? capitalizeFirstLetter(i18n.t('post'))
      : this.props.edit
      ? capitalizeFirstLetter(i18n.t('edit'))
      : capitalizeFirstLetter(i18n.t('reply')),
    previewMode: false,
    loading: false,
    imageLoading: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.tribute = setupTribute();
    this.state = this.emptyState;

    if (this.props.node) {
      if (this.props.edit) {
        this.state.commentForm.edit_id = this.props.node.comment.id;
        this.state.commentForm.parent_id = this.props.node.comment.parent_id;
        this.state.commentForm.content = this.props.node.comment.content;
        this.state.commentForm.creator_id = this.props.node.comment.creator_id;
      } else {
        // A reply gets a new parent id
        this.state.commentForm.parent_id = this.props.node.comment.id;
      }
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
      this.state.commentForm.content = textarea.value;
      this.setState(this.state);
      autosize.update(textarea);
    });
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="mb-3">
        <Prompt
          when={this.state.commentForm.content}
          message={i18n.t('block_leaving')}
        />
        <form
          id={this.formId}
          onSubmit={linkEvent(this, this.handleCommentSubmit)}
        >
          <div class="form-group row">
            <div className={`col-sm-12`}>
              <textarea
                id={this.id}
                className={`form-control ${this.state.previewMode && 'd-none'}`}
                value={this.state.commentForm.content}
                onInput={linkEvent(this, this.handleCommentContentChange)}
                onPaste={linkEvent(this, this.handleImageUploadPaste)}
                required
                disabled={this.props.disabled}
                rows={2}
                maxLength={10000}
              />
              {this.state.previewMode && (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(
                    this.state.commentForm.content
                  )}
                />
              )}
            </div>
          </div>
          <div class="row">
            <div class="col-sm-12">
              <button
                type="submit"
                class="btn btn-sm btn-secondary mr-2"
                disabled={this.props.disabled}
              >
                {this.state.loading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : (
                  <span>{this.state.buttonTitle}</span>
                )}
              </button>
              {this.state.commentForm.content && (
                <button
                  className={`btn btn-sm mr-2 btn-secondary ${this.state
                    .previewMode && 'active'}`}
                  onClick={linkEvent(this, this.handlePreviewToggle)}
                >
                  {i18n.t('preview')}
                </button>
              )}
              {this.props.node && (
                <button
                  type="button"
                  class="btn btn-sm btn-secondary mr-2"
                  onClick={linkEvent(this, this.handleReplyCancel)}
                >
                  {i18n.t('cancel')}
                </button>
              )}
              <a
                href={markdownHelpUrl}
                target="_blank"
                class="d-inline-block float-right text-muted font-weight-bold"
                title={i18n.t('formatting_help')}
              >
                <svg class="icon icon-inline">
                  <use xlinkHref="#icon-help-circle"></use>
                </svg>
              </a>
              <form class="d-inline-block mr-3 float-right text-muted font-weight-bold">
                <label
                  htmlFor={`file-upload-${this.id}`}
                  className={`${UserService.Instance.user && 'pointer'}`}
                  data-tippy-content={i18n.t('upload_image')}
                >
                  <svg class="icon icon-inline">
                    <use xlinkHref="#icon-image"></use>
                  </svg>
                </label>
                <input
                  id={`file-upload-${this.id}`}
                  type="file"
                  accept="image/*,video/*"
                  name="file"
                  class="d-none"
                  disabled={!UserService.Instance.user}
                  onChange={linkEvent(this, this.handleImageUpload)}
                />
              </form>
              {this.state.imageLoading && (
                <svg class="icon icon-spinner spin">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              )}
            </div>
          </div>
        </form>
      </div>
    );
  }

  handleFinished() {
    this.state.previewMode = false;
    this.state.loading = false;
    this.state.commentForm.content = '';
    this.setState(this.state);
    let form: any = document.getElementById(this.formId);
    form.reset();
    if (this.props.node) {
      this.props.onReplyCancel();
    }
    autosize.update(document.querySelector('textarea'));
    this.setState(this.state);
  }

  handleCommentSubmit(i: CommentForm, event: any) {
    event.preventDefault();
    if (i.props.edit) {
      WebSocketService.Instance.editComment(i.state.commentForm);
    } else {
      WebSocketService.Instance.createComment(i.state.commentForm);
    }

    i.state.loading = true;
    i.setState(i.state);
  }

  handleCommentContentChange(i: CommentForm, event: any) {
    i.state.commentForm.content = event.target.value;
    i.setState(i.state);
  }

  handlePreviewToggle(i: CommentForm, event: any) {
    event.preventDefault();
    i.state.previewMode = !i.state.previewMode;
    i.setState(i.state);
  }

  handleReplyCancel(i: CommentForm) {
    i.props.onReplyCancel();
  }

  handleImageUploadPaste(i: CommentForm, event: any) {
    let image = event.clipboardData.files[0];
    if (image) {
      i.handleImageUpload(i, image);
    }
  }

  handleImageUpload(i: CommentForm, event: any) {
    let file: any;
    if (event.target) {
      event.preventDefault();
      file = event.target.files[0];
    } else {
      file = event;
    }

    const imageUploadUrl = `/pictshare/api/upload.php`;
    const formData = new FormData();
    formData.append('file', file);

    i.state.imageLoading = true;
    i.setState(i.state);

    fetch(imageUploadUrl, {
      method: 'POST',
      body: formData,
    })
      .then(res => res.json())
      .then(res => {
        let url = `${window.location.origin}/pictshare/${res.url}`;
        let imageMarkdown =
          res.filetype == 'mp4' ? `[vid](${url}/raw)` : `![](${url})`;
        let content = i.state.commentForm.content;
        content = content ? `${content}\n${imageMarkdown}` : imageMarkdown;
        i.state.commentForm.content = content;
        i.state.imageLoading = false;
        i.setState(i.state);
        let textarea: any = document.getElementById(i.id);
        autosize.update(textarea);
      })
      .catch(error => {
        i.state.imageLoading = false;
        i.setState(i.state);
        toast(error, 'danger');
      });
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);

    // Only do the showing and hiding if logged in
    if (UserService.Instance.user) {
      if (res.op == UserOperation.CreateComment) {
        let data = res.data as CommentResponse;
        if (data.comment.creator_id == UserService.Instance.user.id) {
          this.handleFinished();
        }
      } else if (res.op == UserOperation.EditComment) {
        let data = res.data as CommentResponse;
        if (data.comment.creator_id == UserService.Instance.user.id) {
          this.handleFinished();
        }
      }
    }
  }
}
