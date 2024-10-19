from datetime import datetime, timezone
import grow_pylib
import sys

x = 1

def init():
    global x
    x = 10

def measure():
    global x
    t = datetime.now(timezone.utc)
    m = grow_pylib.WaterLevelMeasurement(t, "testy", x)
    print(sys.executable, sys.path)
    x += 1
    # print(x)
    return [m]
