# socksy

`Dirt simple SOCKS5 CONNECT proxy which binds a network interface`

## Build

```shell
cargo build --release
```

The executable will be `target/release/socksy`

## Usage

See `socksy --help` for up-to-date help info.

```
Dirt simple SOCKS5 CONNECT proxy which binds a network interface

All command line arguments can also be provided as environment variables in SCREAMING_SNAKE_CASE and prepended with `SOCKSY_`. If both are used, the command line arguments take precedence.

For example: `SOCKSY_LISTEN_ADDRESS=127.0.0.1:8888` is the same as `--listen-address=127.0.0.1:8888`

Usage: socksy [OPTIONS]

Options:
  -l, --listen-address <LISTEN_ADDRESS>
          Address to listen for incoming connections on.
          
          Default: 127.0.0.1:8888

  -b, --bind-interface <BIND_INTERFACE>
          Name of the interface to bind to for outgoing connections

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

A bind interface **MUST** be provided.

### Systemd

A parameterised SystemD service file is provided in `dist`.
The parameter is for the bind interface, so to enable the service and bind socksy to `foobar`, use the following:

```
# systemctl enable --now socksy@foobar
```

## License

MIT
