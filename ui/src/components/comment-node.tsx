import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { CommentNode as CommentNodeI, CommentLikeForm, CommentForm as CommentFormI } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { mdToHtml } from '../utils';
import { MomentTime } from './moment-time';
import { CommentForm } from './comment-form';
import { CommentNodes } from './comment-nodes';

interface CommentNodeState {
  showReply: boolean;
  showEdit: boolean;
}

interface CommentNodeProps {
  node: CommentNodeI;
  noIndent?: boolean;
  viewOnly?: boolean;
}

export class CommentNode extends Component<CommentNodeProps, CommentNodeState> {

  private emptyState: CommentNodeState = {
    showReply: false,
    showEdit: false
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleReplyCancel = this.handleReplyCancel.bind(this);
    this.handleCommentLike = this.handleCommentLike.bind(this);
    this.handleCommentDisLike = this.handleCommentDisLike.bind(this);
  }

  render() {
    let node = this.props.node;
    return (
      <div id={`comment-${node.comment.id}`} className={`comment ${node.comment.parent_id  && !this.props.noIndent ? 'ml-4' : ''}`}>
        <div className={`float-left small text-center ${this.props.viewOnly && 'no-click'}`}>
          <div className={`pointer upvote ${node.comment.my_vote == 1 ? 'text-info' : 'text-muted'}`} onClick={linkEvent(node, this.handleCommentLike)}>▲</div>
          <div>{node.comment.score}</div>
          <div className={`pointer downvote ${node.comment.my_vote == -1 && 'text-danger'}`} onClick={linkEvent(node, this.handleCommentDisLike)}>▼</div>
        </div>
        <div className="details ml-4">
          <ul class="list-inline mb-0 text-muted small">
            <li className="list-inline-item">
              <Link to={`/user/${node.comment.creator_id}`}>{node.comment.creator_name}</Link>
            </li>
            <li className="list-inline-item">
              <span>(
                <span className="text-info">+{node.comment.upvotes}</span>
                <span> | </span>
                <span className="text-danger">-{node.comment.downvotes}</span>
                <span>) </span>
              </span>
            </li>
            <li className="list-inline-item">
              <span><MomentTime data={node.comment} /></span>
            </li>
          </ul>
          {this.state.showEdit && <CommentForm node={node} edit onReplyCancel={this.handleReplyCancel} />}
          {!this.state.showEdit &&
            <div>
              <div className="md-div" dangerouslySetInnerHTML={mdToHtml(node.comment.content)} />
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                {!this.props.viewOnly && 
                  <span class="mr-2">
                    <li className="list-inline-item">
                      <span class="pointer" onClick={linkEvent(this, this.handleReplyClick)}>reply</span>
                    </li>
                    {this.myComment && 
                      <li className="list-inline-item">
                        <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
                      </li>
                    }
                    {this.myComment &&
                      <li className="list-inline-item">
                        <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span>
                      </li>
                    }
                  </span>
                }
                <li className="list-inline-item">
                  <Link className="text-muted" to={`/post/${node.comment.post_id}/comment/${node.comment.id}`} target="_blank">link</Link>
                </li>
              </ul>
            </div>
          }
        </div>
        {this.state.showReply && <CommentForm node={node} onReplyCancel={this.handleReplyCancel} />}
        {this.props.node.children && <CommentNodes nodes={this.props.node.children} />}
      </div>
    )
  }

  private get myComment(): boolean {
    return UserService.Instance.loggedIn && this.props.node.comment.creator_id == UserService.Instance.user.id;
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
      content: "*deleted*",
      edit_id: i.props.node.comment.id,
      post_id: i.props.node.comment.post_id,
      parent_id: i.props.node.comment.parent_id,
      auth: null
    };
    WebSocketService.Instance.editComment(deleteForm);
  }

  handleReplyCancel() {
    this.state.showReply = false;
    this.state.showEdit = false;
    this.setState(this.state);
  }


  handleCommentLike(i: CommentNodeI) {

    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: (i.comment.my_vote == 1) ? 0 : 1
    };
    WebSocketService.Instance.likeComment(form);
  }

  handleCommentDisLike(i: CommentNodeI) {
    let form: CommentLikeForm = {
      comment_id: i.comment.id,
      post_id: i.comment.post_id,
      score: (i.comment.my_vote == -1) ? 0 : -1
    };
    WebSocketService.Instance.likeComment(form);
  }
}
