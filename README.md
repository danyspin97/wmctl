# wmctl

**wmctl** is a small tool to get and control wayland compositors (e.g. sway, Hyprland).
Currently it allows to get the information on connected outputs and to wait for
any to get connected or disconnected. It acts as a compositor agnostic interface
to be used in scripts.

## Dependencies

**wmctl** doesn't have any build time dependency, other than a working Rust compiler and a
working toolchain.

## Getting started

To build **wmctl** locally, run:

```bash
$ cargo build --release
```

You can get the connected outputs by running:

```bash
$ wmctl list-outputs
```

You can use **wmctl** for scripts that need to run when when an output gets connected
or disconnected by using the command `wmctl wait-for-output-changes`.

## Projects using wmctl

- [_lumactl_](https://github.com/danyspin97/wmctl)

## License

**wmctl** is licensed under the [GPL-3.0+](https://github.com/danyspin97/wmctl/blob/main/LICENSE.md) license.
