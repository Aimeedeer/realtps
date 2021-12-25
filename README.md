# RealTPS

WIP


## About the software

RealTPS is written in Rust. It contains three parts:
- [`realtps_import`] is the syncing server that keeps running jobs for
  - requesting block data from various blockchain's RPC clients
  - calculating TPS for each blockchain
  - writing and reading from JSON files as the database
- [`realtps_web`] is the [realtps.net] website using [Rocket] framework
- [`realtps_common`] is data structures that are shared between
  `realtps_import` and `realtps_web`
  - abstracted database trait `Db` and its JSON implementation `JsonDb`
  - RealTPS' `Block` data structure that is converted from different
    blockchains' block data
  - a list of `Chain`s RealTPS covered

[`realtps_import`]: src/realtps_import
[`realtps_web`]: src/realtps_web
[`realtps_common`]: src/realtps_common
[realtps.net]: https://realtps.net
[Rocket]: https://rocket.rs

