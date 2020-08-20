import { Component, linkEvent } from 'inferno';
import { Prompt } from 'inferno-router';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  CommunityForm as CommunityFormI,
  UserOperation,
  Category,
  ListCategoriesResponse,
  CommunityResponse,
  WebSocketJsonResponse,
  Community,
} from 'lemmy-js-client';
import { WebSocketService } from '../services';
import { wsJsonToRes, capitalizeFirstLetter, toast, randomStr } from '../utils';
import { i18n } from '../i18next';

import { MarkdownTextArea } from './markdown-textarea';
import { ImageUploadForm } from './image-upload-form';

interface CommunityFormProps {
  community?: Community; // If a community is given, that means this is an edit
  onCancel?(): any;
  onCreate?(community: Community): any;
  onEdit?(community: Community): any;
  enableNsfw: boolean;
}

interface CommunityFormState {
  communityForm: CommunityFormI;
  categories: Array<Category>;
  loading: boolean;
}

export class CommunityForm extends Component<
  CommunityFormProps,
  CommunityFormState
> {
  private id = `community-form-${randomStr()}`;
  private subscription: Subscription;

  private emptyState: CommunityFormState = {
    communityForm: {
      name: null,
      title: null,
      category_id: null,
      nsfw: false,
      icon: null,
      banner: null,
    },
    categories: [],
    loading: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    this.handleCommunityDescriptionChange = this.handleCommunityDescriptionChange.bind(
      this
    );

    this.handleIconUpload = this.handleIconUpload.bind(this);
    this.handleIconRemove = this.handleIconRemove.bind(this);

    this.handleBannerUpload = this.handleBannerUpload.bind(this);
    this.handleBannerRemove = this.handleBannerRemove.bind(this);

    if (this.props.community) {
      this.state.communityForm = {
        name: this.props.community.name,
        title: this.props.community.title,
        category_id: this.props.community.category_id,
        description: this.props.community.description,
        edit_id: this.props.community.id,
        nsfw: this.props.community.nsfw,
        icon: this.props.community.icon,
        banner: this.props.community.banner,
        auth: null,
      };
    }

    this.subscription = WebSocketService.Instance.subject
      .pipe(retryWhen(errors => errors.pipe(delay(3000), take(10))))
      .subscribe(
        msg => this.parseMessage(msg),
        err => console.error(err),
        () => console.log('complete')
      );

    WebSocketService.Instance.listCategories();
  }

  componentDidUpdate() {
    if (
      !this.state.loading &&
      (this.state.communityForm.name ||
        this.state.communityForm.title ||
        this.state.communityForm.description)
    ) {
      window.onbeforeunload = () => true;
    } else {
      window.onbeforeunload = undefined;
    }
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
    window.onbeforeunload = null;
  }

  render() {
    return (
      <>
        <Prompt
          when={
            !this.state.loading &&
            (this.state.communityForm.name ||
              this.state.communityForm.title ||
              this.state.communityForm.description)
          }
          message={i18n.t('block_leaving')}
        />
        <form onSubmit={linkEvent(this, this.handleCreateCommunitySubmit)}>
          {!this.props.community && (
            <div class="form-group row">
              <label class="col-12 col-form-label" htmlFor="community-name">
                {i18n.t('name')}
                <span
                  class="pointer unselectable ml-2 text-muted"
                  data-tippy-content={i18n.t('name_explain')}
                >
                  <svg class="icon icon-inline">
                    <use xlinkHref="#icon-help-circle"></use>
                  </svg>
                </span>
              </label>
              <div class="col-12">
                <input
                  type="text"
                  id="community-name"
                  class="form-control"
                  value={this.state.communityForm.name}
                  onInput={linkEvent(this, this.handleCommunityNameChange)}
                  required
                  minLength={3}
                  maxLength={20}
                  pattern="[a-z0-9_]+"
                  title={i18n.t('community_reqs')}
                />
              </div>
            </div>
          )}
          <div class="form-group row">
            <label class="col-12 col-form-label" htmlFor="community-title">
              {i18n.t('display_name')}
              <span
                class="pointer unselectable ml-2 text-muted"
                data-tippy-content={i18n.t('display_name_explain')}
              >
                <svg class="icon icon-inline">
                  <use xlinkHref="#icon-help-circle"></use>
                </svg>
              </span>
            </label>
            <div class="col-12">
              <input
                type="text"
                id="community-title"
                value={this.state.communityForm.title}
                onInput={linkEvent(this, this.handleCommunityTitleChange)}
                class="form-control"
                required
                minLength={3}
                maxLength={100}
              />
            </div>
          </div>
          <div class="form-group">
            <label>{i18n.t('icon')}</label>
            <ImageUploadForm
              uploadTitle={i18n.t('upload_icon')}
              imageSrc={this.state.communityForm.icon}
              onUpload={this.handleIconUpload}
              onRemove={this.handleIconRemove}
              rounded
            />
          </div>
          <div class="form-group">
            <label>{i18n.t('banner')}</label>
            <ImageUploadForm
              uploadTitle={i18n.t('upload_banner')}
              imageSrc={this.state.communityForm.banner}
              onUpload={this.handleBannerUpload}
              onRemove={this.handleBannerRemove}
            />
          </div>
          <div class="form-group row">
            <label class="col-12 col-form-label" htmlFor={this.id}>
              {i18n.t('sidebar')}
            </label>
            <div class="col-12">
              <MarkdownTextArea
                initialContent={this.state.communityForm.description}
                onContentChange={this.handleCommunityDescriptionChange}
              />
            </div>
          </div>
          <div class="form-group row">
            <label class="col-12 col-form-label" htmlFor="community-category">
              {i18n.t('category')}
            </label>
            <div class="col-12">
              <select
                class="form-control"
                id="community-category"
                value={this.state.communityForm.category_id}
                onInput={linkEvent(this, this.handleCommunityCategoryChange)}
              >
                {this.state.categories.map(category => (
                  <option value={category.id}>{category.name}</option>
                ))}
              </select>
            </div>
          </div>

          {this.props.enableNsfw && (
            <div class="form-group row">
              <div class="col-12">
                <div class="form-check">
                  <input
                    class="form-check-input"
                    id="community-nsfw"
                    type="checkbox"
                    checked={this.state.communityForm.nsfw}
                    onChange={linkEvent(this, this.handleCommunityNsfwChange)}
                  />
                  <label class="form-check-label" htmlFor="community-nsfw">
                    {i18n.t('nsfw')}
                  </label>
                </div>
              </div>
            </div>
          )}
          <div class="form-group row">
            <div class="col-12">
              <button
                type="submit"
                class="btn btn-secondary mr-2"
                disabled={this.state.loading}
              >
                {this.state.loading ? (
                  <svg class="icon icon-spinner spin">
                    <use xlinkHref="#icon-spinner"></use>
                  </svg>
                ) : this.props.community ? (
                  capitalizeFirstLetter(i18n.t('save'))
                ) : (
                  capitalizeFirstLetter(i18n.t('create'))
                )}
              </button>
              {this.props.community && (
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
      </>
    );
  }

  handleCreateCommunitySubmit(i: CommunityForm, event: any) {
    event.preventDefault();
    i.state.loading = true;
    if (i.props.community) {
      WebSocketService.Instance.editCommunity(i.state.communityForm);
    } else {
      WebSocketService.Instance.createCommunity(i.state.communityForm);
    }
    i.setState(i.state);
  }

  handleCommunityNameChange(i: CommunityForm, event: any) {
    i.state.communityForm.name = event.target.value;
    i.setState(i.state);
  }

  handleCommunityTitleChange(i: CommunityForm, event: any) {
    i.state.communityForm.title = event.target.value;
    i.setState(i.state);
  }

  handleCommunityDescriptionChange(val: string) {
    this.state.communityForm.description = val;
    this.setState(this.state);
  }

  handleCommunityCategoryChange(i: CommunityForm, event: any) {
    i.state.communityForm.category_id = Number(event.target.value);
    i.setState(i.state);
  }

  handleCommunityNsfwChange(i: CommunityForm, event: any) {
    i.state.communityForm.nsfw = event.target.checked;
    i.setState(i.state);
  }

  handleCancel(i: CommunityForm) {
    i.props.onCancel();
  }

  handleIconUpload(url: string) {
    this.state.communityForm.icon = url;
    this.setState(this.state);
  }

  handleIconRemove() {
    this.state.communityForm.icon = '';
    this.setState(this.state);
  }

  handleBannerUpload(url: string) {
    this.state.communityForm.banner = url;
    this.setState(this.state);
  }

  handleBannerRemove() {
    this.state.communityForm.banner = '';
    this.setState(this.state);
  }

  parseMessage(msg: WebSocketJsonResponse) {
    let res = wsJsonToRes(msg);
    console.log(msg);
    if (msg.error) {
      toast(i18n.t(msg.error), 'danger');
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (res.op == UserOperation.ListCategories) {
      let data = res.data as ListCategoriesResponse;
      this.state.categories = data.categories;
      if (!this.props.community) {
        this.state.communityForm.category_id = data.categories[0].id;
      }
      this.setState(this.state);
    } else if (res.op == UserOperation.CreateCommunity) {
      let data = res.data as CommunityResponse;
      this.state.loading = false;
      this.props.onCreate(data.community);
    } else if (res.op == UserOperation.EditCommunity) {
      let data = res.data as CommunityResponse;
      this.state.loading = false;
      this.props.onEdit(data.community);
    }
  }
}
