#!/bin/sh

set -e
set +u

# Source environment
. ./checkEnv.sh

# Parse command-line options
. ./parseCMDOpts.sh "$@"

# Check last command return status
if [ $? -ne 0 ]; then
	echo "Could not parse command-line options" >&2
	exit 1
fi

if [ -z "$DEVICE_IP" ]; then
    echo "\$DEVICE_IP is not set, Please use -i option" >&2
    exit 7
fi

if [ -z "$DEVICE_PORT" ]; then
    DEVICE_PORT="5025"
fi

if [ -z "$DEVICE_TYPE" ]; then
    DEVICE_TYPE="Agilent33521a"
fi

if [ -z "$EPICS_CA_MAX_ARRAY_BYTES" ]; then
    export EPICS_CA_MAX_ARRAY_BYTES="50000000"
fi

cd "$IOC_BOOT_DIR"

if [ "$DEVICE_TYPE" = "EgunTest" ]; then
    P="$P" R="$R" DEVICE_IP="$DEVICE_IP" DEVICE_PORT="$DEVICE_PORT" "$IOC_BIN" stEgunTest.cmd
    exit 7
else
    P="$P" R="$R" DEVICE_IP="$DEVICE_IP" DEVICE_PORT="$DEVICE_PORT" "$IOC_BIN" stAgilent33521a.cmd
fi
