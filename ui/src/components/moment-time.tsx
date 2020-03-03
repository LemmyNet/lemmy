import { Component } from 'inferno';
import moment from 'moment';
import { getMomentLanguage, setupTippy } from '../utils';
import { i18n } from '../i18next';

interface MomentTimeProps {
  data: {
    published?: string;
    when_?: string;
    updated?: string;
  };
}

export class MomentTime extends Component<MomentTimeProps, any> {
  constructor(props: any, context: any) {
    super(props, context);

    let lang = getMomentLanguage();

    moment.locale(lang);
  }

  componentDidMount() {
    setupTippy();
  }

  render() {
    if (this.props.data.updated) {
      return (
        <span
          data-tippy-content={this.format(this.props.data.updated)}
          className="font-italics pointer unselectable"
        >
          {i18n.t('modified')} {moment.utc(this.props.data.updated).fromNow()}
        </span>
      );
    } else {
      let str = this.props.data.published || this.props.data.when_;
      return (
        <span
          className="pointer unselectable"
          data-tippy-content={this.format(str)}
        >
          {moment.utc(str).fromNow()}
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
