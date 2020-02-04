const {
  FuseBox,
  Sparky,
  EnvPlugin,
  CSSPlugin,
  WebIndexPlugin,
  QuantumPlugin,
} = require('fuse-box');
// const transformInferno = require('../../dist').default
const transformInferno = require('ts-transform-inferno').default;
const transformClasscat = require('ts-transform-classcat').default;
let fuse, app;
let isProduction = false;
// var setVersion = require('./set_version.js').setVersion;

Sparky.task('config', _ => {
  fuse = new FuseBox({
    homeDir: 'src',
    hash: isProduction,
    output: 'dist/$name.js',
    experimentalFeatures: true,
    cache: !isProduction,
    sourceMaps: !isProduction,
    transformers: {
      before: [transformClasscat(), transformInferno()],
    },
    alias: {
      locale: 'moment/locale',
    },
    plugins: [
      EnvPlugin({ NODE_ENV: isProduction ? 'production' : 'development' }),
      CSSPlugin(),
      WebIndexPlugin({
        title: 'Inferno Typescript FuseBox Example',
        template: 'src/index.html',
        path: isProduction ? '/static' : '/',
      }),
      isProduction &&
        QuantumPlugin({
          bakeApiIntoBundle: 'app',
          treeshake: true,
          uglify: true,
        }),
    ],
  });
  app = fuse.bundle('app').instructions('>index.tsx');
});
// Sparky.task('version', _ => setVersion());
Sparky.task('clean', _ => Sparky.src('dist/').clean('dist/'));
Sparky.task('env', _ => (isProduction = true));
Sparky.task('copy-assets', () =>
  Sparky.src('assets/**/**.*').dest(isProduction ? 'dist/' : 'dist/static')
);
Sparky.task('dev', ['clean', 'config', 'copy-assets'], _ => {
  fuse.dev();
  app.hmr().watch();
  return fuse.run();
});
Sparky.task('prod', ['clean', 'env', 'config', 'copy-assets'], _ => {
  // fuse.dev({ reload: true }); // remove after demo
  return fuse.run();
});
