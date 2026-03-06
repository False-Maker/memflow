import sqlite3
import json
import os

db_path = 'C:/Users/wangx/AppData/Roaming/com.memflow.app/memflow.db'
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

print('=== 向量数据详情 ===')

# 查看所有向量
cursor.execute('SELECT id, activity_id, embedding, created_at FROM vector_embeddings ORDER BY id')
rows = cursor.fetchall()

# 统计唯一向量
unique_vectors = set()
for row in rows:
    unique_vectors.add(row[2])

print(f'总向量数: {len(rows)}')
print(f'唯一向量数: {len(unique_vectors)}')

print()
print('=== 检查是否为占位符向量 ===')

# 占位符向量的特征：从文本 hash 生成
# 相同文本应该生成相同的占位符向量
# 让我检查一下对应的 OCR 文本

cursor.execute('''
    SELECT a.id, a.ocr_text, v.id as vector_id
    FROM activity_logs a
    INNER JOIN vector_embeddings v ON a.id = v.activity_id
    ORDER BY a.id DESC
    LIMIT 10
''')

for row in cursor.fetchall():
    print(f'ActivityID: {row[0]}')
    print(f'OCR文本: {row[1][:100] if row[1] else "NULL"}...')
    print(f'向量ID: {row[2]}')
    print('---')

conn.close()
