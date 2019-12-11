import { Component, linkEvent } from 'inferno';
import { Site, SiteForm as SiteFormI } from '../interfaces';
import { WebSocketService } from '../services';
import { capitalizeFirstLetter } from '../utils';
import * as autosize from 'autosize';
import { i18n } from '../i18next';
import { T } from 'inferno-i18next';

interface SiteFormProps {
  site?: Site; // If a site is given, that means this is an edit
  onCancel?(): any;
}

interface SiteFormState {
  siteForm: SiteFormI;
  loading: boolean;
}

export class SiteForm extends Component<SiteFormProps, SiteFormState> {
  private emptyState: SiteFormState = {
    siteForm: {
      enable_downvotes: true,
      open_registration: true,
      enable_nsfw: true,
      name: null,
    },
    loading: false,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
    if (this.props.site) {
      this.state.siteForm = {
        name: this.props.site.name,
        description: this.props.site.description,
        enable_downvotes: this.props.site.enable_downvotes,
        open_registration: this.props.site.open_registration,
        enable_nsfw: this.props.site.enable_nsfw,
      };
    }
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  render() {
    return (
      <form onSubmit={linkEvent(this, this.handleCreateSiteSubmit)}>
        <h5>{`${
          this.props.site
            ? capitalizeFirstLetter(i18n.t('edit'))
            : capitalizeFirstLetter(i18n.t('name'))
        } ${i18n.t('your_site')}`}</h5>
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="name">#</T>
          </label>
          <div class="col-12">
            <input
              type="text"
              class="form-control"
              value={this.state.siteForm.name}
              onInput={linkEvent(this, this.handleSiteNameChange)}
              required
              minLength={3}
              maxLength={20}
            />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-12 col-form-label">
            <T i18nKey="sidebar">#</T>
          </label>
          <div class="col-12">
            <textarea
              value={this.state.siteForm.description}
              onInput={linkEvent(this, this.handleSiteDescriptionChange)}
              class="form-control"
              rows={3}
              maxLength={10000}
            />
          </div>
        </div>
        <div class="form-group row">
          <div class="col-12">
            <div class="form-check">
              <input
                class="form-check-input"
                type="checkbox"
                checked={this.state.siteForm.enable_downvotes}
                onChange={linkEvent(this, this.handleSiteEnableDownvotesChange)}
              />
              <label class="form-check-label">
                <T i18nKey="enable_downvotes">#</T>
              </label>
            </div>
          </div>
        </div>
        <div class="form-group row">
          <div class="col-12">
            <div class="form-check">
              <input
                class="form-check-input"
                type="checkbox"
                checked={this.state.siteForm.enable_nsfw}
                onChange={linkEvent(this, this.handleSiteEnableNsfwChange)}
              />
              <label class="form-check-label">
                <T i18nKey="enable_nsfw">#</T>
              </label>
            </div>
          </div>
        </div>
        <div class="form-group row">
          <div class="col-12">
            <div class="form-check">
              <input
                class="form-check-input"
                type="checkbox"
                checked={this.state.siteForm.open_registration}
                onChange={linkEvent(
                  this,
                  this.handleSiteOpenRegistrationChange
                )}
              />
              <label class="form-check-label">
                <T i18nKey="open_registration">#</T>
              </label>
            </div>
          </div>
        </div>
        <div class="form-group row">
          <div class="col-12">
            <button type="submit" class="btn btn-secondary mr-2">
              {this.state.loading ? (
                <svg class="icon icon-spinner spin">
                  <use xlinkHref="#icon-spinner"></use>
                </svg>
              ) : this.props.site ? (
                capitalizeFirstLetter(i18n.t('save'))
              ) : (
                capitalizeFirstLetter(i18n.t('create'))
              )}
            </button>
            {this.props.site && (
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

  handleCreateSiteSubmit(i: SiteForm, event: any) {
    event.preventDefault();
    i.state.loading = true;
    if (i.props.site) {
      WebSocketService.Instance.editSite(i.state.siteForm);
    } else {
      WebSocketService.Instance.createSite(i.state.siteForm);
    }
    i.setState(i.state);
  }

  handleSiteNameChange(i: SiteForm, event: any) {
    i.state.siteForm.name = event.target.value;
    i.setState(i.state);
  }

  handleSiteDescriptionChange(i: SiteForm, event: any) {
    i.state.siteForm.description = event.target.value;
    i.setState(i.state);
  }

  handleSiteEnableNsfwChange(i: SiteForm, event: any) {
    i.state.siteForm.enable_nsfw = event.target.checked;
    i.setState(i.state);
  }

  handleSiteOpenRegistrationChange(i: SiteForm, event: any) {
    i.state.siteForm.open_registration = event.target.checked;
    i.setState(i.state);
  }

  handleSiteEnableDownvotesChange(i: SiteForm, event: any) {
    i.state.siteForm.enable_downvotes = event.target.checked;
    i.setState(i.state);
  }

  handleCancel(i: SiteForm) {
    i.props.onCancel();
  }
}
