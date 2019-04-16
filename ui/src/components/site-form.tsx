import { Component, linkEvent } from 'inferno';
import { Site, SiteForm as SiteFormI } from '../interfaces';
import { WebSocketService } from '../services';
import * as autosize from 'autosize';

interface SiteFormProps {
  site?: Site; // If a site is given, that means this is an edit
  onCancel?(): any;
}

interface SiteFormState {
  siteForm: SiteFormI;
  loading: boolean;
}

export class SiteForm extends Component<SiteFormProps, SiteFormState> {
  private emptyState: SiteFormState ={
    siteForm: {
      name: null
    },
    loading: false
  }

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
  }

  componentDidMount() {
    autosize(document.querySelectorAll('textarea'));
  }

  render() {
    return (
      <form onSubmit={linkEvent(this, this.handleCreateSiteSubmit)}>
        <h4>{`${this.props.site ? 'Edit' : 'Name'} your Site`}</h4>
        <div class="form-group row">
          <label class="col-12 col-form-label">Name</label>
          <div class="col-12">
            <input type="text" class="form-control" value={this.state.siteForm.name} onInput={linkEvent(this, this.handleSiteNameChange)} required minLength={3} />
          </div>
        </div>
        <div class="form-group row">
          <label class="col-12 col-form-label">Sidebar</label>
          <div class="col-12">
            <textarea value={this.state.siteForm.description} onInput={linkEvent(this, this.handleSiteDescriptionChange)} class="form-control" rows={3} />
          </div>
        </div>
        <div class="form-group row">
          <div class="col-12">
            <button type="submit" class="btn btn-secondary mr-2">
              {this.state.loading ? 
              <svg class="icon icon-spinner spin"><use xlinkHref="#icon-spinner"></use></svg> : 
              this.props.site ? 'Save' : 'Create'}</button>
              {this.props.site && <button type="button" class="btn btn-secondary" onClick={linkEvent(this, this.handleCancel)}>Cancel</button>}
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
