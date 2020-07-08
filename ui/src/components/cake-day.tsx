import { Component } from 'inferno';
import moment from 'moment';
import { i18n } from '../i18next';

interface CakeDayProps {
  creator_name: string;
  creator_published: string;
}

export class CakeDay extends Component<CakeDayProps, any> {
  render() {
    const { creator_name, creator_published } = this.props;

    return (
      this.isCakeDay(creator_published) && (
        <div
          className="mr-lg-2 d-inline-block unselectable pointer mx-2"
          data-tippy-content={this.cakeDayTippy(creator_name)}
        >
          <svg class="icon icon-inline">
            <use xlinkHref="#icon-cake"></use>
          </svg>
        </div>
      )
    );
  }

  isCakeDay(input: string): boolean {
    const userCreationDate = moment.utc(input).local();
    const currentDate = moment(new Date());

    return (
      userCreationDate.date() === currentDate.date() &&
      userCreationDate.month() === currentDate.month()
    );
  }

  cakeDayTippy(creator_name: string): string {
    return i18n.t('cake_day_info', { creator_name });
  }
}
