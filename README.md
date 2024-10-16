# Reflectron
## production == test

Reflectron is a server configuration and change management tool that builds local test instances as Incus VMs, then mirrors them to production (either bare metal or virtual server) using ZFS replication. 
The generated instances have a read-only root filesystem on ZFS, and boot via zfsbootmenu with SSH support for remote decryption of datasets during boot. All configuration changes 
and updates are then made to the test machines and promoted as-is to the production machines. 

Test systems are identical to production systems down to mac address and ip address on an isolated double-nat virtual network, enabling full pre-production testing of routing, firewall rules, load balancing, and  other network changes.

Automated package manager updates can be configured along with automated smoke tests to prevent breaking updates from distribution packages from being promoted to production - which should be useful the next time the Linux kernel team decides to break ZFS!

## Prerequisites

The VM host running Reflectron requires:

- OpenZFS
- Incus

## Limitations

- Currently only builds Debian12 systems (but extensible if you would like to add others).

- The test systems do not attempt to duplicate the disk layout of the production machine, and instead rely on ZFS to abstract this detail away.

## Status
#### This is pre-alpha code under development, and not ready for production use.

## License
This project is licensed under the Affero General Public License v3.0 (AGPL-3.0).

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Support
If you have any questions or run into any problems, please open an issue in the GitHub repository.