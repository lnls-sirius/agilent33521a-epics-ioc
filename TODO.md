TODO
====

List of features and tasks planned for the future.

## Error Handling

Currently no error handling is present. This can be a problem, especially when
changing one field (for example, the waveform function) results in changes to
another field (for example, the frequency due to the ramp function having a
smaller maximum frequency).

A way to deal with these errors is required. One idea would be to always request
the error status and update an error specific record with the result. The error
record could then fan-out an update to all `-RB` and `-Sts` fields.

Need to decide if alarms should be raised when errors occur.
