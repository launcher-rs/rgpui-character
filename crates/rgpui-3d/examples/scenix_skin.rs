use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rgpui::{
    App, Bounds, Context, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render,
    RenderImage, ScrollDelta, ScrollWheelEvent, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, img, prelude::*, px, rgb, size,
};
use rgpui_platform::application;
use rgpui_3d::Scenix3D;
use rgpui_3d::scenix::{self, PerspectiveCamera, SceneGraph, Vec3};

const RENDER_W: u32 = 800;
const RENDER_H: u32 = 600;

struct SharedState {
    orbit_x: f32,
    orbit_y: f32,
    distance: f32,
    is_dragging: bool,
    drag_start_x: f32,
    drag_start_y: f32,
    render_image: Option<Arc<RenderImage>>,
    fps: f32,
    model_info: String,
    anim_names: Vec<String>,
    current_anim: usize,
    anim_time: f32,
    anim_duration: f32,
    anim_speed: f32,
    anim_paused: bool,
    joint_count: usize,
    skin_count: usize,
}

impl SharedState {
    fn new() -> Self {
        Self {
            orbit_x: 0.0,
            orbit_y: 0.2,
            distance: 3.5,
            is_dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            render_image: None,
            fps: 0.0,
            model_info: String::new(),
            anim_names: Vec::new(),
            current_anim: 0,
            anim_time: 0.0,
            anim_duration: 0.0,
            anim_speed: 1.0,
            anim_paused: false,
            joint_count: 0,
            skin_count: 0,
        }
    }
}

struct SkinView {
    state: Arc<Mutex<SharedState>>,
}

