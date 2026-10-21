## Ask your agent

Run and capture the original. Copy this prompt into your coding agent:

```text
In my GameFrame project, get the original game running where we can watch and capture it.

1. Read docs/observe/running-the-original.md and the re-observe skill.
2. Pick the environment for this game (GameFrame's emulators, DOSBox-X, Wine, a console
   emulator). Tell me what it needs before you install anything.
3. Write a script that starts the game, plays a fixed input and saves frames and state.
4. Record next to each capture the game version, the settings, the seed and the inputs.
5. Keep every capture in the ignored derived/ folder.
```

# Observing the original

How to run the original game, watch it, drive it and capture what it does, so every claim in
the port can rest on something measured. Read these after the [taxonomy](../games/README.md)
has told you what kind of game you have.

- [Running the original](running-the-original.md): where to run it, what to capture, how to
  drive it by script, how to compare.
- [Inputs](inputs.md): every kind of input from outside, where each is read, and how a port
  takes them through one door.
- [Capturing the interface](ui.md): why game interfaces are custom, how to inventory every
  screen, and how to capture and rebuild them.
- [A model to query](object-model.md): an object model of the game, like a web page's, with
  doors for queries, input, time and capture, in both the original and the port.
