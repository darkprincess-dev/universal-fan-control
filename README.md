# Fan Control

Linux PWM Fan Controller written in Rust. This tool allows you to monitor hardware temperatures and control fan speeds based on configurable temperature-to-PWM curves.

## Features

- **Hardware Scanning**: Automatically discover `hwmon` devices, temperature sensors, and PWM controls.
- **Dynamic Control**: Adjust fan speeds using linear interpolation based on real-time temperature data.
- **Aggregation Modes**: Support for `avg`, `min`, and `max` temperature calculation when using multiple sensors.
- **Safety**: Automatically restores original fan control modes (e.g., BIOS/Auto) upon termination (SIGINT/SIGTERM).
- **UID System**: Precise device targeting using a unique addressing system.

## Installation

### Prerequisites
- Rust (Cargo) installed.
- Linux system with `hwmon` support.
- Root privileges (for fan control).

### Build
```bash
cargo build --release
```
The binary will be located at `target/release/fan-control`.

## Usage

### 1. Scan System
Discover your hardware devices and their UIDs.
```bash
./fan-control scan
```
**Example Output:**
```
DEVICE NAME          | ADDRESS (UID Prefix) | HWMON PATH
--------------------------------------------------------------------------------
amdgpu               | 0000:0d:00.0         | /sys/class/hwmon/hwmon4
   ↳ Controls: pwm1
   ↳ Sensors:  temp1_input ...
--------------------------------------------------------------------------------
```

### 2. Run Controller
Start the fan control daemon. This command requires root privileges.

```bash
sudo ./fan-control run [OPTIONS]
```

#### Arguments
- `-t, --temp <UID>...`: One or more temperature sensor UIDs.
- `--tm, --temp_mode <MODE>`: Aggregation mode for multiple sensors (`avg`, `min`, `max`). [Default: `avg`]
- `-f, --fan <UID>...`: One or more fan PWM Uid(s).
- `--fm, --fan_mode <ATTR>`: The attribute to enable manual control (e.g., `_enable`).
- `--fc, --fan_config <TEMP:PERCENT>...`: Temperature to PWM percentage mapping.

#### UID Format
`{ADDRESS}@{TYPE}@{INDEX}@{ATTRIBUTE}`
- `ADDRESS`: Device address (e.g., `0000:0d:00.0` or `0290`).
- `TYPE`: File prefix (e.g., `temp` or `pwm`).
- `INDEX`: (Optional) Sensor/Fan number.
- `ATTRIBUTE`: (Optional) Specific file suffix (e.g., `_input`).

### Full Example
To control a GPU fan based on its temperature:
```bash
sudo ./fan-control run \
  --temp "0000:0d:00.0@temp@1@_input" \
  --fan "0000:0d:00.0@pwm@1" \
  --fm "_enable" \
  --fc "40:20" "60:50" "85:100"
```
*In this example, the fan will run at 20% PWM at 40°C, 50% at 60°C, and 100% at 85°C, with linear interpolation in between.*

## Restoring State
The application tracks the original state of the fan mode attributes. When you stop the program (e.g., pressing `Ctrl+C`), it will automatically write the original values back to the system to return fan control to the BIOS or previous driver settings.

## Running as a Systemd Service
To run `fan-control` automatically on boot, you can create a systemd service.

1. Create a service file: `/etc/systemd/system/fan-control.service`
```ini
[Unit]
Description=PWM Fan Controller
After=network.target

[Service]
Type=simple
# Replace /path/to/your/... with the actual absolute path
ExecStart=/path/to/your/fan-control run \
  --temp "YOUR_TEMP_UID" \
  --fan "YOUR_FAN_UID" \
  --fm "_enable" \
  --fc "40:20" "60:50" "85:100"
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

2. Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable fan-control
sudo systemctl start fan-control
```

