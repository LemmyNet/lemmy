import { Component } from 'inferno';
import * as moment from 'moment';
// import 'moment/locale/de';
import 'moment/locale/zh-cn';
import 'moment/locale/fr';
import 'moment/locale/sv';
import { getLanguage } from '../utils';
import { i18n } from '../i18next';

interface MomentTimeProps {
  data: {
    published?: string;
    when_?: string;
    updated?: string;
  }
}

export class MomentTime extends Component<MomentTimeProps, any> {

  constructor(props: any, context: any) {
    super(props, context);

    // Moment doesnt have zh, only zh-cn
    let lang = getLanguage();
    if (lang == 'zh') {
      lang = 'zh-cn';
    }

    moment.locale(lang);
  }

  render() {
    if (this.props.data.updated) {
      return (
        <span title={this.props.data.updated} className="font-italics">{i18n.t('modified')} {moment.utc(this.props.data.updated).fromNow()}</span>
      )
    } else {
      let str = this.props.data.published || this.props.data.when_;
      return (
        <span title={str}>{moment.utc(str).fromNow()}</span>
      )
    }
  }
}
