httpstime-rs is an implementation of the "Time over HTTPS" specification described by Poul-Henning Kamp here http://phk.freebsd.dk/time/20151129.html and is based off PHK's reference implementation.

Please only run against servers you control or have permission to use for this purpose. This implementation currently DOES NOT contain functionality to cache servers which refuse to provide time service.


[![Build Status](https://travis-ci.org/brayniac/httpstime-rs.svg?branch=master)](https://travis-ci.org/brayniac/httpstime-rs)
[![License](http://img.shields.io/:license-mit-blue.svg)](http://doge.mit-license.org)

## Build

httpstime-rs builds with Cargo which is distributed with Rust. If you don't already have Rust installed, I recommend multirust https://github.com/brson/multirust

Once you have Rust installed, you should cd into the source directory and run:

```shell
cargo build --release
```

This will produce a binary at ./target/release/httpstime-rs which you may copy into a convenient location for your system, or simply run following the usage below

## Usage

```shell
# show usage details
./target/release/httpstime-rs --help
# simple usage
./target/release/httpstime-rs -s www.example.com
```

## Features

* working Time over HTTPS implementation
* verifies SSL certificate of server

## Future work

* add server blacklist cache
* add X-HTTPSTIME header support
