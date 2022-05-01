# node providers

- https://runnode.com/
- https://blockdaemon.com/marketplace/
- https://www.quicknode.com/
- https://nownodes.io/

# disk provisioning

sudo mke2fs -t ext4 /dev/nvme2n1 -i 4096

- -i 4096 creates an inode per 4096-byte block
- much more than the default