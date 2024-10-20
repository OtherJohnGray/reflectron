#!/usr/bin/bash

echo $SUDO_USER
# Check if script is run with sudo (not as root)
if [ "$EUID" -eq 0 ] && [ "$SUDO_USER" = "root" -o -z "$SUDO_USER" ]; then
    echo "This script must be run using sudo, not as root directly."
    echo "Please run: sudo $0"
    exit 1
elif [ "$EUID" -ne 0 ]; then
    echo "This script must be run with sudo privileges."
    echo "Please run: sudo $0"
    exit 1
fi

# Create reflectron group and add current user to it
groupadd reflectron
usermod -a -G reflectron $SUDO_USER

# Create necessary directories
mkdir -p /opt/reflectron/images
touch /opt/reflectron/database
chown -R :reflectron /opt/reflectron
chmod -R 775 /opt/reflectron

mkdir /var/log/reflectron
chown :reflectron /var/log/reflectron
chmod 775 /var/log/reflectron

# Create sudoers file for reflectron
SUDOERS_FILE="/etc/sudoers.d/reflectron"

cat << EOF | sudo tee "$SUDOERS_FILE" > /dev/null
# Allow group reflectron to add and remove virtual networks:
%reflectron ALL=(root) NOPASSWD: /sbin/ip link add name * type bridge
%reflectron ALL=(root) NOPASSWD: /sbin/ip tuntap add name * mode tap
%reflectron ALL=(root) NOPASSWD: /sbin/ip link set * up
%reflectron ALL=(root) NOPASSWD: /sbin/ip link delete *

# Allow group reflectron to run debootstrap
%reflectron ALL=(root) NOPASSWD: $(which debootstrap)

# Allow group reflectron to start VMs in the reflectron network namespace
%reflectron ALL=(root) NOPASSWD: /sbin/ip netns exec reflectron qemu-system-x86_64 *
EOF

# Set correct permissions for the sudoers file
sudo chmod 0440 "$SUDOERS_FILE"

echo "Setup completed. Please log out and log back in for group changes to take effect."