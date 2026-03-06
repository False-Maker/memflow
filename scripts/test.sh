#!/bin/bash
# MemFlow 测试脚本

echo "=== MemFlow 测试脚本 ==="
echo ""

# 检查依赖
echo "1. 检查依赖..."
if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm 未安装"
    exit 1
fi
echo "✅ pnpm 已安装"

if ! command -v cargo &> /dev/null; then
    echo "❌ cargo 未安装"
    exit 1
fi
echo "✅ cargo 已安装"

# 检查 Rust 版本
rust_version=$(rustc --version | cut -d' ' -f2)
echo "✅ Rust 版本: $rust_version"

# 检查 Node 版本
node_version=$(node --version)
echo "✅ Node 版本: $node_version"

echo ""
echo "2. 安装依赖..."
pnpm install

echo ""
echo "3. 检查 Rust 依赖并运行桌面端测试..."
cd src-tauri
cargo check
cargo test
cd ..

echo ""
echo "4. 运行 memflow-core 测试（包含 OCR 质量评估相关用例）..."
cargo test -p memflow-core

echo ""
echo "5. 类型检查..."
pnpm type-check

echo ""
echo "✅ 所有检查完成！"
echo ""
echo "下一步：运行 'pnpm tauri:dev' 启动开发服务器"

