import { Component } from 'inferno';
import * as moment from 'moment';

interface MomentTimeProps {
  data: {
    published: string;
    updated?: string;
  }
}

export class MomentTime extends Component<MomentTimeProps, any> {

  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    if (this.props.data.updated) {
      return (
        <span title={this.props.data.updated} className="font-italics">modified {moment.utc(this.props.data.updated).fromNow()}</span>
      )
    } else {
      return (
        <span title={this.props.data.published}>{moment.utc(this.props.data.published).fromNow()}</span>
      )
    }
  }
}
