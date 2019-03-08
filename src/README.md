# Summary
This is a NES (Nintendo Entertainment System) emulator written in Rust using SDL2 for graphics and input.

## Why NES?
I have already written a [chip 8 emulator](https://github.com/maximveligan/chip_8), so the NES seemed like a good place to go next. It is a fairly well documented system, and is not too difficult for one person to finish. The NES is also a more rewarding system to emulate than the chip 8, since it has so many iconic games for it. The final reason for writing a NES emulator is that I wanted to learn more about how real hardware interfaces.

## Overview
The NES has three main processing units and a few external asics that all run in parallel. Due to the serial nature of software, actually having these components run in parallel is not feasible for this emulator. I use the catch up technique of synchronization where I run the CPU for one instruction, and then pass the amount of cycles elapsed to the APU PPU and any mapper that needs this information.
