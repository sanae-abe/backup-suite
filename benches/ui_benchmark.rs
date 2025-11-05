use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::{self, Write};

// UI関連の関数（実装を想定）

fn format_progress_bar(current: u64, total: u64, width: usize) -> String {
    let percentage = if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64) * 100.0
    };

    let filled = ((current as f64 / total as f64) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let bar = "█".repeat(filled) + &"░".repeat(empty);
    format!("[{}] {:.1}% ({}/{})", bar, percentage, current, total)
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn colorize_text(text: &str, color: &str) -> String {
    let color_code = match color {
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "cyan" => "\x1b[36m",
        _ => "",
    };

    format!("{}{}\x1b[0m", color_code, text)
}

fn format_table_row(columns: &[&str], widths: &[usize]) -> String {
    columns.iter()
        .zip(widths.iter())
        .map(|(col, width)| format!("{:width$}", col, width = width))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn render_spinner(frame: usize) -> String {
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    spinners[frame % spinners.len()].to_string()
}

// ベンチマーク関数

fn bench_format_progress_bar(c: &mut Criterion) {
    let test_cases = vec![
        (0, 100, "開始時"),
        (50, 100, "50%"),
        (100, 100, "完了時"),
        (500, 1000, "大きな数値"),
    ];

    let mut group = c.benchmark_group("format_progress_bar");

    for (current, total, name) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(current, total),
            |b, &(current, total)| {
                b.iter(|| format_progress_bar(black_box(current), black_box(total), black_box(40)));
            },
        );
    }

    group.finish();
}

fn bench_format_file_size(c: &mut Criterion) {
    let test_cases = vec![
        (512, "512B"),
        (1024, "1KB"),
        (1024 * 1024, "1MB"),
        (1024 * 1024 * 1024, "1GB"),
        (1024_u64 * 1024 * 1024 * 1024, "1TB"),
        (123_456_789, "混合サイズ"),
    ];

    let mut group = c.benchmark_group("format_file_size");

    for (bytes, name) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &bytes,
            |b, &bytes| {
                b.iter(|| format_file_size(black_box(bytes)));
            },
        );
    }

    group.finish();
}

fn bench_format_duration(c: &mut Criterion) {
    let test_cases = vec![
        (30, "30秒"),
        (90, "1分30秒"),
        (3600, "1時間"),
        (7265, "2時間1分5秒"),
    ];

    let mut group = c.benchmark_group("format_duration");

    for (seconds, name) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &seconds,
            |b, &seconds| {
                b.iter(|| format_duration(black_box(seconds)));
            },
        );
    }

    group.finish();
}

fn bench_colorize_text(c: &mut Criterion) {
    let colors = vec!["red", "green", "yellow", "blue", "cyan"];
    let text = "テストメッセージ";

    let mut group = c.benchmark_group("colorize_text");

    for color in colors {
        group.bench_with_input(
            BenchmarkId::from_parameter(color),
            &color,
            |b, &color| {
                b.iter(|| colorize_text(black_box(text), black_box(color)));
            },
        );
    }

    group.finish();
}

fn bench_format_table_row(c: &mut Criterion) {
    let columns = vec!["Column1", "Column2", "Column3", "Column4"];
    let widths = vec![15, 20, 25, 10];

    c.bench_function("format_table_row", |b| {
        b.iter(|| format_table_row(black_box(&columns), black_box(&widths)));
    });
}

fn bench_render_spinner(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_spinner");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_frame", |b| {
        let mut frame = 0;
        b.iter(|| {
            let result = render_spinner(black_box(frame));
            frame = (frame + 1) % 10;
            result
        });
    });

    group.finish();
}

fn bench_ui_full_render(c: &mut Criterion) {
    c.bench_function("ui_full_render", |b| {
        b.iter(|| {
            let mut output = Vec::new();

            // ヘッダー
            writeln!(output, "{}", colorize_text("=== バックアップ状況 ===", "cyan")).unwrap();

            // プログレスバー
            let progress = format_progress_bar(500, 1000, 40);
            writeln!(output, "{}", progress).unwrap();

            // 統計情報
            writeln!(output, "処理済み: {}", format_file_size(500 * 1024 * 1024)).unwrap();
            writeln!(output, "経過時間: {}", format_duration(125)).unwrap();

            // スピナー
            writeln!(output, "{} 処理中...", render_spinner(0)).unwrap();

            black_box(output);
        });
    });
}

fn bench_large_table_render(c: &mut Criterion) {
    let rows = 100;
    let columns = vec!["ファイル名", "サイズ", "状態", "日時"];
    let widths = vec![30, 15, 10, 20];

    let mut group = c.benchmark_group("large_table_render");
    group.throughput(Throughput::Elements(rows));

    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut output = Vec::new();

            for i in 0..rows {
                let row_data = vec![
                    &format!("file_{}.txt", i),
                    &format_file_size((i as u64) * 1024),
                    "完了",
                    "2025-11-05 10:00:00",
                ];
                writeln!(output, "{}", format_table_row(&row_data, &widths)).unwrap();
            }

            black_box(output);
        });
    });

    group.finish();
}

fn bench_progress_bar_updates(c: &mut Criterion) {
    let total = 1000;

    let mut group = c.benchmark_group("progress_bar_updates");
    group.throughput(Throughput::Elements(total));

    group.bench_function("frequent_updates", |b| {
        b.iter(|| {
            for i in 0..total {
                black_box(format_progress_bar(i, total, 40));
            }
        });
    });

    group.bench_function("sparse_updates", |b| {
        b.iter(|| {
            for i in (0..total).step_by(10) {
                black_box(format_progress_bar(i, total, 40));
            }
        });
    });

    group.finish();
}

fn bench_interactive_prompt_render(c: &mut Criterion) {
    c.bench_function("interactive_prompt_render", |b| {
        b.iter(|| {
            let mut output = Vec::new();

            writeln!(output, "{}", colorize_text("🎯 バックアップ設定", "cyan")).unwrap();
            writeln!(output, "").unwrap();
            writeln!(output, "対象ディレクトリ: {}", colorize_text("/home/user/documents", "green")).unwrap();
            writeln!(output, "バックアップ先: {}", colorize_text("/backup/dest", "green")).unwrap();
            writeln!(output, "").unwrap();
            writeln!(output, "{}", colorize_text("実行しますか? [Y/n]:", "yellow")).unwrap();

            black_box(output);
        });
    });
}

fn bench_error_message_formatting(c: &mut Criterion) {
    let errors = vec![
        "ファイルが見つかりません",
        "権限がありません",
        "ディスク容量不足",
        "ネットワークエラー",
    ];

    let mut group = c.benchmark_group("error_message_formatting");

    for error in errors {
        group.bench_with_input(
            BenchmarkId::from_parameter(error),
            &error,
            |b, &error| {
                b.iter(|| {
                    let mut output = Vec::new();
                    writeln!(output, "{} {}",
                        colorize_text("❌", "red"),
                        colorize_text(error, "red")
                    ).unwrap();
                    black_box(output);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_progress_bar,
    bench_format_file_size,
    bench_format_duration,
    bench_colorize_text,
    bench_format_table_row,
    bench_render_spinner,
    bench_ui_full_render,
    bench_large_table_render,
    bench_progress_bar_updates,
    bench_interactive_prompt_render,
    bench_error_message_formatting,
);

criterion_main!(benches);
