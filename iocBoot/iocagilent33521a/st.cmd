#!../../bin/linux-x86_64/agilent33521a

## You may have to change agilent33521a to something else
## everywhere it appears in this file

< envPaths

cd "${TOP}"

epicsEnvSet("STREAM_PROTOCOL_PATH", "$(TOP)/agilent33521aApp/db")

## Register all support components
dbLoadDatabase "dbd/agilent33521a.dbd"
agilent33521a_registerRecordDeviceDriver pdbbase

drvAsynIPPortConfigure("AGILENTPORT", "${IPADDR}:${IPPORT} TCP", 0, 0, 0)

## Load record instances
dbLoadRecords("db/agilent33521a.db","P=${P}, R=${R}, PORT=AGILENTPORT")

cd "${TOP}/iocBoot/${IOC}"
iocInit
