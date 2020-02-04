import { Component } from 'inferno';
import {
  CommentNode as CommentNodeI,
  CommunityUser,
  UserView,
} from '../interfaces';
import { CommentNode } from './comment-node';

interface CommentNodesState {}

interface CommentNodesProps {
  nodes: Array<CommentNodeI>;
  moderators?: Array<CommunityUser>;
  admins?: Array<UserView>;
  postCreatorId?: number;
  noIndent?: boolean;
  viewOnly?: boolean;
  locked?: boolean;
  markable?: boolean;
}

export class CommentNodes extends Component<
  CommentNodesProps,
  CommentNodesState
> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <div className="comments">
        {this.props.nodes.map(node => (
          <CommentNode
            node={node}
            noIndent={this.props.noIndent}
            viewOnly={this.props.viewOnly}
            locked={this.props.locked}
            moderators={this.props.moderators}
            admins={this.props.admins}
            postCreatorId={this.props.postCreatorId}
            markable={this.props.markable}
          />
        ))}
      </div>
    );
  }
}
