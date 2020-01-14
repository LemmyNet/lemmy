import { Component, linkEvent } from 'inferno';
import { Link } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  UserOperation,
  Post,
  Comment,
  CommunityUser,
  GetUserDetailsForm,
  SortType,
  ListingType,
  UserDetailsResponse,
  UserView,
  CommentResponse,
  UserSettingsForm,
  LoginResponse,
  BanUserResponse,
  AddAdminResponse,
  DeleteAccountForm,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  msgOp,
  fetchLimit,
  routeSortTypeToEnum,
  capitalizeFirstLetter,
  themes,
  setTheme,
  languages,
  showAvatars,
} from '../utils';
import { PostListing } from './post-listing';
import { SortSelect } from './sort-select';
import { ListingTypeSelect } from './listing-type-select';
import { CommentNodes } from './comment-nodes';
import { MomentTime } from './moment-time';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

enum View {
  Overview,
  Comments,
  Posts,
  Saved,
}

interface UserState {
  user: UserView;
  user_id: number;
  username: string;
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  comments: Array<Comment>;
  posts: Array<Post>;
  saved?: Array<Post>;
  admins: Array<UserView>;
  view: View;
  sort: SortType;
  page: number;
  loading: boolean;
  avatarLoading: boolean;
  userSettingsForm: UserSettingsForm;
  userSettingsLoading: boolean;
  deleteAccountLoading: boolean;
  deleteAccountShowConfirm: boolean;
  deleteAccountForm: DeleteAccountForm;
}

