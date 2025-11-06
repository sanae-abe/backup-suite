# Pipeline並列処理の最適化

## 概要

`src/core/pipeline.rs`のrayon並列処理を最適化し、高性能なバックアップ処理を実現しました。

## 実装された最適化

### 1. 最適な並列度計算

```rust
pub fn optimal_parallelism() -> usize {
    let cpus = num_cpus::get();
    (cpus * 3 / 4).clamp(1, 32)
}
```

**特徴:**
- CPU コア数の75%を使用し、システムリソースを確保
- 最小1スレッド、最大32スレッドに制限
- OS側のプロセス実行を妨げない適切なバランス

**動作例:**
- 4コアCPU: 3スレッド
- 8コアCPU: 6スレッド
- 16コアCPU: 12スレッド
- 64コアCPU: 32スレッド（上限）

### 2. 動的並列度調整

```rust
pub fn dynamic_parallelism(file_count: usize, avg_file_size: u64) -> usize
```

**特徴:**
- ファイル数とファイルサイズに基づいて最適なスレッド数を自動計算
- 小さいファイル（< 1MB）: オーバーヘッド考慮で並列度を抑える
- 大きいファイル（> 100MB）: 並列度を増やして高速化
- ファイル数が少ない場合: 無駄なスレッド生成を回避

**調整ルール:**
1. `file_count < base_parallelism`: ファイル数まで並列度を抑制
2. `avg_file_size < 1MB`: 並列度を半分に削減
3. `avg_file_size > 100MB`: 並列度を4/3倍に増加（最大32）

### 3. カスタムThreadPool設定

```rust
fn create_thread_pool(performance: &PerformanceConfig) -> Result<rayon::ThreadPool> {
    ThreadPoolBuilder::new()
        .num_threads(performance.parallel_threads)
        .thread_name(|i| format!("backup-worker-{}", i))
        .stack_size(8 * 1024 * 1024) // 8MBスタックサイズ
        .build()
}
```

**特徴:**
- 専用のThreadPoolを作成し、グローバルプールへの影響を回避
- スレッド名を明示的に設定し、デバッグを容易化
- 8MBスタックサイズで大容量ファイル処理に対応

### 4. バッチ並列処理

```rust
pub fn process_files_parallel<P: AsRef<Path> + Send + Sync>(
    &self,
    files: &[P],
    master_key: Option<&MasterKey>,
    salt: Option<[u8; 16]>,
) -> Vec<Result<ProcessedData>>
```

**特徴:**
- 複数ファイルを効率的に並列処理
- バッチサイズを設定可能（デフォルト32ファイル/バッチ）
- ワークスティーリングによる負荷分散

**実装詳細:**
```rust
files
    .par_chunks(batch_size)
    .flat_map(|batch| {
        batch.par_iter().map(|file| {
            self.process_file(file, master_key, salt)
        }).collect::<Vec<_>>()
    })
    .collect()
```

### 5. 進捗コールバック付き並列処理

```rust
pub fn process_files_with_progress<P, F>(
    &self,
    files: &[P],
    master_key: Option<&MasterKey>,
    salt: Option<[u8; 16]>,
    progress_callback: F,
) -> Vec<Result<ProcessedData>>
where
    F: Fn(usize, usize) + Send + Sync
```

**特徴:**
- 並列処理中の進捗をリアルタイムで取得
- UIプログレスバーとの統合が容易
- スレッドセーフなコールバック機構

## パフォーマンス設定

### デフォルト設定

```rust
PerformanceConfig {
    parallel_threads: optimal_parallelism(), // CPU コア数の75%
    buffer_size: 1024 * 1024,                // 1MB
    memory_limit: 512 * 1024 * 1024,         // 512MB
    batch_size: 32,                          // 32ファイル/バッチ
}
```

### 高速設定

```rust
let config = PipelineConfig::default().fast();
// parallel_threads: 全コア使用
// buffer_size: 2MB
// memory_limit: 1GB
// batch_size: 64ファイル/バッチ
```

### 品質重視設定

```rust
let config = PipelineConfig::default().best_compression();
// parallel_threads: コア数の半分
// buffer_size: 512KB
// memory_limit: 256MB
// batch_size: 16ファイル/バッチ
```

### カスタム設定

```rust
let perf_config = PerformanceConfig::default()
    .with_parallelism(8)  // 8スレッド
    .with_batch_size(16); // 16ファイル/バッチ

let mut pipeline_config = PipelineConfig::default();
pipeline_config.performance = perf_config;
let pipeline = ProcessingPipeline::new(pipeline_config);
```

