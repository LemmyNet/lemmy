import { Component, linkEvent } from 'inferno';
import { SortType } from '../interfaces';
import { sortingHelpUrl } from '../utils';
import { i18n } from '../i18next';

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

  static getDerivedStateFromProps(props: any): SortSelectState {
    return {
      sort: props.sort,
    };
  }

  render() {
    return (
      <>
        <select
          value={this.state.sort}
          onChange={linkEvent(this, this.handleSortChange)}
          class="custom-select custom-select-sm w-auto mr-2"
        >
          <option disabled>{i18n.t('sort_type')}</option>
          {!this.props.hideHot && (
            <option value={SortType.Hot}>{i18n.t('hot')}</option>
          )}
          <option value={SortType.New}>{i18n.t('new')}</option>
          <option disabled>─────</option>
          <option value={SortType.TopDay}>{i18n.t('top_day')}</option>
          <option value={SortType.TopWeek}>{i18n.t('week')}</option>
          <option value={SortType.TopMonth}>{i18n.t('month')}</option>
          <option value={SortType.TopYear}>{i18n.t('year')}</option>
          <option value={SortType.TopAll}>{i18n.t('all')}</option>
        </select>
        <a
          className="text-muted"
          href={sortingHelpUrl}
          target="_blank"
          rel="noopener"
          title={i18n.t('sorting_help')}
        >
          <svg class={`icon icon-inline`}>
            <use xlinkHref="#icon-help-circle"></use>
          </svg>
        </a>
      </>
    );
  }

  handleSortChange(i: SortSelect, event: any) {
    i.props.onChange(event.target.value);
  }
}
