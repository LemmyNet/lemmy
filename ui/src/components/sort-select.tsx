import { Component, linkEvent } from 'inferno';
import { SortType } from '../interfaces';

import { T } from 'inferno-i18next';

interface SortSelectProps {
  sort: SortType;
  onChange?(val: SortType): any;
  hideHot?: boolean;
}

interface SortSelectState {
  sort: SortType;
}

export class SortSelect extends Component<SortSelectProps, SortSelectState> {
  private emptyState: SortSelectState = {
    sort: this.props.sort,
  };

  constructor(props: any, context: any) {
    super(props, context);
    this.state = this.emptyState;
  }

  render() {
    return (
      <select
        value={this.state.sort}
        onChange={linkEvent(this, this.handleSortChange)}
        class="custom-select custom-select-sm w-auto"
      >
        <option disabled>
          <T i18nKey="sort_type">#</T>
        </option>
        {!this.props.hideHot && (
          <option value={SortType.Hot}>
            <T i18nKey="hot">#</T>
          </option>
        )}
        <option value={SortType.New}>
          <T i18nKey="new">#</T>
        </option>
        <option disabled>─────</option>
        <option value={SortType.TopDay}>
          <T i18nKey="top_day">#</T>
        </option>
        <option value={SortType.TopWeek}>
          <T i18nKey="week">#</T>
        </option>
        <option value={SortType.TopMonth}>
          <T i18nKey="month">#</T>
        </option>
        <option value={SortType.TopYear}>
          <T i18nKey="year">#</T>
        </option>
        <option value={SortType.TopAll}>
          <T i18nKey="all">#</T>
        </option>
      </select>
    );
  }

  handleSortChange(i: SortSelect, event: any) {
    i.state.sort = Number(event.target.value);
    i.setState(i.state);
    i.props.onChange(i.state.sort);
  }
}