## 使用例

### 基本的な並列処理

```rust
use backup_suite::core::pipeline::{ProcessingPipeline, PipelineConfig};

let config = PipelineConfig::default().fast();
let pipeline = ProcessingPipeline::new(config);

let files = vec![
    "/path/to/file1.txt",
    "/path/to/file2.txt",
    "/path/to/file3.txt",
];

let results = pipeline.process_files_parallel(&files, None, None);

for (i, result) in results.iter().enumerate() {
    match result {
        Ok(processed) => {
            println!(
                "File {}: {} -> {} bytes ({}% compression)",
                i,
                processed.metadata.original_size,
                processed.metadata.final_size,
                processed.metadata.compression_ratio
            );
        }
        Err(e) => {
            eprintln!("File {} error: {}", i, e);
        }
    }
}
```

### 進捗表示付き並列処理

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

let pipeline = ProcessingPipeline::new(PipelineConfig::default().fast());

let files = vec![/* ... */];
let total = files.len();

let results = pipeline.process_files_with_progress(
    &files,
    None,
    None,
    |current, total| {
        println!("進捗: {}/{} ({}%)", current, total, current * 100 / total);
    },
);

println!("完了: {}/{} ファイル処理成功",
    results.iter().filter(|r| r.is_ok()).count(),
    total
);
```

### 暗号化付き並列処理

```rust
let (pipeline, salt) = ProcessingPipeline::with_encryption("password123")?;

let files = vec![/* ... */];
let master_key = /* 取得したマスターキー */;

let results = pipeline.process_files_parallel(&files, Some(&master_key), Some(salt));
```

## パフォーマンスベンチマーク

### 理論値（100MBファイル x 100個、8コアCPU）

| 設定 | 並列度 | 予想スループット | 処理時間 |
|------|--------|------------------|----------|
| デフォルト | 6スレッド | 600MB/s | 16.7秒 |
| 高速設定 | 8スレッド | 800MB/s | 12.5秒 |
| 品質重視 | 4スレッド | 400MB/s | 25.0秒 |

### 実測値（ベンチマーク予定）

```bash
cargo bench --bench integration_benchmark -- parallel
```

## 監視とデバッグ

### ThreadPool状態の確認

```rust
let pipeline = ProcessingPipeline::new(config);

// ThreadPoolが正常に作成されているか
assert!(pipeline.is_parallel_ready());

// 現在の並列度
println!("並列度: {}", pipeline.current_parallelism());

// パフォーマンス統計
let stats = pipeline.get_performance_stats();
println!("利用可能スレッド: {}", stats.available_threads);
println!("バッファサイズ: {} bytes", stats.buffer_size);
```

### スレッド名でのプロファイリング

ThreadPoolのスレッド名は`backup-worker-{id}`の形式で設定されているため、
プロファイラー（Instruments、perf等）で簡単に識別できます。

```bash
# macOS Instruments
instruments -t "Time Profiler" ./target/release/backup-suite backup run

# Linux perf
perf record -g ./target/release/backup-suite backup run
perf report
```

## トラブルシューティング

### ThreadPool作成エラー

```rust
// エラー例: "ThreadPool作成エラー: ..."
// 原因: 並列度が0、またはシステムリソース不足
```

**解決策:**
1. 並列度を明示的に設定: `with_parallelism(1)` で最小値を保証
2. メモリ制限を緩和: `memory_limit`を増やす
3. システムリソースを確認: `ulimit -a` でファイル記述子数等を確認

### 並列処理が遅い

**チェックリスト:**
1. ファイルサイズが小さすぎないか（< 1MB）
   - 解決策: `dynamic_parallelism()`で自動調整
2. ディスクI/Oがボトルネックになっていないか
   - 解決策: SSD使用、またはバッファサイズ調整
3. 並列度が高すぎないか
   - 解決策: `with_parallelism()`で適切な値に調整

## 今後の拡張予定

1. **適応的並列度調整**: 実行時にCPU使用率を監視し、動的に並列度を調整
2. **ストリーミング並列処理**: 大容量ファイルを複数チャンクに分割して並列圧縮
3. **メモリプール**: 頻繁なメモリアロケーションを削減
4. **非同期I/O統合**: tokioとの統合でI/O待機時間を削減

## 参考資料

- [rayon公式ドキュメント](https://docs.rs/rayon/)
- [Rust並行プログラミング](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [パフォーマンス最適化ガイド](https://nnethercote.github.io/perf-book/)
