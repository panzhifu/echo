//! echo-vault 基准测试
//!
//! 覆盖以下场景：
//! - 事件路径提取 (`VaultEvent::path`)
//! - 忽略过滤器构建 (`IgnoreFilter::new`)
//! - 忽略过滤器匹配 (`IgnoreFilter::is_ignored`)
//! - 监控器构建 (`VaultWatcher::new` / `with_paths`)
//! - 链式构建器 (`ignore_patterns` + `debounce`)

use std::path::{Path, PathBuf};
use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use echo_vault::{IgnoreFilter, VaultEvent, VaultWatcher};

/// 基准测试：忽略过滤器缓存（get_or_create_filter）
fn bench_ignore_filter_cached(c: &mut Criterion) {
    echo_vault::filter_cache::clear_cache();
    let patterns: Vec<String> = (0..50).map(|i| format!("*.ext{i}")).collect();

    // 预热：首次创建填充缓存
    let _ = echo_vault::filter_cache::get_or_create(PathBuf::from("/"), patterns.clone()).unwrap();

    c.bench_function("ignore_filter_cached_hit", |b| {
        b.iter(|| {
            black_box(
                echo_vault::filter_cache::get_or_create(
                    black_box(PathBuf::from("/")),
                    black_box(patterns.clone()),
                )
                .unwrap(),
            );
        });
    });
}

/// 基准测试：事件路径提取
fn bench_event_path(c: &mut Criterion) {
    let create = VaultEvent::Create {
        path: PathBuf::from("/home/user/vault/notes.md"),
    };
    let rename = VaultEvent::Rename {
        from: PathBuf::from("/home/user/vault/old.md"),
        to: PathBuf::from("/home/user/vault/new.md"),
    };

    c.bench_function("event_path_create", |b| {
        b.iter(|| {
            black_box(create.path());
        });
    });
    c.bench_function("event_path_rename", |b| {
        b.iter(|| {
            black_box(rename.path());
        });
    });
}

/// 基准测试：忽略过滤器构建
fn bench_ignore_filter_new(c: &mut Criterion) {
    let patterns: Vec<String> = (0..50).map(|i| format!("*.ext{i}")).collect();

    c.bench_function("ignore_filter_new_50", |b| {
        b.iter(|| {
            black_box(IgnoreFilter::new(Path::new("/"), black_box(&patterns)).unwrap());
        });
    });

    let patterns_small: Vec<String> = vec!["*.tmp".into(), ".git/".into(), "*.log".into()];
    c.bench_function("ignore_filter_new_3", |b| {
        b.iter(|| {
            black_box(
                IgnoreFilter::new(Path::new("/home/user/vault"), black_box(&patterns_small))
                    .unwrap(),
            );
        });
    });
}

/// 基准测试：忽略过滤器匹配
fn bench_ignore_filter_match(c: &mut Criterion) {
    let filter = IgnoreFilter::new(
        Path::new("/home/user/vault"),
        &[
            "*.tmp".to_string(),
            ".git/".to_string(),
            "node_modules/".to_string(),
            "*.log".to_string(),
            "**/*.bak".to_string(),
        ],
    )
    .unwrap();

    let matched_path = PathBuf::from("/home/user/vault/temp.tmp");
    let unmatched_path = PathBuf::from("/home/user/vault/notes.md");
    let nested_ignored = PathBuf::from("/home/user/vault/.git/config");

    c.bench_function("ignore_filter_match_ignored", |b| {
        b.iter(|| {
            black_box(filter.is_ignored(black_box(&matched_path)));
        });
    });
    c.bench_function("ignore_filter_match_not_ignored", |b| {
        b.iter(|| {
            black_box(filter.is_ignored(black_box(&unmatched_path)));
        });
    });
    c.bench_function("ignore_filter_match_nested_dir", |b| {
        b.iter(|| {
            black_box(filter.is_ignored(black_box(&nested_ignored)));
        });
    });
}

/// 基准测试：VaultWatcher 构建
fn bench_watcher_new(c: &mut Criterion) {
    c.bench_function("watcher_new_single", |b| {
        b.iter(|| {
            black_box(VaultWatcher::new(black_box("/path/to/vault")));
        });
    });

    let paths: Vec<PathBuf> = (0..10)
        .map(|i| PathBuf::from(format!("/path/to/vault{i}")))
        .collect();
    c.bench_function("watcher_with_paths_10", |b| {
        b.iter(|| {
            black_box(VaultWatcher::with_paths(black_box(paths.clone())));
        });
    });
}

/// 基准测试：VaultWatcher 链式构建器
fn bench_watcher_builder(c: &mut Criterion) {
    let patterns: Vec<String> = (0..20).map(|i| format!("*.ext{i}")).collect();

    c.bench_function("watcher_builder_full", |b| {
        b.iter(|| {
            black_box(
                VaultWatcher::new("/path/to/vault")
                    .ignore_patterns(black_box(patterns.clone()))
                    .debounce(black_box(Duration::from_millis(200))),
            );
        });
    });

    c.bench_function("watcher_builder_no_debounce", |b| {
        b.iter(|| {
            black_box(
                VaultWatcher::new("/path/to/vault")
                    .ignore_patterns(black_box(patterns.clone()))
                    .debounce(black_box(Duration::ZERO)),
            );
        });
    });
}

criterion_group!(
    benches,
    bench_event_path,
    bench_ignore_filter_cached,
    bench_ignore_filter_new,
    bench_ignore_filter_match,
    bench_watcher_new,
    bench_watcher_builder,
);
criterion_main!(benches);