export class User extends Component<any, UserState> {
  private subscription: Subscription;
  private emptyState: UserState = {
    user: {
      id: null,
      name: null,
      fedi_name: null,
      published: null,
      number_of_posts: null,
      post_score: null,
      number_of_comments: null,
      comment_score: null,
      banned: null,
      avatar: null,
      show_avatars: null,
      send_notifications_to_email: null,
    },
    user_id: null,
    username: null,
    follows: [],
    moderates: [],
    comments: [],
    posts: [],
    admins: [],
    loading: true,
    avatarLoading: false,
    view: this.getViewFromProps(this.props),
    sort: this.getSortTypeFromProps(this.props),
    page: this.getPageFromProps(this.props),
    userSettingsForm: {
      show_nsfw: null,
      theme: null,
      default_sort_type: null,
      default_listing_type: null,
      lang: null,
      show_avatars: null,
      send_notifications_to_email: null,
      auth: null,
    },
    userSettingsLoading: null,
    deleteAccountLoading: null,
    deleteAccountShowConfirm: false,
    deleteAccountForm: {
      password: null,
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

    this.state.user_id = Number(this.props.match.params.id);
    this.state.username = this.props.match.params.username;

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

  get isCurrentUser() {
    return (
      UserService.Instance.user &&
      UserService.Instance.user.id == this.state.user.id
    );
  }

  getViewFromProps(props: any): View {
    return props.match.params.view
      ? View[capitalizeFirstLetter(props.match.params.view)]
      : View.Overview;
  }

  getSortTypeFromProps(props: any): SortType {
    return props.match.params.sort
      ? routeSortTypeToEnum(props.match.params.sort)
      : SortType.New;
  }

  getPageFromProps(props: any): number {
    return props.match.params.page ? Number(props.match.params.page) : 1;
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  // Necessary for back button for some reason
  componentWillReceiveProps(nextProps: any) {
    if (
      nextProps.history.action == 'POP' ||
      nextProps.history.action == 'PUSH'
    ) {
      this.state.view = this.getViewFromProps(nextProps);
      this.state.sort = this.getSortTypeFromProps(nextProps);
      this.state.page = this.getPageFromProps(nextProps);
      this.setState(this.state);
      this.refetch();
    }
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

  render() {
    return (
      <div class="container">
        {this.state.loading ? (
          <h5>
            <svg class="icon icon-spinner spin">
              <use xlinkHref="#icon-spinner"></use>
            </svg>
          </h5>
        ) : (
          <div class="row">
            <div class="col-12 col-md-8">
              <h5>
                {this.state.user.avatar && showAvatars() && (
                  <img
                    height="80"
                    width="80"
                    src={this.state.user.avatar}
                    class="rounded-circle mr-2"
                  />
                )}
                <span>/u/{this.state.user.name}</span>
              </h5>
              {this.selects()}
              {this.state.view == View.Overview && this.overview()}
              {this.state.view == View.Comments && this.comments()}
              {this.state.view == View.Posts && this.posts()}
              {this.state.view == View.Saved && this.overview()}
              {this.paginator()}
            </div>
            <div class="col-12 col-md-4">
              {this.userInfo()}
              {this.isCurrentUser && this.userSettings()}
              {this.moderates()}
              {this.follows()}
            </div>
          </div>
        )}
      </div>
    );
  }

  selects() {
    return (
      <div className="mb-2">
        <select
          value={this.state.view}
          onChange={linkEvent(this, this.handleViewChange)}
          class="custom-select custom-select-sm w-auto"
        >
          <option disabled>
            <T i18nKey="view">#</T>
          </option>
          <option value={View.Overview}>
            <T i18nKey="overview">#</T>
          </option>
          <option value={View.Comments}>
            <T i18nKey="comments">#</T>
          </option>
          <option value={View.Posts}>
            <T i18nKey="posts">#</T>
          </option>
          <option value={View.Saved}>
            <T i18nKey="saved">#</T>
          </option>
        </select>
        <span class="ml-2">
          <SortSelect
            sort={this.state.sort}
            onChange={this.handleSortChange}
            hideHot
          />
        </span>
        <a
          href={`/feeds/u/${this.state.username}.xml?sort=${
            SortType[this.state.sort]
          }`}
          target="_blank"
        >
          <svg class="icon mx-2 text-muted small">
            <use xlinkHref="#icon-rss">#</use>
          </svg>
        </a>
      </div>
    );
  }

  overview() {
    let combined: Array<{ type_: string; data: Comment | Post }> = [];
    let comments = this.state.comments.map(e => {
      return { type_: 'comments', data: e };
    });
    let posts = this.state.posts.map(e => {
      return { type_: 'posts', data: e };
    });

    combined.push(...comments);
    combined.push(...posts);

    // Sort it
    if (this.state.sort == SortType.New) {
      combined.sort((a, b) => b.data.published.localeCompare(a.data.published));
    } else {
      combined.sort((a, b) => b.data.score - a.data.score);
    }

    return (
      <div>
        {combined.map(i => (
          <div>
            {i.type_ == 'posts' ? (
              <PostListing
                post={i.data as Post}
                admins={this.state.admins}
                showCommunity
                viewOnly
              />
            ) : (
              <CommentNodes
                nodes={[{ comment: i.data as Comment }]}
                admins={this.state.admins}
                noIndent
              />
            )}
          </div>
        ))}
      </div>
    );
  }

  comments() {
    return (
      <div>
        {this.state.comments.map(comment => (
          <CommentNodes
            nodes={[{ comment: comment }]}
            admins={this.state.admins}
            noIndent
          />
        ))}
      </div>
    );
  }

  posts() {
    return (
      <div>
        {this.state.posts.map(post => (
          <PostListing
            post={post}
            admins={this.state.admins}
            showCommunity
            viewOnly
          />
        ))}
      </div>
    );
  }

  userInfo() {
    let user = this.state.user;
    return (
      <div>
        <div class="card border-secondary mb-3">
          <div class="card-body">
            <h5>
              <ul class="list-inline mb-0">
                <li className="list-inline-item">{user.name}</li>
                {user.banned && (
                  <li className="list-inline-item badge badge-danger">
                    <T i18nKey="banned">#</T>
                  </li>
                )}
              </ul>
            </h5>
            <div>
              {i18n.t('joined')} <MomentTime data={user} />
            </div>
            <div class="table-responsive">
              <table class="table table-bordered table-sm mt-2 mb-0">
                <tr>
                  <td>
                    <T
                      i18nKey="number_of_points"
                      interpolation={{ count: user.post_score }}
                    >
                      #
                    </T>
                  </td>
                  <td>
                    <T
                      i18nKey="number_of_posts"
                      interpolation={{ count: user.number_of_posts }}
                    >
                      #
                    </T>
                  </td>
                </tr>
                <tr>
                  <td>
                    <T
                      i18nKey="number_of_points"
                      interpolation={{ count: user.comment_score }}
                    >
                      #
                    </T>
                  </td>
                  <td>
                    <T
                      i18nKey="number_of_comments"
                      interpolation={{ count: user.number_of_comments }}
                    >
                      #
                    </T>
                  </td>
                </tr>
              </table>
            </div>
            {this.isCurrentUser && (
              <button
                class="btn btn-block btn-secondary mt-3"
                onClick={linkEvent(this, this.handleLogoutClick)}
              >
                <T i18nKey="logout">#</T>
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }

  userSettings() {
    return (
      <div>
        <div class="card border-secondary mb-3">
          <div class="card-body">
            <h5>
              <T i18nKey="settings">#</T>
            </h5>
            <form onSubmit={linkEvent(this, this.handleUserSettingsSubmit)}>
              <div class="form-group">
                <label>
                  <T i18nKey="avatar">#</T>
                </label>
                <form class="d-inline">
                  <label
                    htmlFor="file-upload"
                    class="pointer ml-4 text-muted small font-weight-bold"
                  >
                    <img
                      height="80"
                      width="80"
                      src={
                        this.state.userSettingsForm.avatar
                          ? this.state.userSettingsForm.avatar
                          : 'https://via.placeholder.com/300/000?text=Avatar'
                      }
                      class="rounded-circle"
                    />
                  </label>
                  <input
                    id="file-upload"
                    type="file"
                    accept="image/*,video/*"
                    name="file"
                    class="d-none"
                    disabled={!UserService.Instance.user}
                    onChange={linkEvent(this, this.handleImageUpload)}
                  />
                </form>
              </div>
              <div class="form-group">
                <label>
                  <T i18nKey="language">#</T>
                </label>
                <select
                  value={this.state.userSettingsForm.lang}
                  onChange={linkEvent(this, this.handleUserSettingsLangChange)}
                  class="ml-2 custom-select custom-select-sm w-auto"
                >
                  <option disabled>
                    <T i18nKey="language">#</T>
                  </option>
                  <option value="browser">
                    <T i18nKey="browser_default">#</T>
                  </option>
                  <option disabled>──</option>
                  {languages.map(lang => (
                    <option value={lang.code}>{lang.name}</option>
                  ))}
                </select>
              </div>
              <div class="form-group">
                <label>
                  <T i18nKey="theme">#</T>
                </label>
                <select
                  value={this.state.userSettingsForm.theme}
                  onChange={linkEvent(this, this.handleUserSettingsThemeChange)}
                  class="ml-2 custom-select custom-select-sm w-auto"
                >
                  <option disabled>
                    <T i18nKey="theme">#</T>
                  </option>
                  {themes.map(theme => (
                    <option value={theme}>{theme}</option>
                  ))}
                </select>
              </div>
              <form className="form-group">
                <label>
                  <T i18nKey="sort_type" class="mr-2">
                    #
                  </T>
                </label>
                <ListingTypeSelect
                  type_={this.state.userSettingsForm.default_listing_type}
                  onChange={this.handleUserSettingsListingTypeChange}
                />
              </form>
              <form className="form-group">
                <label>
                  <T i18nKey="type" class="mr-2">
                    #
                  </T>
                </label>
                <SortSelect
                  sort={this.state.userSettingsForm.default_sort_type}
                  onChange={this.handleUserSettingsSortTypeChange}
                />
              </form>
              <div class="form-group row">
                <label class="col-lg-3 col-form-label">
                  <T i18nKey="email">#</T>
                </label>
                <div class="col-lg-9">
                  <input
                    type="email"
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
                  <T i18nKey="new_password">#</T>
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    class="form-control"
                    value={this.state.userSettingsForm.new_password}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsNewPasswordChange
                    )}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-5 col-form-label">
                  <T i18nKey="verify_password">#</T>
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    class="form-control"
                    value={this.state.userSettingsForm.new_password_verify}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsNewPasswordVerifyChange
                    )}
                  />
                </div>
              </div>
              <div class="form-group row">
                <label class="col-lg-5 col-form-label">
                  <T i18nKey="old_password">#</T>
                </label>
                <div class="col-lg-7">
                  <input
                    type="password"
                    class="form-control"
                    value={this.state.userSettingsForm.old_password}
                    onInput={linkEvent(
                      this,
                      this.handleUserSettingsOldPasswordChange
                    )}
                  />
                </div>
              </div>
              {WebSocketService.Instance.site.enable_nsfw && (
                <div class="form-group">
                  <div class="form-check">
                    <input
                      class="form-check-input"
                      type="checkbox"
                      checked={this.state.userSettingsForm.show_nsfw}
                      onChange={linkEvent(
                        this,
                        this.handleUserSettingsShowNsfwChange
                      )}
                    />
                    <label class="form-check-label">
                      <T i18nKey="show_nsfw">#</T>
                    </label>
                  </div>
                </div>
              )}
              <div class="form-group">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    checked={this.state.userSettingsForm.show_avatars}
                    onChange={linkEvent(
                      this,
                      this.handleUserSettingsShowAvatarsChange
                    )}
                  />
                  <label class="form-check-label">
                    <T i18nKey="show_avatars">#</T>
                  </label>
                </div>
              </div>
              <div class="form-group">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    disabled={!this.state.user.email}
                    checked={
                      this.state.userSettingsForm.send_notifications_to_email
                    }
                    onChange={linkEvent(
                      this,
                      this.handleUserSettingsSendNotificationsToEmailChange
                    )}
                  />
                  <label class="form-check-label">
                    <T i18nKey="send_notifications_to_email">#</T>
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
                  <T i18nKey="delete_account">#</T>
                </button>
                {this.state.deleteAccountShowConfirm && (
                  <>
                    <div class="my-2 alert alert-danger" role="alert">
                      <T i18nKey="delete_account_confirm">#</T>
                    </div>
                    <input
                      type="password"
                      value={this.state.deleteAccountForm.password}
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
                      <T i18nKey="cancel">#</T>
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
          <div class="card border-secondary mb-3">
            <div class="card-body">
              <h5>
                <T i18nKey="moderates">#</T>
              </h5>
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
          <div class="card border-secondary mb-3">
            <div class="card-body">
              <h5>
                <T i18nKey="subscribed">#</T>
              </h5>
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

  paginator() {
    return (
      <div class="my-2">
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
    let viewStr = View[this.state.view].toLowerCase();
    let sortStr = SortType[this.state.sort].toLowerCase();
    this.props.history.push(
      `/u/${this.state.user.name}/view/${viewStr}/sort/${sortStr}/page/${this.state.page}`
    );
  }

  nextPage(i: User) {
    i.state.page++;
    i.setState(i.state);
    i.updateUrl();
    i.refetch();
  }

  prevPage(i: User) {
    i.state.page--;
    i.setState(i.state);
    i.updateUrl();
    i.refetch();
  }

  refetch() {
    let form: GetUserDetailsForm = {
      user_id: this.state.user_id,
      username: this.state.username,
      sort: SortType[this.state.sort],
      saved_only: this.state.view == View.Saved,
      page: this.state.page,
      limit: fetchLimit,
    };
    WebSocketService.Instance.getUserDetails(form);
  }

  handleSortChange(val: SortType) {
    this.state.sort = val;
    this.state.page = 1;
    this.setState(this.state);
    this.updateUrl();
    this.refetch();
  }

  handleViewChange(i: User, event: any) {
    i.state.view = Number(event.target.value);
    i.state.page = 1;
    i.setState(i.state);
    i.updateUrl();
    i.refetch();
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
    setTheme(event.target.value);
    i.setState(i.state);
  }

  handleUserSettingsLangChange(i: User, event: any) {
    i.state.userSettingsForm.lang = event.target.value;
    i18n.changeLanguage(i.state.userSettingsForm.lang);
    i.setState(i.state);
  }

  handleUserSettingsSortTypeChange(val: SortType) {
    this.state.userSettingsForm.default_sort_type = val;
    this.setState(this.state);
  }

  handleUserSettingsListingTypeChange(val: ListingType) {
    this.state.userSettingsForm.default_listing_type = val;
    this.setState(this.state);
  }

  handleUserSettingsEmailChange(i: User, event: any) {
    i.state.userSettingsForm.email = event.target.value;
    if (i.state.userSettingsForm.email == '' && !i.state.user.email) {
      i.state.userSettingsForm.email = undefined;
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

  handleImageUpload(i: User, event: any) {
    event.preventDefault();
    let file = event.target.files[0];
    const imageUploadUrl = `/pictshare/api/upload.php`;
    const formData = new FormData();
    formData.append('file', file);

    i.state.avatarLoading = true;
    i.setState(i.state);

    fetch(imageUploadUrl, {
      method: 'POST',
      body: formData,
    })
      .then(res => res.json())
      .then(res => {
        let url = `${window.location.origin}/pictshare/${res.url}`;
        if (res.filetype == 'mp4') {
          url += '/raw';
        }
        i.state.userSettingsForm.avatar = url;
        console.log(url);
        i.state.avatarLoading = false;
        i.setState(i.state);
      })
      .catch(error => {
        i.state.avatarLoading = false;
        i.setState(i.state);
        alert(error);
      });
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

  parseMessage(msg: any) {
    console.log(msg);
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.state.deleteAccountLoading = false;
      this.state.avatarLoading = false;
      this.state.userSettingsLoading = false;
      if (msg.error == 'couldnt_find_that_username_or_email') {
        this.context.router.history.push('/');
      }
      this.setState(this.state);
      return;
    } else if (op == UserOperation.GetUserDetails) {
      let res: UserDetailsResponse = msg;
      this.state.user = res.user;
      this.state.comments = res.comments;
      this.state.follows = res.follows;
      this.state.moderates = res.moderates;
      this.state.posts = res.posts;
      this.state.admins = res.admins;
      this.state.loading = false;
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
        this.state.userSettingsForm.email = this.state.user.email;
        this.state.userSettingsForm.send_notifications_to_email = this.state.user.send_notifications_to_email;
        this.state.userSettingsForm.show_avatars =
          UserService.Instance.user.show_avatars;
      }
      document.title = `/u/${this.state.user.name} - ${WebSocketService.Instance.site.name}`;
      window.scrollTo(0, 0);
      this.setState(this.state);
    } else if (op == UserOperation.EditComment) {
      let res: CommentResponse = msg;

      let found = this.state.comments.find(c => c.id == res.comment.id);
      found.content = res.comment.content;
      found.updated = res.comment.updated;
      found.removed = res.comment.removed;
      found.deleted = res.comment.deleted;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      found.score = res.comment.score;

      this.setState(this.state);
    } else if (op == UserOperation.CreateComment) {
      // let res: CommentResponse = msg;
      alert(i18n.t('reply_sent'));
      // this.state.comments.unshift(res.comment); // TODO do this right
      // this.setState(this.state);
    } else if (op == UserOperation.SaveComment) {
      let res: CommentResponse = msg;
      let found = this.state.comments.find(c => c.id == res.comment.id);
      found.saved = res.comment.saved;
      this.setState(this.state);
    } else if (op == UserOperation.CreateCommentLike) {
      let res: CommentResponse = msg;
      let found: Comment = this.state.comments.find(
        c => c.id === res.comment.id
      );
      found.score = res.comment.score;
      found.upvotes = res.comment.upvotes;
      found.downvotes = res.comment.downvotes;
      if (res.comment.my_vote !== null) found.my_vote = res.comment.my_vote;
      this.setState(this.state);
    } else if (op == UserOperation.BanUser) {
      let res: BanUserResponse = msg;
      this.state.comments
        .filter(c => c.creator_id == res.user.id)
        .forEach(c => (c.banned = res.banned));
      this.state.posts
        .filter(c => c.creator_id == res.user.id)
        .forEach(c => (c.banned = res.banned));
      this.setState(this.state);
    } else if (op == UserOperation.AddAdmin) {
      let res: AddAdminResponse = msg;
      this.state.admins = res.admins;
      this.setState(this.state);
    } else if (op == UserOperation.SaveUserSettings) {
      this.state = this.emptyState;
      this.state.userSettingsLoading = false;
      this.setState(this.state);
      let res: LoginResponse = msg;
      UserService.Instance.login(res);
    } else if (op == UserOperation.DeleteAccount) {
      this.state.deleteAccountLoading = false;
      this.state.deleteAccountShowConfirm = false;
      this.setState(this.state);
      this.context.router.history.push('/');
    }
  }
}
