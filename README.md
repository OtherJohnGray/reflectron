# Reflectron
## production == test

- Build and deploy bare metal or virtual servers
- ...with Root-on-ZFS and ZFS Boot Menu
- Deploy OS, configuration, and applications together atomically as a read-only ZFS snapshot
- Simulate your production machines ZPool disk layout and network in a virtualized test environment
- Manage changes with change scripts and helpers, including change rewind and replay
- Test and validate your application, configuration, and OS together as a single unit
- Toggle the routing of DNS-based traffic to the test environment during testing
- Promote changes to production rapidly using ZFS send, read-only clones, overlay mounts, and kexec.

Reflectron is a server configuration and change management tool that builds local test instances as virtual machines, then mirrors them to production (either bare metal or virtual server) using ZFS replication. 
The resulting production instances have a read-only root filesystem on ZFS, and boot via zfsbootmenu with SSH support for remote decryption of datasets during boot. All configuration changes 
and updates are then made to the test machines and promoted as-is to the production machines. 

Test systems are identical to production systems including disk layout, and mac address and ip address in an isolated network namespace, enabling full pre-production testing of routing, firewall rules, load balancing, and  other network changes. Reflectron toggles the network config of the test host to route traffic to test VMs during testing, and reverts it to allow deployment to production servers.

## Prerequisites

The VM host running Reflectron requires:

- OpenZFS
- QEMU

## Limitations

- Currently only builds Debian12 systems.

- Currently hard-codes VM CPU and memory quota.

- Currently hard-codes pool layout to RAIDz5 with no special, cache, or log devices

- Currently hard-codes swap to use MDRAID

## Status
#### This is pre-alpha code under development, and not ready for production use.

## Installation

1. Install Reflectron
```
git clone https://github.com/OtherJohnGray/reflectron.git
cd reflectron
cargo build
sudo ln -s $(pwd)/target/debug/reflectron /usr/local/bin/ref
```
2. Setup reflectron user and required directories
```
sudo ./setup.sh
```


## Usage 
1. Create the inital filesystem image (currently debian only)
```
ref image create debian
```
2. ...TBD. 

## License
This project is licensed under the Affero General Public License v3.0 (AGPL-3.0).

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Support
If you have any questions or run into any problems, please open an issue in the GitHub repository.