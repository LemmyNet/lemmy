import { Component } from 'inferno';
import {
  CommentNode as CommentNodeI,
  CommunityUser,
  UserView,
  CommentSortType,
  SortType,
} from '../interfaces';
import { commentSort, commentSortSortType } from '../utils';
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
  showCommunity?: boolean;
  sort?: CommentSortType;
  sortType?: SortType;
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
        {this.sorter().map(node => (
          <CommentNode
            node={node}
            noIndent={this.props.noIndent}
            viewOnly={this.props.viewOnly}
            locked={this.props.locked}
            moderators={this.props.moderators}
            admins={this.props.admins}
            postCreatorId={this.props.postCreatorId}
            markable={this.props.markable}
            showCommunity={this.props.showCommunity}
            sort={this.props.sort}
            sortType={this.props.sortType}
          />
        ))}
      </div>
    );
  }

  sorter(): Array<CommentNodeI> {
    if (this.props.sort !== undefined) {
      commentSort(this.props.nodes, this.props.sort);
    } else if (this.props.sortType !== undefined) {
      commentSortSortType(this.props.nodes, this.props.sortType);
    }

    return this.props.nodes;
  }
}
