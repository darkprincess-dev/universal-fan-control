import argparse
import glob
import os
import sys
import time
import signal


origin_fan_mode_path = []
origin_fan_mode = []


def get_args():
    parser = argparse.ArgumentParser(description="Linux Universal PWM Fan Controller")
    parser.add_argument(
        "-t",
        "--temp",
        nargs="+",
        type=str,
        required=True,
        help="Enter the physical UID of the tempe device. Multiple are possible. Address and type are required. (ex) {address}@{type}@{index or _attribute}@{_attribute}",
    )
    parser.add_argument(
        "-tm",
        "--temp_mode",
        type=str,
        required=False,
        default="avg",
        help="Please write one of avg, min, max, or both. If none is provided, the average will be used.",
    )
    parser.add_argument(
        "-f",
        "--fan",
        nargs="+",
        type=str,
        required=True,
        help="Enter the physical UID of the fan device. Multiple are possible. Address and type are required. (ex) {address}@{type}@{index or _attribute}@{_attribute}",
    )
    parser.add_argument(
        "-fm",
        "--fan_mode",
        type=str,
        required=True,
        help="Fan activation properties (ex) {_attribute}",
    )
    parser.add_argument(
        "-fc",
        "--fan_config",
        nargs="+",
        type=str,
        required=True,
        help="Enter your desired temperature and fan speed. Multiple are possible. (ex) {temp}:{percent}",
    )
    return parser.parse_args()


def uid_to_path(uid: str) -> str:
    match len(buff := uid.strip().lower().split("@")):
        case 2:
            addr, type_ = buff
            idx = ""
            attr = ""
        case 3:
            addr, type_, idx = buff
            attr = ""
        case 4:
            addr, type_, idx, attr = buff
        case _:
            raise RuntimeError(f"{uid} is not found. Please check the UID.")

    hwmon = None
    for path in glob.glob("/sys/class/hwmon/hwmon*"):
        try:
            link = os.path.join(path, "device")
            if os.path.islink(link) and (addr in os.path.realpath(link).lower()):
                hwmon = path
                break
        except Exception:
            continue
    if hwmon is None:
        raise RuntimeError(f"{uid} is not found. Please check the UID.")

    path = os.path.join(hwmon, f"{type_}{idx}{attr}")
    if os.path.exists(path) == False:
        raise RuntimeError(f"{uid} is not found. Please check the UID.")

    return path


def temp_avg(data: list) -> int:
    return int(sum(data) / len(data))


def temp_min(data: list) -> list:
    return min(data)


def temp_max(data: list) -> list:
    return max(data)


def get_linear_pwm(current_temp: float, config: list[tuple[int, int]]) -> int:
    if current_temp <= config[0][0]:
        return config[0][1]
    if current_temp >= config[-1][0]:
        return config[-1][1]

    for i in range(len(config) - 1):
        t1, p1 = config[i]
        t2, p2 = config[i + 1]
        if t1 <= current_temp <= t2:
            ratio = (current_temp - t1) / (t2 - t1)
            interpolated_pwm = p1 + (p2 - p1) * ratio
            return int(interpolated_pwm)

    return config[-1][1]


def signal_handler(signum, frame):
    global origin_fan_mode_path
    global origin_fan_mode

    for i, p in enumerate(origin_fan_mode_path):
        with open(p, "w") as f:
            f.write(origin_fan_mode[i])
    sys.exit(0)


if __name__ == "__main__":
    if os.geteuid() != 0:
        sys.exit(1)
        raise RuntimeError(f"Error: This script requires root privileges (sudo).")

    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)

    args = get_args()

    temp_path = [uid_to_path(t) for t in args.temp]
    match args.temp_mode:
        case "avg":
            temp_func = temp_avg
        case "min":
            temp_func = temp_min
        case "max":
            temp_func = temp_max
        case _:
            raise RuntimeError(f"This temp mode does not exist. {args.temp_mode}")

    fan_path = [uid_to_path(f) for f in args.fan]
    for f_path in fan_path:
        fan_mode_path = f_path + args.fan_mode
        with open(fan_mode_path, "r") as f:
            origin_fan_mode_path.append(fan_mode_path)
            origin_fan_mode.append(f.read())
        with open(fan_mode_path, "w") as f:
            f.write("1")

    fan_config = []
    for c in args.fan_config:
        temp_thr, percent = map(int, c.split(":"))
        percent = max(0, min(100, percent))
        pwm_val = int(percent * 255 / 100)
        fan_config.append((temp_thr, pwm_val))
        fan_config = sorted(fan_config)

    temp_buff = [0.0] * len(temp_path)
    while True:
        for i, d in enumerate(temp_path):
            with open(d, "r") as f:
                temp_buff[i] = int(f.read().strip()) / 1000.0

        temp = temp_func(temp_buff)
        pwm = get_linear_pwm(temp, fan_config)

        for f_path in fan_path:
            with open(f_path, "w") as f:
                f.write(str(pwm))

        time.sleep(2)
