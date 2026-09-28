# TLS Caster Controller

This application is designed to be the full package for OBS streaming under TLS.

### Requirements
- [League Broadcast](https://bluebottle.gg)
- OBS/Streamlabs OBS
- [This application](https://github.com/MatrixSenpai/caster-control/releases/latest)

### Setup Steps
Ensure League Broadcast is installed. You will need to configure each team manually. All logos
for the teams are a part of this package. You will need to get credentials from MatrixSenpai for LB

Download the `cast.json` file from the [release page](https://github.com/MatrixSenpai/caster-control/releases/latest)
and import into OBS (`Scene Collection > Import`). Ensure audio settings are correct. You will need
to edit Local Caster and Remote Caster, as well as In Game for in-game audio.

Download the `caster-control-vX.X.X-windows.zip` file and unzip it into your caster directory. Then,
configure your OBS by going to `Tools > Websocket Server Settings`. Ensure port is 4455, and either
set or disable completely a server password (disable by unchecking `Enable Authentication`).

Copy `config.example.toml` and rename it `config.toml`. If you set a password, remove the "#" in front
of `obs_password` and set it there. If you changed the port, make sure to update it in `obs_port`.

### Casting Steps

Open VDO.ninja, create a room, and copy the guest link and add your camera and mic.

Open the `caster-control.exe` application **after OBS is opened**. It should automatically
open a web browser window with the dashboard, as well as configure items in OBS to use the info bar.

You will need to set the following items:
- Casters
  - Name
  - VDO.Ninja `Solo View` link (for audio and video)
- Team
  - Logo
  - Name
- Any ticker items
  - Matchup (Team A vs Team B)
  - Production by X
  - Casting by X
  - Any other items that should be shown

Once everything is set correctly, hit "Save" in the top right corner.

### Reporting issues

Next to `caster-control.exe` should be `caster_control.log`. Send that to MatrixSenpai on Discord with
a description of the issue.