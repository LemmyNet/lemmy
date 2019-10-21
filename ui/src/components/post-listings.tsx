import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Post } from '../interfaces';
import { PostListing } from './post-listing';
import { T } from 'inferno-i18next';

interface PostListingsProps {
  posts: Array<Post>;
  showCommunity?: boolean;
}

export class PostListings extends Component<PostListingsProps, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <div>
        {this.props.posts.length > 0 ? (
          this.props.posts.map(post => (
            <>
              <PostListing
                post={post}
                showCommunity={this.props.showCommunity}
              />
              <hr class="d-md-none my-2" />
              <div class="d-none d-md-block my-2"></div>
            </>
          ))
        ) : (
          <>
            <div>
              <T i18nKey="no_posts">#</T>
            </div>
            {this.props.showCommunity !== undefined && (
              <div>
                <T i18nKey="subscribe_to_communities">
                  #<Link to="/communities">#</Link>
                </T>
              </div>
            )}
          </>
        )}
      </div>
    );
  }
}
