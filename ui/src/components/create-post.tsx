import { Component } from 'inferno';
import { PostForm } from './post-form';
import { WebSocketService } from '../services';
import { PostFormParams } from '../interfaces';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

export class CreatePost extends Component<any, any> {
  constructor(props: any, context: any) {
    super(props, context);
    this.handlePostCreate = this.handlePostCreate.bind(this);
  }

  componentDidMount() {
    document.title = `${i18n.t('create_post')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 offset-lg-3 mb-4">
            <h5>
              <T i18nKey="create_post">#</T>
            </h5>
            <PostForm onCreate={this.handlePostCreate} params={this.params} />
          </div>
        </div>
      </div>
    );
  }

  get params(): PostFormParams {
    let urlParams = new URLSearchParams(this.props.location.search);
    let params: PostFormParams = {
      name: urlParams.get('name'),
      community: urlParams.get('community') || this.prevCommunityName,
      body: urlParams.get('body'),
      url: urlParams.get('url'),
    };

    return params;
  }

  get prevCommunityName(): string {
    if (this.props.match.params.name) {
      return this.props.match.params.name;
    } else if (this.props.location.state) {
      let lastLocation = this.props.location.state.prevPath;
      if (lastLocation.includes('/c/')) {
        return lastLocation.split('/c/')[1];
      }
    }
    return undefined;
  }

  handlePostCreate(id: number) {
    this.props.history.push(`/post/${id}`);
  }
}
