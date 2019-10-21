import { Component, linkEvent } from 'inferno';
import { ListingType } from '../interfaces';
import { UserService } from '../services';

import { i18n } from '../i18next';

interface ListingTypeSelectProps {
  type_: ListingType;
  onChange?(val: ListingType): any;
}

interface ListingTypeSelectState {
  type_: ListingType;
}

export class ListingTypeSelect extends Component<
  ListingTypeSelectProps,
  ListingTypeSelectState
> {
  private emptyState: ListingTypeSelectState = {
    type_: this.props.type_,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
  }

  render() {
    return (
      <div class="btn-group btn-group-toggle">
        <label
          className={`btn btn-sm btn-secondary 
            ${this.state.type_ == ListingType.Subscribed && 'active'}
            ${UserService.Instance.user == undefined ? 'disabled' : 'pointer'}
          `}
        >
          <input
            type="radio"
            value={ListingType.Subscribed}
            checked={this.state.type_ == ListingType.Subscribed}
            onChange={linkEvent(this, this.handleTypeChange)}
            disabled={UserService.Instance.user == undefined}
          />
          {i18n.t('subscribed')}
        </label>
        <label
          className={`pointer btn btn-sm btn-secondary ${this.state.type_ ==
            ListingType.All && 'active'}`}
        >
          <input
            type="radio"
            value={ListingType.All}
            checked={this.state.type_ == ListingType.All}
            onChange={linkEvent(this, this.handleTypeChange)}
          />
          {i18n.t('all')}
        </label>
      </div>
    );
  }

  handleTypeChange(i: ListingTypeSelect, event: any) {
    i.state.type_ = Number(event.target.value);
    i.setState(i.state);
    i.props.onChange(i.state.type_);
  }
}
