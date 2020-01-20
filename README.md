# pass-rs

`pass-rs` attempts to be a replacement for [`pass`](https://passwordstore.org/),
written in Rust.

## Inspirations

While `pass-rs` was inspired by [`gopass`](https://github.com/gopasspw/gopass/),
it does not accomplish the same goals, nor does it try to. For example, you will
not find the ability to specify recipients on a per-secret basis (this is only
done on a store or substore basis, utilizing the `.gpg-id` file), it does not
expose an API for use in browser extensions, and it does not support any
cryptography protocol aside from OpenPGP.

## Security

I should probably add a big, red, flashy warning about this: **I am not security
minded in any way and therefore cannot guarantee the security of this
implementation**. I wrote this for fun. The past few weeks while I have been on
break from school, I wanted something to do with Rust aside from reading posts
on [r/rust](https://reddit.com/r/rust), as insightful as they may be. Much of
the code in this project is amateurish at best.

In another vein: there is only one instance of unsafe code, and that is in
[`util::set_permissions_recursive`](./src/util.rs) -- a call to `libc::getuid()`
to check if the user owns the path about be to operated on.

## Dependencies

Before you get started with installing and running `passrs`, you will need a few
dependencies: the first of which is the `gpgme` library.

## Installation

With that infodump out of the way, let's get to the fun stuff. At the moment,
only Linux systems are supported; however, more targets might come down the
line, as I become more competent in Rust.

To install the `passrs` binary, run:

``` sh
$ cargo install pass-rs
```

<sup>the binary intentionally does not have a hyphen, while the crate name
does</sup>

## Differences to unix pass
  - TODO
  - env vars that `pass` supports, but we do not:
    - PASSWORD_STORE_ENABLE_EXTENSIONS
    - PASSWORD_STORE_EXTENSIONS_DIR
    - PASSWORD_STORE_GPG_OPTS
    - GREPOPTIONS

## Licensing
- This software is licensed under the [MIT License](./LICENSE-MIT)
- Portions of this software are derived from [tui-rs](https://github.com/fdehau/tui-rs) examples, under the MIT
    license
- Portions of this software are derived from the [treeline](https://github.com/softprops/treeline) library, under the
    MIT license
- Portions of this software are derived from the [copy_dir](https://github.com/mdunsmuir/copy_dir) library, under the
    MIT license
- Portions of this software are derived from the [grep-cli](https://github.com/BurntSushi/ripgrep/tree/master/grep-cli) library, under the
    MIT license
- Portions of this software are derived from the [terminal_qrcode](https://github.com/calum/terminal_qrcode) library, under
    the MIT license
