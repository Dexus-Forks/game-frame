---
id: steam-client-wine
title: The Windows Steam client in the Wine prefix, for games that need steam_api
kind: layer
runtime: [emupc-wine, wine-host]
provides: [drm:steam, drm:steamstub]
requires: []
vars: [WINEPREFIX, STEAM_APP_ID, GAME_FILES, GAME_FOLDER, GAME_EXE]
seen: ['Banished (2014), in emupc-wine, 2026-10-24']
---

# The Windows Steam client in the Wine prefix, for games that need steam_api

## What it does

A game that calls `SteamAPI_Init` quits at start when no Steam client runs, often with no
window and exit code 53. This module installs the Windows Steam client in the game's Wine
prefix, puts the game in Steam's library, and starts it the way Steam would. The owner signs in
once. Agents never type the owner's credentials.

## Set up

```bash
steam="$WINEPREFIX/drive_c/Program Files (x86)/Steam"
curl -fLo /tmp/SteamSetup.exe https://cdn.cloudflare.steamstatic.com/client/installer/SteamSetup.exe
wine /tmp/SteamSetup.exe /S
(cd "$steam" && wine Steam.exe -silent -no-cef-sandbox -cef-disable-gpu -cef-disable-gpu-compositing >/tmp/steam.log 2>&1 &)
# PERSON: sign in to Steam once, through the VM's screen (VNC). Steam remembers the sign-in.

# Put the game in Steam's library. Started from any other folder, Steam stops it.
mkdir -p "$steam/steamapps/common/$GAME_FOLDER"
cp -R "$GAME_FILES/." "$steam/steamapps/common/$GAME_FOLDER/"
printf '"AppState"\n{\n\t"appid"\t\t"%s"\n\t"installdir"\t\t"%s"\n\t"StateFlags"\t\t"4"\n}\n' \
  "$STEAM_APP_ID" "$GAME_FOLDER" > "$steam/steamapps/appmanifest_$STEAM_APP_ID.acf"
printf '%s\n' "$STEAM_APP_ID" > "$steam/steamapps/common/$GAME_FOLDER/steam_appid.txt"

# The overlay crashes in Direct3D under Wine: switch it off by renaming its DLLs.
for f in GameOverlayRenderer64.dll GameOverlayRenderer.dll; do
  [[ -f "$steam/$f" ]] && mv "$steam/$f" "$steam/$f.off"
done

# Start the game with its app id in the environment, from its folder in the library.
export SteamAppId="$STEAM_APP_ID" SteamGameId="$STEAM_APP_ID"
(cd "$steam/steamapps/common/$GAME_FOLDER" && wine "$GAME_EXE" >/tmp/game.log 2>&1 &)
```

## Check

The game's window appears, and `/tmp/game.log` has no `SteamAPI_Init` failure. `pgrep -f
'[S]team.exe'` shows the client running.

## Pitfalls

- Exits at once, code 53, no window: Steam is not running or not signed in.
- Exits with 53 when started from the library: `steam_api` handed the start to `steam.exe`.
  Set `SteamAppId` and `SteamGameId` in the environment.
- "Application load error 6:0000065432": the game was started outside Steam's library. Copy it
  into `steamapps/common` and write the manifest.
- Stops after the splash screen and uses no CPU: the overlay crashed in
  `gameoverlayrenderer64.dll`. Rename the overlay DLLs.
- A Steam update can bring the overlay DLLs back. Rename them before each run.
