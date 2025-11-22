# universal-fan-control

- Available only on Linux. 
- Requires Python version 3.12 or later.
- There is no installation, just code execution.

## fan_control.py
- Code that controls the fan.
- When specifying a device, you must enter physical {address}@{type}@{index}@{_attribute}
- --temp : device_uid
- --temp_mode : avg or min or max default=avg
- --fan : device_uid
- --fan_mode : _attribute : Attribute value to use when activating the fan
- --fan_config : {temp}:{percent} ... : Multiple entries possible

## device_scan.py
- It shows various information such as the name, physical address, and type of the currently connected device.

## example of use
```
[Unit]
Description=4x mi50 32gb Fan Controller
After=multi-user.target

[Service]
Type=simple
User=root

WorkingDirectory=/opt/universal-fan-control
ExecStart=/usr/bin/python3 /opt/universal-fan-control/fan_control.py \
    --temp 0000:03:00.0@temp@1@_input \
           0000:07:00.0@temp@1@_input \
           0000:0a:00.0@temp@1@_input \
           0000:0d:00.0@temp@1@_input \
    --temp_mode max \
    --fan nct6775.656@pwm@6 \
    --fan_mode _enable \
    --fan_config 40:30 55:60 70:80 85:100

Restart=always
RestartSec=5
Environment=PYTHONUNBUFFERED=1

[Install]
WantedBy=multi-user.target
```
