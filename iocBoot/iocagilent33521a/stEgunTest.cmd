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

## Start any sequence programs
# No sequencer program

# Create manual trigger for Autosave
create_triggered_set("auto_settings_EgunTest.req", "${P}${R}SaveTrg", "P=${P}, R=${R}")
create_monitor_set("auto_settings_EgunTest.req", 5, "P=${P}, R=${R}")
set_savefile_name("auto_settings_EgunTest.req", "auto_settings_${P}${R}.sav")
