# Records explorer

Drop a data file that holds a table of records, or a network capture of a game's messages.
Each record or message becomes a row, and each byte a coloured cell, so fields line up as
columns. A flat column is a constant, such as a magic number or a version. A smooth ramp is a counter.
Noise is data. The strip above the grid marks what each column looks like: constant, counter,
length, or a few values that may be the message type. Group the rows by that byte to see each
type's own layout.

For a data file, the explorer suggests record sizes from the distances at which bytes repeat.
Try the first suggestion, then step the size by one until the columns stand straight. For a
capture (.pcap or .pcapng from Wireshark or tcpdump), choose a flow and a direction.

In a project, `tools/formats/fields.py` prints the same table and draws the same picture,
and `tools/formats/pcap.py` lists and extracts the flows. [Network games](/observe/network)
covers capturing and reading a protocol.

<ClientOnly>
  <RecordsExplorer />
</ClientOnly>
