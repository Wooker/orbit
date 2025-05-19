<div align="center">

![Logo](images/logo.png)

Exokernel-based distributed operating system for microcontrollers
</div>

# Rust version
Use `rustc` version `1.89.0-nightly`.
```
  $ rustc -V
  rustc 1.89.0-nightly (4d051fb30 2025-05-18)
```

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
