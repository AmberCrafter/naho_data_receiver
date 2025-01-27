#!/bin/bash

sudo socat -d -d pty,link=/dev/ttyUSB0,raw,echo=0 pty,link=/dev/ttyUSB1,raw,echo=0
