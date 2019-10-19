import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Community,
  ListCommunitiesResponse,
  CommunityResponse,
  FollowCommunityForm,
  ListCommunitiesForm,
  SortType,
} from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp } from '../utils';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

declare const Sortable: any;

interface CommunitiesState {
  communities: Array<Community>;
  page: number;
  loading: boolean;
}

export class Communities extends Component<any, CommunitiesState> {
  private subscription: Subscription;
  private emptyState: CommunitiesState = {
    communities: [],
    loading: true,
    page: this.getPageFromProps(this.props),
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    this.subscription = WebSocketService.Instance.subject
      .pipe(
        retryWhen(errors =>
          errors.pipe(
            delay(3000),
            take(10)
          )
        )
      )
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    this.refetch();
  }

  getPageFromProps(props: any): number {
    return props.match.params.page ? Number(props.match.params.page) : 1;
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  componentDidMount() {
    document.title = `${i18n.t('communities')} - ${
      WebSocketService.Instance.site.name
    }`;
  }

  // Necessary for back button for some reason
  componentWillReceiveProps(nextProps: any) {
    if (nextProps.history.action == 'POP') {
      this.state = this.emptyState;
      this.state.page = this.getPageFromProps(nextProps);
      this.refetch();
    }
  }

  render() {
    return (
      <div class="container">
        {this.state.loading ? (
          <h5 class="">
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div>
            <h5>
              <T i18nKey="list_of_communities">#</T>
            </h5>
            <div class="table-responsive">
              <table id="community_table" class="table table-sm table-hover">
                <thead class="pointer">
                  <tr>
                    <th>
                      <T i18nKey="name">#</T>
                    </th>
                    <th class="d-none d-lg-table-cell">
                      <T i18nKey="title">#</T>
                    </th>
                    <th>
                      <T i18nKey="category">#</T>
                    </th>
                    <th class="text-right">
                      <T i18nKey="subscribers">#</T>
                    </th>
                    <th class="text-right d-none d-lg-table-cell">
                      <T i18nKey="posts">#</T>
                    </th>
                    <th class="text-right d-none d-lg-table-cell">
                      <T i18nKey="comments">#</T>
                    </th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {this.state.communities.map(community => (
                    <tr>
                      <td>
                        <Link to={`/c/${community.name}`}>
                          {community.name}
                        </Link>
                      </td>
                      <td class="d-none d-lg-table-cell">{community.title}</td>
                      <td>{community.category_name}</td>
                      <td class="text-right">
                        {community.number_of_subscribers}
                      </td>
                      <td class="text-right d-none d-lg-table-cell">
                        {community.number_of_posts}
                      </td>
                      <td class="text-right d-none d-lg-table-cell">
                        {community.number_of_comments}
                      </td>
                      <td class="text-right">
                        {community.subscribed ? (
                          <span
                            class="pointer btn-link"
                            onClick={linkEvent(
                              community.id,
                              this.handleUnsubscribe
                            )}
                          >
                            <T i18nKey="unsubscribe">#</T>
                          </span>
                        ) : (
                          <span
                            class="pointer btn-link"
                            onClick={linkEvent(
                              community.id,
                              this.handleSubscribe
                            )}
                          >
                            <T i18nKey="subscribe">#</T>
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {this.paginator()}
          </div>
        )}
      </div>
    );
  }

  paginator() {
    return (
      <div class="mt-2">
        {this.state.page > 1 && (
          <button
            class="btn btn-sm btn-secondary mr-1"
            onClick={linkEvent(this, this.prevPage)}
          >
            <T i18nKey="prev">#</T>
          </button>
        )}
        <button
          class="btn btn-sm btn-secondary"
          onClick={linkEvent(this, this.nextPage)}
        >
          <T i18nKey="next">#</T>
        </button>
      </div>
    );
  }

  updateUrl() {
    this.props.history.push(`/communities/page/${this.state.page}`);
  }

  nextPage(i: Communities) {
    i.state.page++;
    i.setState(i.state);
    i.updateUrl();
    i.refetch();
  }

  prevPage(i: Communities) {
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.refetch();
  }

  handleUnsubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: false,
    };
    WebSocketService.Instance.followCommunity(form);
  }

  handleSubscribe(communityId: number) {
    let form: FollowCommunityForm = {
      community_id: communityId,
      follow: true,
    };
    WebSocketService.Instance.followCommunity(form);
  }

  refetch() {
    let listCommunitiesForm: ListCommunitiesForm = {
      sort: SortType[SortType.TopAll],
      limit: 100,
      page: this.state.page,
    };

    WebSocketService.Instance.listCommunities(listCommunitiesForm);
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      return;
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.communities = res.communities;
      this.state.communities.sort(
        (a, b) => b.number_of_subscribers - a.number_of_subscribers
      );
      this.state.loading = false;
      window.scrollTo(0, 0);
      this.setState(this.state);
      let table = document.querySelector('#community_table');
      Sortable.initTable(table);
    } else if (op == UserOperation.FollowCommunity) {
      let res: CommunityResponse = msg;
      let found = this.state.communities.find(c => c.id == res.community.id);
      found.subscribed = res.community.subscribed;
      found.number_of_subscribers = res.community.number_of_subscribers;
      this.setState(this.state);
    }
  }
}
