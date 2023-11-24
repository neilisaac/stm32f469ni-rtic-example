#!/bin/bash
set -euo pipefail

cargo flash --chip STM32F469NIHx --target thumbv7em-none-eabihf
