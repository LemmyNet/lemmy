import { Component } from 'inferno';

interface BannerIconHeaderProps {
  banner?: string;
  icon?: string;
}

export class BannerIconHeader extends Component<BannerIconHeaderProps, any> {
  constructor(props: any, context: any) {
    super(props, context);
  }

  render() {
    return (
      <div class="position-relative mb-2">
        {this.props.banner && (
          <img src={this.props.banner} class="banner img-fluid" />
        )}
        {this.props.icon && (
          <img
            src={this.props.icon}
            className={`ml-2 mb-0 ${
              this.props.banner ? 'avatar-pushup' : ''
            } rounded-circle avatar-overlay`}
          />
        )}
      </div>
    );
  }
}
