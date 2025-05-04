<div align="center">

![Logo](images/logo.png)

Exokernel-based distributed operating system for microcontrollers
</div>

# How to build

Orbit uses `bin` alias as main build command. It takes chip name as argument.
For example, to build orbit for `ch32v208wbu6`, run the following command:
```
  cargo bin ch32v208wbu6
```
Command output indicates chip name and applications included in the binary, for example:
```
  ...
  orbit-bin: Chip: ch32v208wbu6
  orbit-bin: Apps: ["calc", "blinky", "systemnumports"]
  ...
```

## Supported chips
- ch32v208wbu6
- ch32x035
