# ram

Real-time RAM and swap monitor for Linux with terminal charts.

Reads `/proc/meminfo` and `/proc/vmstat` to display live memory throughput (page alloc/free rates) and usage gauges in the terminal using [ratatui](https://github.com/ratatui/ratatui).

![ram screenshot](screenshot.png)

## Features

- Memory throughput chart (alloc/free MB/s) with auto-scaled Y-axis
- RAM and swap usage gauge bars with used/total labels
- Configurable refresh rate and scrollback window
- `q`, `Esc`, or `Ctrl+C` to quit

## Install

```bash
cargo install --path .
```

Or grab the binary from `target/release/ram` after:

```bash
cargo build --release
```

## Usage

```
ram                  # default 500ms refresh
ram -r 1000          # 1s refresh
ram -s 120           # 2 min scrollback (default 60s)
```

## Disclaimer

This software was generated with the assistance of AI (Claude, Anthropic). It is provided **as-is**, with **no warranty of any kind**, express or implied. The author(s) accept **no responsibility or liability** for any damage, data loss, or other issues arising from its use. Use entirely at your own risk. You are solely responsible for reviewing the code and determining its suitability for your environment before running it.

## License

MIT
