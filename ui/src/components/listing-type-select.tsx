import { Component, linkEvent } from 'inferno';
import { ListingType } from 'lemmy-js-client';
import { UserService } from '../services';
import { randomStr } from '../utils';
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
  private id = `listing-type-input-${randomStr()}`;

  private emptyState: ListingTypeSelectState = {
    type_: this.props.type_,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
  }

  static getDerivedStateFromProps(props: any): ListingTypeSelectProps {
    return {
      type_: props.type_,
    };
  }

  render() {
    return (
      <div class="btn-group btn-group-toggle flex-wrap mb-2">
        <label
          className={`btn btn-outline-secondary 
            ${this.state.type_ == ListingType.Subscribed && 'active'}
            ${UserService.Instance.user == undefined ? 'disabled' : 'pointer'}
          `}
        >
          <input
            id={`${this.id}-subscribed`}
            type="radio"
            value={ListingType.Subscribed}
            checked={this.state.type_ == ListingType.Subscribed}
            onChange={linkEvent(this, this.handleTypeChange)}
            disabled={UserService.Instance.user == undefined}
          />
          {i18n.t('subscribed')}
        </label>
        <label
          className={`pointer btn btn-outline-secondary ${
            this.state.type_ == ListingType.All && 'active'
          }`}
        >
          <input
            id={`${this.id}-all`}
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
    i.props.onChange(event.target.value);
  }
}
