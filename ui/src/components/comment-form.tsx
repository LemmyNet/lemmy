import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  CommentNode as CommentNodeI,
  CommentForm as CommentFormI,
  WebSocketJsonResponse,
  UserOperation,
  CommentResponse,
} from '../interfaces';
import { capitalizeFirstLetter, wsJsonToRes } from '../utils';
import { WebSocketService, UserService } from '../services';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';
import { MarkdownTextArea } from './markdown-textarea';

interface CommentFormProps {
  postId?: number;
  node?: CommentNodeI;
  onReplyCancel?(): any;
  edit?: boolean;
  disabled?: boolean;
  focus?: boolean;
}

interface CommentFormState {
  commentForm: CommentFormI;
  buttonTitle: string;
  finished: boolean;
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {
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
      ? capitalizeFirstLetter(i18n.t('save'))
      : capitalizeFirstLetter(i18n.t('reply')),
    finished: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.handleCommentSubmit = this.handleCommentSubmit.bind(this);
    this.handleReplyCancel = this.handleReplyCancel.bind(this);

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

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="mb-3">
        {UserService.Instance.user ? (
          <MarkdownTextArea
            initialContent={this.state.commentForm.content}
            buttonTitle={this.state.buttonTitle}
            finished={this.state.finished}
            replyType={!!this.props.node}
            focus={this.props.focus}
            disabled={this.props.disabled}
            onSubmit={this.handleCommentSubmit}
            onReplyCancel={this.handleReplyCancel}
          />
        ) : (
          <div class="alert alert-light" role="alert">
            <svg class="icon icon-inline mr-2">
              <use xlinkHref="#icon-alert-triangle"></use>
            </svg>
            <T i18nKey="must_login" class="d-inline">
              #
              <Link class="alert-link" to="/login">
                #
              </Link>
            </T>
          </div>
        )}
      </div>
    );
  }

  handleFinished(op: UserOperation, data: CommentResponse) {
    let isReply =
      this.props.node !== undefined && data.comment.parent_id !== null;
    let xor =
      +!(data.comment.parent_id !== null) ^ +(this.props.node !== undefined);

    if (
      (data.comment.creator_id == UserService.Instance.user.id &&
        ((op == UserOperation.CreateComment &&
          // If its a reply, make sure parent child match
          isReply &&
          data.comment.parent_id == this.props.node.comment.id) ||
          // Otherwise, check the XOR of the two
          (!isReply && xor))) ||
      // If its a comment edit, only check that its from your user, and that its a
      // text edit only

      (data.comment.creator_id == UserService.Instance.user.id &&
        op == UserOperation.EditComment &&
        data.comment.content)
    ) {
      this.state.finished = true;
      this.setState(this.state);
    }
  }

  handleCommentSubmit(val: string) {
    this.state.commentForm.content = val;
    if (this.props.edit) {
      WebSocketService.Instance.editComment(this.state.commentForm);
    } else {
      WebSocketService.Instance.createComment(this.state.commentForm);
    }
    this.setState(this.state);
  }

  handleReplyCancel() {
    this.props.onReplyCancel();
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);

    // Only do the showing and hiding if logged in
    if (UserService.Instance.user) {
      if (res.op == UserOperation.CreateComment) {
        let data = res.data as CommentResponse;
        this.handleFinished(res.op, data);
      } else if (res.op == UserOperation.EditComment) {
        let data = res.data as CommentResponse;
        this.handleFinished(res.op, data);
      }
    }
  }
}
