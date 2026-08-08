#![warn(clippy::all, clippy::pedantic)]
#![deny(
    clippy::unimplemented,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic
)]
#![forbid(unsafe_code)]

mod app;
mod screens;

use gpui::{AppContext, Bounds, Size, WindowBounds, WindowOptions, px};
use gpui_component::{Root, TitleBar};

/// 应用入口，供 [`main.rs`] 和 [`examples/`] 调用。
pub fn run() {
    // 加载配置并初始化日志系统
    let config_data = echo_core::config::load_config(None).unwrap_or_default();
    if let Err(e) = echo_core::log::init(&config_data.log) {
        eprintln!("Failed to initialize logger: {e}");
    }
    log::info!("Echo application starting");

    // 注册内置图标资源（SVG），否则 `IconName` 的图标无法渲染。
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            // 使用任何 GPUI Component 功能之前必须先调用
            gpui_component::init(cx);

            // 设置窗口初始大小为 900x600，并在屏幕居中
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,                            // 不指定特定屏幕，使用主屏幕
                    Size::new(px(900.0), px(600.0)), // 宽900，高600
                    cx,
                ))),
                titlebar: Some(TitleBar::title_bar_options()),
                ..Default::default() // 其他选项保持默认
            };

            let window = cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| app::EchoApp::new(window, cx, config_data.clone()));
                cx.new(|cx| Root::new(view, window, cx))
            });

            if let Err(err) = window {
                log::error!("Failed to open window: {err:#}");
            }
        });
}
