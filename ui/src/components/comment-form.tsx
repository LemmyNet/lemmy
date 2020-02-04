import { Component, linkEvent } from 'inferno';
import {
  CommentNode as CommentNodeI,
  CommentForm as CommentFormI,
} from '../interfaces';
import {
  capitalizeFirstLetter,
  mdToHtml,
  randomStr,
  markdownHelpUrl,
  toast,
  setupTribute,
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
  imageLoading: boolean;
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {
  private id = `comment-form-${randomStr()}`;
  private tribute: Tribute;
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

  render() {
    return (
      <div class="mb-3">
        <form onSubmit={linkEvent(this, this.handleCommentSubmit)}>
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
                {this.state.buttonTitle}
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
                class="d-inline-block float-right text-muted small font-weight-bold"
              >
                {i18n.t('formatting_help')}
              </a>
              <form class="d-inline-block mr-2 float-right text-muted small font-weight-bold">
                <label
                  htmlFor={`file-upload-${this.id}`}
                  className={`${UserService.Instance.user && 'pointer'}`}
                >
                  {i18n.t('upload_image')}
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

  handleCommentSubmit(i: CommentForm, event: any) {
    event.preventDefault();
    if (i.props.edit) {
      WebSocketService.Instance.editComment(i.state.commentForm);
    } else {
      WebSocketService.Instance.createComment(i.state.commentForm);
    }

    i.state.previewMode = false;
    i.state.commentForm.content = undefined;
    event.target.reset();
    i.setState(i.state);
    if (i.props.node) {
      i.props.onReplyCancel();
    }

    autosize.update(document.querySelector('textarea'));
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
        var textarea: any = document.getElementById(i.id);
        autosize.update(textarea);
      })
      .catch(error => {
        i.state.imageLoading = false;
        i.setState(i.state);
        toast(error, 'danger');
      });
  }
}
