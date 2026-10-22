# File explorer

Drop a file from your game to see what it holds before you decode it. The signature names the
format when it is a known one. In the byte map, blue is text and red speckle is compressed or encrypted data. Regular stripes are a table of fixed-size records (set the width
to a multiple of the record size and they line up). Black is padding. The entropy graph shows
the same along the file. Click anywhere on the map or the graph to see the bytes there.

[Identifying unknown files](/media/files) explains what to look for. In a project,
`tools/triage/filetypes.py` and `tools/triage/binview.py` give the same results for whole
folders.

<ClientOnly>
  <FileExplorer />
</ClientOnly>
