import os
log_file = 'C:/Users/wangx/AppData/Roaming/com.memflow.app/logs/memflow.log.2026-03-05'
with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
    lines = f.readlines()
    for line in lines:
        lower = line.lower()
        if 'warn' in lower and ('embed' in lower or 'api' in lower or '调用' in line or '失败' in line):
            print(line.strip())
