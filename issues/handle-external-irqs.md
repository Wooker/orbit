# Handle external IRQs

Some target chips may have external interrupt controller. QEMU, for example,
has PLIC and cannot handle peripheral interrupts using current kernel interrupt
handling routine.

Need to create API to dispatch interrupt handling to a driver. This may involve
creating some sort of interface, or framework (like in Linux), to bindd some
drivers to kernel-related functionalities.

For orbit these frameworks are called _systems_. With dispatching interrupt
handling to drivers, orbit kernel should offload UART
