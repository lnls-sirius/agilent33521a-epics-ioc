< envPaths
< agilent33521a.config

####################################################

epicsEnvSet("STREAM_PROTOCOL_PATH", "$(TOP)/agilent33521aApp/db")

## Register all support components
dbLoadDatabase("$(TOP)/dbd/agilent33521a.dbd",0,0)
agilent33521a_registerRecordDeviceDriver pdbbase

drvAsynIPPortConfigure("AGILENTPORT", "${DEVICE_IP}:${DEVICE_PORT} TCP", 0, 0, 0)

## Load record instances
dbLoadRecords("db/agilent33521a.db","P=${P}, R=${R}, PORT=AGILENTPORT")

< save_restore.cmd

cd "${TOP}/iocBoot/${IOC}"
iocInit

## Start any sequence programs
# No sequencer program

# Create manual trigger for Autosave
create_triggered_set("auto_settings_agilent33521a.req", "${P}${R}SaveTrg", "P=${P}, R=${R}")
