import { Component, linkEvent } from 'inferno';
import * as moment from 'moment';

interface MomentTimeProps {
  data: {
    published: string;
    updated?: string;
  }
}

export class MomentTime extends Component<MomentTimeProps, any> {

  constructor(props, context) {
    super(props, context);
  }

  render() {
    if (this.props.data.updated) {
      return (
        <span className="font-italics">modified {moment.utc(this.props.data.updated).fromNow()}</span>
      )
    } else {
      return (
        <span>{moment.utc(this.props.data.published).fromNow()}</span>
      )
    }
  }
}
