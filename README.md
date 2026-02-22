# Pico I∴O×16 Firmware

This repository contains the firmware for a custom circuit board that uses
the Raspberry Pi Pico and Pico 2 to control 16 inputs and 16 outputs.
An input is a voltage scaled to 3.3V and read with the Pico's ADC. An output
is a PWM output. The board is controlled from a master devices via a custom RS485
protocol.

The board's intended use is for model train sets. The Pico's outputs are 
connected to motor drivers which can be used to drive the locomotives, LEDs
or other devices (by setting the PWM to 0% or 100% duty cycle). The inputs
can be used to read infra red sensors, buttons or to monitor the current
running through the outputs. Therefore both the board and the firmware
allow for a lot of different use cases.

This repository is divided into three parts:

- `pico_iox16_protocol` contains the protocol definitions and is shared between the
  master and the boards.
- `pico_iox16_firmware` contains the firmware's main loop but without concrete 
  hardware implementation.
- `pico_iox16_pico2` contains the concrete firmware for the Pico 2.

