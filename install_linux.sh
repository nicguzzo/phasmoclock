#!/bin/sh


sudo sh -c "\
mkdir -p /opt/phasmoclock/assets && \
cp target/release/phasmoclock /opt/phasmoclock/ && \
cp assets/icon.png /opt/phasmoclock/assets"

# Copy the desktop file to the Linux application registry
desktop-file-install --dir=$HOME/.local/share/applications assets/phasmoclock.desktop

# Refresh the Linux application cache
update-desktop-database ~/.local/share/applications
