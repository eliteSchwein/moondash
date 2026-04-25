#!/bin/bash
set -e

green=$(echo -en "\e[92m")
yellow=$(echo -en "\e[93m")
red=$(echo -en "\e[91m")
cyan=$(echo -en "\e[96m")
default=$(echo -en "\e[39m")

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

MCSERVICENAME="moondash"
MCCONFIGFILE="/home/$(whoami)/printer_data/config/moondash.cfg"

status_msg(){ echo; echo -e "${yellow}###### $1${default}"; }
ok_msg(){ echo -e "${green}>>>>>> $1${default}"; }
warn_msg(){ echo -e "${red} $1${default}"; }
title_msg(){ echo -e "${cyan}$1${default}"; }

for ARGUMENT in "$@"; do
  KEY=$(echo "$ARGUMENT" | cut -f1 -d=)
  VALUE=$(echo "$ARGUMENT" | cut -f2- -d=)

  case "$KEY" in
    --config_path|--app_config)
      MCCONFIGFILE="${VALUE}"
      ;;
    --service_suffix)
      MCSERVICENAME="${MCSERVICENAME}_${VALUE}"
      ;;
  esac
done

if [[ ${UID} == '0' ]]; then
  warn_msg "You can't run this script as root!"
  exit 1
fi

questions() {
  title_msg "Welcome to the Moondash kiosk installer."

  status_msg "Please enter your Moondash config file"
  read -p "$cyan Config file (leave empty for $MCCONFIGFILE): $default" config_file

  if [[ "$config_file" != "" ]]; then
    MCCONFIGFILE="$config_file"
  fi

  ok_msg "Moondash config file set: $MCCONFIGFILE"
}

setup_custom_apt_repo_placeholder() {
  status_msg "Custom apt repository setup placeholder"
  # TODO: Add your custom apt repository here later.
  # Example later:
  # curl -fsSL https://YOUR_APT_SERVER/key.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/moondash.gpg
  # echo "deb [signed-by=/etc/apt/keyrings/moondash.gpg] https://YOUR_APT_SERVER stable main" | sudo tee /etc/apt/sources.list.d/moondash.list
}

install_packages() {
  status_msg "Update package data"
  sudo apt update

  status_msg "Install kiosk dependencies"
  sudo apt-get -y install --no-install-recommends \
    cage \
    seatd \
    dbus-user-session \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    libgl1-mesa-dri \
    libegl-mesa0 \
    libgles2-mesa \
    git \
    cmake \
    g++ \
    scdoc \
    libevdev-dev

  status_msg "Moondash deb install placeholder"
  # TODO: Later, after adding your apt server:
  # sudo apt-get -y install moondash
}

modify_user() {
  status_msg "Update user permissions"
  sudo usermod -aG video,render,input,tty "$USER"
  sudo loginctl enable-linger "$USER"
}

install_service() {
  "$SCRIPTPATH/generateService.sh" --app_config="$MCCONFIGFILE"
}

install_ydotool_service() {
  status_msg "Install ydotoold user service"

  if [[ -x "$SCRIPTPATH/installYdotoolService.sh" ]]; then
    "$SCRIPTPATH/installYdotoolService.sh"
  else
    warn_msg "installYdotoolService.sh not found or not executable; skipping ydotoold setup."
  fi
}

install_ydotool_binary() {
  status_msg "Install ydotool"

  if command -v ydotool >/dev/null 2>&1 && command -v ydotoold >/dev/null 2>&1; then
    ok_msg "ydotool already installed"
    return
  fi

  sudo apt-get -y install git cmake g++ scdoc libevdev-dev libuinput-dev

  BUILD_DIR="/tmp/ydotool-build"
  rm -rf "$BUILD_DIR"

  git clone https://github.com/ReimuNotMoe/ydotool "$BUILD_DIR"

  cmake -S "$BUILD_DIR" -B "$BUILD_DIR/build"
  cmake --build "$BUILD_DIR/build" -j"$(nproc)"
  sudo cmake --install "$BUILD_DIR/build"

  ok_msg "ydotool installed"
}

questions
setup_custom_apt_repo_placeholder
install_packages
install_ydotool_binary
modify_user
install_ydotool_service
install_service

ok_msg "Installation finished. Reboot is recommended."