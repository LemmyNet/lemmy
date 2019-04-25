import { Component } from 'inferno';
import { CommunityForm } from './community-form';
import { Community } from '../interfaces';

export class CreateCommunity extends Component<any, any> {

  constructor(props: any, context: any) {
    super(props, context);
    this.handleCommunityCreate = this.handleCommunityCreate.bind(this);
  }

  componentDidMount() {
    document.title = "Create Forum - Lemmy";
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            <h5>Create Forum</h5>
            <CommunityForm onCreate={this.handleCommunityCreate}/>
          </div>
        </div>
      </div>
    )
  }

  handleCommunityCreate(community: Community) {
    this.props.history.push(`/f/${community.name}`);
  }
}


