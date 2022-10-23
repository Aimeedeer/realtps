# node providers

- https://runnode.com/
- https://blockdaemon.com/marketplace/
- https://www.quicknode.com/
- https://nownodes.io/

# disk provisioning

sudo mke2fs -t ext4 /dev/nvme2n1 -i 4096

- -i 4096 creates an inode per 4096-byte block
- much more than the default

sudo mount /dev/nvme2n1 /mnt/realtps2/

# swap

```
: /mnt/realtps2
$ sudo fallocate -l 2GB swapfile

: /mnt/realtps2
$ sudo chmod 600 swapfile

: /mnt/realtps2
$ sudo mkswap swapfile
Setting up swapspace version 1, size = 1.9 GiB (1999994880 bytes)
no label, UUID=c31f9c49-53fa-4f2f-bc0f-86c424bc2b01

: /mnt/realtps2
$ sudo swapon swapfile
```

# running

ROCKET_ADDRESS=10.0.0.61 cargo run -j1 --release -p realtps_web

cargo_run -j1 --release -p realtps_import -- import