# 需求与项目边界

## 项目目标

`hardware-probe` 是一个跨平台 Rust crate，用于采集当前主机的基础硬件资源事实，并以可序列化的 `HardwareProfile` 提供给调用方。

## 范围内的功能（In Scope）

- 检测 CPU 逻辑线程数，字段为 `cpu_threads`。
- 检测物理内存总量，字段为 `memory_bytes`，单位为字节。
- 汇总当前系统可见磁盘的可用空间，字段为 `disk_available_bytes`，单位为字节。
- 检测可识别的 GPU/加速器，字段为 `accelerators`；每项包含厂商类别、名称和可选显存字节数。
- 支持 Linux、macOS 和 Windows，并通过编译期条件分发选择平台实现。
- 提供异步 `inspect()` API；将同步系统调用卸载到 Tokio 的阻塞线程池。
- GPU 子系统不可用或检测失败时，仍返回 CPU、内存、磁盘结果，并将 GPU 结果降级为空列表或保留已成功检测到的设备。
- 将硬件结果序列化为 JSON 等 serde 支持的格式。

## 范围外的功能（Out of Scope）

- 不进行模型选择、模型推理或模型兼容性判断。
- 不进行任务调度、设备分配、队列管理或集群编排。
- 不进行资源计费、配额管理、成本估算或用量结算。
- 不进行性能压测、基准测试或吞吐量预测。
- 不进行硬件健康监控、故障诊断、告警或历史趋势记录。
- 不安装、升级、配置或修复显卡驱动及系统依赖。
- 不采集 GPU/CPU 温度、风扇转速、功耗、利用率、时钟频率或进程占用。
- 不承诺返回设备的稳定排序、唯一 ID、拓扑、NUMA 关系或容器宿主机完整视图。
- 不承诺将 Apple Silicon 统一内存拆分成 CPU/GPU 专属容量；Apple GPU 的独立显存返回 `None`。

## 非功能性需求

- 跨平台支持 Linux、macOS 和 Windows；其它目标编译时暴露后备探测器并报告 `UnsupportedPlatform`。
- Linux、macOS、Windows 实现由 `#[cfg(target_os = "...")]` 在编译期选择，对调用方隐藏平台差异。
- GPU 探测错误不得使整份硬件探测失败。
- `inspect()` 不在 Tokio 异步执行器线程中直接执行阻塞探测。
- 除非后续平台 API 确有需要，不引入 `unsafe`；本项目源码不使用 `unsafe`。
- 公开数据类型可通过 serde 序列化和反序列化。
- 错误使用 `thiserror` 提供结构化、可显示的诊断。

## 约束与假设

- CPU、内存、磁盘信息依赖 `sysinfo` 所支持的系统接口及当前用户可见的资源视图。
- NVIDIA 显卡信息依赖目标系统上可加载的 NVML 库和可访问的驱动；Rust 依赖本身不捆绑显卡驱动。
- Linux 的 AMD/Intel 显卡识别依赖可读的 `/sys/class/drm/card*/device` sysfs 节点。
- macOS 的显卡信息依赖可执行 `system_profiler SPDisplaysDataType -json`。
- Windows 的非 NVIDIA 显卡信息依赖 PowerShell 和 `Get-CimInstance Win32_VideoController`。
- 调用方在受信任的本机环境运行；PowerShell 命令固定，不接受外部命令字符串或用户参数。
- 容器、权限、驱动、挂载命名空间和系统策略可能限制可见数据；结果仅表示探测进程可见的信息。

## 验收标准

- 在支持平台上，`inspect().await` 返回 CPU 逻辑线程数、物理内存总量、磁盘可用空间总和和加速器列表。
- 在没有 GPU 或 GPU 驱动不可用的 Linux 容器中，`inspect()` 仍返回 `Ok`，且 `accelerators` 可以为空。
- Linux sysfs 中 AMD/Intel GPU 的厂商 ID 可映射为对应 `AcceleratorKind`，并读取可用的 VRAM 字节数。
- macOS JSON 可解析 Apple、AMD、Intel 等可识别加速器；Apple GPU 显存不错误地报告为整机统一内存。
- Windows 单个 CIM `AdapterRAM` 不被窄化整数转换溢出；32 位上限哨兵值按未知容量处理。
- GPU 检测失败不覆盖已经成功检测到的其他平台 GPU。
- 非 Linux/macOS/Windows 目标的后备 `inspect()` 返回 `UnsupportedPlatform`。
- 序列化结果包含契约字段名，类型与单位符合 `docs/interface.md` 的映射。
- `cargo fmt --check`、`cargo test` 与当前主机 `cargo check` 通过。
