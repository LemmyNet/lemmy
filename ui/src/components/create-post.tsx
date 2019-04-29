import { Component } from 'inferno';
import { PostForm } from './post-form';

export class CreatePost extends Component<any, any> {

  constructor(props: any, context: any) {
    super(props, context);
    this.handlePostCreate = this.handlePostCreate.bind(this);
  }

  componentDidMount() {
    document.title = "Create Post - Lemmy";
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-lg-6 mb-4">
            <h5>Create a Post</h5>
            <PostForm onCreate={this.handlePostCreate} prevCommunityName={this.prevCommunityName} />
          </div>
        </div>
      </div>
    )
  }

  get prevCommunityName(): string {
    if (this.props.location.state) {
      let lastLocation = this.props.location.state.prevPath;
      if (lastLocation.includes("/c/")) {
        return lastLocation.split("/c/")[1];
      }    
    }
    return undefined;
  }

  handlePostCreate(id: number) {
    this.props.history.push(`/post/${id}`);
  }
}


