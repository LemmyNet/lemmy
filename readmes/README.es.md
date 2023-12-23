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
  <span>Español</span> |
  <a href="README.ru.md">Русский</a> |
  <a href="README.zh.hans.md">汉语</a> |
  <a href="README.zh.hant.md">漢語</a> |
  <a href="README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-lemmy.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/LemmyNet/lemmy-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-lemmy.org">Lemmy</a></h3>
  <p align="center">
    Un agregador de enlaces / alternativa a Menéame - Reddit para el fediverso. 
    <br />
    <br />
    <a href="https://join-lemmy.org">Unirse a Lemmy</a>
    ·
    <a href="https://join-lemmy.org/docs/es/index.html">Documentación</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Reportar Errores (bugs)</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Solicitar Características</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/blob/main/RELEASES.md">Lanzamientos</a>
    ·
    <a href="https://join-lemmy.org/docs/es/code_of_conduct.html">Código de Conducta</a>
  </p>
</p>

## Sobre El Proyecto

| Escritorio                                                                                                 | Móvil                                                                                                       |
| ---------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/mobile_pic.webp) |

[Lemmy](https://github.com/LemmyNet/lemmy) es similar a sitios como [Menéame](https://www.meneame.net/), [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), o [Hacker News](https://news.ycombinator.com/): te subscribes a los foros que te interesan, publicas enlaces y debates, luego votas y comentas en ellos. Entre bastidores, es muy diferente; cualquiera puede gestionar fácilmente un servidor, y todos estos servidores son federados (piensa en el correo electrónico), y conectados al mismo universo, llamado [Fediverso](https://es.wikipedia.org/wiki/Fediverso).

Para un agregador de enlaces, esto significa que un usuario registrado en un servidor puede suscribirse a los foros de otro servidor, y puede mantener discusiones con usuarios registrados en otros lugares.

El objetivo general es crear una alternativa a reddit y otros agregadores de enlaces, fácilmente auto-hospedada, descentralizada, fuera de su control e intromisión corporativa.

Cada servidor lemmy puede establecer su propia política de moderación; nombrando a los administradores del sitio y a los moderadores de la comunidad para mantener alejados a los trolls, y fomentar un entorno saludable y no tóxico en el que puedan sentirse cómodos contribuyendo.

_Nota: Las APIs WebSocket y HTTP actualmente son inestables_

### ¿Por qué se llama Lemmy?

- Cantante principal de [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- El [videojuego de la vieja escuela](https://es.wikipedia.org/wiki/Lemmings).
- El [Koopa de Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- Los [roedores peludos](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

### Creado Con

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

# Características

- Código abierto, [Licencia AGPL](/LICENSE).
- Auto-hospedado, fácil de desplegar (deploy).
  - Viene con [Docker](#docker) y [Ansible](#ansible).
- Interfaz limpia y fácil de usar. Apta para dispositivos móviles.
  - Sólo se requiere como mínimo un nombre de usuario y una contraseñar para inscribirse!
  - Soporte de avatar de usuario.
  - Hilos de comentarios actualizados en directo.
  - Puntuaciones completas de los votos `(+/-)` como en el antiguo reddit.
  - Temas, incluidos los claros, los oscuros, y los solarizados.
  - Emojis con soporte de autocompletado. Empieza tecleando `:`
    - _Ejemplo_ `miau :cat:` => `miau 🐈`
  - Etiquetado de Usuarios con `@`, etiquetado de Comunidades con `!`.
    - _Ejemplo_ `@miguel@lemmy.ml me invitó a la comunidad !gaming@lemmy.ml`
  - Carga de imágenes integrada tanto en las publicaciones como en los comentarios.
  - Una publicación puede consistir en un título y cualquier combinación de texto propio, una URL o nada más.
  - Notificaciones, sobre las respuestas a los comentarios y cuando te etiquetan.
    - Las notificaciones se pueden enviar por correo electrónico.
    - Soporte para mensajes privados.
  - Soporte de i18n / internacionalización.
  - Fuentes RSS / Atom para Todo `All`, Suscrito `Subscribed`, Bandeja de entrada `inbox`, Usuario `User`, y Comunidad `Community`.
- Soporte para la publicación cruzada (cross-posting).
  - **búsqueda de publicaciones similares** al crear una nueva. Ideal para comunidades de preguntas y respuestas.
- Capacidades de moderación.
  - Registros públicos de moderación.
  - Puedes pegar las publicaciones a la parte superior de las comunidades.
  - Tanto los administradores del sitio, como los moderadores de la comunidad, pueden nombrar a otros moderadores.
  - Puedes bloquear, eliminar y restaurar publicaciones y comentarios.
  - Puedes banear y desbanear usuarios de las comunidades y del sitio.
  - Puedes transferir el sitio y las comunidades a otros.
- Puedes borrar completamente tus datos, reemplazando todas las publicaciones y comentarios.
- Soporte para publicaciones y comunidades NSFW.
- Alto rendimiento.
  - El servidor está escrito en rust.
  - El front end está comprimido (gzipped) en `~80kB`.
  - El front end funciona sin javascript (sólo lectura).
  - Soporta arm64 / Raspberry Pi.

## Instalación

- [Docker](https://join-lemmy.org/docs/es/administration/install_docker.html)
- [Ansible](https://join-lemmy.org/docs/es/administration/install_ansible.html)

## Proyectos de Lemmy

### Aplicaciones

- [lemmy-ui - La aplicación web oficial para lemmy](https://github.com/LemmyNet/lemmy-ui)
- [Lemmur - Un cliente móvil para Lemmy (Android, Linux, Windows)](https://github.com/LemmurOrg/lemmur)
- [Remmel - Una aplicación IOS nativa](https://github.com/uuttff8/Lemmy-iOS)

### Librerías

- [lemmy-js-client](https://github.com/LemmyNet/lemmy-js-client)
- [Kotlin API ( en desarrollo )](https://github.com/eiknat/lemmy-client)
- [Dart API client ( en desarrollo )](https://github.com/LemmurOrg/lemmy_api_client)

## Apoyo / Donación

Lemmy es un software libre y de código abierto, lo que significa que no hay publicidad, monetización o capital de riesgo, nunca. Tus donaciones apoyan directamente el desarrollo a tiempo completo del proyecto.

- [Apoya en Liberapay](https://liberapay.com/Lemmy).
- [Apoya en Patreon](https://www.patreon.com/dessalines).
- [Apoya en OpenCollective](https://opencollective.com/lemmy).
- [Lista de patrocinadores](https://join-lemmy.org/sponsors).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Contribuir

- [Instrucciones para contribuir](https://join-lemmy.org/docs/es/contributing/contributing.html)
- [Desarrollo por Docker](https://join-lemmy.org/docs/es/contributing/docker_development.html)
- [Desarrollo Local](https://join-lemmy.org/docs/es/contributing/local_development.html)

### Traducciones

Si quieres ayudar con la traducción, echa un vistazo a [Weblate](https://weblate.yerbamate.ml/projects/lemmy/). También puedes ayudar [traduciendo la documentación](https://github.com/LemmyNet/lemmy-docs#adding-a-new-language).

## Contacto

- [Mastodon](https://mastodon.social/@LemmyDev)
- [Matrix](https://matrix.to/#/#lemmy:matrix.org)

## Repositorios del código

- [GitHub](https://github.com/LemmyNet/lemmy)
- [Gitea](https://yerbamate.ml/LemmyNet/lemmy)
- [Codeberg](https://codeberg.org/LemmyNet/lemmy)

## Creditos

Logo hecho por Andy Cuccaro (@andycuccaro) bajo la licencia CC-BY-SA 4.0.
