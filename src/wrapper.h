#include <libudev.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <mntent.h>
#include <sys/mount.h>

struct DeviceInfo {
    struct udev* udev;
    struct udev_device* block;
    struct udev_device* scsi;
    struct udev_device* usb;
};

