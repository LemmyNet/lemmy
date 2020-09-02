import { Component, linkEvent } from 'inferno';
import { Helmet } from 'inferno-helmet';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  CommunityUser,
  SortType,
  ListingType,
  UserView,
  UserSettingsForm,
  LoginResponse,
  DeleteAccountForm,
  WebSocketJsonResponse,
  GetSiteResponse,
  UserDetailsResponse,
  AddAdminResponse,
} from 'lemmy-js-client';
import { UserDetailsView } from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  wsJsonToRes,
  fetchLimit,
  routeSortTypeToEnum,
  capitalizeFirstLetter,
  themes,
  setTheme,
  languages,
  toast,
  setupTippy,
  getLanguage,
  mdToHtml,
  elementUrl,
  favIconUrl,
} from '../utils';
import { UserListing } from './user-listing';
import { SortSelect } from './sort-select';
import { ListingTypeSelect } from './listing-type-select';
import { MomentTime } from './moment-time';
import { i18n } from '../i18next';
import moment from 'moment';
import { UserDetails } from './user-details';
import { MarkdownTextArea } from './markdown-textarea';
import { ImageUploadForm } from './image-upload-form';
import { BannerIconHeader } from './banner-icon-header';

interface UserState {
  user: UserView;
  user_id: number;
  username: string;
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  view: UserDetailsView;
  sort: SortType;
  page: number;
  loading: boolean;
  userSettingsForm: UserSettingsForm;
  userSettingsLoading: boolean;
  deleteAccountLoading: boolean;
  deleteAccountShowConfirm: boolean;
  deleteAccountForm: DeleteAccountForm;
  siteRes: GetSiteResponse;
}

interface UserProps {
  view: UserDetailsView;
  sort: SortType;
  page: number;
  user_id: number | null;
  username: string;
}

interface UrlParams {
  view?: string;
  sort?: SortType;
  page?: number;
}

export class User extends Component<any, UserState> {
  private subscription: Subscription;
  private emptyState: UserState = {
    user: {
      id: null,
      name: null,
      published: null,
      number_of_posts: null,
      post_score: null,
      number_of_comments: null,
      comment_score: null,
      banned: null,
      avatar: null,
      actor_id: null,
      local: null,
    },
    user_id: null,
    username: null,
    follows: [],
    moderates: [],
    loading: true,
    view: User.getViewFromProps(this.props.match.view),
    sort: User.getSortTypeFromProps(this.props.match.sort),
    page: User.getPageFromProps(this.props.match.page),
    userSettingsForm: {
      show_nsfw: null,
      theme: null,
      default_sort_type: null,
      default_listing_type: null,
      lang: null,
      show_avatars: null,
      send_notifications_to_email: null,
      auth: null,
      bio: null,
      preferred_username: null,
    },
    userSettingsLoading: null,
    deleteAccountLoading: null,
    deleteAccountShowConfirm: false,
    deleteAccountForm: {
      password: null,
    },
    siteRes: {
      admins: [],
      banned: [],
      online: undefined,
      site: {
        id: undefined,
        name: undefined,
        creator_id: undefined,
        published: undefined,
        creator_name: undefined,
        number_of_users: undefined,
        number_of_posts: undefined,
        number_of_comments: undefined,
        number_of_communities: undefined,
        enable_downvotes: undefined,
        open_registration: undefined,
        enable_nsfw: undefined,
        icon: undefined,
        banner: undefined,
        creator_preferred_username: undefined,
      },
      version: undefined,
      my_user: undefined,
      federated_instances: undefined,
    },
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;
    this.handleSortChange = this.handleSortChange.bind(this);
    this.handleUserSettingsSortTypeChange = this.handleUserSettingsSortTypeChange.bind(
      this
    );
    this.handleUserSettingsListingTypeChange = this.handleUserSettingsListingTypeChange.bind(
      this
    );
    this.handlePageChange = this.handlePageChange.bind(this);
    this.handleUserSettingsBioChange = this.handleUserSettingsBioChange.bind(
      this
    );

    this.handleAvatarUpload = this.handleAvatarUpload.bind(this);
    this.handleAvatarRemove = this.handleAvatarRemove.bind(this);

    this.handleBannerUpload = this.handleBannerUpload.bind(this);
    this.handleBannerRemove = this.handleBannerRemove.bind(this);

    this.state.user_id = Number(this.props.match.params.id) || null;
    this.state.username = this.props.match.params.username;

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.getSite();
    setupTippy();
  }

