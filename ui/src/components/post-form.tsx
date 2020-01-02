import { Component, linkEvent } from 'inferno';
import { PostListings } from './post-listings';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  PostForm as PostFormI,
  PostFormParams,
  Post,
  PostResponse,
  UserOperation,
  Community,
  ListCommunitiesResponse,
  ListCommunitiesForm,
  SortType,
  SearchForm,
  SearchType,
  SearchResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  msgOp,
  getPageTitle,
  validURL,
  capitalizeFirstLetter,
  markdownHelpUrl,
  archiveUrl,
  mdToHtml,
  debounce,
  isImage,
} from '../utils';
import * as autosize from 'autosize';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface PostFormProps {
  post?: Post; // If a post is given, that means this is an edit
  params?: PostFormParams;
  onCancel?(): any;
  onCreate?(id: number): any;
  onEdit?(post: Post): any;
}

interface PostFormState {
  postForm: PostFormI;
  communities: Array<Community>;
  loading: boolean;
  imageLoading: boolean;
  previewMode: boolean;
  suggestedTitle: string;
  suggestedPosts: Array<Post>;
  crossPosts: Array<Post>;
}

export class PostForm extends Component<PostFormProps, PostFormState> {
  private subscription: Subscription;
  private emptyState: PostFormState = {
    postForm: {
      name: null,
      nsfw: false,
      auth: null,
      community_id: null,
      creator_id: UserService.Instance.user
        ? UserService.Instance.user.id
        : null,
    },
    communities: [],
    loading: false,
    imageLoading: false,
    previewMode: false,
    suggestedTitle: undefined,
    suggestedPosts: [],
    crossPosts: [],
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.fetchSimilarPosts = debounce(this.fetchSimilarPosts).bind(this);
    this.fetchPageTitle = debounce(this.fetchPageTitle).bind(this);

    this.state = this.emptyState;

    if (this.props.post) {
      this.state.postForm = {
        body: this.props.post.body,
        // NOTE: debouncing breaks both these for some reason, unless you use defaultValue
        name: this.props.post.name,
        community_id: this.props.post.community_id,
        edit_id: this.props.post.id,
        creator_id: this.props.post.creator_id,
        url: this.props.post.url,
        nsfw: this.props.post.nsfw,
        auth: null,
      };
    }

    if (this.props.params) {
      this.state.postForm.name = this.props.params.name;
      if (this.props.params.url) {
        this.state.postForm.url = this.props.params.url;
      }
      if (this.props.params.body) {
        this.state.postForm.body = this.props.params.body;
      }
    }

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

    let listCommunitiesForm: ListCommunitiesForm = {
      sort: SortType[SortType.TopAll],
      limit: 9999,
    };

    WebSocketService.Instance.listCommunities(listCommunitiesForm);
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div>
        <form onSubmit={linkEvent(this, this.handlePostSubmit)}>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">
              <T i18nKey="url">#</T>
            </label>
            <div class="col-sm-10">
              <input
                type="url"
                class="form-control"
                value={this.state.postForm.url}
                onInput={linkEvent(this, this.handlePostUrlChange)}
              />
              {this.state.suggestedTitle && (
                <div
                  class="mt-1 text-muted small font-weight-bold pointer"
                  onClick={linkEvent(this, this.copySuggestedTitle)}
                >
                  <T
                    i18nKey="copy_suggested_title"
                    interpolation={{ title: this.state.suggestedTitle }}
                  >
                    #
                  </T>
                </div>
              )}
              <form>
                <label
                  htmlFor="file-upload"
                  className={`${UserService.Instance.user &&
                    'pointer'} d-inline-block mr-2 float-right text-muted small font-weight-bold`}
                >
                  <T i18nKey="upload_image">#</T>
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
              {validURL(this.state.postForm.url) && (
                <a
                  href={`${archiveUrl}/?run=1&url=${encodeURIComponent(
                    this.state.postForm.url
                  )}`}
                  target="_blank"
                  class="mr-2 d-inline-block float-right text-muted small font-weight-bold"
                >
                  <T i18nKey="archive_link">#</T>
                </a>
              )}
              {this.state.imageLoading && (
                <svg class="icon icon-spinner spin">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              )}
              {isImage(this.state.postForm.url) && (
                <img src={this.state.postForm.url} class="img-fluid" />
              )}
              {this.state.crossPosts.length > 0 && (
                <>
                  <div class="my-1 text-muted small font-weight-bold">
                    <T i18nKey="cross_posts">#</T>
                  </div>
                  <PostListings showCommunity posts={this.state.crossPosts} />
                </>
              )}
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">
              <T i18nKey="title">#</T>
            </label>
            <div class="col-sm-10">
              <textarea
                value={this.state.postForm.name}
                onInput={linkEvent(this, this.handlePostNameChange)}
                class="form-control"
                required
                rows={2}
                minLength={3}
                maxLength={100}
              />
              {this.state.suggestedPosts.length > 0 && (
                <>
                  <div class="my-1 text-muted small font-weight-bold">
                    <T i18nKey="related_posts">#</T>
                  </div>
                  <PostListings posts={this.state.suggestedPosts} />
                </>
              )}
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label">
              <T i18nKey="body">#</T>
            </label>
            <div class="col-sm-10">
              <textarea
                value={this.state.postForm.body}
                onInput={linkEvent(this, this.handlePostBodyChange)}
                className={`form-control ${this.state.previewMode && 'd-none'}`}
                rows={4}
                maxLength={10000}
              />
              {this.state.previewMode && (
                <div
                  className="md-div"
                  dangerouslySetInnerHTML={mdToHtml(this.state.postForm.body)}
                />
              )}
              {this.state.postForm.body && (
                <button
                  className={`mt-1 mr-2 btn btn-sm btn-secondary ${this.state
                    .previewMode && 'active'}`}
                  onClick={linkEvent(this, this.handlePreviewToggle)}
                >
                  <T i18nKey="preview">#</T>
                </button>
              )}
              <a
                href={markdownHelpUrl}
                target="_blank"
                class="d-inline-block float-right text-muted small font-weight-bold"
              >
                <T i18nKey="formatting_help">#</T>
              </a>
            </div>
          </div>
          {!this.props.post && (
            <div class="form-group row">
              <label class="col-sm-2 col-form-label">
                <T i18nKey="community">#</T>
              </label>
              <div class="col-sm-10">
                <select
                  class="form-control"
                  value={this.state.postForm.community_id}
                  onInput={linkEvent(this, this.handlePostCommunityChange)}
                >
                  {this.state.communities.map(community => (
                    <option value={community.id}>{community.name}</option>
                  ))}
                </select>
              </div>
            </div>
          )}
          {WebSocketService.Instance.site.enable_nsfw && (
            <div class="form-group row">
              <div class="col-sm-10">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    type="checkbox"
                    checked={this.state.postForm.nsfw}
                    onChange={linkEvent(this, this.handlePostNsfwChange)}
                  />
                  <label class="form-check-label">
                    <T i18nKey="nsfw">#</T>
                  </label>
                </div>
              </div>
            </div>
          )}
          <div class="form-group row">
            <div class="col-sm-10">
              <button type="submit" class="btn btn-secondary mr-2">
                {this.state.loading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : this.props.post ? (
                  capitalizeFirstLetter(i18n.t('save'))
                ) : (
                  capitalizeFirstLetter(i18n.t('create'))
                )}
              </button>
              {this.props.post && (
                <button
                  type="button"
                  class="btn btn-secondary"
                  onClick={linkEvent(this, this.handleCancel)}
                >
                  <T i18nKey="cancel">#</T>
                </button>
              )}
            </div>
          </div>
        </form>
      </div>
    );
  }

  handlePostSubmit(i: PostForm, event: any) {
    event.preventDefault();
    if (i.props.post) {
      WebSocketService.Instance.editPost(i.state.postForm);
    } else {
      WebSocketService.Instance.createPost(i.state.postForm);
    }
    i.state.loading = true;
    i.setState(i.state);
  }

  copySuggestedTitle(i: PostForm) {
    i.state.postForm.name = i.state.suggestedTitle;
    i.state.suggestedTitle = undefined;
    i.setState(i.state);
  }

  handlePostUrlChange(i: PostForm, event: any) {
    i.state.postForm.url = event.target.value;
    i.setState(i.state);
    i.fetchPageTitle();
  }

  fetchPageTitle() {
    if (validURL(this.state.postForm.url)) {
      let form: SearchForm = {
        q: this.state.postForm.url,
        type_: SearchType[SearchType.Url],
        sort: SortType[SortType.TopAll],
        page: 1,
        limit: 6,
      };

      WebSocketService.Instance.search(form);

      // Fetch the page title
      getPageTitle(this.state.postForm.url).then(d => {
        this.state.suggestedTitle = d;
        this.setState(this.state);
      });
    } else {
      this.state.suggestedTitle = undefined;
      this.state.crossPosts = [];
    }
  }

  handlePostNameChange(i: PostForm, event: any) {
    i.state.postForm.name = event.target.value;
    i.setState(i.state);
    i.fetchSimilarPosts();
  }

  fetchSimilarPosts() {
    let form: SearchForm = {
      q: this.state.postForm.name,
      type_: SearchType[SearchType.Posts],
      sort: SortType[SortType.TopAll],
      community_id: this.state.postForm.community_id,
      page: 1,
      limit: 6,
    };

    if (this.state.postForm.name !== '') {
      WebSocketService.Instance.search(form);
    } else {
      this.state.suggestedPosts = [];
    }

    this.setState(this.state);
  }

  handlePostBodyChange(i: PostForm, event: any) {
    i.state.postForm.body = event.target.value;
    i.setState(i.state);
  }

  handlePostCommunityChange(i: PostForm, event: any) {
    i.state.postForm.community_id = Number(event.target.value);
    i.setState(i.state);
  }

  handlePostNsfwChange(i: PostForm, event: any) {
    i.state.postForm.nsfw = event.target.checked;
    i.setState(i.state);
  }

  handleCancel(i: PostForm) {
    i.props.onCancel();
  }

  handlePreviewToggle(i: PostForm, event: any) {
    event.preventDefault();
    i.state.previewMode = !i.state.previewMode;
    i.setState(i.state);
  }

  handleImageUpload(i: PostForm, event: any) {
    event.preventDefault();
    let file = event.target.files[0];
    const imageUploadUrl = `/pictshare/api/upload.php`;
    const formData = new FormData();
    formData.append('file', file);

    i.state.imageLoading = true;
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
        i.state.postForm.url = url;
        i.state.imageLoading = false;
        i.setState(i.state);
      })
      .catch(error => {
        i.state.imageLoading = false;
        i.setState(i.state);
        alert(error);
      });
  }

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (op == UserOperation.ListCommunities) {
      let res: ListCommunitiesResponse = msg;
      this.state.communities = res.communities;
      if (this.props.post) {
        this.state.postForm.community_id = this.props.post.community_id;
      } else if (this.props.params && this.props.params.community) {
        let foundCommunityId = res.communities.find(
          r => r.name == this.props.params.community
        ).id;
        this.state.postForm.community_id = foundCommunityId;
      } else {
        this.state.postForm.community_id = res.communities[0].id;
      }
      this.setState(this.state);
    } else if (op == UserOperation.CreatePost) {
      this.state.loading = false;
      let res: PostResponse = msg;
      this.props.onCreate(res.post.id);
    } else if (op == UserOperation.EditPost) {
      this.state.loading = false;
      let res: PostResponse = msg;
      this.props.onEdit(res.post);
    } else if (op == UserOperation.Search) {
      let res: SearchResponse = msg;

      if (res.type_ == SearchType[SearchType.Posts]) {
        this.state.suggestedPosts = res.posts;
      } else if (res.type_ == SearchType[SearchType.Url]) {
        this.state.crossPosts = res.posts;
      }
      this.setState(this.state);
    }
  }
}
