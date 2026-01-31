我们将对 `crates/memflow-mcp/Cargo.toml` 进行修复，以解决 Windows 环境下的 TLS 和版本冲突问题。

### 1. 解决版本冲突与依赖管理
目前 `Cargo.toml` 中显式依赖了 `ort-sys`，这是一种底层依赖，容易与 `fastembed` 内部引入的 `ort` 版本产生冲突或特性不一致。
我们将：
- **移除** 显式的 `ort-sys` 依赖。
- **添加** `ort` 依赖（版本 `2.0.0-rc.11`），这与 `fastembed` 2.1.1 内部使用的版本保持一致。
- 这样做可以让 `ort` 高层 crate 统一管理底层的 `ort-sys`，确保版本严格对齐。

### 2. 修复 TLS 协议缺失与下载问题
Windows 环境下，下载模型或动态库时经常因为 SSL/TLS 证书问题失败。
我们将：
- 在 `ort` 依赖中启用 `load-dynamic` 和 `copy-dylibs` 特性。
    - `load-dynamic`: 允许 `ort` 自动下载和加载 ONNX Runtime 动态库。
    - `copy-dylibs`: **关键修复**，确保在 Windows 上将下载的 `.dll` 文件复制到可执行文件目录，防止运行时找不到库。
- 确保下载器使用系统原生 TLS (Schannel)，这在 Windows 上比 `rustls` 更稳定且不需要额外配置根证书。如果 `ort` 默认使用 `reqwest`，它通常会自动适配；如果需要显式配置，我们会通过特性标志启用。

### 3. 修改后的 Cargo.toml 预览
```toml
[dependencies]
# ...
# Keep stable v2 fastembed
fastembed = "2.1.1"

# Explicitly depend on ort to fix Windows DLL issues and ensure version alignment
# load-dynamic: Auto-download ONNX Runtime
# copy-dylibs: Auto-copy DLLs to target dir (Critical for Windows)
ort = { version = "2.0.0-rc.11", features = ["load-dynamic", "copy-dylibs"] }
```

此方案保持了 `fastembed` 的稳定 v2 版本，同时修复了底层引擎在 Windows 上的部署问题。