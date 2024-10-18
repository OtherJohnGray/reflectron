#!/usr/bin/bash

adduser reflectron
mkdir /opt/reflectron
chown reflectron:reflectron /opt/reflectron
mkdir /var/log/reflectron
chown reflectron:reflectron /var/log/reflectron
