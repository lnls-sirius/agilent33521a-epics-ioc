#!/bin/sh

# Run run*.sh scripts with procServ
/usr/local/bin/procServ -f -n agilent33521a -i ^C^D 20000 ./runAgilent33521a.sh "$@"
