#!/bin/sh

# Use default if not set
if [ -z "${AGILENT33521A_DEVICE_TELNET_PORT}" ]; then
    AGILENT33521A_DEVICE_TELNET_PORT=20000
fi

# Run run*.sh scripts with procServ
/usr/local/bin/procServ -f -n agilent33521a${AGILENT33521A_INSTANCE} -i ^C^D ${AGILENT33521A_DEVICE_TELNET_PORT} ./runAgilent33521a.sh "$@"
