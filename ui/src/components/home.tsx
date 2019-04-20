import { Component } from 'inferno';
import { Main } from './main';
import { ListingType } from '../interfaces';

export class Home extends Component<any, any> {

  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <Main type={this.listType()}/>
    )
  }

  listType(): ListingType { 
    return (this.props.match.path == '/all') ? ListingType.All : ListingType.Subscribed;
  }
}
