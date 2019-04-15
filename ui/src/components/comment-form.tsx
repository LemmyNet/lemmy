import { Component, linkEvent } from 'inferno';
import { CommentNode as CommentNodeI, CommentForm as CommentFormI } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import * as autosize from 'autosize';

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
}

export class CommentForm extends Component<CommentFormProps, CommentFormState> {

  private emptyState: CommentFormState = {
    commentForm: {
      auth: null,
      content: null,
      post_id: this.props.node ? this.props.node.comment.post_id : this.props.postId,
      creator_id: UserService.Instance.loggedIn ? UserService.Instance.user.id : null,
    },
    buttonTitle: !this.props.node ? "Post" : this.props.edit ? "Edit" : "Reply",
  }

  constructor(props: any, context: any) {
    super(props, context);

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
    autosize(document.querySelectorAll('textarea'));
  }

  render() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleCommentSubmit)}>
          <div class="form-group row">
            <div class="col-sm-12">
              <textarea class="form-control" value={this.state.commentForm.content} onInput={linkEvent(this, this.handleCommentContentChange)} placeholder="Comment here" required disabled={this.props.disabled}/>
            </div>
          </div>
          <div class="row">
            <div class="col-sm-12">
              <button type="submit" class="btn btn-sm btn-secondary mr-2" disabled={this.props.disabled}>{this.state.buttonTitle}</button>
              {this.props.node && <button type="button" class="btn btn-sm btn-secondary" onClick={linkEvent(this, this.handleReplyCancel)}>Cancel</button>}
            </div>
          </div>
        </form>
      </div>
    );
  }

  handleCommentSubmit(i: CommentForm, event: any) {
    if (i.props.edit) {
      WebSocketService.Instance.editComment(i.state.commentForm);
    } else {
      WebSocketService.Instance.createComment(i.state.commentForm);
    }

    i.state.commentForm.content = undefined;
    i.setState(i.state);
    event.target.reset();
    if (i.props.node) {
      i.props.onReplyCancel();
    }
  }

  handleCommentContentChange(i: CommentForm, event: any) {
    i.state.commentForm.content = event.target.value;
    i.setState(i.state);
  }

  handleReplyCancel(i: CommentForm) {
    i.props.onReplyCancel();
  }
}
