<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/LemmyNet/lemmy.svg)
[![Build Status](https://cloud.drone.io/api/badges/LemmyNet/lemmy/status.svg)](https://cloud.drone.io/LemmyNet/lemmy/)
[![GitHub issues](https://img.shields.io/github/issues-raw/LemmyNet/lemmy.svg)](https://github.com/LemmyNet/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
[![Translation status](http://weblate.yerbamate.ml/widgets/lemmy/-/lemmy/svg-badge.svg)](http://weblate.yerbamate.ml/engage/lemmy/)
[![License](https://img.shields.io/github/license/LemmyNet/lemmy.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/LemmyNet/lemmy?style=social)
[![Delightful Humane Tech](https://codeberg.org/teaserbot-labs/delightful-humane-design/raw/branch/main/humane-tech-badge.svg)](https://codeberg.org/teaserbot-labs/delightful-humane-design)

</div>

<p align="center">
  <a href="../README.md">English</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.ru.md">Русский</a> |
  <span>汉语</span> |
  <a href="README.zh.hant.md">漢語</a> |
  <a href="README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-lemmy.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/LemmyNet/lemmy-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-lemmy.org">Lemmy</a></h3>
  <p align="center">
    一个联邦宇宙的链接聚合器和论坛。
    <br />
    <br />
    <a href="https://join-lemmy.org">加入 Lemmy</a>
    ·
    <a href="https://join-lemmy.org/docs/en/index.html">文档</a>
    ·
    <a href="https://matrix.to/#/#lemmy-space:matrix.org">Matrix 群组</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">报告缺陷</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">请求新特性</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/blob/main/RELEASES.md">发行版</a>
    ·
    <a href="https://join-lemmy.org/docs/en/code_of_conduct.html">行为准则</a>
  </p>
</p>

## 关于项目

| 桌面应用                                                                                                   | 移动应用                                                                                                    |
| ---------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/mobile_pic.webp) |

[Lemmy](https://github.com/LemmyNet/lemmy) 与 [Reddit](https://reddit.com)、[Lobste.rs](https://lobste.rs) 或 [Hacker News](https://news.ycombinator.com/) 等网站类似：你可以订阅你感兴趣的论坛，发布链接和讨论，然后进行投票或评论。但在幕后，Lemmy 和他们不同——任何人都可以很容易地运行一个服务器，所有服务器都是联邦式的（想想电子邮件），并连接到 [联邦宇宙](https://zh.wikipedia.org/wiki/%E8%81%94%E9%82%A6%E5%AE%87%E5%AE%99)。

对于一个链接聚合器来说，这意味着在一个服务器上注册的用户可以订阅任何其他服务器上的论坛，并可以与其他地方注册的用户进行讨论。

它是 Reddit 和其他链接聚合器的一个易于自托管的、分布式的替代方案，不受公司的控制和干涉。

每个 Lemmy 服务器都可以设置自己的管理政策；任命全站管理员和社区版主来阻止引战和钓鱼的用户，并培养一个健康、无毒的环境，让所有人都能放心地作出贡献。

### 为什么叫 Lemmy？

- 来自 [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U) 的主唱。
- 老式的 [电子游戏](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>)。
- [超级马里奥中的库巴](https://www.mariowiki.com/Lemmy_Koopa)。
- [毛茸茸的啮齿动物](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/)。

### 采用以下项目构建

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## 特性

- 开源，采用 [AGPL 协议](/LICENSE)。
- 可自托管，易于部署。
  - 附带 [Docker](https://join-lemmy.org/docs/en/administration/install_docker.html) 或 [Ansible](https://join-lemmy.org/docs/en/administration/install_ansible.html)。
- 干净、移动设备友好的界面。
  - 仅需用户名和密码就可以注册!
  - 支持用户头像。
  - 实时更新的评论串。
  - 类似旧版 Reddit 的评分功能 `(+/-)`。
  - 主题，有深色 / 浅色主题和 Solarized 主题。
  - Emoji 和自动补全。输入 `:` 开始。
  - 通过 `@` 提及用户，`!` 提及社区。
  - 在帖子和评论中都集成了图片上传功能。
  - 一个帖子可以由一个标题和自我文本的任何组合组成，一个 URL，或没有其他。
  - 评论回复和提及时的通知。
    - 通知可通过电子邮件发送。
    - 支持私信。
  - i18n（国际化）支持。
  - `All`、`Subscribed`、`Inbox`、`User` 和 `Community` 的 RSS / Atom 订阅。
- 支持多重发布。
  - 在创建新的帖子时，有 _相似帖子_ 的建议，对问答式社区很有帮助。
- 监管能力。
  - 公开的修改日志。
  - 可以把帖子在社区置顶。
  - 既有网站管理员，也有可以任命其他版主社区版主。
  - 可以锁定、删除和恢复帖子和评论。
  - 可以禁止和解禁社区和网站的用户。
  - 可以将网站和社区转让给其他人。
- 可以完全删除你的数据，替换所有的帖子和评论。
- NSFW 帖子 / 社区支持。
- 高性能。
  - 服务器采用 Rust 编写。
  - 前端 gzip 后约 `~80kB`。
  - 支持 arm64 架构和树莓派。

## 安装

- [Docker](https://join-lemmy.org/docs/en/administration/install_docker.html)
- [Ansible](https://join-lemmy.org/docs/en/administration/install_ansible.html)

## Lemmy 项目

### 应用

- [lemmy-ui - Lemmy 的官方网页应用](https://github.com/LemmyNet/lemmy-ui)
- [Lemmur - 一个 Lemmy 的移动客户端（支持安卓、Linux、Windows）](https://github.com/LemmurOrg/lemmur)
- [Jerboa - 一个由 Lemmy 的开发者打造的原生 Android 应用](https://github.com/dessalines/jerboa)
- [Remmel - 一个原生 iOS 应用](https://github.com/uuttff8/Lemmy-iOS)

### 库

- [lemmy-js-client](https://github.com/LemmyNet/lemmy-js-client)
- [Kotlin API (尚在开发)](https://github.com/eiknat/lemmy-client)
- [Dart API client](https://github.com/LemmurOrg/lemmy_api_client)

## 支持和捐助

Lemmy 是免费的开源软件，无广告，无营利，无风险投资。您的捐款直接支持我们全职开发这一项目。

- [在 Liberapay 上支持](https://liberapay.com/Lemmy)。
- [在 Patreon 上支持](https://www.patreon.com/dessalines)。
- [在 OpenCollective 上支持](https://opencollective.com/lemmy)。
- [赞助者列表](https://join-lemmy.org/sponsors)。

### 加密货币

- 比特币：`1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- 以太坊: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- 门罗币：`41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`
- 艾达币：`addr1q858t89l2ym6xmrugjs0af9cslfwvnvsh2xxp6x4dcez7pf5tushkp4wl7zxfhm2djp6gq60dk4cmc7seaza5p3slx0sakjutm`

## 贡献

- [贡献指南](https://join-lemmy.org/docs/en/contributing/contributing.html)
- [Docker 开发](https://join-lemmy.org/docs/en/contributing/docker_development.html)
- [本地开发](https://join-lemmy.org/docs/en/contributing/local_development.html)

### 翻译

如果你想帮助翻译，请至 [Weblate](https://weblate.yerbamate.ml/projects/lemmy/)；也可以 [翻译文档](https://github.com/LemmyNet/lemmy-docs#adding-a-new-language)。

## 联系

- [Mastodon](https://mastodon.social/@LemmyDev)
- [Lemmy 支持论坛](https://lemmy.ml/c/lemmy_support)

## 代码镜像

- [GitHub](https://github.com/LemmyNet/lemmy)
- [Gitea](https://yerbamate.ml/LemmyNet/lemmy)
- [Codeberg](https://codeberg.org/LemmyNet/lemmy)

## 致谢

Logo 由 Andy Cuccaro (@andycuccaro) 制作，采用 CC-BY-SA 4.0 协议释出。
