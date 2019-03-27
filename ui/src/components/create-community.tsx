import { Component, linkEvent } from 'inferno';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { CommunityForm, UserOperation } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

import { Community } from '../interfaces';

interface State {
  communityForm: CommunityForm;
}

export class CreateCommunity extends Component<any, State> {
  private subscription: Subscription;

  private emptyState: State = {
    communityForm: {
      name: null,
    }
  }

  constructor(props, context) {
    super(props, context);

    this.state = this.emptyState;
    
    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log("complete")
      );
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            {this.communityForm()}
          </div>
        </div>
      </div>
    )
  }

  communityForm() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handleCreateCommunitySubmit)}>
          <h3>Create Forum</h3>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">Name</label>
            <div class="col-sm-10">
              <input type="text" class="form-control" value={this.state.communityForm.name} onInput={linkEvent(this, this.handleCommunityNameChange)} required minLength={3} />
            </div>
          </div>
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary">Create</button>
            </div>
          </div>
        </form>
      </div>
    );
  }
  
  handleCreateCommunitySubmit(i: CreateCommunity, event) {
    event.preventDefault();
    WebSocketService.Instance.createCommunity(i.state.communityForm);
  }

  handleCommunityNameChange(i: CreateCommunity, event) {
    i.state.communityForm.name = event.target.value;
  }

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    console.log(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else {
      if (op == UserOperation.CreateCommunity) {
        let community: Community = msg.community;
        this.props.history.push(`/community/${community.id}`);
      }
    }
  }

}
