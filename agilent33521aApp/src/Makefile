TOP = ../..

include $(TOP)/configure/CONFIG

# Build an IOC application
PROD_IOC = agilent33521a

agilent33521a_SRCS_DEFAULT += main.cpp

agilent33521a_LIBS += $(EPICS_BASE_IOC_LIBS)

include $(TOP)/configure/RULES
