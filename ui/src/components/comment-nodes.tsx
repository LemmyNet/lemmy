import { Component } from 'inferno';
import { CommentNode as CommentNodeI } from '../interfaces';
import { CommentNode } from './comment-node';

interface CommentNodesState {
}

interface CommentNodesProps {
  nodes: Array<CommentNodeI>;
  noIndent?: boolean;
  viewOnly?: boolean;
}

export class CommentNodes extends Component<CommentNodesProps, CommentNodesState> {

  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <div className="comments">
        {this.props.nodes.map(node =>
          <CommentNode node={node} noIndent={this.props.noIndent} viewOnly={this.props.viewOnly}/>
        )}
      </div>
    )
  }
}

