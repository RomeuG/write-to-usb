#include "wrapper.h"
#include <libudev.h>

struct udev_device* get_child(struct udev* udev, struct udev_device* parent, const char* subsystem)
{
    struct udev_device* child = NULL;
    struct udev_enumerate *enumerate = udev_enumerate_new(udev);

    udev_enumerate_add_match_parent(enumerate, parent);
    udev_enumerate_add_match_subsystem(enumerate, subsystem);
    udev_enumerate_scan_devices(enumerate);

    struct udev_list_entry *devices = udev_enumerate_get_list_entry(enumerate);
    struct udev_list_entry *entry;

    udev_list_entry_foreach(entry, devices) {
        const char *path = udev_list_entry_get_name(entry);
        child = udev_device_new_from_syspath(udev, path);
        break;
    }

    udev_enumerate_unref(enumerate);
    return child;
}

// sysattr {idVendor, manufacturer, removable, idProduct, bDeviceClass, product}
// udevadm info --attribute-walk --path=$(udevadm info --query=path --name=/dev/sdg)
struct DeviceInfo* get_device_info_by_block_impl(struct udev* udev, const char* device_dev_path)
{
    struct DeviceInfo* device_info = NULL;
    struct udev_enumerate* enumerate = udev_enumerate_new(udev);

    udev_enumerate_add_match_subsystem(enumerate, "scsi");
    udev_enumerate_add_match_property(enumerate, "DEVTYPE", "scsi_device");
    udev_enumerate_scan_devices(enumerate);

    struct udev_list_entry *devices = udev_enumerate_get_list_entry(enumerate);
    struct udev_list_entry *entry;

    udev_list_entry_foreach(entry, devices) {
        const char* path = udev_list_entry_get_name(entry);
        struct udev_device* scsi = udev_device_new_from_syspath(udev, path);

        struct udev_device* block = get_child(udev, scsi, "block");
        struct udev_device* scsi_disk = get_child(udev, scsi, "scsi_disk");

        struct udev_device* usb
            = udev_device_get_parent_with_subsystem_devtype(scsi, "usb", "usb_device");

        if (block && scsi_disk && usb) {
            const char* device_path = udev_device_get_devnode(block);

            if (strcmp(device_path, device_dev_path) == 0) {
                device_info = malloc(sizeof(struct DeviceInfo));
                device_info->block = block;
                device_info->scsi = scsi;
                device_info->usb = usb;

                if (scsi_disk) {
                    udev_device_unref(scsi_disk);
                }

                break;
            }
        }

        if (block) {
            udev_device_unref(block);
        }

        if (scsi_disk) {
            udev_device_unref(scsi_disk);
        }

        udev_device_unref(scsi);
    }

    udev_enumerate_unref(enumerate);

    return device_info;
}

// by idVendor and idProduct
struct DeviceInfo* get_device_info_by_vp_impl(struct udev* udev, const char* id_vendor, const char* id_product)
{
    struct DeviceInfo* device_info = NULL;
    struct udev_enumerate* enumerate = udev_enumerate_new(udev);

    udev_enumerate_add_match_subsystem(enumerate, "scsi");
    udev_enumerate_add_match_property(enumerate, "DEVTYPE", "scsi_device");
    udev_enumerate_scan_devices(enumerate);

    struct udev_list_entry *devices = udev_enumerate_get_list_entry(enumerate);
    struct udev_list_entry *entry;

    udev_list_entry_foreach(entry, devices) {
        const char* path = udev_list_entry_get_name(entry);
        struct udev_device* scsi = udev_device_new_from_syspath(udev, path);

        struct udev_device* block = get_child(udev, scsi, "block");
        struct udev_device* scsi_disk = get_child(udev, scsi, "scsi_disk");

        struct udev_device* usb
            = udev_device_get_parent_with_subsystem_devtype(scsi, "usb", "usb_device");

        if (block && scsi_disk && usb) {
            const char* usb_id_vendor = udev_device_get_sysattr_value(usb, "idVendor");
            const char* usb_id_product = udev_device_get_sysattr_value(usb, "idProduct");

            if (strcmp(id_vendor, usb_id_vendor) == 0 && strcmp(id_product, usb_id_product) == 0) {
                device_info = malloc(sizeof(struct DeviceInfo));
                device_info->block = block;
                device_info->scsi = scsi;
                device_info->usb = usb;

                if (scsi_disk) {
                    udev_device_unref(scsi_disk);
                }

                break;
            }
        }

        if (block) {
            udev_device_unref(block);
        }

        if (scsi_disk) {
            udev_device_unref(scsi_disk);
        }

        udev_device_unref(scsi);
    }

    udev_enumerate_unref(enumerate);

    return device_info;
}

struct DeviceInfo* get_device_info_by_block(const char* device_path) {
    struct udev *udev = udev_new();

    struct DeviceInfo* device_info = get_device_info_by_block_impl(udev, device_path);
    if (device_info != NULL) {
        device_info->udev = udev;
    } else {
        udev_unref(udev);
    }

    return device_info;
}

struct DeviceInfo* get_device_info_by_vp(const char* id_vendor, const char* id_product) {
    struct udev *udev = udev_new();

    struct DeviceInfo* device_info = get_device_info_by_vp_impl(udev, id_vendor, id_product);
    if (device_info != NULL) {
        device_info->udev = udev;
    } else {
        udev_unref(udev);
    }

    return device_info;
}
