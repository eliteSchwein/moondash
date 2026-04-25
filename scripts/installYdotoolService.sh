#!/bin/bash
set -e

green=$(echo -en "\e[92m")
yellow=$(echo -en "\e[93m")
red=$(echo -en "\e[91m")
default=$(echo -en "\e[39m")

status_msg(){ echo; echo -e "${yellow}###### $1${default}"; }
ok_msg(){ echo -e "${green}>>>>>> $1${default}"; }
warn_msg(){ echo -e "${red} $1${default}"; }

if [[ ${UID} == '0' ]]; then
  warn_msg "Run this as the normal kiosk user, not root."
  exit 1
fi

YDOTOOL_BIN="/usr/local/bin/ydotool"
YDOTOOLD_BIN="/usr/local/bin/ydotoold"

if [[ ! -x "$YDOTOOLD_BIN" ]]; then
  warn_msg "$YDOTOOLD_BIN not found. Build/install ydotool first."
  exit 1
fi

status_msg "Allow user access to uinput"
sudo usermod -aG input "$USER"

sudo tee /etc/udev/rules.d/80-uinput.rules > /dev/null <<'EOF'
KERNEL=="uinput", GROUP="input", MODE="0660", OPTIONS+="static_node=uinput"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger || true

status_msg "Install ydotoold user service"
sudo tee /etc/systemd/system/ydotoold.service > /dev/null <<EOF
[Unit]
Description=ydotool daemon
After=systemd-user-sessions.service
After=moonraker.service

[Service]
Type=simple
ExecStart=/usr/local/bin/ydotoold
Restart=always
RestartSec=1

[Install]
WantedBy=multi-user.target
EOF

loginctl enable-linger "$USER" || sudo loginctl enable-linger "$USER"

sudo systemctl daemon-reload
systemctl --user daemon-reload
systemctl --user enable ydotoold.service
systemctl --user restart ydotoold.service

ok_msg "ydotoold user service installed."

if [[ -x "$YDOTOOL_BIN" ]]; then
  status_msg "Test command:"
  echo "$YDOTOOL_BIN mousemove -a 2000 2000"
fi

warn_msg "If ydotool still fails until logout/reboot, reboot once so group and udev changes apply."