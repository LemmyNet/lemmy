import { Component, linkEvent } from 'inferno';

import { LoginForm, PostForm, UserOperation } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { msgOp } from '../utils';

interface State {
  postForm: PostForm;
}

let emptyState: State = {
  postForm: {
    name: null,
    url: null,
    attributed_to: null
  }
}

export class CreatePost extends Component<any, State> {

  constructor(props, context) {
    super(props, context);

    this.state = emptyState;

    WebSocketService.Instance.subject.subscribe(
      (msg) => this.parseMessage(msg),
      (err) => console.error(err),
      () => console.log('complete')
    );
  }


  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            create post
            {/* {this.postForm()} */}
          </div>
        </div>
      </div>
    )
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else {
    }
  }

}
