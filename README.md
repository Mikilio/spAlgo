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
    ❕ If you have direnv installed `devenv shell` will be called automatically otherwise call it manually.

### Commands

- `pull-data <region>` downloads graph data for specific region.
- `prepare-tests` prepare test by generating result to compare by.
- `cargo test --release` run all tests.
- `cargo install --version 0.11.0 iai-callgrind-runner` to install the callgrind runner
- `cargo bench` run all benchmarks. 
- `benchmark-profile` run benchmarks with profiling enabled.

### Available regions

        Name | Description
        _____|_______________________
        USA  | Full USA 	
        CTR  | Central USA 	
        W 	 | Western USA 	
        E 	 | Eastern USA 	
        LKS  | Great Lakes 	
        CAL  | California and Nevada 	
        NE 	 | Northeast USA 	
        NW 	 | Northwest USA 	
        FLA  | Florida 	
        COL  | Colorado 	
        BAY  | San Francisco Bay Area 	
        NY 	 | New York City"

## Implemented algorithms

### Priority Queues
- D-ary Heaps for d ∈ (2,4,8,16)
- D-ary Heaps without Lookup
- Sorted List
- Pairing Heap

### Searches
- single-source shortest path using Dijkstras algorithm
- shortest path queries using early abortion
- shortest path queries using bidirectional search
- all-pairs shortest path using Dijkstras algorithm
- all-pairs shortest path using Warshall-Floyd algorithm

## Disclaimer
This project is designed to be purely a reference implementation for my thesis. 
It will thus not be updated unless it generated enough interest. This library in
its current form is not ready for production, because it lacks compatibility and
some crucial state-of-art optimizations. Feel free to take a look at the thesis
for a full overview of performed optimizations.
