# Agilent 33521A EPICS IOC

This is an EPICS IOC for the Agilent 33521A 30MHz Function/Arbitrary Waveform
Generator.

## Building

To build the IOC, you can follow the standard EPICS build procedure for IOC
applications. The following EPICS modules are required:

- Asyn (tested with version 4.26)
- StreamDevice (tested with version 2.7.7)

The path to the required EPICS modules should be configured as Makefile
variables in the file `configure/RELEASE`, as the example below:

    # file: configure/RELEASE.local
    SUPPORT = /opt/epics/synApps_5_8/support
    ASYN = $(SUPPORT)/asyn-4-26
    STREAM = /opt/epics/stream

Afterwards, from the root repository directory, run `make install`. At any time,
if a rebuild is required, simply run the following commands:

    make clean uninstall
    make install

## Running

To run the IOC application, you need to configure the following environment
variables:

- `P`: first part of the prefix for the PV names
- `R`: second part of the prefix for the PV names
- `DEVICE_IP`: IP address of the device to connect to
- `DEVICE_PORT`: IP port of the device to connect to (the default port on the device
  is usually 5025)

With the variables exported, you can run the `stAgilent33521a.cmd` file from the
`iocBoot/iocagilent33521a` directory:

    cd iocBoot/iocagilent33521a
    P="MYLAB:" R="TEST:" DEVICE_IP="192.168.1.100" DEVICE_PORT=5025 ./stAgilent33521a.cmd

If your shell fails to execute the `stAgilent33521a.cmd` file you can run the application
binary directly by passing the `stAgilent33521a.cmd` file name as a parameter:

    cd iocBoot/iocagilent33521a
    P="MYLAB:" R="TEST:" DEVICE_IP="192.168.1.100" DEVICE_PORT=5025 ../../bin/<HOST-ARCH>/agilent33521a stAgilent33521a.cmd

where `<HOST-ARCH>` is the output folder with the binary for your specific host
architecture. If you built the IOC with no cross-compilation configured, there
should only be one output directory.

## Supported Features

For now, the following features are supported:

- Enabling/Disabling the channel output;
- Selecting the waveform function;
- Configuring the waveform frequency, amplitude, DC offset and phase;
- Configuring square wave duty cycle;
- Configuring ramp function symmetry;
- Configuring pulse width and leading and trailing edge times;
- Configuring pseudo-random bit stream bit rate, edge time and sequence type;
- Configuring noise function bandwidth;
- Configuring arbitrary waveform samples file and sample rate.
- Loading arbitrary waveform file from array of points.
