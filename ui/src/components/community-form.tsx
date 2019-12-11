import { Component, linkEvent } from 'inferno';
import { Subscription } from 'rxjs';
import { retryWhen, delay, take } from 'rxjs/operators';
import {
  CommunityForm as CommunityFormI,
  UserOperation,
  Category,
  ListCategoriesResponse,
  CommunityResponse,
} from '../interfaces';
import { WebSocketService } from '../services';
import { msgOp, capitalizeFirstLetter } from '../utils';
import * as autosize from 'autosize';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

import { Community } from '../interfaces';

interface CommunityFormProps {
  community?: Community; // If a community is given, that means this is an edit
  onCancel?(): any;
  onCreate?(community: Community): any;
  onEdit?(community: Community): any;
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
  private subscription: Subscription;

  private emptyState: CommunityFormState = {
    communityForm: {
      name: null,
      title: null,
      category_id: null,
      nsfw: false,
    },
    categories: [],
    loading: false,
  };

  constructor(props: any, context: any) {
    super(props, context);

    this.state = this.emptyState;

    if (this.props.community) {
      this.state.communityForm = {
        name: this.props.community.name,
        title: this.props.community.title,
        category_id: this.props.community.category_id,
        description: this.props.community.description,
        edit_id: this.props.community.id,
        nsfw: this.props.community.nsfw,
        auth: null,
      };
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

    WebSocketService.Instance.listCategories();
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  componentWillUnmount() {
    this.subscription.unsubscribe();
  }

  render() {
    return (
      <form onSubmit={linkEvent(this, this.handleCreateCommunitySubmit)}>
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="name">#</T>
          </label>
          <div class="col-12">
            <input
              type="text"
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
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="title">#</T>
          </label>
          <div class="col-12">
            <input
              type="text"
              value={this.state.communityForm.title}
              onInput={linkEvent(this, this.handleCommunityTitleChange)}
              class="form-control"
              required
              minLength={3}
              maxLength={100}
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="sidebar">#</T>
          </label>
          <div class="col-12">
            <textarea
              value={this.state.communityForm.description}
              onInput={linkEvent(this, this.handleCommunityDescriptionChange)}
              class="form-control"
              rows={3}
              maxLength={10000}
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="category">#</T>
          </label>
          <div class="col-12">
            <select
              class="form-control"
              value={this.state.communityForm.category_id}
              onInput={linkEvent(this, this.handleCommunityCategoryChange)}
            >
              {this.state.categories.map(category => (
                <option value={category.id}>{category.name}</option>
              ))}
            </select>
          </div>
        </div>

        {WebSocketService.Instance.site.enable_nsfw && (
          <div class="form-group row">
            <div class="col-12">
              <div class="form-check">
                <input
                  class="form-check-input"
                  type="checkbox"
                  checked={this.state.communityForm.nsfw}
                  onChange={linkEvent(this, this.handleCommunityNsfwChange)}
                />
                <label class="form-check-label">
                  <T i18nKey="nsfw">#</T>
                </label>
              </div>
            </div>
          </div>
        )}
        <div class="form-group row">
          <div class="col-12">
            <button type="submit" class="btn btn-secondary mr-2">
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
                <T i18nKey="cancel">#</T>
              </button>
            )}
          </div>
        </div>
      </form>
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

  handleCommunityDescriptionChange(i: CommunityForm, event: any) {
    i.state.communityForm.description = event.target.value;
    i.setState(i.state);
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

  parseMessage(msg: any) {
    let op: UserOperation = msgOp(msg);
    console.log(msg);
    if (msg.error) {
      alert(i18n.t(msg.error));
      this.state.loading = false;
      this.setState(this.state);
      return;
    } else if (op == UserOperation.ListCategories) {
      let res: ListCategoriesResponse = msg;
      this.state.categories = res.categories;
      if (!this.props.community) {
        this.state.communityForm.category_id = res.categories[0].id;
      }
      this.setState(this.state);
    } else if (op == UserOperation.CreateCommunity) {
      let res: CommunityResponse = msg;
      this.state.loading = false;
      this.props.onCreate(res.community);
    }
    // TODO is ths necessary
    else if (op == UserOperation.EditCommunity) {
      let res: CommunityResponse = msg;
      this.state.loading = false;
      this.props.onEdit(res.community);
    }
  }
}
