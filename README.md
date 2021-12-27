# Real TPS

Real TPS measures the current number of transactions per second committed by
various blockchains.

## About the software

RealTPS is written in Rust. It contains three parts:

- [`realtps_import`] is the syncing server that keeps running jobs for
  - requesting block data from various blockchains' RPC clients,
    and storing them to disk,
  - calculating TPS for each blockchain.
- [`realtps_web`] is the [realtps.net] website, built on top of the
  [Rocket] framework.
- [`realtps_common`] is data structures that are shared between
  `realtps_import` and `realtps_web`
  - abstracted database trait `Db` and its JSON implementation `JsonDb`
  - RealTPS' `Block` data structure that is converted from different
    blockchains' block data
  - implementations of `Chain` for various RPC protocols

[`realtps_import`]: src/realtps_import
[`realtps_web`]: src/realtps_web
[`realtps_common`]: src/realtps_common
[realtps.net]: https://realtps.net
[Rocket]: https://rocket.rs

## About the algorithm

RealTPS uses a simple method of counting the transactions in every block over
the time period spanning from one week ago until the present moment, then
dividing that total number of transactions by the number of seconds from the
beginning of the first block until the end of the last block.

Full details are on [the website].

[the website]: https://realtps.net/about

## Run RealTPS yourself 

```
$ git clone https://github.com/Aimeedeer/realtps
$ cd realtps
$ RUST_LOG=info cargo run -p realtps_import
```

You'll see the `db` directory for fetched data under the root.
You can kill it any time or just keep it running.

With the data in `db`, you can see the list of results by running the website:

```
$ cargo run -p realtps_web
```

And check it in your browser.

To update data for a specific chain, run `realtps_import` with arguments.
e.g.

```
$ RUST_LOG=info cargo run -p realtps_import -- import --chain polygon
    Finished dev [unoptimized + debuginfo] target(s) in 0.33s
     Running `target/debug/realtps_import import --chain polygon`
[realtps_import] creating client for polygon at https://polygon-rpc.com
[realtps_import] node version for polygon: bor/v0.2.12-stable-488ea2bc/linux-amd64/go1.17
[realtps_import::import] beginning import for polygon
[realtps_import::import] importing at least 43020 blocks for polygon
[realtps_import::import] fast-forwarding chain polygon from block 23004376
[realtps_import::import] fast-forwarded chain polygon to block 23004283
```

Have fun!

## License

MIT
