import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Post, SortType } from 'lemmy-js-client';
import { postSort } from '../utils';
import { PostListing } from './post-listing';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface PostListingsProps {
  posts: Array<Post>;
  showCommunity?: boolean;
  removeDuplicates?: boolean;
  sort?: SortType;
  enableDownvotes: boolean;
  enableNsfw: boolean;
}

export class PostListings extends Component<PostListingsProps, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <div>
        {this.props.posts.length > 0 ? (
          this.outer().map(post => (
            <>
              <PostListing
                post={post}
                showCommunity={this.props.showCommunity}
                enableDownvotes={this.props.enableDownvotes}
                enableNsfw={this.props.enableNsfw}
              />
              <hr class="my-3" />
            </>
          ))
        ) : (
          <>
            <div>{i18n.t('no_posts')}</div>
            {this.props.showCommunity !== undefined && (
              <T i18nKey="subscribe_to_communities">
                #<Link to="/communities">#</Link>
              </T>
            )}
          </>
        )}
      </div>
    );
  }

  outer(): Array<Post> {
    let out = this.props.posts;
    if (this.props.removeDuplicates) {
      out = this.removeDuplicates(out);
    }

    if (this.props.sort !== undefined) {
      postSort(out, this.props.sort, this.props.showCommunity == undefined);
    }

    return out;
  }

  removeDuplicates(posts: Array<Post>): Array<Post> {
    // A map from post url to list of posts (dupes)
    let urlMap = new Map<string, Array<Post>>();

    // Loop over the posts, find ones with same urls
    for (let post of posts) {
      if (
        post.url &&
        !post.deleted &&
        !post.removed &&
        !post.community_deleted &&
        !post.community_removed
      ) {
        if (!urlMap.get(post.url)) {
          urlMap.set(post.url, [post]);
        } else {
          urlMap.get(post.url).push(post);
        }
      }
    }

    // Sort by oldest
    // Remove the ones that have no length
    for (let e of urlMap.entries()) {
      if (e[1].length == 1) {
        urlMap.delete(e[0]);
      } else {
        e[1].sort((a, b) => a.published.localeCompare(b.published));
      }
    }

    for (let i = 0; i < posts.length; i++) {
      let post = posts[i];
      if (post.url) {
        let found = urlMap.get(post.url);
        if (found) {
          // If its the oldest, add
          if (post.id == found[0].id) {
            post.duplicates = found.slice(1);
          }
          // Otherwise, delete it
          else {
            posts.splice(i--, 1);
          }
        }
      }
    }

    return posts;
  }
}
