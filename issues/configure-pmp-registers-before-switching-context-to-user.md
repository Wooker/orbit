# Configure PMP registers before switching contextt to user

PMP must be configured prior switching to user driver/application. Generally,
it must cover driver/application code for reading/execution. Additionally,
in case of the driver, PMP must cover claimed peripherals' registers for
reading/writing.
