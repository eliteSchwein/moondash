#!/bin/bash
set -e

yellow=$(echo -en "\e[93m")
green=$(echo -en "\e[92m")
red=$(echo -en "\e[91m")
default=$(echo -en "\e[39m")

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
  esac
done

if [[ ${UID} == '0' ]]; then
  warn_msg "You can't run this script as root!"
  exit 1
fi

status_msg "Installing labwc autostart and config"

mkdir -p "$HOME/.config/labwc"

cat > "$HOME/.config/labwc/autostart" <<EOF
#!/bin/sh
unclutter --timeout 2 --jitter 1 --ignore-scrolling &
exec /usr/bin/moondash --app-config "$MCCONFIGFILE"
EOF

chmod +x "$HOME/.config/labwc/autostart"

cat > "$HOME/.config/labwc/rc.xml" <<'EOF'
<?xml version="1.0"?>
<openbox_config>
</openbox_config>
EOF

ok_msg "labwc config installed"