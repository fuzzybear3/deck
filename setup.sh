#!/bin/bash

echo "Running steam deck setup script"

#!/usr/bin/env bash
# setup_steamdeck.sh  –  Quick-start SSH + writable-FS for Valve Steam Deck
# Run with:  sudo ./setup_steamdeck.sh

set -euo pipefail

########################################
# Helper functions
########################################
err()  { printf "\e[31mERROR:\e[0m %s\n" "$*" >&2; exit 1; }
info() { printf "\e[36m==>\e[0m %s\n" "$*"; }

# a) Detect Steam Deck by product name
if [[ -r /sys/class/dmi/id/product_name ]]; then
  prod=$(tr -d '\0' < /sys/class/dmi/id/product_name)
else
  prod="unknown"
fi

# b) Detect SteamOS in /etc/os-release
if [[ -r /etc/os-release ]]; then
  source /etc/os-release
  os_id=$ID
else
  os_id="unknown"
fi


# c) Require pacman to be available
command -v pacman >/dev/null || err "pacman not found – this doesn't look like SteamOS"

# d) Consolidated verdict
if [[ ! "$prod" =~ (Jupiter|Steam|Valve) ]] || [[ "$os_id" != "steamos" ]]; then
  err "This script is intended for a Valve Steam Deck running SteamOS; detected product='$prod' os_id='$os_id'"
fi
info "Steam Deck detected (product='$prod', os_id='$os_id'). Continuing..."


info "First you need to set a passward for the deck. Please enter a passward"

# passwd

sudo systemctl start sshd
sudo systemctl enable sshd


git clone https://github.com/fuzzybear3/deck.git

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo steamos-readonly disable

sudo pacman-key --init
sudo pacman-key --populate archlinux holo

sudo pacman -Syu

sudo pacman -S base-devel

sudo pacman -Sy --needed pkgconf alsa-lib

sudo pacman -S --overwrite='*' --noconfirm alsa-lib

sudo pacman -S --overwrite='*' --noconfirm systemd-libs