impl Render for SkinView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.state.lock().unwrap();
        let fps = s.fps;
        let info = s.model_info.clone();
        let ox = s.orbit_x;
        let oy = s.orbit_y;
        let dist = s.distance;
        let anim_names = s.anim_names.clone();
        let current_anim = s.current_anim;
        let anim_time = s.anim_time;
        let anim_duration = s.anim_duration;
        let anim_speed = s.anim_speed;
        let anim_paused = s.anim_paused;
        let joint_count = s.joint_count;
        let skin_count = s.skin_count;

        let img_elem = match &s.render_image {
            Some(img_ref) => div()
                .size(px(RENDER_W as f32))
                .size(px(RENDER_H as f32))
                .child(img(img_ref.clone())),
            None => div()
                .size(px(RENDER_W as f32))
                .size(px(RENDER_H as f32))
                .bg(rgb(0xffffff))
                .flex()
                .items_center()
                .justify_center()
                .text_xl()
                .text_color(rgb(0x333333))
                .child("加载中..."),
        };
        drop(s);

        let state = self.state.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.0))
            .bg(rgb(0xffffff))
            .size(px(900.0))
            .size(px(780.0))
            .child(
                div().py(px(8.0)).child(
                    div()
                        .text_2xl()
                        .text_color(rgb(0x222222))
                        .child("rgpui + scenix 骨骼动画"),
                ),
            )
            .child(
                div()
                    .cursor_grab()
                    .on_mouse_down(MouseButton::Left, {
                        let state = state.clone();
                        move |ev: &MouseDownEvent, _, _| {
                            let mut s = state.lock().unwrap();
                            s.is_dragging = true;
                            s.drag_start_x = ev.position.x.as_f32();
                            s.drag_start_y = ev.position.y.as_f32();
                        }
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let state = state.clone();
                        move |_: &MouseUpEvent, _, _| {
                            state.lock().unwrap().is_dragging = false;
                        }
                    })
                    .on_mouse_move({
                        let state = state.clone();
                        move |ev: &MouseMoveEvent, _, _| {
                            let mut s = state.lock().unwrap();
                            if s.is_dragging {
                                let dx = ev.position.x.as_f32() - s.drag_start_x;
                                let dy = ev.position.y.as_f32() - s.drag_start_y;
                                s.drag_start_x = ev.position.x.as_f32();
                                s.drag_start_y = ev.position.y.as_f32();
                                s.orbit_x += dx * 0.008;
                                s.orbit_y += dy * 0.008;
                                s.orbit_y = s.orbit_y.clamp(-1.5, 1.5);
                            }
                        }
                    })
                    .on_scroll_wheel({
                        let state = state;
                        move |ev: &ScrollWheelEvent, _, _| {
                            let mut s = state.lock().unwrap();
                            let delta = match ev.delta {
                                ScrollDelta::Pixels(d) => d.y.as_f32(),
                                ScrollDelta::Lines(d) => d.y * 20.0,
                            };
                            s.distance -= delta * 0.05;
                            s.distance = s.distance.clamp(1.0, 50.0);
                        }
                    })
                    .child(img_elem),
            )
            // 动画控制栏
            .child(
                div()
                    .py(px(6.0))
                    .px(px(8.0))
                    .flex()
                    .gap(px(12.0))
                    .items_center()
                    .child(
                        div()
                            .text_color(rgb(0x1565c0))
                            .text_sm()
                            .child(format!("FPS: {:.0}", fps)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x555555))
                            .text_sm()
                            .child(info),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x777777))
                            .text_sm()
                            .child(format!("关节: {} | 皮肤: {}", joint_count, skin_count)),
                    ),
            )
            // 动画播放控制
            .child(
                div()
                    .py(px(4.0))
                    .px(px(8.0))
                    .flex()
                    .gap(px(8.0))
                    .items_center()
                    .child(
                        div()
                            .text_color(rgb(0xe65100))
                            .text_sm()
                            .child(format!(
                                "动画: {}",
                                anim_names
                                    .get(current_anim)
                                    .map(|n| n.as_str())
                                    .unwrap_or("无")
                            )),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x666666))
                            .text_sm()
                            .child(format!(
                                "{:.2}s / {:.2}s",
                                anim_time, anim_duration
                            )),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x666666))
                            .text_sm()
                            .child(format!("速度: {:.1}x", anim_speed)),
                    )
                    .child(
                        div()
                            .text_color(if anim_paused {
                                rgb(0xc62828)
                            } else {
                                rgb(0x2e7d32)
                            })
                            .text_sm()
                            .child(if anim_paused { "暂停" } else { "播放" }),
                    ),
            )
            // 提示信息
            .child(
                div()
                    .py(px(4.0))
                    .px(px(8.0))
                    .flex()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_color(rgb(0x999999))
                            .text_xs()
                            .child("拖拽旋转 | 滚轮缩放"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x999999))
                            .text_xs()
                            .child(format!(
                                "视角 ({:.0}°, {:.0}°) | 距离 {:.1}",
                                ox.to_degrees(),
                                oy.to_degrees(),
                                dist
                            )),
                    ),
            )
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut model_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                model_path = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    if model_path.is_none() {
        let default_path = format!("{}/examples/3d/person.glb", env!("CARGO_MANIFEST_DIR"));
        if std::path::Path::new(&default_path).exists() {
            model_path = Some(default_path);
        }
    }

    let shared = Arc::new(Mutex::new(SharedState::new()));
    let render_shared = shared.clone();

    std::thread::spawn(move || {
        let mut ctx: Scenix3D = smol::block_on(async {
            Scenix3D::new(RENDER_W, RENDER_H)
                .await
                .expect("创建 3D 上下文失败")
        });

        // 设置白色背景
        ctx.set_clear_color(1.0, 1.0, 1.0, 1.0);

        let mut loaded_model: Option<SceneGraph> = None;
        let mut info: String = "未加载模型".into();
        let mut anim_names: Vec<String> = Vec::new();

                        if let Some(ref path) = model_path {
            let loader = scenix::GltfLoader::new();
            match loader.load_file(path) {
                Ok(asset) => match ctx.register_gltf_asset(&asset) {
                    Ok(_) => {
                        // 加载蒙皮和动画（需要完整的 asset）
                        let skin_result = ctx.load_gltf_skins(path, &asset);
                        let mut names = ctx.animation_names();

                        // 如果没有动画数据，生成程序化行走动画
                        if names.is_empty() && ctx.joint_count() > 0 {
                            ctx.generate_walk_animation(0.8, 0.5);
                            names = ctx.animation_names();
                        }

                        if !names.is_empty() {
                            anim_names = names;
                        }
                        info = format!(
                            "{} | {} 网格, {} 材质{}",
                            std::path::Path::new(path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                            asset.meshes.len(),
                            asset.materials.len(),
                            if !anim_names.is_empty() {
                                format!(" | {} 动画", anim_names.len())
                            } else {
                                String::new()
                            }
                        );
                        if skin_result.is_err() && anim_names.is_empty() {
                            info.push_str(" | 蒙皮加载失败");
                        }
                        loaded_model = Some(asset.scene);
                    }
                    Err(e) => {
                        info = format!("GPU 注册失败: {e}");
                    }
                },
                Err(e) => {
                    info = format!("加载失败: {e}");
                }
            }
        }

        {
            let mut s = render_shared.lock().unwrap();
            s.model_info = info;
            s.anim_names = anim_names;
            s.joint_count = ctx.joint_count();
            s.skin_count = ctx.skinned_mesh_count();
        }

        let mut frame_times: Vec<f32> = Vec::with_capacity(30);
        let mut last_time = Instant::now();

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last_time).as_secs_f32();
            last_time = frame_start;

            let (mut ox, oy, dist) = {
                let s = render_shared.lock().unwrap();
                (s.orbit_x, s.orbit_y, s.distance)
            };

            ox += 0.008;

            {
                let mut s = render_shared.lock().unwrap();
                s.orbit_x = ox;
            }

            let cam_pos = Vec3::new(
                dist * ox.sin() * oy.cos(),
                dist * oy.sin(),
                dist * ox.cos() * oy.cos(),
            );
            let camera =
                PerspectiveCamera::new(45.0, RENDER_W as f32 / RENDER_H as f32, 0.1, 100.0)
                    .position(cam_pos)
                    .target(Vec3::ZERO);

            if let Some(ref mut scene) = loaded_model {
                // 每帧推进动画
                ctx.advance_animation(dt);
                let result = ctx.render(scene, &camera).expect("渲染失败");
                let render_image = Arc::new(result.into_render_image());

                // 同步动画状态到 UI
                {
                    let mut s = render_shared.lock().unwrap();
                    s.render_image = Some(render_image);
                    s.anim_time = ctx.animation_time();
                    s.anim_duration = ctx.animation_duration();
                    s.anim_speed = ctx.animation_speed();
                    s.anim_paused = ctx.is_animation_paused();
                }
            }

            let elapsed = frame_start.elapsed().as_secs_f32();
            frame_times.push(elapsed);
            if frame_times.len() > 30 {
                frame_times.remove(0);
            }
            let avg_fps = frame_times.len() as f32 / frame_times.iter().sum::<f32>();
            {
                let mut s = render_shared.lock().unwrap();
                s.fps = avg_fps;
            }

            let frame_time = frame_start.elapsed();
            let target = Duration::from_millis(33);
            if frame_time < target {
                std::thread::sleep(target - frame_time);
            }
        }
    });

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(780.0)), cx);
        let state = shared;

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // window_background: WindowBackgroundAppearance::Transparent,
                titlebar: Some(TitlebarOptions {
                    title: Some("rgpui + scenix 骨骼动画".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            move |_, cx| {
                let s = state;
                cx.new(move |cx| {
                    cx.spawn(async move |this, cx| {
                        loop {
                            smol::Timer::after(Duration::from_millis(33)).await;
                            let _ = this.update(cx, |_, cx| cx.notify());
                        }
                    })
                    .detach();
                    SkinView { state: s }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
