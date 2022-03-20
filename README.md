# write-to-usb

A small program that writes raw bytes to a device. I personally use this to write to a USB
device by its `vendorid` and `productid` instead of using the device file directly, with
the goal of not destroying my own system.

# Requirements

To run and compile this you need to be in a GNU/Linux system and have access to `libudev`.

# Usage

```
write-to-usb 0.1.0

USAGE:
    write-to-usb [OPTIONS] --vendorid <VENDORID> --productid <PRODUCTID> --input <INPUT>

OPTIONS:
    -b, --bskip <BSKIP>            Number of bytes to skip before writing [default: 131072]
    -h, --help                     Print help information
    -i, --input <INPUT>            Input file
    -p, --productid <PRODUCTID>    USB device product id
    -v, --vendorid <VENDORID>      USB device vendor id
    -V, --version                  Print version information
```

# License

Copyright 2022 Romeu Gomes

Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"), to deal in the Software
without restriction, including without limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
