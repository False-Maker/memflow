import sqlite3
import json
from datetime import datetime

db_path = 'C:/Users/wangx/AppData/Roaming/com.memflow.app/memflow.db'
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

print('=== 向量创建时间 ===')
cursor.execute('SELECT id, activity_id, created_at FROM vector_embeddings ORDER BY id DESC LIMIT 10')
for row in cursor.fetchall():
    ts = row[2]
    dt = datetime.fromtimestamp(ts) if ts else 'N/A'
    print(f'ID: {row[0]}, ActivityID: {row[1]}, 时间: {dt}')

print()
print('=== 检查配置中是否有新的向量 ===')
cursor.execute('''
    SELECT v.id, v.activity_id, a.ocr_text, a.created_at, v.created_at as vector_created
    FROM vector_embeddings v
    INNER JOIN activity_logs a ON v.activity_id = a.id
    ORDER BY v.id DESC
    LIMIT 5
''')
for row in cursor.fetchall():
    ocr = row[2]
    ocr_preview = ocr[:50] if ocr else 'NULL'
    print(f'ActivityID: {row[1]}')
    print(f'  OCR: {ocr_preview}...')
    print(f'  活动创建: {datetime.fromtimestamp(row[3]) if row[3] else "N/A"}')
    print(f'  向量创建: {datetime.fromtimestamp(row[4]) if row[4] else "N/A"}')
    print()

conn.close()
