import os
log_file = 'C:/Users/wangx/AppData/Roaming/com.memflow.app/logs/memflow.log.2026-03-05'
with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
    lines = f.readlines()
    for line in lines:
        lower = line.lower()
        # 搜索关键日志
        if '未配置' in line or '占位' in line or 'placeholder' in lower or 'fallback' in lower:
            print(line.strip())
