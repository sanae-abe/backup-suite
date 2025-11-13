#!/bin/bash
# backup-suite用のZsh補完スクリプトを生成して多言語化
# 対応言語: 日本語(ja), 簡体字中国語(zh-CN), 繁体字中国語(zh-TW)

set -e

# 言語設定（デフォルト: システムのLANGから判定）
LANG_CODE="${1:-auto}"

# 自動判定
if [ "$LANG_CODE" = "auto" ]; then
    case "$LANG" in
        ja_*|jp_*) LANG_CODE="ja" ;;
        zh_CN*) LANG_CODE="zh-CN" ;;
        zh_TW*|zh_HK*) LANG_CODE="zh-TW" ;;
        *) LANG_CODE="en" ;;
    esac
fi

echo "Generating Zsh completion script..."
backup-suite completion zsh > ~/.zfunc/_backup-suite

if [ "$LANG_CODE" = "en" ]; then
    echo "✅ English completion script generated"
    exit 0
fi

echo "Translating to ${LANG_CODE}..."

case "$LANG_CODE" in
    ja)
        # 日本語
        sed -i '' \
          -e "s/Add backup target (with interactive file selector)/バックアップ対象を追加（対話的選択対応）/g" \
          -e "s/List backup targets/バックアップ対象一覧を表示/g" \
          -e "s/Remove backup target/バックアップ対象を削除/g" \
          -e "s/Clear all backup targets/すべてのバックアップ対象を削除/g" \
          -e "s/Run backup (with encryption and compression support)/バックアップを実行（暗号化・圧縮対応）/g" \
          -e "s/Restore from backup/バックアップから復元/g" \
          -e "s/Clean up old backups/古いバックアップを削除/g" \
          -e "s/Show backup status/バックアップ状態を表示/g" \
          -e "s/Show backup history/バックアップ履歴を表示/g" \
          -e "s/Show interactive dashboard/対話的ダッシュボードを表示/g" \
          -e "s/Open backup directory/バックアップディレクトリを開く/g" \
          -e "s/Generate shell completion scripts/シェル補完スクリプトを生成/g" \
          -e "s/Manage backup schedule/バックアップスケジュールを管理/g" \
          -e "s/Configuration management/設定管理/g" \
          -e "s/Smart rule-based intelligent backup management/スマートルールベースのバックアップ管理/g" \
          -e "s/Enable automatic backup/自動バックアップを有効化/g" \
          -e "s/Disable automatic backup/自動バックアップを無効化/g" \
          -e "s/Show schedule status/スケジュール状態を表示/g" \
          -e "s/Setup backup schedule/バックアップスケジュールを設定/g" \
          -e "s/Show help for schedule commands/スケジュールコマンドのヘルプを表示/g" \
          -e "s/Set backup destination directory/バックアップ保存先を設定/g" \
          -e "s/Get current backup destination directory/現在のバックアップ保存先を取得/g" \
          -e "s/Set backup retention days/バックアップ保持日数を設定/g" \
          -e "s/Get current backup retention days/現在のバックアップ保持日数を取得/g" \
          -e "s/Open configuration file in default editor/デフォルトエディタで設定ファイルを開く/g" \
          -e "s/Show help for config commands/設定コマンドのヘルプを表示/g" \
          -e "s/Detect anomalies in backup history/バックアップ履歴の異常を検出/g" \
          -e "s/Analyze file importance/ファイル重要度を分析/g" \
          -e "s/Suggest exclude patterns/除外パターンを提案/g" \
          -e "s/Auto-configure backup settings with smart rules/スマートルールで自動設定/g" \
          -e "s/Show help for smart commands/Smartコマンドのヘルプを表示/g" \
          ~/.zfunc/_backup-suite
        ;;
    zh-CN)
        # 简体中文
        sed -i '' \
          -e "s/Add backup target (with interactive file selector)/添加备份目标（支持交互式选择）/g" \
          -e "s/List backup targets/列出备份目标/g" \
          -e "s/Remove backup target/删除备份目标/g" \
          -e "s/Clear all backup targets/清除所有备份目标/g" \
          -e "s/Run backup (with encryption and compression support)/运行备份（支持加密和压缩）/g" \
          -e "s/Restore from backup/从备份恢复/g" \
          -e "s/Clean up old backups/清理旧备份/g" \
          -e "s/Show backup status/显示备份状态/g" \
          -e "s/Show backup history/显示备份历史/g" \
          -e "s/Show interactive dashboard/显示交互式仪表板/g" \
          -e "s/Open backup directory/打开备份目录/g" \
          -e "s/Generate shell completion scripts/生成Shell补全脚本/g" \
          -e "s/Manage backup schedule/管理备份计划/g" \
          -e "s/Configuration management/配置管理/g" \
          -e "s/Smart rule-based intelligent backup management/基于智能规则的备份管理/g" \
          -e "s/Enable automatic backup/启用自动备份/g" \
          -e "s/Disable automatic backup/禁用自动备份/g" \
          -e "s/Show schedule status/显示计划状态/g" \
          -e "s/Setup backup schedule/设置备份计划/g" \
          -e "s/Show help for schedule commands/显示计划命令帮助/g" \
          -e "s/Set backup destination directory/设置备份目标目录/g" \
          -e "s/Get current backup destination directory/获取当前备份目标目录/g" \
          -e "s/Set backup retention days/设置备份保留天数/g" \
          -e "s/Get current backup retention days/获取当前备份保留天数/g" \
          -e "s/Open configuration file in default editor/在默认编辑器中打开配置文件/g" \
          -e "s/Show help for config commands/显示配置命令帮助/g" \
          -e "s/Detect anomalies in backup history/检测备份历史中的异常/g" \
          -e "s/Analyze file importance/分析文件重要性/g" \
          -e "s/Suggest exclude patterns/建议排除模式/g" \
          -e "s/Auto-configure backup settings with smart rules/使用智能规则自动配置/g" \
          -e "s/Show help for smart commands/显示Smart命令帮助/g" \
          ~/.zfunc/_backup-suite
        ;;
    zh-TW)
        # 繁體中文
        sed -i '' \
          -e "s/Add backup target (with interactive file selector)/新增備份目標（支援互動式選擇）/g" \
          -e "s/List backup targets/列出備份目標/g" \
          -e "s/Remove backup target/刪除備份目標/g" \
          -e "s/Clear all backup targets/清除所有備份目標/g" \
          -e "s/Run backup (with encryption and compression support)/執行備份（支援加密和壓縮）/g" \
          -e "s/Restore from backup/從備份還原/g" \
          -e "s/Clean up old backups/清理舊備份/g" \
          -e "s/Show backup status/顯示備份狀態/g" \
          -e "s/Show backup history/顯示備份歷史/g" \
          -e "s/Show interactive dashboard/顯示互動式儀表板/g" \
          -e "s/Open backup directory/開啟備份目錄/g" \
          -e "s/Generate shell completion scripts/產生Shell補全腳本/g" \
          -e "s/Manage backup schedule/管理備份排程/g" \
          -e "s/Configuration management/組態管理/g" \
          -e "s/Smart rule-based intelligent backup management/基於智慧規則的備份管理/g" \
          -e "s/Enable automatic backup/啟用自動備份/g" \
          -e "s/Disable automatic backup/停用自動備份/g" \
          -e "s/Show schedule status/顯示排程狀態/g" \
          -e "s/Setup backup schedule/設定備份排程/g" \
          -e "s/Show help for schedule commands/顯示排程指令說明/g" \
          -e "s/Set backup destination directory/設定備份目標目錄/g" \
          -e "s/Get current backup destination directory/取得目前備份目標目錄/g" \
          -e "s/Set backup retention days/設定備份保留天數/g" \
          -e "s/Get current backup retention days/取得目前備份保留天數/g" \
          -e "s/Open configuration file in default editor/在預設編輯器中開啟組態檔/g" \
          -e "s/Show help for config commands/顯示組態指令說明/g" \
          -e "s/Detect anomalies in backup history/檢測備份歷史中的異常/g" \
          -e "s/Analyze file importance/分析檔案重要性/g" \
          -e "s/Suggest exclude patterns/建議排除模式/g" \
          -e "s/Auto-configure backup settings with smart rules/使用智慧規則自動組態/g" \
          -e "s/Show help for smart commands/顯示Smart指令說明/g" \
          ~/.zfunc/_backup-suite
        ;;
esac

echo "✅ Completion script generated and translated to ${LANG_CODE}"
echo "📍 Location: ~/.zfunc/_backup-suite"
echo ""
echo "Usage:"
echo "  Auto-detect:     ./scripts/generate-completion.sh"
echo "  Japanese:        ./scripts/generate-completion.sh ja"
echo "  Simplified CN:   ./scripts/generate-completion.sh zh-CN"
echo "  Traditional TW:  ./scripts/generate-completion.sh zh-TW"
echo "  English:         ./scripts/generate-completion.sh en"
echo ""
echo "To reload in current shell:"
echo "  source ~/.zfunc/_backup-suite"
