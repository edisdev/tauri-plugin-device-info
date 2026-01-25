# Linux

## Implementation

Linux support uses the filesystem-based approach, reading from `/sys` and `/proc`.

## Technologies Used

- **sysfs** (`/sys`) - DMI hardware information
- **procfs** (`/proc`) - Hostname and system info
- **xrandr** - Display refresh rate
- **sysinfo** crate - Storage information
- **battery** crate - Battery status

## Data Sources

| Information | File Path |
|-------------|-----------|
| Hostname | `/proc/sys/kernel/hostname` or `/etc/hostname` |
| Manufacturer | `/sys/class/dmi/id/sys_vendor` |
| Model | `/sys/class/dmi/id/product_name` |
| UUID | `/sys/class/dmi/id/product_uuid` or `/etc/machine-id` |
| Serial | `/sys/class/dmi/id/product_serial` |

## Testing

```bash
# Show all DMI information
ls -la /sys/class/dmi/id/
cat /sys/class/dmi/id/*

# Specific fields
cat /sys/class/dmi/id/sys_vendor
cat /sys/class/dmi/id/product_name
cat /sys/class/dmi/id/product_uuid

# Hostname
cat /proc/sys/kernel/hostname

# Refresh rate (X11)
xrandr --current | grep '*'
```

## Permission Requirements

Some files may require root access:

```bash
# Serial number often requires root
sudo cat /sys/class/dmi/id/product_serial

# UUID may require root on some systems
sudo cat /sys/class/dmi/id/product_uuid
```

## Special Cases

### ARM Devices

On ARM devices (Raspberry Pi, etc.), DMI information may not be available. Device tree is used instead:

```bash
cat /proc/device-tree/model
```

### Virtual Machines

Virtual machines show hypervisor information:

```bash
# QEMU/KVM
cat /sys/class/dmi/id/sys_vendor
# Output: QEMU
```

### Wayland

Display refresh rate detection uses xrandr (X11). On Wayland-only systems, refresh rate may be unavailable.
