#!/usr/bin/bash

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

# Install polkit rules
cp ./99-reflectron.rules /etc/polkit-1/rules.d/
chown root:root /etc/polkit-1/rules.d/99-reflectron.rules
chmod 644 /etc/polkit-1/rules.d/99-reflectron.rules

# Create necessary directories
mkdir -p /opt/reflectron/images
mkdir /opt/reflectron/database
chown -R :reflectron /opt/reflectron
chmod -R 775 /opt/reflectron
chmod g+s /opt/reflectron/database

mkdir /var/log/reflectron
chown :reflectron /var/log/reflectron
chmod 775 /var/log/reflectron

echo "Setup completed. Please log out and log back in for group changes to take effect."