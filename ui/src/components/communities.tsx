import { Component, linkEvent } from 'inferno';
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
  WebSocketJsonResponse,
  GetSiteResponse,
} from '../interfaces';
import { WebSocketService } from '../services';
import { wsJsonToRes, toast } from '../utils';
import { CommunityLink } from './community-link';
import { i18n } from '../i18next';

declare const Sortable: any;

const communityLimit = 100;

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
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    this.refetch();
    WebSocketService.Instance.getSite();
  }

  getPageFromProps(props: any): number {
    return props.match.params.page ? Number(props.match.params.page) : 1;
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
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
            <h5>{i18n.t('list_of_communities')}</h5>
            <div class="table-responsive">
              <table id="community_table" class="table table-sm table-hover">
                <thead class="pointer">
                  <tr>
                    <th>{i18n.t('name')}</th>
                    <th class="d-none d-lg-table-cell">{i18n.t('title')}</th>
                    <th>{i18n.t('category')}</th>
                    <th class="text-right">{i18n.t('subscribers')}</th>
                    <th class="text-right d-none d-lg-table-cell">
                      {i18n.t('posts')}
                    </th>
                    <th class="text-right d-none d-lg-table-cell">
                      {i18n.t('comments')}
                    </th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {this.state.communities.map(community => (
                    <tr>
                      <td>
                        <CommunityLink community={community} />
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
                            {i18n.t('unsubscribe')}
                          </span>
                        ) : (
                          <span
                            class="pointer btn-link"
                            onClick={linkEvent(
                              community.id,
                              this.handleSubscribe
                            )}
                          >
                            {i18n.t('subscribe')}
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
            {i18n.t('prev')}
          </button>
        )}

        {this.state.communities.length > 0 && (
          <button
            class="btn btn-sm btn-secondary"
            onClick={linkEvent(this, this.nextPage)}
          >
            {i18n.t('next')}
          </button>
        )}
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
      limit: communityLimit,
      page: this.state.page,
    };

    WebSocketService.Instance.listCommunities(listCommunitiesForm);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      return;
    } else if (res.op == UserOperation.ListCommunities) {
      let data = res.data as ListCommunitiesResponse;
      this.state.communities = data.communities;
      this.state.communities.sort(
        (a, b) => b.number_of_subscribers - a.number_of_subscribers
      );
      this.state.loading = false;
      window.scrollTo(0, 0);
      this.setState(this.state);
      let table = document.querySelector('#community_table');
      Sortable.initTable(table);
    } else if (res.op == UserOperation.FollowCommunity) {
      let data = res.data as CommunityResponse;
      let found = this.state.communities.find(c => c.id == data.community.id);
      found.subscribed = data.community.subscribed;
      found.number_of_subscribers = data.community.number_of_subscribers;
      this.setState(this.state);
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      document.title = `${i18n.t('communities')} - ${data.site.name}`;
    }
  }
}
