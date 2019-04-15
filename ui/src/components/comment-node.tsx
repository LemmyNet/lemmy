import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { CommentNode as CommentNodeI, CommentLikeForm, CommentForm as CommentFormI, BanFromCommunityForm, CommunityUser, AddModToCommunityForm } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { mdToHtml, getUnixTime } from '../utils';
import { MomentTime } from './moment-time';
import { CommentForm } from './comment-form';
import { CommentNodes } from './comment-nodes';

interface CommentNodeState {
  showReply: boolean;
  showEdit: boolean;
  showRemoveDialog: boolean;
  removeReason: string;
  showBanDialog: boolean;
  banReason: string;
  banExpires: string;
}

interface CommentNodeProps {
  node: CommentNodeI;
  noIndent?: boolean;
  viewOnly?: boolean;
  locked?: boolean;
  moderators: Array<CommunityUser>;
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
              <Link className="text-info" to={`/user/${node.comment.creator_id}`}>{node.comment.creator_name}</Link>
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
          {this.state.showEdit && <CommentForm node={node} edit onReplyCancel={this.handleReplyCancel} disabled={this.props.locked} />}
          {!this.state.showEdit &&
            <div>
              <div className="md-div" dangerouslySetInnerHTML={mdToHtml(node.comment.removed ? '*removed*' : node.comment.content)} />
              <ul class="list-inline mb-1 text-muted small font-weight-bold">
                {!this.props.viewOnly && 
                  <span class="mr-2">
                    <li className="list-inline-item">
                      <span class="pointer" onClick={linkEvent(this, this.handleReplyClick)}>reply</span>
                    </li>
                    {this.myComment && 
                      <>
                      <li className="list-inline-item">
                        <span class="pointer" onClick={linkEvent(this, this.handleEditClick)}>edit</span>
                      </li>
                      <li className="list-inline-item">
                        <span class="pointer" onClick={linkEvent(this, this.handleDeleteClick)}>delete</span>
                      </li>
                    </>
                    }
                    {this.canMod &&
                      <>
                      <li className="list-inline-item">
                        {!this.props.node.comment.removed ? 
                        <span class="pointer" onClick={linkEvent(this, this.handleModRemoveShow)}>remove</span> :
                        <span class="pointer" onClick={linkEvent(this, this.handleModRemoveSubmit)}>restore</span>
                        }
                      </li>
                      {!this.isMod &&
                        <>
                          <li className="list-inline-item">
                            {!this.props.node.comment.banned ? 
                            <span class="pointer" onClick={linkEvent(this, this.handleModBanShow)}>ban</span> :
                            <span class="pointer" onClick={linkEvent(this, this.handleModBanSubmit)}>unban</span>
                            }
                          </li>
                        </>
                      }
                      {!this.props.node.comment.banned &&
                        <li className="list-inline-item">
                          <span class="pointer" onClick={linkEvent(this, this.handleAddModToCommunity)}>{`${this.isMod ? 'remove' : 'appoint'} as mod`}</span>
                        </li>
                      }
                    </>
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
        {this.state.showRemoveDialog && 
          <form class="form-inline" onSubmit={linkEvent(this, this.handleModRemoveSubmit)}>
            <input type="text" class="form-control mr-2" placeholder="Reason" value={this.state.removeReason} onInput={linkEvent(this, this.handleModRemoveReasonChange)} />
            <button type="submit" class="btn btn-secondary">Remove Comment</button>
          </form>
        }
        {this.state.showBanDialog && 
        <form onSubmit={linkEvent(this, this.handleModBanSubmit)}>
          <div class="form-group row">
            <label class="col-form-label">Reason</label>
            <input type="text" class="form-control mr-2" placeholder="Optional" value={this.state.banReason} onInput={linkEvent(this, this.handleModBanReasonChange)} />
          </div>
          <div class="form-group row">
            <label class="col-form-label">Expires</label>
            <input type="date" class="form-control mr-2" placeholder="Expires" value={this.state.banExpires} onInput={linkEvent(this, this.handleModBanExpiresChange)} />
          </div>
          <div class="form-group row">
            <button type="submit" class="btn btn-secondary">Ban {this.props.node.comment.creator_name}</button>
          </div>
        </form>
        }
        {this.state.showReply && <CommentForm node={node} onReplyCancel={this.handleReplyCancel} disabled={this.props.locked} />}
        {this.props.node.children && <CommentNodes nodes={this.props.node.children} locked={this.props.locked} moderators={this.props.moderators}/>}
      </div>
    )
  }

  get myComment(): boolean {
    return UserService.Instance.loggedIn && this.props.node.comment.creator_id == UserService.Instance.user.id;
  }

  get canMod(): boolean {

    // You can do moderator actions only on the mods added after you.
    if (UserService.Instance.loggedIn) {
      let modIds = this.props.moderators.map(m => m.user_id);
      let yourIndex = modIds.findIndex(id => id == UserService.Instance.user.id);
      if (yourIndex == -1) {
        return false;
      } else { 
        console.log(modIds);
        modIds = modIds.slice(0, yourIndex+1); // +1 cause you cant mod yourself
        console.log(modIds);
        return !modIds.includes(this.props.node.comment.creator_id);
      }
    } else {
      return false;
    }

  }

  get isMod(): boolean {
    return this.props.moderators.map(m => m.user_id).includes(this.props.node.comment.creator_id);
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
      creator_id: i.props.node.comment.creator_id,
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

  handleModRemoveShow(i: CommentNode) {
    i.state.showRemoveDialog = true;
    i.setState(i.state);
  }

  handleModRemoveReasonChange(i: CommentNode, event: any) {
    i.state.removeReason = event.target.value;
    i.setState(i.state);
  }

  handleModRemoveSubmit(i: CommentNode) {
    let form: CommentFormI = {
      content: i.props.node.comment.content,
      edit_id: i.props.node.comment.id,
      creator_id: i.props.node.comment.creator_id,
      post_id: i.props.node.comment.post_id,
      parent_id: i.props.node.comment.parent_id,
      removed: !i.props.node.comment.removed,
      reason: i.state.removeReason,
      auth: null
    };
    WebSocketService.Instance.editComment(form);

    i.state.showRemoveDialog = false;
    i.setState(i.state);
  }

  handleModBanShow(i: CommentNode) {
    i.state.showBanDialog = true;
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

  handleModBanSubmit(i: CommentNode) {
    let form: BanFromCommunityForm = {
      user_id: i.props.node.comment.creator_id,
      community_id: i.props.node.comment.community_id,
      ban: !i.props.node.comment.banned,
      reason: i.state.banReason,
      expires: getUnixTime(i.state.banExpires),
    };
    WebSocketService.Instance.banFromCommunity(form);

    i.state.showBanDialog = false;
    i.setState(i.state);
  }

  handleAddModToCommunity(i: CommentNode) {
    let form: AddModToCommunityForm = {
      user_id: i.props.node.comment.creator_id,
      community_id: i.props.node.comment.community_id,
      added: !i.isMod,
    };
    WebSocketService.Instance.addModToCommunity(form);
    i.setState(i.state);
  }
}
