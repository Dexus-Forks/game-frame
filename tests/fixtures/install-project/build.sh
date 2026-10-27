#!/bin/sh
# A stand-in build for tools/install/install_app.py's checks: out/game prints which build it is,
# the folder it runs in and its arguments.
mkdir -p out
printf '#!/bin/sh\necho "%s cwd=$PWD args=$*"\n' "$1" > out/game
printf '#!/bin/sh\n' > out/helper
chmod +x out/game out/helper
