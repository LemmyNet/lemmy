import { Component, linkEvent } from 'inferno';
import { Prompt } from 'inferno-router';
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
  GetSiteResponse,
  WebSocketJsonResponse,
} from '../interfaces';
import { WebSocketService, UserService } from '../services';
import {
  wsJsonToRes,
  getPageTitle,
  validURL,
  capitalizeFirstLetter,
  markdownHelpUrl,
  archiveUrl,
  mdToHtml,
  debounce,
  isImage,
  toast,
  randomStr,
  setupTribute,
  setupTippy,
  emojiPicker,
  pictrsDeleteToast,
} from '../utils';
import autosize from 'autosize';
import Tribute from 'tributejs/src/Tribute.js';
import emojiShortName from 'emoji-short-name';
import Selectr from 'mobius1-selectr';
import { i18n } from '../i18next';

const MAX_POST_TITLE_LENGTH = 200;

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
  enable_nsfw: boolean;
}

export class PostForm extends Component<PostFormProps, PostFormState> {
  private id = `post-form-${randomStr()}`;
  private tribute: Tribute;
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
    enable_nsfw: undefined,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.fetchSimilarPosts = debounce(this.fetchSimilarPosts).bind(this);
    this.fetchPageTitle = debounce(this.fetchPageTitle).bind(this);

