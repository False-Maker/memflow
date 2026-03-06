-- 为活动日志增加 OCR 质量评估字段（与 memflow-core 保持一致）
-- 说明：
-- - ocr_cer: Character Error Rate（字符级错误率），0.0 越接近表示越好
-- - ocr_wer: Word  Error Rate（词级错误率），0.0 越接近表示越好
-- - ocr_quality: 归一化到 [0,1] 的综合评分，1.0 表示最佳
-- 这些字段默认为空，仅在需要做质量评估/回归分析时写入。

ALTER TABLE activity_logs ADD COLUMN ocr_cer REAL;
ALTER TABLE activity_logs ADD COLUMN ocr_wer REAL;
ALTER TABLE activity_logs ADD COLUMN ocr_quality REAL;

