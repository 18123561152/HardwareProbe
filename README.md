# hardware-probe

跨平台 Rust 硬件检测库和命令行示例，可采集当前进程可见的 CPU、内存、磁盘及 GPU/加速器信息。

## 功能

- 检测 CPU 型号和物理内存总量。
- 检测当前工作目录所在盘/文件系统的可用空间，不累加其他磁盘。
- 尽力识别 GPU 厂商、设备名称和可报告的显存容量。
- 在 Linux、macOS、Windows 上通过编译期平台分发使用独立实现。
- 提供异步 `inspect()` API，将同步探测放入 Tokio 阻塞线程池。
- 硬件结果支持 Serde 序列化和反序列化。
- GPU 探测失败时降级，不影响 CPU、内存和磁盘信息的返回。

## 平台支持

| 平台 | GPU 探测方式 | 相关系统依赖 |
| --- | --- | --- |
| Linux | NVIDIA 使用 NVML；AMD/Intel/Other 通过 DRM sysfs | NVIDIA 驱动/NVML；`/sys/class/drm` |
| macOS | 解析 `system_profiler SPDisplaysDataType -json` | `system_profiler` |
| Windows | NVIDIA 使用 NVML；AMD/Intel/Other 通过 CIM 查询 | NVIDIA 驱动/NVML；PowerShell 和 `Get-CimInstance` |

GPU 信息依赖驱动、系统工具和权限，无法检测时加速器列表可能为空或不完整。Apple Silicon 使用统一内存，其 `memory_bytes` 输出系统物理内存总量，不代表独立 VRAM。其他操作系统会编译后备实现，调用探测接口时返回 `UnsupportedPlatform`。

## 运行示例

```sh
cargo run --bin hardware-probe
```

命令会将探测结果以 JSON 格式打印到标准输出。

## 作为库使用

在 Tokio 异步上下文中调用 `SystemHardwareProbe::inspect()`：

```rust
use hardware_probe::SystemHardwareProbe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = SystemHardwareProbe::new().inspect().await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}
```

在其他 Rust 项目中，可通过本地路径或 Git 依赖引入本 crate。

## 输出格式

```json
{
  "cpu": "Example CPU Model",
  "memory_bytes": 34359738368,
  "disk_available_bytes": 512000000000,
  "accelerators": [
    {
      "kind": "Nvidia",
      "name": "NVIDIA Example GPU",
      "memory_bytes": 12884901888
    }
  ]
}
```

`cpu` 为操作系统报告的 CPU 型号，无法获取时为 `"Unknown CPU"`。内存和磁盘容量单位均为字节。Apple Silicon 的 `memory_bytes` 是统一系统内存总量；其他加速器显存未知时为 `null`。没有检测到加速器时，`accelerators` 为空数组。

## 公共接口

- `SystemHardwareProbe::new()`：创建平台对应的硬件探测器。
- `SystemHardwareProbe::inspect()`：异步返回 `Result<HardwareProfile, HardwareProbeError>`。
- `HardwareProfile`：包含 `cpu`、`memory_bytes`、`disk_available_bytes` 和 `accelerators`。
- `AcceleratorProfile`：包含 `kind`、`name` 和可选的 `memory_bytes`。
- `AcceleratorKind`：加速器类别，包括 `Nvidia`、`Amd`、`Intel`、`Apple` 和 `Other`。
- `Platform`、`detect_platform()`、`current_os()`、`current_arch()`：提供编译目标平台信息。

详细字段语义、错误说明和契约映射见[接口文档](docs/interface.md)。

## 项目边界

本项目只采集基础硬件事实，不负责模型选择、任务调度、资源计费、性能压测、硬件健康监控、驱动安装，也不采集温度、功耗或利用率。结果仅反映当前进程通过操作系统接口可见的资源；容器和权限限制可能影响结果。

更多内容见[需求与项目边界](docs/requirements.md)和[项目内部结构](docs/architecture.md)。

## 开发与测试

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo check --all-targets
```

## 许可证

MIT 或 Apache-2.0，详见 `Cargo.toml`。
