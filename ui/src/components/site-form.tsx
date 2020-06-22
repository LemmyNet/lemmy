import { Component, linkEvent } from 'inferno';
import { Prompt } from 'inferno-router';
import { Site, SiteForm as SiteFormI } from '../interfaces';
import { WebSocketService } from '../services';
import { capitalizeFirstLetter, randomStr, setupTribute } from '../utils';
import autosize from 'autosize';
import Tribute from 'tributejs/src/Tribute.js';
import { i18n } from '../i18next';

interface SiteFormProps {
  site?: Site; // If a site is given, that means this is an edit
  onCancel?(): any;
}

interface SiteFormState {
  siteForm: SiteFormI;
  loading: boolean;
}

export class SiteForm extends Component<SiteFormProps, SiteFormState> {
  private id = `site-form-${randomStr()}`;
  private tribute: Tribute;
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

    this.tribute = setupTribute();
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
    var textarea: any = document.getElementById(this.id);
    autosize(textarea);
    this.tribute.attach(textarea);
    textarea.addEventListener('tribute-replaced', () => {
      this.state.siteForm.description = textarea.value;
      this.setState(this.state);
      autosize.update(textarea);
    });
  }

  // Necessary to stop the loading
  componentWillReceiveProps() {
    this.state.loading = false;
    this.setState(this.state);
  }

  render() {
    return (
      <>
        <Prompt
          when={
            !this.state.loading &&
            !this.props.site &&
            (this.state.siteForm.name || this.state.siteForm.description)
          }
          message={i18n.t('block_leaving')}
        />
        <form onSubmit={linkEvent(this, this.handleCreateSiteSubmit)}>
          <h5>{`${
            this.props.site
              ? capitalizeFirstLetter(i18n.t('save'))
              : capitalizeFirstLetter(i18n.t('name'))
          } ${i18n.t('your_site')}`}</h5>
          <div class="form-group row">
            <label class="col-12 col-form-label" htmlFor="create-site-name">
              {i18n.t('name')}
            </label>
            <div class="col-12">
              <input
                type="text"
                id="create-site-name"
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
            <label class="col-12 col-form-label" htmlFor={this.id}>
              {i18n.t('sidebar')}
            </label>
            <div class="col-12">
              <textarea
                id={this.id}
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
                  id="create-site-downvotes"
                  type="checkbox"
                  checked={this.state.siteForm.enable_downvotes}
                  onChange={linkEvent(
                    this,
                    this.handleSiteEnableDownvotesChange
                  )}
                />
                <label class="form-check-label" htmlFor="create-site-downvotes">
                  {i18n.t('enable_downvotes')}
                </label>
              </div>
            </div>
          </div>
          <div class="form-group row">
            <div class="col-12">
              <div class="form-check">
                <input
                  class="form-check-input"
                  id="create-site-enable-nsfw"
                  type="checkbox"
                  checked={this.state.siteForm.enable_nsfw}
                  onChange={linkEvent(this, this.handleSiteEnableNsfwChange)}
                />
                <label
                  class="form-check-label"
                  htmlFor="create-site-enable-nsfw"
                >
                  {i18n.t('enable_nsfw')}
                </label>
              </div>
            </div>
          </div>
          <div class="form-group row">
            <div class="col-12">
              <div class="form-check">
                <input
                  class="form-check-input"
                  id="create-site-open-registration"
                  type="checkbox"
                  checked={this.state.siteForm.open_registration}
                  onChange={linkEvent(
                    this,
                    this.handleSiteOpenRegistrationChange
                  )}
                />
                <label
                  class="form-check-label"
                  htmlFor="create-site-open-registration"
                >
                  {i18n.t('open_registration')}
                </label>
              </div>
            </div>
          </div>
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
                  {i18n.t('cancel')}
                </button>
              )}
            </div>
          </div>
        </form>
      </>
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
