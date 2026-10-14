# 接口文档

## 公开 API

crate 路径为 `hardware_probe`。所有公开字段和枚举均实现 serde 序列化与反序列化。

### `HardwareProfile`

```rust
pub struct HardwareProfile {
    pub cpu_threads: usize,
    pub memory_bytes: u64,
    pub disk_available_bytes: u64,
    pub accelerators: Vec<AcceleratorProfile>,
}
```

- `cpu_threads`：系统可见的逻辑 CPU 线程数，不是物理核心数。
- `memory_bytes`：物理内存总量，单位为字节。
- `disk_available_bytes`：进程当前工作目录所在文件系统的可用空间，单位为字节；Windows 对应工作目录所在卷，Linux/macOS 对应挂载该目录的文件系统。不累加其他磁盘。
- `accelerators`：成功识别的加速器列表。无 GPU 或 GPU 探测失败时可为空。

### `AcceleratorProfile`

```rust
pub struct AcceleratorProfile {
    pub kind: AcceleratorKind,
    pub name: String,
    pub memory_bytes: Option<u64>,
}
```

- `kind`：加速器厂商/类别。
- `name`：操作系统或 GPU 驱动报告的设备名称；Linux 无法取得型号时使用可读的厂商和 DRM 卡号。
- `memory_bytes`：设备可报告的专用显存容量，单位为字节。不可用或无法区分独立容量时为 `None`。

### `AcceleratorKind`

```rust
pub enum AcceleratorKind {
    Nvidia,
    Amd,
    Intel,
    Apple,
}
```

JSON 序列化值使用 Rust 变体名称（例如 `"Nvidia"`）。

### `Platform`

```rust
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Other,
}
```

表示当前编译目标对应的平台；JSON 序列化值使用变体名称。

### `HardwareProbeError`

```rust
pub enum HardwareProbeError {
    DetectionFailed(String),
    GpuDriverUnavailable(String),
    DiskError(String),
    UnsupportedPlatform(String),
}
```

- `DetectionFailed`：阻塞探测任务无法启动/完成等整体探测失败。
- `GpuDriverUnavailable`：平台内部 GPU 探测器用于标记驱动或 GPU 查询不可用；`inspect()` 会捕获这类 GPU 错误并降级，不会因 GPU 单独失败而返回此错误。
- `DiskError`：无法读取当前工作目录，或该目录无法匹配到 `sysinfo` 可见磁盘。
- `UnsupportedPlatform`：后备探测器在非 Linux/macOS/Windows 目标上被调用。

该类型实现 `std::error::Error` 和 `Display`。

### `SystemHardwareProbe::inspect()`

```rust
pub async fn inspect(&self) -> Result<HardwareProfile, HardwareProbeError>;
```

无参数（除接收者 `&self`）。通过 Tokio `spawn_blocking` 执行阻塞的系统探测；支持平台成功时返回 `HardwareProfile`。GPU 探测失败仅影响加速器列表。阻塞任务失败时返回 `DetectionFailed`；不支持目标返回 `UnsupportedPlatform`。

### `detect_platform()`

```rust
pub const fn detect_platform() -> Platform;
```

无参数，返回当前编译目标的平台枚举；未知目标返回 `Platform::Other`。

### `current_os()` 与 `current_arch()`

```rust
pub const fn current_os() -> &'static str;
pub const fn current_arch() -> &'static str;
```

无参数，分别返回 Rust 编译器的 `target_os` 和 `target_arch` 字符串，例如 `"windows"`、`"x86_64"`。返回值描述编译目标，而非运行时可变属性。

## 调用示例

```rust
use hardware_probe::SystemHardwareProbe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = SystemHardwareProbe::new().inspect().await?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}
```

## burncloud-node 契约映射

| 本项目字段 | 契约字段 | 含义/转换 |
| --- | --- | --- |
| `HardwareProfile::cpu_threads` | `cpu_threads` | 逻辑 CPU 线程数，保持 `usize` 数值 |
| `HardwareProfile::memory_bytes` | `memory_bytes` | 物理内存总量，字节 |
| `HardwareProfile::disk_available_bytes` | `disk_available_bytes` | 当前工作目录所在文件系统的可用空间，字节 |
| `HardwareProfile::accelerators` | `accelerators` | 加速器数组 |
| `AcceleratorProfile::kind` | `kind` | `Nvidia`、`Amd`、`Intel` 或 `Apple` |
| `AcceleratorProfile::name` | `name` | 设备名称字符串 |
| `AcceleratorProfile::memory_bytes` | `memory_bytes` | 显存字节数；`None` 序列化为 `null`，未知/统一内存不冒充专用显存 |
