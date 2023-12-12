# Building Lemmy Images

Lemmy's images are meant to be **built** on `linux/amd64`,
but they can be **executed** on both `linux/amd64` and `linux/arm64`.

To do so we need to use a _cross toolchain_ whose goal is to build
**from** amd64 **to** arm64.

Namely, we need to link the _lemmy_server_ with `pq` and `openssl`
shared libraries and a few others, and they need to be in `arm64`,
indeed.

The toolchain we use to cross-compile is specifically tailored for
Lemmy's needs, see [the image repository][image-repo].

#### References

- [The Linux Documentation Project on Shared Libraries][tldp-lib]

[tldp-lib]: https://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html
[image-repo]: https://github.com/raskyld/lemmy-cross-toolchains
