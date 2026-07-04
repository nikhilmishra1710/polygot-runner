import subprocess
import os
import time

child = subprocess.Popen(["sleep", "30"])

print(child.pid, flush=True)

while True:
    time.sleep(1)