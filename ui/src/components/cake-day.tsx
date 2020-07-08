import { Component } from 'inferno';
import { i18n } from '../i18next';

interface CakeDayProps {
  creator_name: string;
  is_post_creator?: boolean;
}

export class CakeDay extends Component<CakeDayProps, any> {
  render() {
    const { creator_name, is_post_creator } = this.props;

    return (
      <div
        className={`mr-lg-2 d-inline-block unselectable pointer${
          is_post_creator ? ' mx-2' : ''
        }`}
        data-tippy-content={this.cakeDayTippy(creator_name)}
      >
        <svg class="icon icon-inline">
          <use xlinkHref="#icon-cake"></use>
        </svg>
      </div>
    );
  }

  cakeDayTippy(creator_name: string): string {
    return i18n.t('cake_day_info', { creator_name });
  }
}
