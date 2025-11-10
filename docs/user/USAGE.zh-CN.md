# 使用指南

详细说明 Backup Suite v1.0.0 的所有功能和实用使用方法。

## 📋 目录

- [基本概念](#基本概念)
- [命令参考](#命令参考)
- [实践工作流程](#实践工作流程)
- [配置文件详细说明](#配置文件详细说明)
- [高级用法](#高级用法)
- [最佳实践](#最佳实践)

## 🎯 基本概念

### 优先级系统
Backup Suite 通过 3 个优先级管理备份：

| 优先级 | 用途 | 推荐频率 | 示例 |
|--------|------|----------|-----|
| **high** | 重要·紧急文件 | 每天 | 正在处理的项目、重要文档 |
| **medium** | 普通文件 | 每周 | 已完成项目、照片 |
| **low** | 归档 | 每月 | 旧文件、参考资料 |

### 类别系统
可以按用途将文件分类管理：
- `development` - 开发项目
- `work` - 工作文件
- `personal` - 个人文件
- `creative` - 设计·创作
- `archive` - 归档

### 目标类型
- `file` - 单个文件
- `directory` - 目录（递归）

## 📝 命令参考

### `add` - 添加备份目标

#### 基本语法
```bash
backup-suite add [PATH] [OPTIONS]
```

#### 选项
- `--priority <PRIORITY>` - 设置优先级（high/medium/low，默认：medium）
- `--category <CATEGORY>` - 设置类别（默认：user）
- `--interactive` - 交互式文件选择模式

#### 使用示例

```bash
# 基本添加
backup-suite add ~/Documents/project --priority high --category development

# 指定类别
backup-suite add ~/Photos --priority medium --category personal

# 交互式选择（省略路径或使用 --interactive）
backup-suite add --interactive
backup-suite add  # 省略路径时自动切换到交互模式

# 添加当前目录
backup-suite add . --priority high --category work

# 添加多个文件（脚本化）
for dir in ~/project1 ~/project2 ~/project3; do
    backup-suite add "$dir" --priority high --category development
done
```

#### 执行示例和输出
```bash
$ backup-suite add ~/Documents/important --priority high --category work
✅ 已添加："/Users/user/Documents/important"

$ backup-suite add --interactive
# 启动 skim 界面
# 使用模糊查找器选择文件/目录
✅ 已添加："/Users/user/selected/path"
```

---

### `list` (`ls`) - 备份目标列表

#### 基本语法
```bash
backup-suite list [OPTIONS]
backup-suite ls [OPTIONS]  # 别名
```

#### 选项
- `--priority <PRIORITY>` - 仅显示指定优先级

#### 使用示例

```bash
# 显示所有目标
backup-suite list

# 仅显示高优先级
backup-suite list --priority high

# 使用别名
backup-suite ls --priority medium
```

#### 执行示例和输出
```bash
$ backup-suite list
📋 备份目标列表
1. "/Users/user/Documents/project" [High] development
2. "/Users/user/Photos" [Medium] personal
3. "/Users/user/Archive" [Low] archive
合计：3 项

$ backup-suite list --priority high
📋 备份目标列表
1. "/Users/user/Documents/project" [High] development
合计：1 项
```

---

### `remove` - 删除备份目标

#### 基本语法
```bash
backup-suite remove [PATH] [OPTIONS]
```

#### 选项
- `--interactive` - 交互式目标选择模式

#### 使用示例

```bash
# 通过路径删除
backup-suite remove ~/Documents/old-project

# 交互式删除
backup-suite remove --interactive

# 省略路径时自动切换到交互模式
backup-suite remove
```

#### 执行示例和输出
```bash
$ backup-suite remove ~/Documents/old-project
✅ 已删除："/Users/user/Documents/old-project"

$ backup-suite remove --interactive
# 显示现有目标的选择 UI
选择要删除的备份目标：
> /Users/user/Documents/project [High] development
  /Users/user/Photos [Medium] personal
  /Users/user/Archive [Low] archive
✅ 已删除："/Users/user/Documents/project"
```

---

### `clear` (`rm`) - 批量删除

#### 基本语法
```bash
backup-suite clear [OPTIONS]
backup-suite rm [OPTIONS]  # 别名
```

#### 选项
- `--priority <PRIORITY>` - 批量删除指定优先级的目标
- `--all` - 删除所有目标

#### 使用示例

```bash
# 删除所有低优先级目标
backup-suite clear --priority low

# 删除所有目标（注意！）
backup-suite clear --all

# 使用别名
backup-suite rm --priority medium
```

#### 执行示例和输出
```bash
$ backup-suite clear --priority low
✅ 已删除 2 项

$ backup-suite clear --all
✅ 已删除 5 项
```

---

### `run` - 执行备份

#### 基本语法
```bash
backup-suite run [OPTIONS]
```

#### 选项
- `--priority <PRIORITY>` - 仅执行指定优先级
- `--category <CATEGORY>` - 仅执行指定类别
- `--dry-run` - 试运行（不实际执行，仅确认）
- `--encrypt` - 启用 AES-256-GCM 加密
- `--password <PASSWORD>` - 加密密码（省略时显示提示）
- `--compress <TYPE>` - 压缩算法（zstd/gzip/none，默认：zstd）
- `--compress-level <LEVEL>` - 压缩级别（zstd：1-22，gzip：1-9，默认：3）

#### 使用示例

```bash
# 备份所有目标
backup-suite run

# 仅备份高优先级
backup-suite run --priority high

# 仅备份特定类别
backup-suite run --category development

# 加密备份（AES-256-GCM）
backup-suite run --encrypt --password "your-password"
backup-suite run --encrypt  # 通过提示输入密码

# 压缩备份（zstd 高速压缩）
backup-suite run --compress zstd --compress-level 3

# 压缩备份（gzip 注重兼容性）
backup-suite run --compress gzip --compress-level 6

# 加密 + 压缩备份
backup-suite run --encrypt --compress zstd

# 试运行（仅确认）
backup-suite run --dry-run

# 中优先级试运行
backup-suite run --priority medium --dry-run

# 加密 + 压缩 + 指定类别
backup-suite run --encrypt --compress zstd --category work
```

#### 执行示例和输出
```bash
$ backup-suite run --priority high
🚀 备份执行
📊 结果：150/150 成功，25.67 MB

$ backup-suite run --dry-run
🚀 备份执行（试运行）
📋 检测到：300 个文件

$ backup-suite run --encrypt --compress zstd
加密密码：****
🚀 备份执行（加密，压缩：zstd）
📊 结果：150/150 成功，12.34 MB（压缩后）

$ backup-suite run --category development
🚀 备份执行（类别：development）
📊 结果：75/75 成功，18.42 MB
```

---

### `restore` - 恢复备份

#### 基本语法
```bash
backup-suite restore [OPTIONS]
```

#### 选项
- `--from <PATTERN>` - 指定恢复源备份（模式匹配）
- `--to <PATH>` - 指定恢复目标目录（默认：./.restored）
- `--password <PASSWORD>` - 解密密码（加密备份的情况下，省略时显示提示）

#### 使用示例

```bash
# 从最新备份恢复
backup-suite restore

# 从特定日期的备份恢复
backup-suite restore --from backup-20251104

# 指定自定义恢复路径
backup-suite restore --to ~/recovered-files

# 将特定备份恢复到特定位置
backup-suite restore --from backup-20251104 --to ~/project-recovery

# 恢复加密备份
backup-suite restore --password "your-password"
backup-suite restore --from backup-20251104 --password "your-password" --to ~/restored

# 加密备份（密码提示）
backup-suite restore  # 检测到加密文件时自动要求输入密码
```

#### 执行示例和输出
```bash
$ backup-suite restore
🔄 开始恢复："/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ 已将备份恢复到 "./.restored/backup_20251104_143000"
  恢复的文件数：150（加密：0 个文件）

$ backup-suite restore --from backup-20251104 --to ~/recovered
🔄 开始恢复："/Users/user/backup-suite/backups/backup-20251104-143000" → "/Users/user/recovered/backup_20251104_143000"
✅ 已将备份恢复到 "/Users/user/recovered/backup_20251104_143000"
  恢复的文件数：150（加密：0 个文件）

$ backup-suite restore --password "my-password"
🔄 开始恢复："/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ 已将备份恢复到 "./.restored/backup_20251104_143000"
  恢复的文件数：150（加密：150 个文件）
```

---

### `cleanup` - 删除旧备份

#### 基本语法
```bash
backup-suite cleanup [OPTIONS]
```

#### 选项
- `--days <DAYS>` - 删除超过指定天数的备份（默认：30）
- `--dry-run` - 试运行（不删除，仅确认）

#### 使用示例

```bash
# 删除 30 天前的备份（默认）
backup-suite cleanup

# 删除 7 天前的备份
backup-suite cleanup --days 7

# 试运行（确认删除目标）
backup-suite cleanup --days 30 --dry-run

# 删除 1 年前的备份
backup-suite cleanup --days 365
```

#### 执行示例和输出
```bash
$ backup-suite cleanup --days 7 --dry-run
🗑️ 删除："/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 删除："/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 已删除 2 项（试运行）

$ backup-suite cleanup --days 7
🗑️ 删除："/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 删除："/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 已删除 2 项
```

---

### `status` - 显示当前状态

#### 基本语法
```bash
backup-suite status
```

#### 使用示例和输出
```bash
$ backup-suite status
📊 状态
  保存路径："/Users/user/backup-suite/backups"
  目标：15
    高：5
    中：7
    低：3
```

---

### `history` - 显示备份历史

#### 基本语法
```bash
backup-suite history [OPTIONS]
```

#### 选项
- `--days <DAYS>` - 显示的历史天数（默认：7）

#### 使用示例

```bash
# 过去 7 天的历史（默认）
backup-suite history

# 过去 30 天的历史
backup-suite history --days 30

# 过去 1 天的历史
backup-suite history --days 1
```

#### 执行示例和输出
```bash
$ backup-suite history --days 7
📜 备份历史（7 天）
1. ✅ 2025-11-04 14:30:00
   /Users/user/backup-suite/backups/backup-20251104-143000：150 个文件，25.67 MB
2. ✅ 2025-11-03 14:30:00
   /Users/user/backup-suite/backups/backup-20251103-143000：148 个文件，25.23 MB
```

---

### `schedule` - 计划任务管理

#### 基本语法
```bash
backup-suite schedule <ACTION> [OPTIONS]
```

#### 子命令

##### `setup` - 计划设置
```bash
backup-suite schedule setup [OPTIONS]
```

**选项：**
- `--high <FREQUENCY>` - 高优先级执行频率（默认：daily）
- `--medium <FREQUENCY>` - 中优先级执行频率（默认：weekly）
- `--low <FREQUENCY>` - 低优先级执行频率（默认：monthly）

**频率选项：**
- `daily` - 每天 2:00 AM
- `weekly` - 每周日 2:00 AM
- `monthly` - 每月 1 日 2:00 AM
- `hourly` - 每小时（开发·测试用）

```bash
# 默认设置
backup-suite schedule setup

# 自定义频率设置
backup-suite schedule setup --high daily --medium weekly --low monthly

# 全部设为每周
backup-suite schedule setup --high weekly --medium weekly --low weekly
```

##### `enable` - 启用自动备份
```bash
backup-suite schedule enable [OPTIONS]
```

**选项：**
- `--priority <PRIORITY>` - 仅启用特定优先级

```bash
# 启用所有优先级的自动备份
backup-suite schedule enable

# 仅启用高优先级
backup-suite schedule enable --priority high

# 仅启用中优先级
backup-suite schedule enable --priority medium
```

##### `disable` - 禁用自动备份
```bash
backup-suite schedule disable [OPTIONS]
```

**选项：**
- `--priority <PRIORITY>` - 仅禁用特定优先级

```bash
# 禁用所有优先级的自动备份
backup-suite schedule disable

# 仅禁用高优先级
backup-suite schedule disable --priority high
```

##### `status` - 检查计划状态
```bash
backup-suite schedule status
```

#### 执行示例和输出
```bash
$ backup-suite schedule setup --high daily --medium weekly --low monthly
📅 高优先级计划设置完成：daily
📅 中优先级计划设置完成：weekly
📅 低优先级计划设置完成：monthly

$ backup-suite schedule enable
✅ 自动备份已启用

$ backup-suite schedule status
📅 计划设置
  已启用：✅
  高优先级：daily
  中优先级：weekly
  低优先级：monthly

📋 实际计划状态
  high：✅ 已启用
  medium：✅ 已启用
  low：✅ 已启用
```

---

### `config` - 配置管理

#### 基本语法
```bash
backup-suite config <ACTION> [ARGS]
```

#### 子命令

##### `set-destination` - 更改备份保存路径
```bash
backup-suite config set-destination <PATH>
```

**参数：**
- `<PATH>` - 新的备份保存目录路径（支持波浪号扩展）

```bash
# 更改为外部硬盘
backup-suite config set-destination /Volumes/ExternalHDD/backups

# 更改为主目录内（波浪号扩展）
backup-suite config set-destination ~/Documents/backups

# 更改为 NAS
backup-suite config set-destination /mnt/nas/backup-suite
```

##### `get-destination` - 显示当前备份保存路径
```bash
backup-suite config get-destination
```

```bash
$ backup-suite config get-destination
📁 当前备份路径
  "/Users/user/backup-suite/backups"
```

##### `open` - 在编辑器中打开配置文件
```bash
backup-suite config open
```

**行为：**
- 使用环境变量 `$EDITOR` 或 `$VISUAL` 指定的编辑器打开
- 在 macOS 上，环境变量未设置时使用 `open` 命令（默认编辑器）
- 在 Linux 上，回退到 `nano`
- 在 Windows 上，回退到 `notepad`

```bash
# 使用默认编辑器打开
backup-suite config open

# 使用指定的编辑器打开
EDITOR=vim backup-suite config open
EDITOR=code backup-suite config open  # VS Code
```

---

### `ai` - AI 驱动的智能备份管理（需要 `--features smart`）

要使用 AI 功能，需要在构建时使用 `--features smart` 标志。

```bash
# 启用 AI 功能构建
cargo build --release --features smart
cargo install --path . --features smart
```

#### 子命令

##### `ai detect` - 异常检测

从历史记录中检测统计上异常的备份。

**基本语法：**
```bash
backup-suite smart detect [OPTIONS]
```

**选项：**
- `--days <DAYS>` - 分析的历史天数（默认：7）
- `--format <FORMAT>` - 输出格式：table/json/detailed（默认：table）

**使用示例：**
```bash
# 检测过去 7 天的异常（默认）
backup-suite smart detect

# 详细分析过去 14 天
backup-suite smart detect --days 14 --format detailed

# 以 JSON 格式输出
backup-suite smart detect --format json
```

**执行示例和输出：**
```
🤖 AI 异常检测报告（过去 7 天）

┌────┬──────────────────┬──────────┬──────────┬─────────────────────┐
│ No │ 检测时间          │ 异常类型  │ 置信度    │ 说明                 │
├────┼──────────────────┼──────────┼──────────┼─────────────────────┤
│ 1  │ 2025-11-09 03:15 │ 大小激增  │ 95.3%    │ 文件大小为正常的3倍   │
└────┴──────────────────┴──────────┴──────────┴─────────────────────┘

📊 摘要：检测到 1 个异常
💡 推荐操作：将 ~/Downloads 的临时文件添加到排除设置
```

**性能**：< 1ms（100 条历史记录）

---

##### `ai analyze` - 文件重要性分析

按重要程度对目录中的文件进行分类，优化备份策略。

**基本语法：**
```bash
backup-suite smart analyze <PATH> [OPTIONS]
```

**参数：**
- `<PATH>` - 要分析的目录路径

**选项：**
- `--suggest-priority` - 根据推荐优先级建议命令
- `--detailed` - 显示详细的分析结果

**使用示例：**
```bash
# 分析目录重要性
backup-suite smart analyze ~/documents

# 显示详细的重要性分数
backup-suite smart analyze ~/documents --detailed

# 显示优先级建议
backup-suite smart analyze ~/projects --suggest-priority
```

**评估标准：**
- **高重要性（80-100 分）**：源代码、文档、配置文件
- **中重要性（40-79 分）**：图像、数据文件
- **低重要性（0-39 分）**：日志、临时文件

**执行示例和输出：**
```
🤖 AI 文件重要性分析：~/Documents

  重要性分数：90/100
  推荐优先级：High
  类别：文档
  理由：PDF 文件（频繁更新）

$ backup-suite smart analyze ~/projects --suggest-priority
🤖 AI 文件重要性分析：~/projects

  重要性分数：95/100
  推荐优先级：High
  类别：Rust 项目
  理由：检测到 Cargo.toml（开发中项目）

💡 推荐命令：backup-suite add "/Users/user/projects" --priority High
```

---

##### `ai suggest-exclude` - 排除模式推荐

自动检测不必要的文件，推荐排除模式。

**基本语法：**
```bash
backup-suite smart suggest-exclude <PATH> [OPTIONS]
```

**参数：**
- `<PATH>` - 要分析的目录路径

**选项：**
- `--apply` - 自动将推荐模式应用到配置文件
- `--confidence <VALUE>` - 最小置信度（0.0-1.0，默认：0.8）

**使用示例：**
```bash
# 显示排除模式推荐
backup-suite smart suggest-exclude ~/projects

# 自动将推荐模式应用到配置
backup-suite smart suggest-exclude ~/projects --apply

# 将最小置信度设为 50%（显示更多候选）
backup-suite smart suggest-exclude ~/projects --confidence 0.5
```

**执行示例和输出：**
```bash
$ backup-suite smart suggest-exclude ~/projects
🤖 AI 排除模式推荐：~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ 模式              │ 减少量    │ 置信度    │ 理由                 │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm 依赖（可重新生成）│
│ target/          │ 1.87 GB  │ 99%      │ Rust 构建产物        │
│ .cache/          │ 0.45 GB  │ 95%      │ 缓存目录              │
└──────────────────┴──────────┴──────────┴─────────────────────┘

💡 总减少量：4.66 GB（备份时间约缩短 30%）
```

---

##### `ai auto-configure` - AI 自动配置

分析目录并自动生成最佳备份配置。

**基本语法：**
```bash
backup-suite smart auto-configure <PATHS>... [OPTIONS]
```

**参数：**
- `<PATHS>...` - 要配置的目录路径（可指定多个）

**选项：**
- `--dry-run` - 试运行（不应用配置，仅确认）
- `--interactive` - 交互模式（确认每个子目录和排除模式）
- `--max-depth <DEPTH>` - 子目录探索深度（1 = 仅直接子目录，默认：1）

**使用示例：**
```bash
# 自动分析和配置（分别评估子目录）
backup-suite smart auto-configure ~/data

# 以交互方式确认并配置（确认子目录和排除模式）
backup-suite smart auto-configure ~/data --interactive

# 试运行（不应用配置，仅确认）
backup-suite smart auto-configure ~/data --dry-run

# 指定子目录探索深度（最多 2 层）
backup-suite smart auto-configure ~/data --max-depth 2

# 一次配置多个目录
backup-suite smart auto-configure ~/projects ~/documents ~/photos
```

**功能：**
- **分别评估每个子目录的重要性**：为每个目录设置最佳优先级
- **自动检测并应用排除模式**：自动排除 `node_modules/`、`target/`、`.cache/` 等
- **自动判断项目类型**：Rust、Node.js、Python 等
- **仅应用置信度 80% 以上的模式**：防止误检

**执行示例和输出：**
```
🤖 AI 自动配置
分析中："/Users/user/projects"
  📁 发现 3 个子目录：3
    评估中："/Users/user/projects/web-app"
      推荐优先级：High（分数：95）
      📋 排除模式建议：3
        - node_modules（99.0%，预计减少 2.34 GB）
        - .cache（95.0%，预计减少 0.45 GB）
        - .*\.tmp$（99.0%，预计减少 0.00 GB）
      📝 排除模式：node_modules、.cache、.*\.tmp$
      ✅ 已添加到配置
    评估中："/Users/user/projects/rust-cli"
      推荐优先级：High（分数：95）
      📋 排除模式建议：2
        - target（99.0%，预计减少 1.87 GB）
        - .cache（95.0%，预计减少 0.12 GB）
      📝 排除模式：target、.cache
      ✅ 已添加到配置
    评估中："/Users/user/projects/archive"
      推荐优先级：Low（分数：30）
      ✅ 已添加到配置

自动配置已完成
  添加的项目：3
  总减少量：4.78 GB（备份时间约缩短 35%）
```

**最佳实践：**

1. **首次使用 `--dry-run` 确认**：确认配置内容后再应用
   ```bash
   backup-suite smart auto-configure ~/projects --dry-run
   ```

2. **使用交互模式进行细粒度控制**：对重要项目使用交互模式确认
   ```bash
   backup-suite smart auto-configure ~/projects --interactive
   ```

3. **调整深度**：如果子项目较多，可增加深度
   ```bash
   backup-suite smart auto-configure ~/projects --max-depth 2
   ```

4. **确认排除模式**：配置后使用 `backup-suite list` 确认排除模式
   ```bash
   backup-suite list
   ```

---

## 🎯 实践工作流程

### 开发者工作流程

```bash
# 1. 添加当前项目为高优先级
backup-suite add ~/projects/current-project --priority high --category development

# 2. 将已完成项目迁移到中优先级
backup-suite remove ~/projects/current-project
backup-suite add ~/projects/current-project --priority medium --category development

# 3. 将旧项目设为低优先级归档
backup-suite add ~/projects/old-project --priority low --category archive

# 4. 自动化每日高优先级备份
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high

# 5. 定期检查历史
backup-suite dashboard
backup-suite history --days 7
```

### 摄影师工作流程

```bash
# 1. 以高优先级管理当前拍摄会话
backup-suite add ~/Photos/2025/current-session --priority high --category creative

# 2. 以中优先级保存编辑完成的照片
backup-suite add ~/Photos/2025/edited --priority medium --category creative

# 3. 归档旧照片
backup-suite add ~/Photos/2023 --priority low --category archive

# 4. 配置每周创作备份
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable

# 5. 存储管理
backup-suite cleanup --days 90  # 删除 3 个月前的备份
```

---

## 💡 最佳实践

### 优先级设置指南

#### `high` 优先级的适当使用
```bash
# ✅ 适当
backup-suite add ~/current-work-project --priority high --category development
backup-suite add ~/.ssh --priority high --category security
backup-suite add ~/Documents/contracts --priority high --category legal

# ❌ 应避免
backup-suite add ~/Downloads --priority high  # 临时文件应为低优先级
backup-suite add ~/Music --priority high      # 娱乐内容应为中~低优先级
```

#### `medium` 优先级的适当使用
```bash
# ✅ 适当
backup-suite add ~/Photos/2025 --priority medium --category personal
backup-suite add ~/Documents/references --priority medium --category reference
backup-suite add ~/.config --priority medium --category config
```

#### `low` 优先级的适当使用
```bash
# ✅ 适当
backup-suite add ~/Archive/old-projects --priority low --category archive
backup-suite add ~/Downloads --priority low --category temp
backup-suite add ~/Desktop/old-files --priority low --category cleanup
```

### 排除模式最佳实践

#### 开发项目
```toml
[[targets]]
path = "/Users/user/projects/web-app"
exclude_patterns = [
    "node_modules",      # NPM 依赖
    ".git",             # Git 历史（大容量）
    "build",            # 构建产物
    "dist",             # 分发构建
    "*.log",            # 日志文件
    ".env",             # 环境变量（敏感信息）
    "coverage",         # 测试覆盖率
    ".nyc_output"       # 覆盖率临时文件
]
```

#### 创作·设计项目
```toml
[[targets]]
path = "/Users/user/creative/video-project"
exclude_patterns = [
    "*.tmp",            # 临时文件
    "cache",            # 缓存目录
    "render",           # 渲染临时文件
    "*.autosave",       # 自动保存文件
    ".DS_Store"         # macOS 系统文件
]
```

---

## 📞 支持·联系

如有使用方法不明之处：

1. **GitHub Issues**：[问题·Bug 报告](https://github.com/user/backup-suite/issues)
2. **Discussions**：[社区咨询](https://github.com/user/backup-suite/discussions)
3. **Documentation**：[其他文档](../README.md#文档)

---

**下一步**：有关更多技术细节，请查看 [架构文档](../development/ARCHITECTURE.md)。
