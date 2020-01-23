import { Component } from 'inferno';
import { PrivateMessageForm } from './private-message-form';
import { WebSocketService } from '../services';
import { PrivateMessageFormParams } from '../interfaces';
import { toast } from '../utils';
import { i18n } from '../i18next';

export class CreatePrivateMessage extends Component<any, any> {
  constructor(props: any, context: any) {
    super(props, context);
    this.handlePrivateMessageCreate = this.handlePrivateMessageCreate.bind(
      this
    );
  }

  componentDidMount() {
    document.title = `${i18n.t('create_private_message')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 offset-lg-3 mb-4">
            <h5>{i18n.t('create_private_message')}</h5>
            <PrivateMessageForm
              onCreate={this.handlePrivateMessageCreate}
              params={this.params}
            />
          </div>
        </div>
      </div>
    );
  }

  get params(): PrivateMessageFormParams {
    let urlParams = new URLSearchParams(this.props.location.search);
    let params: PrivateMessageFormParams = {
      recipient_id: Number(urlParams.get('recipient_id')),
    };

    return params;
  }

  handlePrivateMessageCreate() {
    toast(i18n.t('message_sent'));

    // Navigate to the front
    this.props.history.push(`/`);
  }
}
