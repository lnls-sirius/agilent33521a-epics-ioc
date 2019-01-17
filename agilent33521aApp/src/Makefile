TOP = ../..

include $(TOP)/configure/CONFIG

# Build an IOC application
PROD_IOC = agilent33521a

DBD += agilent33521a.dbd

agilent33521a_DBD += base.dbd

agilent33521a_SRCS_DEFAULT += main.cpp
agilent33521a_SRCS += agilent33521a_registerRecordDeviceDriver.cpp

agilent33521a_LIBS += $(EPICS_BASE_IOC_LIBS)

include $(TOP)/configure/RULES
#----------------------------------------
#  ADD RULES AFTER THIS LINE

agilent33521a_DBD += asyn.dbd
agilent33521a_DBD += drvAsynIPPort.dbd
agilent33521a_DBD += stream.dbd
agilent33521a_DBD += asSupport.dbd

agilent33521a_LIBS += asyn
agilent33521a_LIBS += stream
agilent33521a_LIBS += autosave
