#!/usr/bin/python3

import serial
from time import sleep

uart = serial.Serial('/dev/ttyUSB1', 9600, timeout=1)

# ascii_data
f = open("./ascii_data.txt", 'r')
readline = f.readline()
while readline:
    buf = readline.strip()
    data = serial.to_bytes(buf.encode())
    # print(data)
    uart.write(data)
    readline = f.readline()
f.close()

# bytes_data
f = open("./bytes_data.txt", 'r')
readline = f.readline()
while readline:
    buf = readline.strip().split(" ")
    buf = [int(f"0x{val}", 16) for val in buf]
    data = serial.to_bytes(buf)
    # print(data)
    uart.write(data)
    readline = f.readline()
f.close()
