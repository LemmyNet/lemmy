import { Component } from 'inferno';
import moment from 'moment';
import { getMomentLanguage, capitalizeFirstLetter } from '../utils';
import { i18n } from '../i18next';

interface MomentTimeProps {
  data: {
    published?: string;
    when_?: string;
    updated?: string;
  };
  showAgo?: boolean;
}

export class MomentTime extends Component<MomentTimeProps, any> {
  constructor(props: any, context: any) {
    super(props, context);

    let lang = getMomentLanguage();

    moment.locale(lang);
  }

  render() {
    if (this.props.data.updated) {
      return (
        <span
          data-tippy-content={`${capitalizeFirstLetter(
            i18n.t('modified')
          )} ${this.format(this.props.data.updated)}`}
          className="font-italics pointer unselectable"
        >
          <svg class="icon icon-inline mr-1">
            <use xlinkHref="#icon-edit-2"></use>
          </svg>
          {moment.utc(this.props.data.updated).fromNow(!this.props.showAgo)}
        </span>
      );
    } else {
      let str = this.props.data.published || this.props.data.when_;
      return (
        <span
          className="pointer unselectable"
          data-tippy-content={this.format(str)}
        >
          {moment.utc(str).fromNow(!this.props.showAgo)}
        </span>
      );
    }
  }

  format(input: string): string {
    return moment
      .utc(input)
      .local()
      .format('LLLL');
  }
}
