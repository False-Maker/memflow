import sqlite3
import json

db_path = 'C:/Users/wangx/AppData/Roaming/com.memflow.app/memflow.db'
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

# 查看向量统计
print('=== 向量统计 ===')
cursor.execute('SELECT COUNT(*) FROM vector_embeddings')
total = cursor.fetchone()[0]
print(f'总向量数: {total}')

cursor.execute("SELECT COUNT(*) FROM activity_logs WHERE ocr_text IS NOT NULL AND ocr_text != ''")
with_ocr = cursor.fetchone()[0]
print(f'有OCR文本的活动数: {with_ocr}')

cursor.execute('''
    SELECT COUNT(*) FROM activity_logs a
    LEFT JOIN vector_embeddings v ON a.id = v.activity_id
    WHERE a.ocr_text IS NOT NULL AND a.ocr_text != '' AND v.activity_id IS NULL
''')
pending = cursor.fetchone()[0]
print(f'待向量化的活动数: {pending}')

print()
print('=== 向量表示例 (前3条) ===')
cursor.execute('SELECT id, activity_id, embedding FROM vector_embeddings LIMIT 3')
for row in cursor.fetchall():
    print(f'ID: {row[0]}, ActivityID: {row[1]}')
    emb = json.loads(row[2])
    print(f'  向量维度: {len(emb)}')
    print(f'  前5个值: {emb[:5]}')

conn.close()
