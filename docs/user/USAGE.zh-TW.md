# 使用指南

詳細說明 Backup Suite v1.0.0 的所有功能和實用使用方法。

## 📋 目錄

- [基本概念](#基本概念)
- [指令參考](#指令參考)
- [實踐工作流程](#實踐工作流程)
- [設定檔詳細說明](#設定檔詳細說明)
- [進階用法](#進階用法)
- [最佳實務](#最佳實務)

## 🎯 基本概念

### 優先順序系統
Backup Suite 透過 3 個優先順序管理備份：

| 優先順序 | 用途 | 建議頻率 | 範例 |
|--------|------|----------|-----|
| **high** | 重要·緊急檔案 | 每天 | 正在處理的專案、重要文件 |
| **medium** | 普通檔案 | 每週 | 已完成專案、照片 |
| **low** | 封存 | 每月 | 舊檔案、參考資料 |

### 類別系統
可以按用途將檔案分類管理：
- `development` - 開發專案
- `work` - 工作檔案
- `personal` - 個人檔案
- `creative` - 設計·創作
- `archive` - 封存

### 目標類型
- `file` - 單一檔案
- `directory` - 目錄（遞迴）

## 📝 指令參考

### `add` - 新增備份目標

#### 基本語法
```bash
backup-suite add [PATH] [OPTIONS]
```

#### 選項
- `--priority <PRIORITY>` - 設定優先順序（high/medium/low，預設：medium）
- `--category <CATEGORY>` - 設定類別（預設：user）
- `--interactive` - 互動式檔案選擇模式

#### 使用範例

```bash
# 基本新增
backup-suite add ~/Documents/project --priority high --category development

# 指定類別
backup-suite add ~/Photos --priority medium --category personal

# 互動式選擇（省略路徑或使用 --interactive）
backup-suite add --interactive
backup-suite add  # 省略路徑時自動切換到互動模式

# 新增目前目錄
backup-suite add . --priority high --category work

# 新增多個檔案（指令碼化）
for dir in ~/project1 ~/project2 ~/project3; do
    backup-suite add "$dir" --priority high --category development
done
```

#### 執行範例和輸出
```bash
$ backup-suite add ~/Documents/important --priority high --category work
✅ 已新增："/Users/user/Documents/important"

$ backup-suite add --interactive
# 啟動 skim 介面
# 使用模糊尋找器選擇檔案/目錄
✅ 已新增："/Users/user/selected/path"
```

---

### `list` (`ls`) - 備份目標列表

#### 基本語法
```bash
backup-suite list [OPTIONS]
backup-suite ls [OPTIONS]  # 別名
```

#### 選項
- `--priority <PRIORITY>` - 僅顯示指定優先順序

#### 使用範例

```bash
# 顯示所有目標
backup-suite list

# 僅顯示高優先順序
backup-suite list --priority high

# 使用別名
backup-suite ls --priority medium
```

#### 執行範例和輸出
```bash
$ backup-suite list
📋 備份目標列表
1. "/Users/user/Documents/project" [High] development
2. "/Users/user/Photos" [Medium] personal
3. "/Users/user/Archive" [Low] archive
合計：3 項

$ backup-suite list --priority high
📋 備份目標列表
1. "/Users/user/Documents/project" [High] development
合計：1 項
```

---

### `remove` - 刪除備份目標

#### 基本語法
```bash
backup-suite remove [PATH] [OPTIONS]
```

#### 選項
- `--interactive` - 互動式目標選擇模式

#### 使用範例

```bash
# 透過路徑刪除
backup-suite remove ~/Documents/old-project

# 互動式刪除
backup-suite remove --interactive

# 省略路徑時自動切換到互動模式
backup-suite remove
```

#### 執行範例和輸出
```bash
$ backup-suite remove ~/Documents/old-project
✅ 已刪除："/Users/user/Documents/old-project"

$ backup-suite remove --interactive
# 顯示現有目標的選擇 UI
選擇要刪除的備份目標：
> /Users/user/Documents/project [High] development
  /Users/user/Photos [Medium] personal
  /Users/user/Archive [Low] archive
✅ 已刪除："/Users/user/Documents/project"
```

---

### `clear` (`rm`) - 批次刪除

#### 基本語法
```bash
backup-suite clear [OPTIONS]
backup-suite rm [OPTIONS]  # 別名
```

#### 選項
- `--priority <PRIORITY>` - 批次刪除指定優先順序的目標
- `--all` - 刪除所有目標

#### 使用範例

```bash
# 刪除所有低優先順序目標
backup-suite clear --priority low

# 刪除所有目標（注意！）
backup-suite clear --all

# 使用別名
backup-suite rm --priority medium
```

#### 執行範例和輸出
```bash
$ backup-suite clear --priority low
✅ 已刪除 2 項

$ backup-suite clear --all
✅ 已刪除 5 項
```

---

### `run` - 執行備份

#### 基本語法
```bash
backup-suite run [OPTIONS]
```

#### 選項
- `--priority <PRIORITY>` - 僅執行指定優先順序
- `--category <CATEGORY>` - 僅執行指定類別
- `--dry-run` - 試執行（不實際執行，僅確認）
- `--encrypt` - 啟用 AES-256-GCM 加密
- `--password <PASSWORD>` - 加密密碼（省略時顯示提示）
- `--compress <TYPE>` - 壓縮演算法（zstd/gzip/none，預設：zstd）
- `--compress-level <LEVEL>` - 壓縮等級（zstd：1-22，gzip：1-9，預設：3）

#### 使用範例

```bash
# 備份所有目標
backup-suite run

# 僅備份高優先順序
backup-suite run --priority high

# 僅備份特定類別
backup-suite run --category development

# 加密備份（AES-256-GCM）
backup-suite run --encrypt --password "your-password"
backup-suite run --encrypt  # 透過提示輸入密碼

# 壓縮備份（zstd 高速壓縮）
backup-suite run --compress zstd --compress-level 3

# 壓縮備份（gzip 注重相容性）
backup-suite run --compress gzip --compress-level 6

# 加密 + 壓縮備份
backup-suite run --encrypt --compress zstd

# 試執行（僅確認）
backup-suite run --dry-run

# 中優先順序試執行
backup-suite run --priority medium --dry-run

# 加密 + 壓縮 + 指定類別
backup-suite run --encrypt --compress zstd --category work
```

#### 執行範例和輸出
```bash
$ backup-suite run --priority high
🚀 備份執行
📊 結果：150/150 成功，25.67 MB

$ backup-suite run --dry-run
🚀 備份執行（試執行）
📋 偵測到：300 個檔案

$ backup-suite run --encrypt --compress zstd
加密密碼：****
🚀 備份執行（加密，壓縮：zstd）
📊 結果：150/150 成功，12.34 MB（壓縮後）

$ backup-suite run --category development
🚀 備份執行（類別：development）
📊 結果：75/75 成功，18.42 MB
```

---

### `restore` - 復原備份

#### 基本語法
```bash
backup-suite restore [OPTIONS]
```

#### 選項
- `--from <PATTERN>` - 指定復原來源備份（模式比對）
- `--to <PATH>` - 指定復原目標目錄（預設：./.restored）
- `--password <PASSWORD>` - 解密密碼（加密備份的情況下，省略時顯示提示）

#### 使用範例

```bash
# 從最新備份復原
backup-suite restore

# 從特定日期的備份復原
backup-suite restore --from backup-20251104

# 指定自訂復原路徑
backup-suite restore --to ~/recovered-files

# 將特定備份復原到特定位置
backup-suite restore --from backup-20251104 --to ~/project-recovery

# 復原加密備份
backup-suite restore --password "your-password"
backup-suite restore --from backup-20251104 --password "your-password" --to ~/restored

# 加密備份（密碼提示）
backup-suite restore  # 偵測到加密檔案時自動要求輸入密碼
```

#### 執行範例和輸出
```bash
$ backup-suite restore
🔄 開始復原："/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ 已將備份復原到 "./.restored/backup_20251104_143000"
  復原的檔案數：150（加密：0 個檔案）

$ backup-suite restore --from backup-20251104 --to ~/recovered
🔄 開始復原："/Users/user/backup-suite/backups/backup-20251104-143000" → "/Users/user/recovered/backup_20251104_143000"
✅ 已將備份復原到 "/Users/user/recovered/backup_20251104_143000"
  復原的檔案數：150（加密：0 個檔案）

$ backup-suite restore --password "my-password"
🔄 開始復原："/Users/user/backup-suite/backups/backup-20251104-143000" → "./.restored/backup_20251104_143000"
✅ 已將備份復原到 "./.restored/backup_20251104_143000"
  復原的檔案數：150（加密：150 個檔案）
```

---

### `cleanup` - 刪除舊備份

#### 基本語法
```bash
backup-suite cleanup [OPTIONS]
```

#### 選項
- `--days <DAYS>` - 刪除超過指定天數的備份（預設：30）
- `--dry-run` - 試執行（不刪除，僅確認）

#### 使用範例

```bash
# 刪除 30 天前的備份（預設）
backup-suite cleanup

# 刪除 7 天前的備份
backup-suite cleanup --days 7

# 試執行（確認刪除目標）
backup-suite cleanup --days 30 --dry-run

# 刪除 1 年前的備份
backup-suite cleanup --days 365
```

#### 執行範例和輸出
```bash
$ backup-suite cleanup --days 7 --dry-run
🗑️ 刪除："/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 刪除："/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 已刪除 2 項（試執行）

$ backup-suite cleanup --days 7
🗑️ 刪除："/Users/user/backup-suite/backups/backup-20251028-143000"
🗑️ 刪除："/Users/user/backup-suite/backups/backup-20251029-143000"
✅ 已刪除 2 項
```

---

### `status` - 顯示目前狀態

#### 基本語法
```bash
backup-suite status
```

#### 使用範例和輸出
```bash
$ backup-suite status
📊 狀態
  儲存路徑："/Users/user/backup-suite/backups"
  目標：15
    高：5
    中：7
    低：3
```

---

### `history` - 顯示備份歷史

#### 基本語法
```bash
backup-suite history [OPTIONS]
```

#### 選項
- `--days <DAYS>` - 顯示的歷史天數（預設：7）

#### 使用範例

```bash
# 過去 7 天的歷史（預設）
backup-suite history

# 過去 30 天的歷史
backup-suite history --days 30

# 過去 1 天的歷史
backup-suite history --days 1
```

#### 執行範例和輸出
```bash
$ backup-suite history --days 7
📜 備份歷史（7 天）
1. ✅ 2025-11-04 14:30:00
   /Users/user/backup-suite/backups/backup-20251104-143000：150 個檔案，25.67 MB
2. ✅ 2025-11-03 14:30:00
   /Users/user/backup-suite/backups/backup-20251103-143000：148 個檔案，25.23 MB
```

---

### `schedule` - 排程任務管理

#### 基本語法
```bash
backup-suite schedule <ACTION> [OPTIONS]
```

#### 子指令

##### `setup` - 排程設定
```bash
backup-suite schedule setup [OPTIONS]
```

**選項：**
- `--high <FREQUENCY>` - 高優先順序執行頻率（預設：daily）
- `--medium <FREQUENCY>` - 中優先順序執行頻率（預設：weekly）
- `--low <FREQUENCY>` - 低優先順序執行頻率（預設：monthly）

**頻率選項：**
- `daily` - 每天 2:00 AM
- `weekly` - 每週日 2:00 AM
- `monthly` - 每月 1 日 2:00 AM
- `hourly` - 每小時（開發·測試用）

```bash
# 預設設定
backup-suite schedule setup

# 自訂頻率設定
backup-suite schedule setup --high daily --medium weekly --low monthly

# 全部設為每週
backup-suite schedule setup --high weekly --medium weekly --low weekly
```

##### `enable` - 啟用自動備份
```bash
backup-suite schedule enable [OPTIONS]
```

**選項：**
- `--priority <PRIORITY>` - 僅啟用特定優先順序

```bash
# 啟用所有優先順序的自動備份
backup-suite schedule enable

# 僅啟用高優先順序
backup-suite schedule enable --priority high

# 僅啟用中優先順序
backup-suite schedule enable --priority medium
```

##### `disable` - 停用自動備份
```bash
backup-suite schedule disable [OPTIONS]
```

**選項：**
- `--priority <PRIORITY>` - 僅停用特定優先順序

```bash
# 停用所有優先順序的自動備份
backup-suite schedule disable

# 僅停用高優先順序
backup-suite schedule disable --priority high
```

##### `status` - 檢查排程狀態
```bash
backup-suite schedule status
```

#### 執行範例和輸出
```bash
$ backup-suite schedule setup --high daily --medium weekly --low monthly
📅 高優先順序排程設定完成：daily
📅 中優先順序排程設定完成：weekly
📅 低優先順序排程設定完成：monthly

$ backup-suite schedule enable
✅ 自動備份已啟用

$ backup-suite schedule status
📅 排程設定
  已啟用：✅
  高優先順序：daily
  中優先順序：weekly
  低優先順序：monthly

📋 實際排程狀態
  high：✅ 已啟用
  medium：✅ 已啟用
  low：✅ 已啟用
```

---

### `config` - 設定管理

#### 基本語法
```bash
backup-suite config <ACTION> [ARGS]
```

#### 子指令

##### `set-destination` - 變更備份儲存路徑
```bash
backup-suite config set-destination <PATH>
```

**參數：**
- `<PATH>` - 新的備份儲存目錄路徑（支援波浪號展開）

```bash
# 變更為外接硬碟
backup-suite config set-destination /Volumes/ExternalHDD/backups

# 變更為主目錄內（波浪號展開）
backup-suite config set-destination ~/Documents/backups

# 變更為 NAS
backup-suite config set-destination /mnt/nas/backup-suite
```

##### `get-destination` - 顯示目前備份儲存路徑
```bash
backup-suite config get-destination
```

```bash
$ backup-suite config get-destination
📁 目前備份路徑
  "/Users/user/backup-suite/backups"
```

##### `open` - 在編輯器中開啟設定檔
```bash
backup-suite config open
```

**行為：**
- 使用環境變數 `$EDITOR` 或 `$VISUAL` 指定的編輯器開啟
- 在 macOS 上，環境變數未設定時使用 `open` 指令（預設編輯器）
- 在 Linux 上，回退到 `nano`
- 在 Windows 上，回退到 `notepad`

```bash
# 使用預設編輯器開啟
backup-suite config open

# 使用指定的編輯器開啟
EDITOR=vim backup-suite config open
EDITOR=code backup-suite config open  # VS Code
```

---

### `ai` - AI 驅動的智慧備份管理（需要 `--features ai`）

要使用 AI 功能，需要在建置時使用 `--features ai` 旗標。

```bash
# 啟用 AI 功能建置
cargo build --release --features ai
cargo install --path . --features ai
```

#### 子指令

##### `ai detect` - 異常偵測

從歷史記錄中偵測統計上異常的備份。

**基本語法：**
```bash
backup-suite ai detect [OPTIONS]
```

**選項：**
- `--days <DAYS>` - 分析的歷史天數（預設：7）
- `--format <FORMAT>` - 輸出格式：table/json/detailed（預設：table）

**使用範例：**
```bash
# 偵測過去 7 天的異常（預設）
backup-suite ai detect

# 詳細分析過去 14 天
backup-suite ai detect --days 14 --format detailed

# 以 JSON 格式輸出
backup-suite ai detect --format json
```

**執行範例和輸出：**
```
🤖 AI 異常偵測報告（過去 7 天）

┌────┬──────────────────┬──────────┬──────────┬─────────────────────┐
│ No │ 偵測時間          │ 異常類型  │ 信賴度    │ 說明                 │
├────┼──────────────────┼──────────┼──────────┼─────────────────────┤
│ 1  │ 2025-11-09 03:15 │ 大小激增  │ 95.3%    │ 檔案大小為正常的3倍   │
└────┴──────────────────┴──────────┴──────────┴─────────────────────┘

📊 摘要：偵測到 1 個異常
💡 建議操作：將 ~/Downloads 的暫存檔案加入排除設定
```

**效能**：< 1ms（100 條歷史記錄）

---

##### `ai analyze` - 檔案重要性分析

按重要程度對目錄中的檔案進行分類，最佳化備份策略。

**基本語法：**
```bash
backup-suite ai analyze <PATH> [OPTIONS]
```

**參數：**
- `<PATH>` - 要分析的目錄路徑

**選項：**
- `--suggest-priority` - 根據建議優先順序建議指令
- `--detailed` - 顯示詳細的分析結果

**使用範例：**
```bash
# 分析目錄重要性
backup-suite ai analyze ~/documents

# 顯示詳細的重要性分數
backup-suite ai analyze ~/documents --detailed

# 顯示優先順序建議
backup-suite ai analyze ~/projects --suggest-priority
```

**評估標準：**
- **高重要性（80-100 分）**：原始碼、文件、設定檔
- **中重要性（40-79 分）**：影像、資料檔案
- **低重要性（0-39 分）**：日誌、暫存檔案

**執行範例和輸出：**
```
🤖 AI 檔案重要性分析：~/Documents

  重要性分數：90/100
  建議優先順序：High
  類別：文件
  理由：PDF 檔案（頻繁更新）

$ backup-suite ai analyze ~/projects --suggest-priority
🤖 AI 檔案重要性分析：~/projects

  重要性分數：95/100
  建議優先順序：High
  類別：Rust 專案
  理由：偵測到 Cargo.toml（開發中專案）

💡 建議指令：backup-suite add "/Users/user/projects" --priority High
```

---

##### `ai suggest-exclude` - 排除模式建議

自動偵測不必要的檔案，建議排除模式。

**基本語法：**
```bash
backup-suite ai suggest-exclude <PATH> [OPTIONS]
```

**參數：**
- `<PATH>` - 要分析的目錄路徑

**選項：**
- `--apply` - 自動將建議模式套用到設定檔
- `--confidence <VALUE>` - 最小信賴度（0.0-1.0，預設：0.8）

**使用範例：**
```bash
# 顯示排除模式建議
backup-suite ai suggest-exclude ~/projects

# 自動將建議模式套用到設定
backup-suite ai suggest-exclude ~/projects --apply

# 將最小信賴度設為 50%（顯示更多候選）
backup-suite ai suggest-exclude ~/projects --confidence 0.5
```

**執行範例和輸出：**
```bash
$ backup-suite ai suggest-exclude ~/projects
🤖 AI 排除模式建議：~/projects

┌──────────────────┬──────────┬──────────┬─────────────────────┐
│ 模式              │ 減少量    │ 信賴度    │ 理由                 │
├──────────────────┼──────────┼──────────┼─────────────────────┤
│ node_modules/    │ 2.34 GB  │ 99%      │ npm 相依（可重新產生）│
│ target/          │ 1.87 GB  │ 99%      │ Rust 建置產物        │
│ .cache/          │ 0.45 GB  │ 95%      │ 快取目錄              │
└──────────────────┴──────────┴──────────┴─────────────────────┘

💡 總減少量：4.66 GB（備份時間約縮短 30%）
```

---

##### `ai auto-configure` - AI 自動設定

分析目錄並自動產生最佳備份設定。

**基本語法：**
```bash
backup-suite ai auto-configure <PATHS>... [OPTIONS]
```

**參數：**
- `<PATHS>...` - 要設定的目錄路徑（可指定多個）

**選項：**
- `--dry-run` - 試執行（不套用設定，僅確認）
- `--interactive` - 互動模式（確認每個子目錄和排除模式）
- `--max-depth <DEPTH>` - 子目錄探索深度（1 = 僅直接子目錄，預設：1）

**使用範例：**
```bash
# 自動分析和設定（分別評估子目錄）
backup-suite ai auto-configure ~/data

# 以互動方式確認並設定（確認子目錄和排除模式）
backup-suite ai auto-configure ~/data --interactive

# 試執行（不套用設定，僅確認）
backup-suite ai auto-configure ~/data --dry-run

# 指定子目錄探索深度（最多 2 層）
backup-suite ai auto-configure ~/data --max-depth 2

# 一次設定多個目錄
backup-suite ai auto-configure ~/projects ~/documents ~/photos
```

**功能：**
- **分別評估每個子目錄的重要性**：為每個目錄設定最佳優先順序
- **自動偵測並套用排除模式**：自動排除 `node_modules/`、`target/`、`.cache/` 等
- **自動判斷專案類型**：Rust、Node.js、Python 等
- **僅套用信賴度 80% 以上的模式**：防止誤檢

**執行範例和輸出：**
```
🤖 AI 自動設定
分析中："/Users/user/projects"
  📁 發現 3 個子目錄：3
    評估中："/Users/user/projects/web-app"
      建議優先順序：High（分數：95）
      📋 排除模式建議：3
        - node_modules（99.0%，預計減少 2.34 GB）
        - .cache（95.0%，預計減少 0.45 GB）
        - .*\.tmp$（99.0%，預計減少 0.00 GB）
      📝 排除模式：node_modules、.cache、.*\.tmp$
      ✅ 已加入設定
    評估中："/Users/user/projects/rust-cli"
      建議優先順序：High（分數：95）
      📋 排除模式建議：2
        - target（99.0%，預計減少 1.87 GB）
        - .cache（95.0%，預計減少 0.12 GB）
      📝 排除模式：target、.cache
      ✅ 已加入設定
    評估中："/Users/user/projects/archive"
      建議優先順序：Low（分數：30）
      ✅ 已加入設定

自動設定已完成
  加入的項目：3
  總減少量：4.78 GB（備份時間約縮短 35%）
```

**最佳實務：**

1. **首次使用 `--dry-run` 確認**：確認設定內容後再套用
   ```bash
   backup-suite ai auto-configure ~/projects --dry-run
   ```

2. **使用互動模式進行細粒度控制**：對重要專案使用互動模式確認
   ```bash
   backup-suite ai auto-configure ~/projects --interactive
   ```

3. **調整深度**：如果子專案較多，可增加深度
   ```bash
   backup-suite ai auto-configure ~/projects --max-depth 2
   ```

4. **確認排除模式**：設定後使用 `backup-suite list` 確認排除模式
   ```bash
   backup-suite list
   ```

---

## 🎯 實踐工作流程

### 開發者工作流程

```bash
# 1. 新增目前專案為高優先順序
backup-suite add ~/projects/current-project --priority high --category development

# 2. 將已完成專案遷移到中優先順序
backup-suite remove ~/projects/current-project
backup-suite add ~/projects/current-project --priority medium --category development

# 3. 將舊專案設為低優先順序封存
backup-suite add ~/projects/old-project --priority low --category archive

# 4. 自動化每日高優先順序備份
backup-suite schedule setup --high daily
backup-suite schedule enable --priority high

# 5. 定期檢查歷史
backup-suite dashboard
backup-suite history --days 7
```

### 攝影師工作流程

```bash
# 1. 以高優先順序管理目前拍攝工作階段
backup-suite add ~/Photos/2025/current-session --priority high --category creative

# 2. 以中優先順序儲存編輯完成的照片
backup-suite add ~/Photos/2025/edited --priority medium --category creative

# 3. 封存舊照片
backup-suite add ~/Photos/2023 --priority low --category archive

# 4. 設定每週創作備份
backup-suite schedule setup --high daily --medium weekly --low monthly
backup-suite schedule enable

# 5. 儲存管理
backup-suite cleanup --days 90  # 刪除 3 個月前的備份
```

---

## 💡 最佳實務

### 優先順序設定指南

#### `high` 優先順序的適當使用
```bash
# ✅ 適當
backup-suite add ~/current-work-project --priority high --category development
backup-suite add ~/.ssh --priority high --category security
backup-suite add ~/Documents/contracts --priority high --category legal

# ❌ 應避免
backup-suite add ~/Downloads --priority high  # 暫存檔案應為低優先順序
backup-suite add ~/Music --priority high      # 娛樂內容應為中~低優先順序
```

#### `medium` 優先順序的適當使用
```bash
# ✅ 適當
backup-suite add ~/Photos/2025 --priority medium --category personal
backup-suite add ~/Documents/references --priority medium --category reference
backup-suite add ~/.config --priority medium --category config
```

#### `low` 優先順序的適當使用
```bash
# ✅ 適當
backup-suite add ~/Archive/old-projects --priority low --category archive
backup-suite add ~/Downloads --priority low --category temp
backup-suite add ~/Desktop/old-files --priority low --category cleanup
```

### 排除模式最佳實務

#### 開發專案
```toml
[[targets]]
path = "/Users/user/projects/web-app"
exclude_patterns = [
    "node_modules",      # NPM 相依
    ".git",             # Git 歷史（大容量）
    "build",            # 建置產物
    "dist",             # 分發建置
    "*.log",            # 日誌檔案
    ".env",             # 環境變數（敏感資訊）
    "coverage",         # 測試覆蓋率
    ".nyc_output"       # 覆蓋率暫存檔案
]
```

#### 創作·設計專案
```toml
[[targets]]
path = "/Users/user/creative/video-project"
exclude_patterns = [
    "*.tmp",            # 暫存檔案
    "cache",            # 快取目錄
    "render",           # 算繪暫存檔案
    "*.autosave",       # 自動儲存檔案
    ".DS_Store"         # macOS 系統檔案
]
```

---

## 📞 支援·聯絡

如有使用方法不明之處：

1. **GitHub Issues**：[問題·Bug 回報](https://github.com/user/backup-suite/issues)
2. **Discussions**：[社群諮詢](https://github.com/user/backup-suite/discussions)
3. **Documentation**：[其他文件](../README.md#文件)

---

**下一步**：有關更多技術細節，請查看 [架構文件](../development/ARCHITECTURE.md)。
