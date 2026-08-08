//! echo-app 应用逻辑基准测试。
//!
//! 覆盖以下场景：
//! - 应用状态派生 (`AppState::from_vault`)
//! - 仓库选择应用 (`apply_vault_selection`)

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use echo_app::app_logic::{AppState, apply_vault_selection};
use echo_core::config::ConfigData;

/// 基准测试：应用状态派生
fn bench_app_state(c: &mut Criterion) {
    let mut with_vault = ConfigData::default();
    with_vault.vault.path = Some("/home/user/notes".into());
    let no_vault = ConfigData::default();

    c.bench_function("app_state_vault_loaded", |b| {
        b.iter(|| {
            black_box(AppState::from_vault(black_box(&with_vault.vault)));
        });
    });
    c.bench_function("app_state_no_vault", |b| {
        b.iter(|| {
            black_box(AppState::from_vault(black_box(&no_vault.vault)));
        });
    });
}

/// 基准测试：仓库选择应用
fn bench_apply_vault_selection(c: &mut Criterion) {
    c.bench_function("apply_vault_selection_single", |b| {
        b.iter(|| {
            let mut config = ConfigData::default();
            apply_vault_selection(&mut config, black_box("/path/to/vault"));
            black_box(config);
        });
    });

    c.bench_function("apply_vault_selection_100_recent", |b| {
        b.iter(|| {
            let mut config = ConfigData::default();
            for i in 0..100 {
                apply_vault_selection(&mut config, format!("/path/to/vault{i}"));
            }
            black_box(config);
        });
    });
}

criterion_group!(benches, bench_app_state, bench_apply_vault_selection);
criterion_main!(benches);