  get isCurrentUser() {
    return (
      UserService.Instance.user &&
      UserService.Instance.user.id == this.state.user.id
    );
  }

  static getViewFromProps(view: string): UserDetailsView {
    return view ? UserDetailsView[view] : UserDetailsView.Overview;
  }

  static getSortTypeFromProps(sort: string): SortType {
    return sort ? routeSortTypeToEnum(sort) : SortType.New;
  }

  static getPageFromProps(page: number): number {
    return page ? Number(page) : 1;
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  static getDerivedStateFromProps(props: any): UserProps {
    return {
      view: this.getViewFromProps(props.match.params.view),
      sort: this.getSortTypeFromProps(props.match.params.sort),
      page: this.getPageFromProps(props.match.params.page),
      user_id: Number(props.match.params.id) || null,
      username: props.match.params.username,
    };
  }

  componentDidUpdate(lastProps: any, _lastState: UserState, _snapshot: any) {
    // Necessary if you are on a post and you click another post (same route)
    if (
      lastProps.location.pathname.split('/')[2] !==
      lastProps.history.location.pathname.split('/')[2]
    ) {
      // Couldnt get a refresh working. This does for now.
      location.reload();
    }
  }

  get documentTitle(): string {
    if (this.state.siteRes.site.name) {
      return `@${this.state.username} - ${this.state.siteRes.site.name}`;
    } else {
      return 'Lemmy';
    }
  }

  get favIcon(): string {
    return this.state.siteRes.site.icon
      ? this.state.siteRes.site.icon
      : favIconUrl;
  }

  render() {
    return (
      <div class="container">
        <Helmet title={this.documentTitle}>
          <link
            id="favicon"
            rel="icon"
            type="image/x-icon"
            href={this.favIcon}
          />
        </Helmet>
        <div class="row">
          <div class="col-12 col-md-8">
            {this.state.loading ? (
              <h5>
                <svg class="icon icon-spinner spin">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              </h5>
            ) : (
              <>
                {this.userInfo()}
                <hr />
              </>
            )}
            {!this.state.loading && this.selects()}
            <UserDetails
              user_id={this.state.user_id}
              username={this.state.username}
              sort={this.state.sort}
              page={this.state.page}
              limit={fetchLimit}
              enableDownvotes={this.state.siteRes.site.enable_downvotes}
              enableNsfw={this.state.siteRes.site.enable_nsfw}
              admins={this.state.siteRes.admins}
              view={this.state.view}
              onPageChange={this.handlePageChange}
            />
          </div>

          {!this.state.loading && (
            <div class="col-12 col-md-4">
              {this.isCurrentUser && this.userSettings()}
              {this.moderates()}
              {this.follows()}
            </div>
          )}
        </div>
      </div>
    );
  }

  viewRadios() {
    return (
      <div class="btn-group btn-group-toggle flex-wrap mb-2">
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.view == UserDetailsView.Overview && 'active'}
          `}
        >
          <input
            type="radio"
            value={UserDetailsView.Overview}
            checked={this.state.view === UserDetailsView.Overview}
            onChange={linkEvent(this, this.handleViewChange)}
          />
          {i18n.t('overview')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.view == UserDetailsView.Comments && 'active'}
          `}
        >
          <input
            type="radio"
            value={UserDetailsView.Comments}
            checked={this.state.view == UserDetailsView.Comments}
            onChange={linkEvent(this, this.handleViewChange)}
          />
          {i18n.t('comments')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.view == UserDetailsView.Posts && 'active'}
          `}
        >
          <input
            type="radio"
            value={UserDetailsView.Posts}
            checked={this.state.view == UserDetailsView.Posts}
            onChange={linkEvent(this, this.handleViewChange)}
          />
          {i18n.t('posts')}
        </label>
        <label
          className={`btn btn-outline-secondary pointer
            ${this.state.view == UserDetailsView.Saved && 'active'}
          `}
        >
          <input
            type="radio"
            value={UserDetailsView.Saved}
            checked={this.state.view == UserDetailsView.Saved}
            onChange={linkEvent(this, this.handleViewChange)}
          />
          {i18n.t('saved')}
        </label>
      </div>
    );
  }

  selects() {
    return (
      <div className="mb-2">
        <span class="mr-3">{this.viewRadios()}</span>
        <SortSelect
          sort={this.state.sort}
          onChange={this.handleSortChange}
          hideHot
        />
        <a
          href={`/feeds/u/${this.state.username}.xml?sort=${this.state.sort}`}
          target="_blank"
          rel="noopener"
          title="RSS"
        >
          <svg class="icon mx-2 text-muted small">
            <use xlinkHref="#icon-rss">#</use>
          </svg>
        </a>
      </div>
    );
  }

  userInfo() {
    let user = this.state.user;

    return (
      <div>
        <BannerIconHeader
          banner={this.state.user.banner}
          icon={this.state.user.avatar}
        />
        <div class="mb-3">
          <div class="">
            <div class="mb-0 d-flex flex-wrap">
              <div>
                {user.preferred_username && (
                  <h5 class="mb-0">{user.preferred_username}</h5>
                )}
                <ul class="list-inline mb-2">
                  <li className="list-inline-item">
                    <UserListing
                      user={user}
                      realLink
                      useApubName
                      muted
                      hideAvatar
                    />
                  </li>
                  {user.banned && (
                    <li className="list-inline-item badge badge-danger">
                      {i18n.t('banned')}
                    </li>
                  )}
                </ul>
              </div>
              <div className="flex-grow-1 unselectable pointer mx-2"></div>
              {this.isCurrentUser ? (
                <button
                  class="d-flex align-self-start btn btn-secondary ml-2"
                  onClick={linkEvent(this, this.handleLogoutClick)}
                >
                  {i18n.t('logout')}
                </button>
              ) : (
                <>
                  <a
                    className={`d-flex align-self-start btn btn-secondary ml-2 ${
                      !this.state.user.matrix_user_id && 'invisible'
                    }`}
                    target="_blank"
                    rel="noopener"
                    href={`https://matrix.to/#/${this.state.user.matrix_user_id}`}
                  >
                    {i18n.t('send_secure_message')}
                  </a>
                  <Link
                    class="d-flex align-self-start btn btn-secondary ml-2"
                    to={`/create_private_message?recipient_id=${this.state.user.id}`}
                  >
                    {i18n.t('send_message')}
                  </Link>
                </>
              )}
            </div>
            {user.bio && (
              <div className="d-flex align-items-center mb-2">
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(user.bio)}
                />
              </div>
            )}
            <div>
              <ul class="list-inline mb-2">
                <li className="list-inline-item badge badge-light">
                  {i18n.t('number_of_posts', { count: user.number_of_posts })}
                </li>
                <li className="list-inline-item badge badge-light">
                  {i18n.t('number_of_comments', {
                    count: user.number_of_comments,
                  })}
                </li>
              </ul>
            </div>
            <div class="text-muted">
              {i18n.t('joined')} <MomentTime data={user} showAgo />
            </div>
            <div className="d-flex align-items-center text-muted mb-2">
              <svg class="icon">
                <use xlinkHref="#icon-cake"></use>
              </svg>
              <span className="ml-2">
                {i18n.t('cake_day_title')}{' '}
                {moment.utc(user.published).local().format('MMM DD, YYYY')}
              </span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  userSettings() {
    return (
      <div>
        <div class="card bg-transparent border-secondary mb-3">
          <div class="card-body">
            <h5>{i18n.t('settings')}</h5>
            <form onSubmit={linkEvent(this, this.handleUserSettingsSubmit)}>
              <div class="form-group">
                <label>{i18n.t('avatar')}</label>
                <ImageUploadForm
                  uploadTitle={i18n.t('upload_avatar')}
                  imageSrc={this.state.userSettingsForm.avatar}
                  onUpload={this.handleAvatarUpload}
                  onRemove={this.handleAvatarRemove}
                  rounded
                />
              </div>
              <div class="form-group">
                <label>{i18n.t('banner')}</label>
                <ImageUploadForm
                  uploadTitle={i18n.t('upload_banner')}
                  imageSrc={this.state.userSettingsForm.banner}
                  onUpload={this.handleBannerUpload}
                  onRemove={this.handleBannerRemove}
                />
              </div>
              <div class="form-group">
                <label>{i18n.t('language')}</label>
                <select
                  value={this.state.userSettingsForm.lang}
                  onChange={linkEvent(this, this.handleUserSettingsLangChange)}
                  class="ml-2 custom-select w-auto"
                >
                  <option disabled>{i18n.t('language')}</option>
                  <option value="browser">{i18n.t('browser_default')}</option>
                  <option disabled>──</option>
                  {languages.map(lang => (
                    <option value={lang.code}>{lang.name}</option>
                  ))}
                </select>
              </div>
              <div class="form-group">
                <label>{i18n.t('theme')}</label>
                <select
                  value={this.state.userSettingsForm.theme}
                  onChange={linkEvent(this, this.handleUserSettingsThemeChange)}
                  class="ml-2 custom-select w-auto"
                >
                  <option disabled>{i18n.t('theme')}</option>
                  {themes.map(theme => (
                    <option value={theme}>{theme}</option>
                  ))}
                </select>
              </div>
              <form className="form-group">
                <label>
                  <div class="mr-2">{i18n.t('sort_type')}</div>
                </label>
                <ListingTypeSelect
                  type_={
                    Object.values(ListingType)[
                      this.state.userSettingsForm.default_listing_type
                    ]
                  }
                  showLocal={
                    this.state.siteRes.federated_instances &&
                    this.state.siteRes.federated_instances.length > 0
                  }
                  onChange={this.handleUserSettingsListingTypeChange}
                />
              </form>
              <form className="form-group">
                <label>
                  <div class="mr-2">{i18n.t('type')}</div>
                </label>
                <SortSelect
                  sort={
                    Object.values(SortType)[
                      this.state.userSettingsForm.default_sort_type
                    ]
                  }
                  onChange={this.handleUserSettingsSortTypeChange}
                />
              </form>
              <div class="form-group row">
                <label class="col-lg-5 col-form-label">
                  {i18n.t('display_name')}
                </label>
                <div class="col-lg-7">
                  <input
                    type="text"
                    class="form-control"
                    placeholder={i18n.t('optional')}
                    value={this.state.userSettingsForm.preferred_username}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsPreferredUsernameChange
                    )}
                    pattern="^(?!@)(.+)$"
                    minLength={3}
                    maxLength={20}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-3 col-form-label" htmlFor="user-bio">
                  {i18n.t('bio')}
                </label>
                <div class="col-lg-9">
                  <MarkdownTextArea
                    initialContent={this.state.userSettingsForm.bio}
                    onContentChange={this.handleUserSettingsBioChange}
                    maxLength={300}
                    hideNavigationWarnings
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-3 col-form-label" htmlFor="user-email">
                  {i18n.t('email')}
                </label>
                <div class="col-lg-9">
                  <input
                    type="email"
                    id="user-email"
                    class="form-control"
                    placeholder={i18n.t('optional')}
                    value={this.state.userSettingsForm.email}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsEmailChange
                    )}
                    minLength={3}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-5 col-form-label">
                  <a href={elementUrl} target="_blank" rel="noopener">
                    {i18n.t('matrix_user_id')}
                  </a>
                </label>
                <div class="col-lg-7">
                  <input
                    type="text"
                    class="form-control"
                    placeholder="@user:example.com"
                    value={this.state.userSettingsForm.matrix_user_id}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsMatrixUserIdChange
                    )}
                    minLength={3}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-5 col-form-label" htmlFor="user-password">
                  {i18n.t('new_password')}
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    id="user-password"
                    class="form-control"
                    value={this.state.userSettingsForm.new_password}
                    autoComplete="new-password"
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsNewPasswordChange
                    )}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label
                  class="col-lg-5 col-form-label"
                  htmlFor="user-verify-password"
                >
                  {i18n.t('verify_password')}
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    id="user-verify-password"
                    class="form-control"
                    value={this.state.userSettingsForm.new_password_verify}
                    autoComplete="new-password"
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsNewPasswordVerifyChange
                    )}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label
                  class="col-lg-5 col-form-label"
                  htmlFor="user-old-password"
                >
                  {i18n.t('old_password')}
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    id="user-old-password"
                    class="form-control"
                    value={this.state.userSettingsForm.old_password}
                    autoComplete="new-password"
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsOldPasswordChange
                    )}
                  />
                </div>
              </div>
              {this.state.siteRes.site.enable_nsfw && (
                <div class="form-group">
                  <div class="form-check">
                    <input
                      class="form-check-input"
                      id="user-show-nsfw"
                      type="checkbox"
                      checked={this.state.userSettingsForm.show_nsfw}
                      onChange={linkEvent(
                        this,
                        this.handleUserSettingsShowNsfwChange
                      )}
                    />
                    <label class="form-check-label" htmlFor="user-show-nsfw">
                      {i18n.t('show_nsfw')}
                    </label>
                  </div>
                </div>
              )}
              <div class="form-group">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    id="user-show-avatars"
                    type="checkbox"
                    checked={this.state.userSettingsForm.show_avatars}
                    onChange={linkEvent(
                      this,
                      this.handleUserSettingsShowAvatarsChange
                    )}
                  />
                  <label class="form-check-label" htmlFor="user-show-avatars">
                    {i18n.t('show_avatars')}
                  </label>
                </div>
              </div>
              <div class="form-group">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    id="user-send-notifications-to-email"
                    type="checkbox"
                    disabled={!this.state.userSettingsForm.email}
                    checked={
                      this.state.userSettingsForm.send_notifications_to_email
                    }
                    onChange={linkEvent(
                      this,
                      this.handleUserSettingsSendNotificationsToEmailChange
                    )}
                  />
                  <label
                    class="form-check-label"
                    htmlFor="user-send-notifications-to-email"
                  >
                    {i18n.t('send_notifications_to_email')}
                  </label>
                </div>
              </div>
              <div class="form-group">
                <button type="submit" class="btn btn-block btn-secondary mr-4">
                  {this.state.userSettingsLoading ? (
                    <svg class="icon icon-spinner spin">
                      <use xlinkHref="#icon-spinner"></use>
                    </svg>
                  ) : (
                    capitalizeFirstLetter(i18n.t('save'))
                  )}
                </button>
              </div>
              <hr />
              <div class="form-group mb-0">
                <button
                  class="btn btn-block btn-danger"
                  onClick={linkEvent(
                    this,
                    this.handleDeleteAccountShowConfirmToggle
                  )}
                >
                  {i18n.t('delete_account')}
                </button>
                {this.state.deleteAccountShowConfirm && (
                  <>
                    <div class="my-2 alert alert-danger" role="alert">
                      {i18n.t('delete_account_confirm')}
                    </div>
                    <input
                      type="password"
                      value={this.state.deleteAccountForm.password}
                      autoComplete="new-password"
                      onInput={linkEvent(
                        this,
                        this.handleDeleteAccountPasswordChange
                      )}
                      class="form-control my-2"
                    />
                    <button
                      class="btn btn-danger mr-4"
                      disabled={!this.state.deleteAccountForm.password}
                      onClick={linkEvent(this, this.handleDeleteAccount)}
                    >
                      {this.state.deleteAccountLoading ? (
                        <svg class="icon icon-spinner spin">
                          <use xlinkHref="#icon-spinner"></use>
                        </svg>
                      ) : (
                        capitalizeFirstLetter(i18n.t('delete'))
                      )}
                    </button>
                    <button
                      class="btn btn-secondary"
                      onClick={linkEvent(
                        this,
                        this.handleDeleteAccountShowConfirmToggle
                      )}
                    >
                      {i18n.t('cancel')}
                    </button>
                  </>
                )}
              </div>
            </form>
          </div>
        </div>
      </div>
    );
  }

  moderates() {
    return (
      <div>
        {this.state.moderates.length > 0 && (
          <div class="card bg-transparent border-secondary mb-3">
            <div class="card-body">
              <h5>{i18n.t('moderates')}</h5>
              <ul class="list-unstyled mb-0">
                {this.state.moderates.map(community => (
                  <li>
                    <Link to={`/c/${community.community_name}`}>
                      {community.community_name}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </div>
    );
  }

  follows() {
    return (
      <div>
        {this.state.follows.length > 0 && (
          <div class="card bg-transparent border-secondary mb-3">
            <div class="card-body">
              <h5>{i18n.t('subscribed')}</h5>
              <ul class="list-unstyled mb-0">
                {this.state.follows.map(community => (
                  <li>
                    <Link to={`/c/${community.community_name}`}>
                      {community.community_name}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </div>
    );
  }

  updateUrl(paramUpdates: UrlParams) {
    const page = paramUpdates.page || this.state.page;
    const viewStr = paramUpdates.view || UserDetailsView[this.state.view];
    const sortStr = paramUpdates.sort || this.state.sort;
    this.props.history.push(
      `/u/${this.state.username}/view/${viewStr}/sort/${sortStr}/page/${page}`
    );
  }

  handlePageChange(page: number) {
    this.updateUrl({ page });
  }

  handleSortChange(val: SortType) {
    this.updateUrl({ sort: val, page: 1 });
  }

  handleViewChange(i: User, event: any) {
    i.updateUrl({
      view: UserDetailsView[Number(event.target.value)],
      page: 1,
    });
  }

  handleUserSettingsShowNsfwChange(i: User, event: any) {
    i.state.userSettingsForm.show_nsfw = event.target.checked;
    i.setState(i.state);
  }

  handleUserSettingsShowAvatarsChange(i: User, event: any) {
    i.state.userSettingsForm.show_avatars = event.target.checked;
    UserService.Instance.user.show_avatars = event.target.checked; // Just for instant updates
    i.setState(i.state);
  }

  handleUserSettingsSendNotificationsToEmailChange(i: User, event: any) {
    i.state.userSettingsForm.send_notifications_to_email = event.target.checked;
    i.setState(i.state);
  }

  handleUserSettingsThemeChange(i: User, event: any) {
    i.state.userSettingsForm.theme = event.target.value;
    setTheme(event.target.value, true);
    i.setState(i.state);
  }

  handleUserSettingsLangChange(i: User, event: any) {
    i.state.userSettingsForm.lang = event.target.value;
    i18n.changeLanguage(getLanguage(i.state.userSettingsForm.lang));
    i.setState(i.state);
  }

  handleUserSettingsSortTypeChange(val: SortType) {
    this.state.userSettingsForm.default_sort_type = Object.keys(
      SortType
    ).indexOf(val);
    this.setState(this.state);
  }

  handleUserSettingsListingTypeChange(val: ListingType) {
    this.state.userSettingsForm.default_listing_type = Object.keys(
      ListingType
    ).indexOf(val);
    this.setState(this.state);
  }

  handleUserSettingsEmailChange(i: User, event: any) {
    i.state.userSettingsForm.email = event.target.value;
    i.setState(i.state);
  }

  handleUserSettingsBioChange(val: string) {
    this.state.userSettingsForm.bio = val;
    this.setState(this.state);
  }

  handleAvatarUpload(url: string) {
    this.state.userSettingsForm.avatar = url;
    this.setState(this.state);
  }

  handleAvatarRemove() {
    this.state.userSettingsForm.avatar = '';
    this.setState(this.state);
  }

  handleBannerUpload(url: string) {
    this.state.userSettingsForm.banner = url;
    this.setState(this.state);
  }

  handleBannerRemove() {
    this.state.userSettingsForm.banner = '';
    this.setState(this.state);
  }

  handleUserSettingsPreferredUsernameChange(i: User, event: any) {
    i.state.userSettingsForm.preferred_username = event.target.value;
    i.setState(i.state);
  }

  handleUserSettingsMatrixUserIdChange(i: User, event: any) {
    i.state.userSettingsForm.matrix_user_id = event.target.value;
    if (
      i.state.userSettingsForm.matrix_user_id == '' &&
      !i.state.user.matrix_user_id
    ) {
      i.state.userSettingsForm.matrix_user_id = undefined;
    }
    i.setState(i.state);
  }

  handleUserSettingsNewPasswordChange(i: User, event: any) {
    i.state.userSettingsForm.new_password = event.target.value;
    if (i.state.userSettingsForm.new_password == '') {
      i.state.userSettingsForm.new_password = undefined;
    }
    i.setState(i.state);
  }

  handleUserSettingsNewPasswordVerifyChange(i: User, event: any) {
    i.state.userSettingsForm.new_password_verify = event.target.value;
    if (i.state.userSettingsForm.new_password_verify == '') {
      i.state.userSettingsForm.new_password_verify = undefined;
    }
    i.setState(i.state);
  }

  handleUserSettingsOldPasswordChange(i: User, event: any) {
    i.state.userSettingsForm.old_password = event.target.value;
    if (i.state.userSettingsForm.old_password == '') {
      i.state.userSettingsForm.old_password = undefined;
    }
    i.setState(i.state);
  }

  handleUserSettingsSubmit(i: User, event: any) {
    event.preventDefault();
    i.state.userSettingsLoading = true;
    i.setState(i.state);

    WebSocketService.Instance.saveUserSettings(i.state.userSettingsForm);
  }

  handleDeleteAccountShowConfirmToggle(i: User, event: any) {
    event.preventDefault();
    i.state.deleteAccountShowConfirm = !i.state.deleteAccountShowConfirm;
    i.setState(i.state);
  }

  handleDeleteAccountPasswordChange(i: User, event: any) {
    i.state.deleteAccountForm.password = event.target.value;
    i.setState(i.state);
  }

  handleLogoutClick(i: User) {
    UserService.Instance.logout();
    i.context.router.history.push('/');
  }

  handleDeleteAccount(i: User, event: any) {
    event.preventDefault();
    i.state.deleteAccountLoading = true;
    i.setState(i.state);

    WebSocketService.Instance.deleteAccount(i.state.deleteAccountForm);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    console.log(msg);
    const res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      if (msg.error == 'couldnt_find_that_username_or_email') {
        this.context.router.history.push('/');
      }
      this.setState({
        deleteAccountLoading: false,
        userSettingsLoading: false,
      });
      return;
    } else if (res.op == UserOperation.GetUserDetails) {
      // Since the UserDetails contains posts/comments as well as some general user info we listen here as well
      // and set the parent state if it is not set or differs
      const data = res.data as UserDetailsResponse;

      if (this.state.user.id !== data.user.id) {
        this.state.user = data.user;
        this.state.follows = data.follows;
        this.state.moderates = data.moderates;

        if (this.isCurrentUser) {
          this.state.userSettingsForm.show_nsfw =
            UserService.Instance.user.show_nsfw;
          this.state.userSettingsForm.theme = UserService.Instance.user.theme
            ? UserService.Instance.user.theme
            : 'darkly';
          this.state.userSettingsForm.default_sort_type =
            UserService.Instance.user.default_sort_type;
          this.state.userSettingsForm.default_listing_type =
            UserService.Instance.user.default_listing_type;
          this.state.userSettingsForm.lang = UserService.Instance.user.lang;
          this.state.userSettingsForm.avatar = UserService.Instance.user.avatar;
          this.state.userSettingsForm.banner = UserService.Instance.user.banner;
          this.state.userSettingsForm.preferred_username =
            UserService.Instance.user.preferred_username;
          this.state.userSettingsForm.show_avatars =
            UserService.Instance.user.show_avatars;
          this.state.userSettingsForm.email = UserService.Instance.user.email;
          this.state.userSettingsForm.bio = UserService.Instance.user.bio;
          this.state.userSettingsForm.send_notifications_to_email =
            UserService.Instance.user.send_notifications_to_email;
          this.state.userSettingsForm.matrix_user_id =
            UserService.Instance.user.matrix_user_id;
        }
        this.state.loading = false;
        this.setState(this.state);
      }
    } else if (res.op == UserOperation.SaveUserSettings) {
      const data = res.data as LoginResponse;
      UserService.Instance.login(data);
      this.state.user.bio = this.state.userSettingsForm.bio;
      this.state.user.preferred_username = this.state.userSettingsForm.preferred_username;
      this.state.user.banner = this.state.userSettingsForm.banner;
      this.state.user.avatar = this.state.userSettingsForm.avatar;
      this.state.userSettingsLoading = false;
      this.setState(this.state);

      window.scrollTo(0, 0);
    } else if (res.op == UserOperation.DeleteAccount) {
      this.setState({
        deleteAccountLoading: false,
        deleteAccountShowConfirm: false,
      });
      this.context.router.history.push('/');
    } else if (res.op == UserOperation.GetSite) {
      const data = res.data as GetSiteResponse;
      this.state.siteRes = data;
      this.setState(this.state);
    } else if (res.op == UserOperation.AddAdmin) {
      const data = res.data as AddAdminResponse;
      this.state.siteRes.admins = data.admins;
      this.setState(this.state);
    }
  }
}
