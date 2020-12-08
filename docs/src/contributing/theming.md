# Theming Guide

Lemmy uses [Bootstrap v4](https://getbootstrap.com/), and very few custom css classes, so any bootstrap v4 compatible theme should work fine.

## Creating

- Use a tool like [bootstrap.build](https://bootstrap.build/) to create a bootstrap v4 theme. Export the `bootstrap.min.css` once you're done, and save the `_variables.scss` too.

## Testing

- To test out a theme, you can either use your browser's web tools, or a plugin like stylus to copy-paste a theme, when viewing Lemmy.

## Adding

1. Fork the [lemmy-ui](https://github.com/LemmyNet/lemmy-ui).
1. Copy `{my-theme-name}.min.css` to `src/assets/css/themes`. (You can also copy the `_variables.scss` here if you want).
1. Go to `src/shared/utils.ts` and add `{my-theme-name}` to the themes list.
1. Test locally
1. Do a pull request with those changes.
