//! 配置系统基准测试

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use echo_core::config::{ConfigData, Layers, LogLevel, VaultEntry, save_config, validate};

/// 创建一个包含多个仓库条目和完整日志配置的测试配置
fn create_test_config() -> ConfigData {
    let mut config = ConfigData::default();
    config.vault.path = Some("/home/user/notes".to_string());
    config.vault.auto_index = true;
    // 添加多个 recent entries
    for i in 0..100 {
        config.vault.recent.push(VaultEntry {
            path: format!("/home/user/notes{}", i),
            name: Some(format!("Notes {}", i)),
            last_opened: Some("2026-08-08T10:00:00Z".to_string()),
        });
    }
    config.log.level = LogLevel::Debug;
    config.log.console_output = true;
    config.log.file_output = true;
    config.log.file_path = Some("/tmp/echo.log".to_string());
    config
}

/// 基准测试：TOML 序列化
fn bench_serialize(c: &mut Criterion) {
    let config = create_test_config();
    c.bench_function("serialize_config", |b| {
        b.iter(|| {
            black_box(toml::to_string(black_box(&config)).unwrap());
        });
    });
}

/// 基准测试：TOML 反序列化
fn bench_deserialize(c: &mut Criterion) {
    let config = create_test_config();
    let toml_str = toml::to_string(&config).unwrap();
    c.bench_function("deserialize_config", |b| {
        b.iter(|| {
            black_box(toml::from_str::<ConfigData>(black_box(&toml_str)).unwrap());
        });
    });
}

/// 基准测试：配置合并
fn bench_merge(c: &mut Criterion) {
    let mut base = ConfigData::default();
    base.vault.path = Some("/base/path".to_string());
    let mut overlay = ConfigData::default();
    overlay.vault.path = Some("/overlay/path".to_string());
    c.bench_function("merge_configs", |b| {
        b.iter(|| {
            let layers = Layers {
                global: Some(base.clone()),
                workspace: Some(overlay.clone()),
            };
            black_box(layers.merge().unwrap());
        });
    });
}

/// 基准测试：配置校验
fn bench_validate(c: &mut Criterion) {
    let config = create_test_config();
    c.bench_function("validate_config", |b| {
        b.iter(|| {
            validate(black_box(&config)).unwrap();
        });
    });
}

/// 基准测试：添加最近使用的仓库
fn bench_add_recent(c: &mut Criterion) {
    c.bench_function("add_recent", |b| {
        b.iter(|| {
            let mut config = ConfigData::default();
            for i in 0..100 {
                config.vault.add_recent(format!("/path/to/vault{}", i));
            }
            black_box(config);
        });
    });
}

/// 基准测试：保存配置到磁盘
fn bench_save_config(c: &mut Criterion) {
    let config = create_test_config();
    let dir = std::env::temp_dir().join(format!("echo-bench-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    c.bench_function("save_config", |b| {
        b.iter(|| {
            save_config(black_box(&config), black_box(&path)).unwrap();
        });
    });
    let _ = std::fs::remove_dir_all(&dir);
}

criterion_group!(
    benches,
    bench_serialize,
    bench_deserialize,
    bench_merge,
    bench_validate,
    bench_add_recent,
    bench_save_config
);
criterion_main!(benches);
