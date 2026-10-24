---
id: disc-drive
title: The game's disc as a CD-ROM drive with its label, for games that check for the disc
kind: layer
runtime: [wine-host]
provides: [media:cd, media:cd-check]
requires: [wine-host]
vars: [WINEPREFIX, DISC_DIR, DISC_LABEL]
seen: []
---

# The game's disc as a CD-ROM drive with its label, for games that check for the disc

## What it does

Many games look for the first drive whose type is CD-ROM and whose volume label matches, and
ask for the disc when they find none. Under Wine on the host, map a folder that holds the disc's
files as a CD-ROM drive with the disc's label. In `emupc-wine`, attach the disc with
`emupc.py init --disc` instead: Wine on Linux reads the type and label from the device.

## Set up

```bash
ln -sfn "$DISC_DIR" "$WINEPREFIX/dosdevices/d:"
# PERSON: in winecfg, Drives, select D:, Advanced, set the type to CD-ROM and the label to $DISC_LABEL.
```

## Check

`wine cmd /c vol d:` prints the disc's label.

## Pitfalls

- The game asks for its disc though the drive is there: some file failed to open
  ([insert-cd-file-open-failed](../../knowhow/insert-cd-file-open-failed.md)).
- The type and label can also be set in the registry, so a person need not open winecfg (not
  confirmed: write the keys here when a game proves them).
