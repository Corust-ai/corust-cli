# Contributing

This repository contains the installer script (`install.sh`), the issue
tracker, and distribution docs for the Corust CLI. **CLI source code lives in a
separate repository** and is not accepted here.

## Reporting bugs or requesting features

Open an issue using the
[issue templates](https://github.com/Corust-ai/corust-cli/issues/new/choose).
Please include:

- The version reported by `corust --version`
- Your OS and architecture (output of `uname -sm`)
- Steps to reproduce

## Contributing to `install.sh`

Pull requests that fix installer bugs or improve platform support are welcome.

1. Fork the repository and create a branch.
2. Test your changes locally on at least macOS and Linux.
3. Verify the script is POSIX-compliant by running it with `dash` or
   `checkbashisms`.
4. Open a pull request with a clear description.

## Contributing to the CLI itself

Source changes (features, bug fixes, refactors) should be proposed in the main
CLI repository. If you're unsure where that is, open an issue here and we'll
point you in the right direction.

## Updating docs

README changes and clarifications are welcome via PR.
