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

  handleCancel(i: SiteForm) {
    i.props.onCancel();
  }
}
