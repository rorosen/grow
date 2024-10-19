from datetime import datetime, timezone
import grow_pylib

raise Exception('I know Python!')
print("init")
    # hum = 5

def measure():
    print("running")
    t = datetime.now(timezone.utc)
    m = grow_pylib.AirMeasurement(t, "test", 1, 2, 3)
    return [m]
