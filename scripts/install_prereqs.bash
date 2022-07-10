#!/bin/bash
if [[ $EUID -ne 0 ]]; then
    echo "This script must be ran as root."
else
    # Install libssl-dev
    apt-get install libssl-dev -y
    # Install Flawfinder
    apt-get install flawfinder -y
fi