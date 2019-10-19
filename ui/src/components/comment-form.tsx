import { Component, linkEvent } from 'inferno';
import {
  CommentNode as CommentNodeI,
  CommentForm as CommentFormI,
  SearchForm,
  SearchType,
  SortType,
  UserOperation,
  SearchResponse,
} from '../interfaces';
import { Subscription } from 'rxjs';
import {
  capitalizeFirstLetter,
  mentionDropdownFetchLimit,
  msgOp,
  mdToHtml,
  randomStr,
  markdownHelpUrl,
} from '../utils';
import { WebSocketService, UserService } from '../services';
import * as autosize from 'autosize';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';
import Tribute from 'tributejs/src/Tribute.js';
import * as emojiShortName from 'emoji-short-name';

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
  private userSub: Subscription;
  private communitySub: Subscription;
  private tribute: any;
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

    this.tribute = new Tribute({
      collection: [
        // Emojis
        {
          trigger: ':',
          menuItemTemplate: (item: any) => {
            let emoji = `:${item.original.key}:`;
            return `${item.original.val} ${emoji}`;
          },
          selectTemplate: (item: any) => {
            return `:${item.original.key}:`;
          },
          values: Object.entries(emojiShortName).map(e => {
            return { key: e[1], val: e[0] };
          }),
          allowSpaces: false,
          autocompleteMode: true,
          menuItemLimit: mentionDropdownFetchLimit,
        },
        // Users
        {
          trigger: '@',
          selectTemplate: (item: any) => {
            return `[/u/${item.original.key}](/u/${item.original.key})`;
          },
          values: (text: string, cb: any) => {
            this.userSearch(text, (users: any) => cb(users));
          },
          allowSpaces: false,
          autocompleteMode: true,
          menuItemLimit: mentionDropdownFetchLimit,
        },

        // Communities
        {
          trigger: '#',
          selectTemplate: (item: any) => {
            return `[/c/${item.original.key}](/c/${item.original.key})`;
          },
          values: (text: string, cb: any) => {
            this.communitySearch(text, (communities: any) => cb(communities));
          },
          allowSpaces: false,
          autocompleteMode: true,
          menuItemLimit: mentionDropdownFetchLimit,
        },
      ],
    });

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
                  <T i18nKey="preview">#</T>
                </button>
              )}
              {this.props.node && (
                <button
                  type="button"
                  class="btn btn-sm btn-secondary mr-2"
                  onClick={linkEvent(this, this.handleReplyCancel)}
                >
                  <T i18nKey="cancel">#</T>
                </button>
              )}
              <a
                href={markdownHelpUrl}
                target="_blank"
                class="d-inline-block float-right text-muted small font-weight-bold"
              >
                <T i18nKey="formatting_help">#</T>
              </a>
              <form class="d-inline-block mr-2 float-right text-muted small font-weight-bold">
                <label
                  htmlFor={`file-upload-${this.id}`}
                  className={`${UserService.Instance.user && 'pointer'}`}
                >
                  <T i18nKey="upload_image">#</T>
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

  handleImageUpload(i: CommentForm, event: any) {
    event.preventDefault();
    let file = event.target.files[0];
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
        let markdown =
          res.filetype == 'mp4' ? `[vid](${url}/raw)` : `![](${url})`;
        let content = i.state.commentForm.content;
        content = content ? `${content} ${markdown}` : markdown;
        i.state.commentForm.content = content;
        i.state.imageLoading = false;
        i.setState(i.state);
      })
      .catch(error => {
        i.state.imageLoading = false;
        i.setState(i.state);
        alert(error);
      });
  }

  userSearch(text: string, cb: any) {
    if (text) {
      let form: SearchForm = {
        q: text,
        type_: SearchType[SearchType.Users],
        sort: SortType[SortType.TopAll],
        page: 1,
        limit: mentionDropdownFetchLimit,
      };

      WebSocketService.Instance.search(form);

      this.userSub = WebSocketService.Instance.subject.subscribe(
        msg => {
          let op: UserOperation = msgOp(msg);
          if (op == UserOperation.Search) {
            let res: SearchResponse = msg;
            let users = res.users.map(u => {
              return { key: u.name };
            });
            cb(users);
            this.userSub.unsubscribe();
          }
        },
        err => console.error(err),
        () => console.log('complete')
      );
    } else {
      cb([]);
    }
  }

  communitySearch(text: string, cb: any) {
    if (text) {
      let form: SearchForm = {
        q: text,
        type_: SearchType[SearchType.Communities],
        sort: SortType[SortType.TopAll],
        page: 1,
        limit: mentionDropdownFetchLimit,
      };

      WebSocketService.Instance.search(form);

      this.communitySub = WebSocketService.Instance.subject.subscribe(
        msg => {
          let op: UserOperation = msgOp(msg);
          if (op == UserOperation.Search) {
            let res: SearchResponse = msg;
            let communities = res.communities.map(u => {
              return { key: u.name };
            });
            cb(communities);
            this.communitySub.unsubscribe();
          }
        },
        err => console.error(err),
        () => console.log('complete')
      );
    } else {
      cb([]);
    }
  }
}
