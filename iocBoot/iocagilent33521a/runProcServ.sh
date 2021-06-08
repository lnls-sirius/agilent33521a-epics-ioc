#!/bin/sh

set -e
set +u

# Parse command-line options
. ./parseCMDOpts.sh "$@"

UNIX_SOCKET=""
if [ -z "${DEVICE_TELNET_PORT}" ]; then
    UNIX_SOCKET="true"
fi

if [ -z "${AGILENT33521A_INSTANCE}" ]; then
   AGILENT33521A_INSTANCE="1"
fi

set -u

# Use UNIX socket telnet port is not set
if [ "${UNIX_SOCKET}" ]; then
    /usr/local/bin/procServ \
        --logfile - \
        --foreground \
        --name agilent33521a_${AGILENT33521A_INSTANCE} \
        --ignore ^C^D \
        unix:./procserv.sock \
            ./runAgilent33521a.sh "$@"
else
    /usr/local/bin/procServ \
        --logfile - \
        --foreground \
        --name agilent33521a_${AGILENT33521A_INSTANCE} \
        --ignore ^C^D \
        ${DEVICE_TELNET_PORT} \
            ./runAgilent33521a.sh "$@"
fi
