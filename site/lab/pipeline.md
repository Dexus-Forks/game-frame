# Rendering pipeline

A game draws each frame in five stages. The game's state becomes geometry. The geometry becomes
calls. A device turns the calls into pixels and shows them. A 3D card of 1997 took over only the
fourth stage. So a game that ran on a software renderer and on a card gave both the same calls,
and only the device changed.

Move the square below and watch every stage change. The numbers in stage 2 and the calls in
stage 3 are what a port must match exactly. The three pictures in stage 4 come from the same
calls. The guide [The rendering pipeline](/hardware/rendering-pipeline) explains each stage,
where it ran on each machine, and how to build a port the same way.

## Where each stage ran

Pick a machine to see who did each stage, what crossed the seam between the game and the
device, and what to check.

<ClientOnly>
  <PipelineExplorer />
</ClientOnly>

## One frame, three devices

<ClientOnly>
  <PipelineDemo />
</ClientOnly>

Everything here is drawn in your browser from a texture made by the page. No game's files are
used.
