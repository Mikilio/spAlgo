# spAlgo

**A collection of shortest path algorithms**

## Getting started
### Install Nix
```
sh <(curl -L https://nixos.org/nix/install) --daemon
```
### Install Cachix (recommended)
```
nix-env -iA cachix -f https://cachix.org/api/v1/install
cachix use devenv
```
### Install [devenv](https://github.com/cachix/devenv)
```
nix-env -if https://install.devenv.sh/latest
```
### Commands

- `devenv ci` builds your developer environment and makes sure that all checks pass.
- `devenv shell` activates your developer environment.
- `devenv update` updates and pins inputs from devenv.yaml into devenv.lock.
- `devenv gc` deletes unused environments to save disk space.
- `devenv up` starts processes.

    ❕ If you have direnv installed `devenv shell` will be called automatically

