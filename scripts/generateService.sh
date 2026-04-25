#!/bin/bash
set -e

green=$(echo -en "\e[92m")
yellow=$(echo -en "\e[93m")
red=$(echo -en "\e[91m")
default=$(echo -en "\e[39m")

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

MCSERVICENAME="moondash"
MCCONFIGFILE="/home/$(whoami)/printer_data/config/moondash.cfg"

status_msg(){ echo; echo -e "${yellow}###### $1${default}"; }
ok_msg(){ echo -e "${green}>>>>>> $1${default}"; }
warn_msg(){ echo -e "${red} $1${default}"; }

for ARGUMENT in "$@"; do
  KEY=$(echo "$ARGUMENT" | cut -f1 -d=)
  VALUE=$(echo "$ARGUMENT" | cut -f2- -d=)

  case "$KEY" in
    --config_path|--app_config)
      MCCONFIGFILE="${VALUE}"
      ;;
    --service_suffix)
      if [[ "$VALUE" != "" && "$VALUE" != "moondash" ]]; then
        MCSERVICENAME="${MCSERVICENAME}_${VALUE}"
      fi
      ;;
  esac
done

if [[ ${UID} == '0' ]]; then
  warn_msg "You can't run this script as root!"
  exit 1
fi

install_labwc_autostart() {
  status_msg "Installing labwc autostart"

  mkdir -p "$HOME/.config/labwc"

  cat > "$HOME/.config/labwc/autostart" <<EOF
#!/bin/sh
exec /usr/bin/moondash --app-config "$MCCONFIGFILE"
EOF

  chmod +x "$HOME/.config/labwc/autostart"
}

install_systemd_service() {
  status_msg "Installing Moondash kiosk unit file"

  MCUID="$(id -u "$USER")"

  SERVICE=$(<"$SCRIPTPATH/Moondash.service")
  SERVICE=$(sed "s/MC_USER/$USER/g" <<< "$SERVICE")
  SERVICE=$(sed "s/MC_UID/$MCUID/g" <<< "$SERVICE")

  echo "$SERVICE" | sudo tee "/etc/systemd/system/$MCSERVICENAME.service" > /dev/null

  sudo systemctl daemon-reload
  sudo systemctl enable "$MCSERVICENAME.service"
  sudo systemctl restart "$MCSERVICENAME.service"

  ok_msg "Installed and restarted $MCSERVICENAME.service"
}

install_labwc_autostart
install_systemd_service