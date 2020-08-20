import { Component, linkEvent } from 'inferno';
import { Post } from 'lemmy-js-client';
import { mdToHtml } from '../utils';
import { i18n } from '../i18next';

interface FramelyCardProps {
  post: Post;
}

interface FramelyCardState {
  expanded: boolean;
}

export class IFramelyCard extends Component<
  FramelyCardProps,
  FramelyCardState
> {
  private emptyState: FramelyCardState = {
    expanded: false,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
  }

  render() {
    let post = this.props.post;
    return (
      <>
        {post.embed_title && !this.state.expanded && (
          <div class="card bg-transparent border-secondary mt-3 mb-2">
            <div class="row">
              <div class="col-12">
                <div class="card-body">
                  <h5 class="card-title d-inline">
                    {post.embed_html ? (
                      <span
                        class="unselectable pointer"
                        onClick={linkEvent(this, this.handleIframeExpand)}
                        data-tippy-content={i18n.t('expand_here')}
                      >
                        {post.embed_title}
                      </span>
                    ) : (
                      <span>
                        <a
                          class="text-body"
                          target="_blank"
                          href={post.url}
                          rel="noopener"
                        >
                          {post.embed_title}
                        </a>
                      </span>
                    )}
                  </h5>
                  <span class="d-inline-block ml-2 mb-2 small text-muted">
                    <a
                      class="text-muted font-italic"
                      target="_blank"
                      href={post.url}
                      rel="noopener"
                    >
                      {new URL(post.url).hostname}
                      <svg class="ml-1 icon">
                        <use xlinkHref="#icon-external-link"></use>
                      </svg>
                    </a>
                    {post.embed_html && (
                      <span
                        class="ml-2 pointer text-monospace"
                        onClick={linkEvent(this, this.handleIframeExpand)}
                        data-tippy-content={i18n.t('expand_here')}
                      >
                        {this.state.expanded ? '[-]' : '[+]'}
                      </span>
                    )}
                  </span>
                  {post.embed_description && (
                    <div
                      className="card-text small text-muted md-div"
                      dangerouslySetInnerHTML={mdToHtml(post.embed_description)}
                    />
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
        {this.state.expanded && (
          <div
            class="mt-3 mb-2"
            dangerouslySetInnerHTML={{ __html: post.embed_html }}
          />
        )}
      </>
    );
  }

  handleIframeExpand(i: IFramelyCard) {
    i.state.expanded = !i.state.expanded;
    i.setState(i.state);
  }
}
