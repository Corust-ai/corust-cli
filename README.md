# corust-cli

Installer and distribution channel for the **Corust CLI**, a terminal UI client
for the [Corust](https://corust.ai) agent.

> Source code lives in a separate repository. This repo only hosts the
> installer script, issue tracker, and release assets.

## Installation

### Quick install (recommended)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://corust.ai/install.sh | sh
```

Pin to a specific version:

```bash
CORUST_VERSION=v0.4.2 curl --proto '=https' --tlsv1.2 -sSf https://corust.ai/install.sh | sh
```

Install to a custom location:

```bash
INSTALL_DIR=/opt/bin curl --proto '=https' --tlsv1.2 -sSf https://corust.ai/install.sh | sh
```

### Homebrew (macOS / Linux)

```bash
brew install Corust-ai/cli/corust
```

### Manual download

Pre-built binaries for macOS (arm64, x64) and Linux (x64) are available on the
[Releases page](https://github.com/Corust-ai/homebrew-cli/releases).

## Usage

```bash
corust            # launch interactive TUI
corust exec "..."  # non-interactive mode
corust sessions    # list saved sessions
corust resume      # resume most recent session
```

Run `corust --help` for all options.

## Uninstall

```bash
rm "$(command -v corust)"
```

## Supported platforms

| OS      | Architecture | Status |
|---------|--------------|--------|
| macOS   | arm64        | ✅ |
| macOS   | x86_64       | ✅ |
| Linux   | x86_64       | ✅ |
| Linux   | arm64        | ❌ (planned) |
| Windows | x86_64       | ❌ (use manual download) |

## Issues & feature requests

Please file bugs and feature requests on this repository's
[issue tracker](https://github.com/Corust-ai/corust-cli/issues/new/choose).

## License

[MIT](LICENSE)
