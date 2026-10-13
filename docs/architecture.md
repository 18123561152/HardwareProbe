# 项目内部结构

## 目录树

```text
.
├── Cargo.toml                 # crate 元数据、通用依赖和按目标配置的 NVML 依赖
├── docs/
│   ├── requirements.md        # 需求、非目标、假设和验收标准
│   ├── interface.md           # 对外类型、方法、错误和契约映射
│   └── architecture.md        # 模块分层、平台策略和数据流
└── src/
    ├── lib.rs                 # crate 根、公开导出和平台辅助函数
    ├── main.rs                # 可运行的硬件探测与 JSON 输出示例
    ├── error.rs               # 统一结构化错误类型
    ├── types.rs               # 可序列化的硬件结果类型
    └── probe/
        ├── mod.rs             # 编译期平台选择及公开探测器再导出
        ├── common.rs          # sysinfo CPU、内存与磁盘采集
        ├── linux.rs           # Linux NVML 与 DRM sysfs GPU 检测
        ├── macos.rs           # macOS system_profiler JSON GPU 检测
        ├── windows.rs         # Windows NVML 与 PowerShell/CIM GPU 检测
        └── unsupported.rs     # 其它目标的 UnsupportedPlatform 后备实现
```

## 模块分层

```text
types ─┐
       ├──> probe/common ──> probe/{linux,macos,windows,unsupported}
error ─┘                            │
                                    v
                              probe/mod (cfg + pub use)
                                    │
                                    v
                                   lib
                                    │
                                  main
```

`types` 和 `error` 定义平台无关数据/错误；`common` 聚合共享资源；具体 OS 模块只负责该平台 GPU 数据；`probe/mod.rs` 在编译期挑选唯一实现；`lib.rs` 提供稳定的 crate 公共入口。

## 编译期分发

平台模块只在对应 target 编译，统一类型名由条件 `pub use` 导出。调用方始终使用 `hardware_probe::SystemHardwareProbe`：

```rust
#[cfg(target_os = "linux")]
pub use self::linux::SystemHardwareProbe;
#[cfg(target_os = "macos")]
pub use self::macos::SystemHardwareProbe;
#[cfg(target_os = "windows")]
pub use self::windows::SystemHardwareProbe;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub use self::unsupported::SystemHardwareProbe;
```

因此平台实现不会混编在同一 OS 文件里，也无需运行时分支选择探测器。`detect_platform()` 以相同的 `target_os` cfg 映射为 `Platform`。

## GPU 检测策略

| 平台 | NVIDIA 方案 | 其他 GPU 方案 | 统一内存处理 |
| --- | --- | --- | --- |
| Linux | `nvml-wrapper` 初始化 NVML 并枚举设备 | 遍历 `/sys/class/drm/card*/device`，按 PCI vendor ID 识别 AMD/Intel，读取 `mem_info_vram_total` | 不适用 |
| macOS | 解析 `system_profiler SPDisplaysDataType -json` 的设备信息 | 从同一 JSON 解析 Apple/AMD/Intel 厂商与型号 | Apple 设备显存为 `None`，不把系统内存总量填入独立显存 |
| Windows | `nvml-wrapper` 初始化 NVML 并枚举设备 | PowerShell 执行 `Get-CimInstance Win32_VideoController` 并解析 JSON；`AdapterRAM` 先安全解析为 `u64`，32 位上限按未知容量处理 | 不适用 |

Linux sysfs 枚举会跳过 NVIDIA vendor ID，避免与 NVML 返回的 NVIDIA 设备重复。单个设备读取失败时不影响其他设备。

## 数据流

1. `main.rs` 构造 `SystemHardwareProbe` 并调用 `inspect().await`。
2. `inspect()` 把同步工作提交给 `tokio::task::spawn_blocking`。
3. 编译期所选平台实现调用 `probe/common.rs`，通过 `sysinfo` 读取 CPU、物理内存和磁盘可用空间。
4. 平台模块探测 GPU；GPU 子探测器失败时丢弃失败分支结果，并保留其余来源已找到的设备。
5. 平台实现把通用资源和加速器组成 `HardwareProfile` 返回；调用方可直接使用或通过 serde 序列化。

## 错误处理与降级

- 公共 CPU/内存读取使用 `sysinfo`；磁盘汇总采用 checked addition，发生 `u64` 溢出时返回 `DiskError`。
- `spawn_blocking` 任务加入失败或 panic 时映射为 `DetectionFailed`。
- NVML 初始化、查询、系统命令启动/退出、JSON 解析、sysfs 读取错误均视为 GPU 子系统错误；上层吞并错误并返回空列表或部分成功列表。
- `GpuDriverUnavailable` 用于平台内部标记 GPU 驱动/API 不可用。因 GPU 降级约定，`inspect()` 不向调用方传播该变体。
- 不支持的编译目标由独立后备模块实现探测器，调用时返回 `UnsupportedPlatform`。

## 扩展点

- 新增 FreeBSD：在 `src/probe/freebsd.rs` 实现相同探测器接口，在 `probe/mod.rs` 添加 `#[cfg(target_os = "freebsd")]` 再导出，并更新 `detect_platform()`、Cargo 平台依赖、文档和交叉编译检查。
- 新增 GPU 指标：先在公共类型中添加平台无关字段并规定单位/可选语义，再由具备可靠 API 的平台模块分别采集；来源不可用时保留可选值和 GPU 降级原则，并补充 serde/解析测试。
