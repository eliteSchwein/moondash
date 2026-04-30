[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Version](https://img.shields.io/github/package-json/v/Ludwig-3D/moondash)
[![PR](https://img.shields.io/github/issues-pr/Ludwig-3D/moondash)](https://github.com/eliteSchwein/mooncord/pulls)
[![Issues](https://img.shields.io/github/issues/Ludwig-3D/moondash)](https://github.com/eliteSchwein/mooncord/issues)
<br>
#### Tested:
![Tested](https://img.shields.io/badge/rpi-4-brightgreen)
![Tested](https://img.shields.io/badge/rpi-3-brightgreen)

![Screenshot_1](/images/img.png)

## Discord Community Server

[![Discord](https://discord.com/api/guilds/865168027652456448/widget.png?style=banner2)](https://discord.gg/TdxUZFZtVa)

## What is Moondash?

Moondash is a Touch Interface for [Moonraker](https://github.com/Arksine/moonraker).  
It is heavily inspired by the [X1Plus](https://github.com/X1Plus/X1Plus) Project and the [Mainsail](https://github.com/mainsail-crew/mainsail).  
It supports the [AFC Klipper Addon](https://github.com/ArmoredTurtle/AFC-Klipper-Add-On) with multiple units (tested with boxturtles only sofar, feel free to sponsor other units :D).  
Its based of [Tauri](https://github.com/tauri-apps/tauri), [Vue](https://github.com/vuejs) and [Vuetify](https://vuetifyjs.com/en/), it uses [Labwc](https://github.com/labwc/labwc) as Compositer and only Supports Wayland.  
Moondash might start on X11 but it will throw alot of erros and warnings because its not designed for super old software.  
  
Moondash is kept super simple but yet powerful, for example you can define up to 3 shortcut buttons for Macros, including indicator state.  
You can also switch between Light and Darkmode, change the colors and also change languages (currently english and german only).  
There are also some handy tools, for example PID Tuning Tool.  
Moondash got greated because i find [Klipperscreen](https://github.com/KlipperScreen/KlipperScreen) overcomplicated, it offers too much for such a small screen and that leads to many many sub menus and odd controls.  
Klipperscreen is a awesome projects but yeah its time for a "minimalist" change.  

Keep in Mind, this project is around 80% vibe coded, because this is my first tauri project.  
The 20% are primary Frontend Changes to make it more unified and also more performant (the whole app needs less ram and cpu then klipper with many mcus and some plugins),  
so there will be glitches, odd code and other weird stuff. This will be patched out over time, follow the issues and the project for the progress.  

Important Note: only debian bookworm and trixie are supported, 32bit arm support is limited. 32bit support for x86 isnt present, i dont support stoneage tech. 64bit is fully supported for x86 and arm.

## How to Install?

```shell
git clone https://github.com/Ludwig-3D/moondash
cd moondash
./scripts/install.sh

# if you want you can remove the moondash repo, the updates will come over apt.
# but keep in mind, there might be some service and labwc config updates.
# i will announce on release if there are any changes to the service and labwc files.

cd ~
sudo rm -rf moondash
```  
  
  
  
## How to Uninstall?

```shell
sudo apt remove moondash
sudo systemctl disable --now moondash
sudo rm /etc/systemd/system/moondash.conf
sudo systemctl daemon reload
cd ~
rm -rf moondash
```