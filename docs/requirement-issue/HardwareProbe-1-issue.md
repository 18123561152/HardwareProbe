## Align accelerator contract and unknown-vendor classification

## 背景与目标

- **背景**：硬件检测模块已完成 Windows、macOS、Linux 三个平台的实测，能够返回 CPU、内存、磁盘和加速器信息。当前实现与验收材料在加速器枚举契约和未知厂商处理方式上仍不一致。
- **目标**：统一硬件检测模块的公开契约，使未知厂商设备可被保留并以 `Other` 表示，同时保持三平台已有检测能力。模型硬件兼容性判断不属于本模块。

## 需求列表

| # | 需求 | 是否必需 |
| --- | --- | --- |
| 1 | 统一 `HardwareProfile` 与 `AcceleratorKind` 公开契约 | 是 |
| 2 | 将无法识别的加速器厂商映射为 `Other` | 是 |
| 3 | 保持 Windows、macOS、Linux 三平台硬件检测能力 | 是 |
| 4 | 同步更新模块文档和验证用例 | 是 |

## 每项需求的期望行为与期望结果

| # | 需求 | 预期行为 | 期望结果 |
| --- | --- | --- | --- |
| 1 | 统一公开契约 | `HardwareProfile.cpu` 使用 `String`；`AcceleratorKind` 包含 `Nvidia`、`Amd`、`Intel`、`Apple`、`Other`；Serde 使用对应变体名称 | 公开结构体、枚举和 JSON 输出与最终契约一致，不再使用 `cpu_threads` |
| 2 | 未知厂商映射 | 在 Windows、macOS、Linux 探测到设备但无法识别厂商时，将其 `kind` 设置为 `Other`，同时保留设备名称和可获取的 `memory_bytes` | 未知厂商设备出现在 `accelerators` 中，序列化结果包含 `"kind": "Other"`；已识别厂商的分类行为不回退 |
| 3 | 跨平台检测保持可用 | 保持现有 Windows、macOS、Linux 平台分发和硬件采集逻辑 | 三个平台均能完成检测并返回 CPU、内存、磁盘和加速器信息；GPU 不可用时仍按现有降级约定返回基础硬件信息 |
| 4 | 文档和验证同步 | 更新 `docs/requirements.md`、`docs/interface.md` 及相关测试，使其反映最终契约和未知厂商行为 | 文档不再保留冲突的 `cpu_threads` 或缺少 `Other` 的枚举定义；测试覆盖 `Other` 的构造、Serde 和平台解析行为 |

## 范围边界

- **做什么**：调整加速器公开枚举、未知厂商分类逻辑、契约文档和验证用例；保持已有三平台硬件检测能力。
- **不做什么**：不增加进度百分比；不在 `hardware-probe` 中实现模型硬件兼容性判断、模型要求类型、页面跳转或页面展示逻辑。

## 待确认问题

- 无
