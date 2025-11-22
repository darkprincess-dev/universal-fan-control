import os
import glob

def scan_system():
    print(f"{'DEVICE NAME':<20} | {'ADDRESS (UID Prefix)':<20} | {'HWMON PATH'}")
    print("-" * 80)
    
    base_dir = '/sys/class/hwmon'
    for path in sorted(glob.glob(os.path.join(base_dir, 'hwmon*'))):
        # 1. 이름 읽기
        name = "Unknown"
        try:
            with open(os.path.join(path, 'name'), 'r') as f:
                name = f.read().strip()
        except: pass

        # 2. 주소(Address) 추출
        address = "Virtual/Unknown"
        try:
            link = os.path.join(path, 'device')
            if os.path.islink(link):
                real_path = os.path.realpath(link)
                # 경로의 맨 마지막 부분이 보통 주소임 (0000:0d:00.0 또는 0290)
                address = os.path.basename(real_path)
                
                # PCI 주소의 경우 앞에 0000:이 붙는데, 보통 떼고 씁니다 (취향 차이)
                if address.startswith("0000:"):
                    address = address[5:] 
        except: pass

        print(f"{name:<20} | {address:<20} | {path}")
        
        # 3. 내부 센서/팬 파일 미리보기 (유용한 것만)
        sensor_files = glob.glob(os.path.join(path, '*_input')) # 온도/팬속
        pwm_files = glob.glob(os.path.join(path, 'pwm[0-9]*'))  # 제어
        
        # 제어 가능한 파일(PWM)이 있으면 출력
        if pwm_files:
            pwm_indices = sorted([os.path.basename(p) for p in pwm_files])
            print(f"   ↳ Controls: {', '.join(pwm_indices)}")
            
        # 온도 센서가 있으면 출력 (너무 많으면 3개만)
        temps = sorted([os.path.basename(t) for t in sensor_files if 'temp' in t])
        if temps:
            print(f"   ↳ Sensors:  {', '.join(temps[:3])} ...")
        print("-" * 80)

if __name__ == "__main__":
    scan_system()