    this.tribute = setupTribute();
    this.setupEmojiPicker();

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
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
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
    WebSocketService.Instance.getSite();
  }

  componentDidMount() {
    var textarea: any = document.getElementById(this.id);
    autosize(textarea);
    this.tribute.attach(textarea);
    textarea.addEventListener('tribute-replaced', () => {
      this.state.postForm.body = textarea.value;
      this.setState(this.state);
      autosize.update(textarea);
    });
    setupTippy();
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <div>
        <Prompt
          when={
            !this.state.loading &&
            (this.state.postForm.name ||
              this.state.postForm.url ||
              this.state.postForm.body)
          }
          message={i18n.t('block_leaving')}
        />
        <form onSubmit={linkEvent(this, this.handlePostSubmit)}>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label" htmlFor="post-url">
              {i18n.t('url')}
            </label>
            <div class="col-sm-10">
              <input
                type="url"
                id="post-url"
                class="form-control"
                value={this.state.postForm.url}
                onInput={linkEvent(this, this.handlePostUrlChange)}
                onPaste={linkEvent(this, this.handleImageUploadPaste)}
              />
              {this.state.suggestedTitle && (
                <div
                  class="mt-1 text-muted small font-weight-bold pointer"
                  onClick={linkEvent(this, this.copySuggestedTitle)}
                >
                  {i18n.t('copy_suggested_title', {
                    title: this.state.suggestedTitle,
                  })}
                </div>
              )}
              <form>
                <label
                  htmlFor="file-upload"
                  className={`${
                    UserService.Instance.user && 'pointer'
                  } d-inline-block float-right text-muted font-weight-bold`}
                  data-tippy-content={i18n.t('upload_image')}
                >
                  <svg class="icon icon-inline">
                    <use xlinkHref="#icon-image"></use>
                  </svg>
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
                  {i18n.t('archive_link')}
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
                    {i18n.t('cross_posts')}
                  </div>
                  <PostListings showCommunity posts={this.state.crossPosts} />
                </>
              )}
            </div>
          </div>
          <div class="form-group row">
            <label class="col-sm-2 col-form-label" htmlFor="post-title">
              {i18n.t('title')}
            </label>
            <div class="col-sm-10">
              <textarea
                value={this.state.postForm.name}
                id="post-title"
                onInput={linkEvent(this, this.handlePostNameChange)}
                class="form-control"
                required
                rows={2}
                minLength={3}
                maxLength={MAX_POST_TITLE_LENGTH}
              />
              {this.state.suggestedPosts.length > 0 && (
                <>
                  <div class="my-1 text-muted small font-weight-bold">
                    {i18n.t('related_posts')}
                  </div>
                  <PostListings posts={this.state.suggestedPosts} />
                </>
              )}
            </div>
          </div>

          <div class="form-group row">
            <label class="col-sm-2 col-form-label" htmlFor={this.id}>
              {i18n.t('body')}
            </label>
            <div class="col-sm-10">
              <textarea
                id={this.id}
                value={this.state.postForm.body}
                onInput={linkEvent(this, this.handlePostBodyChange)}
                className={`form-control ${this.state.previewMode && 'd-none'}`}
                rows={4}
                maxLength={10000}
              />
              {this.state.previewMode && (
                <div
                  className="card card-body md-div"
                  dangerouslySetInnerHTML={mdToHtml(this.state.postForm.body)}
                />
              )}
              {this.state.postForm.body && (
                <button
                  className={`mt-1 mr-2 btn btn-sm btn-secondary ${
                    this.state.previewMode && 'active'
                  }`}
                  onClick={linkEvent(this, this.handlePreviewToggle)}
                >
                  {i18n.t('preview')}
                </button>
              )}
              <a
                href={markdownHelpUrl}
                target="_blank"
                class="d-inline-block float-right text-muted font-weight-bold"
                title={i18n.t('formatting_help')}
              >
                <svg class="icon icon-inline">
                  <use xlinkHref="#icon-help-circle"></use>
                </svg>
              </a>
              <span
                onClick={linkEvent(this, this.handleEmojiPickerClick)}
                class="pointer unselectable d-inline-block mr-3 float-right text-muted font-weight-bold"
                data-tippy-content={i18n.t('emoji_picker')}
              >
                <svg class="icon icon-inline">
                  <use xlinkHref="#icon-smile"></use>
                </svg>
              </span>
            </div>
          </div>
          {!this.props.post && (
            <div class="form-group row">
              <label class="col-sm-2 col-form-label" htmlFor="post-community">
                {i18n.t('community')}
              </label>
              <div class="col-sm-10">
                <select
                  class="form-control"
                  id="post-community"
                  value={this.state.postForm.community_id}
                  onInput={linkEvent(this, this.handlePostCommunityChange)}
                >
                  <option>{i18n.t('select_a_community')}</option>
                  {this.state.communities.map(community => (
                    <option value={community.id}>{community.name}</option>
                  ))}
                </select>
              </div>
            </div>
          )}
          {this.state.enable_nsfw && (
            <div class="form-group row">
              <div class="col-sm-10">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    id="post-nsfw"
                    type="checkbox"
                    checked={this.state.postForm.nsfw}
                    onChange={linkEvent(this, this.handlePostNsfwChange)}
                  />
                  <label class="form-check-label" htmlFor="post-nsfw">
                    {i18n.t('nsfw')}
                  </label>
                </div>
              </div>
            </div>
          )}
          <div class="form-group row">
            <div class="col-sm-10">
              <button
                disabled={
                  !this.state.postForm.community_id || this.state.loading
                }
                type="submit"
                class="btn btn-secondary mr-2"
              >
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
                  {i18n.t('cancel')}
                </button>
              )}
            </div>
          </div>
        </form>
      </div>
    );
  }

  setupEmojiPicker() {
    emojiPicker.on('emoji', twemojiHtmlStr => {
      if (this.state.postForm.body == null) {
        this.state.postForm.body = '';
      }
      var el = document.createElement('div');
      el.innerHTML = twemojiHtmlStr;
      let nativeUnicode = (el.childNodes[0] as HTMLElement).getAttribute('alt');
      let shortName = `:${emojiShortName[nativeUnicode]}:`;
      this.state.postForm.body += shortName;
      this.setState(this.state);
    });
  }

  handlePostSubmit(i: PostForm, event: any) {
    event.preventDefault();

    // Coerce empty url string to undefined
    if (i.state.postForm.url && i.state.postForm.url === '') {
      i.state.postForm.url = undefined;
    }

    if (i.props.post) {
      WebSocketService.Instance.editPost(i.state.postForm);
    } else {
      WebSocketService.Instance.createPost(i.state.postForm);
    }
    i.state.loading = true;
    i.setState(i.state);
  }

  copySuggestedTitle(i: PostForm) {
    i.state.postForm.name = i.state.suggestedTitle.substring(
      0,
      MAX_POST_TITLE_LENGTH
    );
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

  handleImageUploadPaste(i: PostForm, event: any) {
    let image = event.clipboardData.files[0];
    if (image) {
      i.handleImageUpload(i, image);
    }
  }

  handleImageUpload(i: PostForm, event: any) {
    let file: any;
    if (event.target) {
      event.preventDefault();
      file = event.target.files[0];
    } else {
      file = event;
    }

    const imageUploadUrl = `/pictrs/image`;
    const formData = new FormData();
    formData.append('images[]', file);

    i.state.imageLoading = true;
    i.setState(i.state);

    fetch(imageUploadUrl, {
      method: 'POST',
      body: formData,
    })
      .then(res => res.json())
      .then(res => {
        console.log('pictrs upload:');
        console.log(res);
        if (res.msg == 'ok') {
          let hash = res.files[0].file;
          let url = `${window.location.origin}/pictrs/image/${hash}`;
          let deleteToken = res.files[0].delete_token;
          let deleteUrl = `${window.location.origin}/pictrs/image/delete/${deleteToken}/${hash}`;
          i.state.postForm.url = url;
          i.state.imageLoading = false;
          i.setState(i.state);
          pictrsDeleteToast(
            i18n.t('click_to_delete_picture'),
            i18n.t('picture_deleted'),
            deleteUrl
          );
        } else {
          i.state.imageLoading = false;
          i.setState(i.state);
          toast(JSON.stringify(res), 'danger');
        }
      })
      .catch(error => {
        i.state.imageLoading = false;
        i.setState(i.state);
        toast(error, 'danger');
      });
  }

  handleEmojiPickerClick(_i: PostForm, event: any) {
    emojiPicker.togglePicker(event.target);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (res.op == UserOperation.ListCommunities) {
      let data = res.data as ListCommunitiesResponse;
      this.state.communities = data.communities;
      if (this.props.post) {
        this.state.postForm.community_id = this.props.post.community_id;
      } else if (this.props.params && this.props.params.community) {
        let foundCommunityId = data.communities.find(
          r => r.name == this.props.params.community
        ).id;
        this.state.postForm.community_id = foundCommunityId;
      } else {
        // By default, the null valued 'Select a Community'
      }
      this.setState(this.state);

      // Set up select searching
      let selectId: any = document.getElementById('post-community');
      if (selectId) {
        let selector = new Selectr(selectId, { nativeDropdown: false });
        selector.on('selectr.select', option => {
          this.state.postForm.community_id = Number(option.value);
          this.setState(this.state);
        });
      }
    } else if (res.op == UserOperation.CreatePost) {
      let data = res.data as PostResponse;
      if (data.post.creator_id == UserService.Instance.user.id) {
        this.state.loading = false;
        this.props.onCreate(data.post.id);
      }
    } else if (res.op == UserOperation.EditPost) {
      let data = res.data as PostResponse;
      if (data.post.creator_id == UserService.Instance.user.id) {
        this.state.loading = false;
        this.props.onEdit(data.post);
      }
    } else if (res.op == UserOperation.Search) {
      let data = res.data as SearchResponse;

      if (data.type_ == SearchType[SearchType.Posts]) {
        this.state.suggestedPosts = data.posts;
      } else if (data.type_ == SearchType[SearchType.Url]) {
        this.state.crossPosts = data.posts;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.GetSite) {
      let data = res.data as GetSiteResponse;
      this.state.enable_nsfw = data.site.enable_nsfw;
      this.setState(this.state);
    }
  }
}
