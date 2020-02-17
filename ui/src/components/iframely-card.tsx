import { Component, linkEvent } from 'inferno';
import { FramelyData } from '../interfaces';
import { mdToHtml } from '../utils';

interface FramelyCardProps {
  iframely: FramelyData;
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
    let iframely = this.props.iframely;
    return (
      <>
        {iframely.title && !this.state.expanded && (
          <div class="card mt-3 mb-2">
            <div class="row">
              <div class="col-12">
                <div class="card-body">
                  <h5 class="card-title d-inline">
                    <span>
                      <a class="text-body" target="_blank" href={iframely.url}>
                        {iframely.title}
                      </a>
                    </span>
                  </h5>
                  <span class="d-inline-block ml-2 mb-2 small text-muted">
                    <a
                      class="text-muted font-italic"
                      target="_blank"
                      href={iframely.url}
                    >
                      {new URL(iframely.url).hostname}
                      <svg class="ml-1 icon">
                        <use xlinkHref="#icon-external-link"></use>
                      </svg>
                    </a>
                    {iframely.html && (
                      <span
                        class="ml-2 pointer"
                        onClick={linkEvent(this, this.handleIframeExpand)}
                      >
                        {this.state.expanded ? '[-]' : '[+]'}
                      </span>
                    )}
                  </span>
                  {iframely.description && (
                    <div
                      className="card-text small text-muted md-div"
                      dangerouslySetInnerHTML={mdToHtml(iframely.description)}
                    />
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
        {this.state.expanded && (
          <div
            class="my-2"
            dangerouslySetInnerHTML={{ __html: iframely.html }}
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
