# Writers, editors and mods

A reader that decodes every file is only the first step of modding. A mod also needs a writer
that gives every file back as it was. It needs an editor that changes only what was edited. It
needs a way to load the changed file into the unchanged original. This page sets out those
steps in order. They come from the POD: Planet of Death project. Its changed text tables, menu
images, tracks and cars load in the original game with no patch ([case study](../case-studies/pod.md)).

## 1. A writer that gives every file back

Decode each file of the player's copy, write it back, and compare every byte. Only then change
one value.

```bash
.venv/bin/python tools/formats/validate.py decoders/tracks.py 'derived/extracted/**/*.trk' --write
```

`--write` calls the decoder's `encode(result)`. For each file that does not come back
identical, it names the first byte that differs ([re-format](../../skills/re-format/SKILL.md)).
Readers built for viewers lose the parts below. Writers must keep them
([round trip of every file](../knowhow/writer-round-trip-every-file.md)).

- **The stored form beside the friendly view**: raw palettes beside RGB, fixed point beside
  floats, each name field whole with the bytes after its terminator, the unused tail of the
  last block.
- **Pools and tables as stored.** A string pool may not be in key order. A pool rebuilt from
  the keys then gives other bytes. Append changed strings. When the pool ends without a
  terminator, add one first.
- **The reader's own walk.** A scan by signature is enough for a viewer. A writer must write
  from the boundaries the reader followed.
- **Containers recomputed after a change.** When entry offsets count raw bytes (checksum words
  included), a grown part moves every later entry. Test by growing one part and reading every
  entry to its end. Encode checksums again after any change.

## 2. An editor that changes only what was edited

- **Patch stored fields at offsets the reader recorded.** Never write positions back from a
  float view. A 32-bit float keeps 24 bits, so 16.16 positions beyond 256 units lose their low
  bits ([float views](../knowhow/editor-float-view-changes-bytes.md)). An edit adds a rounded
  fixed-point delta to the stored integer.
- **One edit type for the window and the command line.** Make every change one value of one
  type. Make the window and a command-line mode both produce those values. Then an agent, a
  test or a script can do everything a person can
  ([one edit type](../knowhow/context-editor-one-edit-type.md)).
- **Keep the bytes each edit replaced**, so that undo and redo are exact. Decode again after
  each edit, so that the view shows what will be saved.
- **Write derived data again.** Level files store bounds, normals and planes computed from the
  vertices. Find them before you move geometry. `tools/formats/derived.py` tests the usual
  candidates against every unexplained field over every record
  ([derived geometry data](../knowhow/context-geometry-derived-data.md)).
- **Settle the axes from the running game.** A level drawn mirrored in height can look right
  from above. Read an object's position from memory over geometry you know. Then find the face
  under it ([axes](../knowhow/axes-measure-in-the-running-game.md)).
- **Show the result before writing.** An 8-bit picture maps to the nearest palette colour.
  Show the mapped picture, so that the modder sees what the game will show.

An editor is trustworthy when it passes four tests. A save with no edits equals the original on
every file. Each kind of edit saves, reads back and finds exactly its change. An edit and its
undo give the original bytes. One edit of each kind loads in the original game.

## 3. Loading a changed file into the original

- **Find the file-open path first.** Many CD-era games open every data file through one
  wrapper that tries the install folder before the disc. A changed file at the same relative
  path under the install folder then replaces the disc's copy, with no patch
  ([install folder first](../knowhow/context-mods-install-folder-first.md)). Trace the
  wrapper and its path builder. List the folder numbers and the roots each one tries. Test
  with one changed string.
- **Keep a mod as a folder laid out like the disc**. Installing the mod is then a copy.
  Removing it is a delete:

  ```bash
  .venv/bin/python tools/live/modfolder.py install mods/blue-sky derived/wine/drive_c/GAME
  .venv/bin/python tools/live/modfolder.py list derived/wine/drive_c/GAME
  .venv/bin/python tools/live/modfolder.py clear derived/wine/drive_c/GAME
  ```

- **Check the value where the game uses it.** The game can later replace a value the loader
  read with one computed from settings. A handling mod in the file then changes nothing in the
  race. Read the value back from the running game's memory where the rules read it
  ([a value replaced at run time](../knowhow/context-mod-value-replaced-at-run-time.md)).
- **Judge a mod against a stock run made the same way.** Clear every mod before you record a
  reference run ([mods in a reference recording](../knowhow/mods-loaded-in-a-reference-recording.md)).

## 4. Keep additions apart

Writers, editors and mod loaders are additions to an exact port. Keep them behind build options.
Keep one build that has none of them, and a switch that turns them all off for one run
([re-port](../../skills/re-port/SKILL.md), "Build it so it can be checked").
