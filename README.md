# universal-fan-control

- Available only on Linux. 
- Requires Python version 3.12 or later.

## fan_control.py
- Code that controls the fan.
- When specifying a device, you must enter physical {address}@{type}@{index}@{_attribute}
- --temp : device_uid
- --temp_mode : avg or min or max default=avg
- --fan : device_uid
- --fan_mode : _attribute : Attribute value to use when activating the fan default=_mode
- --fan_config : {temp}:{percent} ... : Multiple entries possible

## device_scan.py
- It shows various information such as the name, physical address, and type of the currently connected device.
