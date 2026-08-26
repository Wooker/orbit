# Check claim status of peripheral on driver initialization

The kernel must check if the peripheral is free or has been claimed on driver
initialization. If the peripheral is already claimed, the driver must cancel its
initialization and unload from memory.

This probably alters API of application container *init* method.
