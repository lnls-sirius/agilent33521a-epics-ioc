#!/bin/sh

IOC_NAME="agilent33521a"

IOC_BOOT_DIR="$(dirname "$0")"
IOC_BIN_DIR="$IOC_BOOT_DIR/../../bin"

if [ -n "$EPICS_HOST_ARCH" ]; then
    IOC_BIN="../../bin/$EPICS_HOST_ARCH/$IOC_NAME"
elif [ "$(ls "$IOC_BIN_DIR" | wc -l)" -eq 1 ]; then
    HOST_ARCH_GUESS="$(ls "$IOC_BIN_DIR")"
    IOC_BIN="../../bin/$HOST_ARCH_GUESS/$IOC_NAME"
else
    if [ -z "$(ls "$IOC_BIN_DIR")" ]; then
        echo "No IOC binaries found Did you run make install in the root directory?" >&2
    else
        echo 'Multiple IOC binaries found. Please set the $EPICS_HOST_ARCH environment variable with the appropriate value.' >&2
        echo "    Available options: $(ls "$IOC_BIN_DIR")" >&2
    fi

    exit 1
fi

if [ -z "$IPPORT" ]; then
    IPPORT=5025
fi

while [ "$#" -gt 0 ]; do
    case "$1" in
        "-P") P="$2" ;;
        "-R") R="$2" ;;
        "-i"|"--ip-addr") IPADDR="$2" ;;
        "-p"|"--ip-port") IPPORT="$2" ;;
        *) echo "Usage:" >&2
            echo "  $0 -i IPADDR -p IPPORT [-P P_VAL] [-R R_VAL]" >&2
            echo >&2
            echo " Options:" >&2
            echo "  -i or --ip-addr    Configure IP address to connect to device" >&2
            echo "  -p or --ip-port    Configure IP port number to connect to device" >&2
            echo "  -P                 Configure value of \$(P) macro" >&2
            echo "  -R                 Configure value of \$(R) macro" >&2
            exit 2
    esac

    shift 2
done

if [ -z "$IPADDR" ]; then
    echo "IP address not set. Please use -i option or set \$IPADDR environment variable" >&2
    exit 3
fi

cd "$IOC_BOOT_DIR"

IPADDR="$IPADDR" IPPORT="$IPPORT" P="$P" R="$R" "$IOC_BIN" st.cmd
