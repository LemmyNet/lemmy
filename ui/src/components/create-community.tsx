import { Component, linkEvent } from 'inferno';
import { CommunityForm } from './community-form';

export class CreateCommunity extends Component<any, any> {

  constructor(props, context) {
    super(props, context);
    this.handleCommunityCreate = this.handleCommunityCreate.bind(this);
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            <h4>Create Forum</h4>
            <CommunityForm onCreate={this.handleCommunityCreate}/>
          </div>
        </div>
      </div>
    )
  }

  handleCommunityCreate(id: number) {
    this.props.history.push(`/community/${id}`);
  }
}


