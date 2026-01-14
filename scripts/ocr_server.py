#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
MemFlow OCR API 服务
基于 FastAPI + RapidOCR (OpenVINO)
"""
import io
import sys
from pathlib import Path

# 配置标准输出为 UTF-8
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

from fastapi import FastAPI, File, UploadFile
from fastapi.responses import JSONResponse
import uvicorn

# 初始化 RapidOCR（使用 OpenVINO）
print("[*] 正在初始化 RapidOCR (OpenVINO)...")

# 获取配置文件路径
script_dir = Path(__file__).parent.parent
config_path = script_dir / "default_rapidocr.yaml"

if config_path.exists():
    print(f"[*] 使用配置文件: {config_path}")
    from rapidocr import RapidOCR
    ocr_engine = RapidOCR(config_path=str(config_path))
else:
    print(f"[!] 配置文件不存在: {config_path}")
    print("[*] 使用默认配置（可能会使用 onnxruntime）")
    from rapidocr import RapidOCR
    ocr_engine = RapidOCR()

print("[OK] RapidOCR 初始化完成")

app = FastAPI(
    title="MemFlow OCR API",
    description="基于 RapidOCR 的 OCR 服务",
    version="1.0.0"
)


@app.get("/")
async def root():
    return {"status": "ok", "message": "MemFlow OCR API"}


@app.post("/ocr")
async def ocr(image: UploadFile = File(...)):
    """
    OCR 识别接口
    
    - **image**: 图片文件 (PNG/JPG/BMP)
    
    返回格式:
    ```json
    {
        "0": {"rec_txt": "识别文本", "dt_boxes": [[x,y]...], "score": "0.99"},
        "1": {...}
    }
    ```
    """
    try:
        # 读取图片
        contents = await image.read()
        
        # 执行 OCR (RapidOCR v3 返回 RapidOCROutput 对象)
        result = ocr_engine(contents)
        
        # 转换为 API 格式
        output = {}
        
        if result and result.txts:
            boxes = result.boxes if result.boxes is not None else []
            txts = result.txts if result.txts else []
            scores = result.scores if result.scores else []
            
            for i, (txt, score) in enumerate(zip(txts, scores)):
                box = boxes[i].tolist() if i < len(boxes) and hasattr(boxes[i], 'tolist') else []
                output[str(i)] = {
                    "rec_txt": txt,
                    "dt_boxes": box,
                    "score": str(score)
                }
        
        return JSONResponse(content=output)
        
    except Exception as e:
        import traceback
        traceback.print_exc()
        return JSONResponse(
            status_code=500,
            content={"error": str(e)}
        )


def main():
    import argparse
    parser = argparse.ArgumentParser(description="MemFlow OCR API Server")
    parser.add_argument("-ip", "--host", default="127.0.0.1", help="监听地址")
    parser.add_argument("-p", "--port", type=int, default=9003, help="监听端口")
    args = parser.parse_args()
    
    print(f"\n[*] 启动 OCR 服务: http://{args.host}:{args.port}")
    print(f"[*] API 文档: http://{args.host}:{args.port}/docs")
    print("[*] 按 Ctrl+C 停止服务\n")
    
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
