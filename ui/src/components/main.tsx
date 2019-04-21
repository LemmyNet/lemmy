import { Component } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from "rxjs";
import { retryWhen, delay, take } from 'rxjs/operators';
import { UserOperation, CommunityUser, GetFollowedCommunitiesResponse, ListCommunitiesForm, ListCommunitiesResponse, Community, SortType, GetSiteResponse, ListingType } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import { PostListings } from './post-listings';
import { msgOp, repoUrl, mdToHtml } from '../utils';


interface MainProps {
  type: ListingType;
}

interface MainState {
  subscribedCommunities: Array<CommunityUser>;
  trendingCommunities: Array<Community>;
  site: GetSiteResponse;
  loading: boolean;
}

export class Main extends Component<MainProps, MainState> {

  private subscription: Subscription;
  private emptyState: MainState = {
    subscribedCommunities: [],
    trendingCommunities: [],
    site: {
      op: null,
      site: {
        id: null,
        name: null,
        creator_id: null,
        creator_name: null,
        published: null,
        number_of_users: null,
        number_of_posts: null,
        number_of_comments: null,
      },
      admins: [],
      banned: [],
    },
    loading: true
  }

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.subscription = WebSocketService.Instance.subject
    .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
    .subscribe(
      (msg) => this.parseMessage(msg),
        (err) => console.error(err),
        () => console.log('complete')
    );

    WebSocketService.Instance.getSite();

    if (UserService.Instance.user) {
      WebSocketService.Instance.getFollowedCommunities();
    }

    let listCommunitiesForm: ListCommunitiesForm = {
      sort: SortType[SortType.New],
      limit: 6
    }

    WebSocketService.Instance.listCommunities(listCommunitiesForm);
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div class="container">
        <div class="row">
          <div class="col-12 col-md-8">
            <PostListings type={this.props.type} />
          </div>
          <div class="col-12 col-md-4">
            {this.state.loading ? 
            <h5><svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg></h5> : 
            <div>
              {this.trendingCommunities()}
              {UserService.Instance.user && this.state.subscribedCommunities.length > 0 && 
              <div>
                <h5>Subscribed forums</h5>
                <ul class="list-inline"> 
                  {this.state.subscribedCommunities.map(community =>
                    <li class="list-inline-item"><Link to={`/community/${community.community_id}`}>{community.community_name}</Link></li>
                  )}
                </ul>
              </div>
              }
              {this.landing()}
            </div>
            }
          </div>
        </div>
      </div>
    )
  }

  trendingCommunities() {
    return (
      <div>
        <h5>Trending <Link class="text-white" to="/communities">forums</Link></h5> 
        <ul class="list-inline"> 
          {this.state.trendingCommunities.map(community =>
            <li class="list-inline-item"><Link to={`/community/${community.id}`}>{community.name}</Link></li>
          )}
        </ul>
      </div>
    )
  }

  landing() {
    return (
      <div>
        <h5>{`${this.state.site.site.name}`}</h5>
        <ul class="my-1 list-inline">
          <li className="list-inline-item badge badge-light">{this.state.site.site.number_of_users} Users</li>
          <li className="list-inline-item badge badge-light">{this.state.site.site.number_of_posts} Posts</li>
          <li className="list-inline-item badge badge-light">{this.state.site.site.number_of_comments} Comments</li>
          <li className="list-inline-item"><Link className="badge badge-light" to="/modlog">Modlog</Link></li>
        </ul>
        <ul class="list-inline small"> 
          <li class="list-inline-item">admins: </li>
          {this.state.site.admins.map(admin =>
            <li class="list-inline-item"><Link class="text-info" to={`/user/${admin.id}`}>{admin.name}</Link></li>
          )}
        </ul>
        {this.state.site.site.description && 
          <div>
            <hr />
            <div className="md-div" dangerouslySetInnerHTML={mdToHtml(this.state.site.site.description)} />
            <hr />
          </div>
        }
        <h5>Welcome to 
          <svg class="icon mx-2"><use xlinkHref="#icon-mouse"></use></svg>
          <a href={repoUrl}>Lemmy<sup>Beta</sup></a>
        </h5>
        <p>Lemmy is a <a href="https://en.wikipedia.org/wiki/Link_aggregation">link aggregator</a> / reddit alternative, intended to work in the <a href="https://en.wikipedia.org/wiki/Fediverse">fediverse</a>.</p>
        <p>Its self-hostable, has live-updating comment threads, and is tiny (<code>~80kB</code>). Federation into the ActivityPub network is on the roadmap.</p>
        <p>This is a <b>very early beta version</b>, and a lot of features are currently broken or missing.</p>
        <p>Suggest new features or report bugs <a href={repoUrl}>here.</a></p>
        <p>Made with <a href="https://www.rust-lang.org">Rust</a>, <a href="https://actix.rs/">Actix</a>, <a href="https://www.infernojs.org">Inferno</a>, <a href="https://www.typescriptlang.org/">Typescript</a>.</p>
      </div>
    )
  }

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(msg.error);
      return;
    } else if (op == UserOperation.GetFollowedCommunities) {
      let res: GetFollowedCommunitiesResponse = msg;
      this.state.subscribedCommunities = res.communities;
      this.state.loading = false;
      this.setState(this.state);
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.trendingCommunities = res.communities;
      this.state.loading = false;
      this.setState(this.state);
    } else if (op == UserOperation.GetSite) {
      let res: GetSiteResponse = msg;

      // This means it hasn't been set up yet
      if (!res.site) {
        this.context.router.history.push("/setup");
      }
      this.state.site.admins = res.admins;
      this.state.site.site = res.site;
      this.state.site.banned = res.banned;
      this.setState(this.state);
    } 
  }
}

