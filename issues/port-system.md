# Port system

Create system API for kernel ports. The port system will improve experience of
bringing initial support for newer chips because bringing up will not require
port driver implementation. Additionally, port will not rely only on UART.
Instead, port system implementors can use any protocol, thus, making the device
more versatile.
