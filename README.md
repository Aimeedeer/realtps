# RealTPS

ReaoTPS measures the current number of transactions per second
committed by various blockchains.


## About the software

RealTPS is written in Rust. It contains three parts:
- [`realtps_import`] is the syncing server that keeps running jobs for
  - requesting block data from various blockchain's RPC clients
  - calculating TPS for each blockchain
  - writing and reading from JSON files as the database
- [`realtps_web`] is the [realtps.net] website building on top of the
  [Rocket] framework
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

## About the algorithm

RealTPS uses a simple method of counting the transactions in every
block over the time period spanning from one week ago until the
present moment, then dividing that total number of transactions by the
number of seconds from the beginning of the first block until the end
of the last block.


### Chain specific

A large proportion of transactions reported by Solana nodes are
validator `vote` transactions, which are part of Solana's consensus
mechanism.  As most chains do not expose this type of information as a
standard transaction, and to make a more useful comparison, we do not
include vote transactions in our TPS calculations.

