< envPaths

epicsEnvSet("TOP", "../..")

< agilent33521a.config

####################################################

epicsEnvSet("STREAM_PROTOCOL_PATH", "$(TOP)/agilent33521aApp/Db")

## Register all support components
dbLoadDatabase("$(TOP)/dbd/agilent33521a.dbd",0,0)
agilent33521a_registerRecordDeviceDriver pdbbase

drvAsynIPPortConfigure("AGILENTPORT", "${DEVICE_IP}:${DEVICE_PORT} TCP", 0, 0, 0)

## Load record instances
dbLoadRecords("$(TOP)/agilent33521aApp/Db/egunTest.db","P=${P}, R=${R}, PORT=AGILENTPORT")

< save_restore.cmd

iocInit

