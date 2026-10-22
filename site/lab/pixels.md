# Pixel explorer

Drop a memory dump, a screen saved by a DOS game, or any file that may hold a picture, and try
its bytes as pixels. A wrong width shows as slanted stripes. Change it by one until the picture
stands straight, then move the offset to the start of the picture. A 64,000-byte block that
shows a screen at 320 by 200 is a raw 256-colour screen. Drop a palette as well (768 bytes of
RGB, or a file that holds one) to see the real colours.

The modes are those of the PC's video hardware ([2D graphics](/hardware/graphics-2d)). In a
project, `tools/formats/rawimage.py` writes the same picture as a PNG, for comparing with the
port's frames.

<ClientOnly>
  <PixelExplorer />
</ClientOnly>
