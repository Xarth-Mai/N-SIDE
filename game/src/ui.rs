//! Neighborhood Signal visual/interaction experiment over the actual Viewer world
//! Sample records are explicitly identified; no quest or save state is fabricated
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    text::{FontWeight, LineHeight},
    ui::InteractionDisabled,
};
use serde::{Deserialize, Serialize};

pub const FONT: &str = "ui/fonts/NotoSansSC-VF.ttf";

#[derive(Resource, Deserialize)]
struct Tokens {
    colors: Colors,
    title_size: f32,
    heading_size: f32,
    body_size: f32,
    line_height: f32,
    panel_radius: f32,
    button_radius: f32,
    panel_slide_ms: f32,
}

#[derive(Deserialize)]
struct Colors {
    base: String,
    raised: String,
    text: String,
    secondary: String,
    focus: String,
    warm: String,
    dream: String,
    warning: String,
}

fn color(hex: &str) -> Color {
    Srgba::hex(hex)
        .expect("UI tokens have validated sRGB colors")
        .into()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Page {
    World,
    #[default]
    Menu,
    Records,
    Settings,
    District,
}

#[derive(Resource, Debug, Serialize)]
pub struct SignalUi {
    pub page: Page,
    pub focus: usize,
    pub selected_record: usize,
    pub scale: f32,
    pub reduced_motion: bool,
    pub device: &'static str,
    menu_focus: usize,
    saved_record_scroll: f32,
    pub scroll: f32,
    pub input_consumed: bool,
}

impl Default for SignalUi {
    fn default() -> Self {
        Self {
            page: Page::Menu,
            focus: 0,
            selected_record: 0,
            scale: 1.0,
            reduced_motion: false,
            device: "keyboard_mouse",
            menu_focus: 0,
            saved_record_scroll: 0.0,
            scroll: 0.0,
            input_consumed: false,
        }
    }
}

impl SignalUi {
    pub fn is_open(&self) -> bool {
        self.page != Page::World
    }

    fn count(&self) -> usize {
        match self.page {
            Page::World => 0,
            Page::Menu => 5,
            Page::Records | Page::Settings => 3,
            Page::District => 1,
        }
    }

    fn back(&mut self) {
        if self.page == Page::Records {
            self.saved_record_scroll = self.scroll;
        }
        match self.page {
            Page::World => {
                self.page = Page::Menu;
                self.focus = self.menu_focus;
            }
            Page::Menu => {
                self.menu_focus = self.focus;
                self.page = Page::World;
            }
            _ => {
                self.page = Page::Menu;
                self.focus = self.menu_focus;
            }
        }
        self.scroll = 0.0;
    }

    fn activate(&mut self) {
        match self.page {
            Page::Menu => {
                self.menu_focus = self.focus;
                match self.focus {
                    0 => self.page = Page::World,
                    1 => return, // Quest runtime is not implemented
                    2 => self.page = Page::Records,
                    3 => self.page = Page::District,
                    _ => self.page = Page::Settings,
                }
                self.focus = if self.page == Page::Records {
                    self.selected_record
                } else {
                    0
                };
                self.scroll = if self.page == Page::Records {
                    self.saved_record_scroll
                } else {
                    0.0
                };
            }
            Page::Records => {
                if self.focus == 2 {
                    self.back();
                } else if self.selected_record != self.focus {
                    self.selected_record = self.focus;
                    self.scroll = 0.0;
                    self.saved_record_scroll = 0.0;
                }
            }
            Page::Settings => match self.focus {
                0 => self.scale = if self.scale == 1.0 { 1.25 } else { 1.0 },
                1 => self.reduced_motion = !self.reduced_motion,
                _ => self.back(),
            },
            Page::District => self.back(),
            Page::World => {}
        }
    }
}

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Control {
    label: String,
    index: usize,
    disabled: bool,
}
#[derive(Component)]
struct Detail;
#[derive(Component)]
struct EnterSlide(f32);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiInput;

pub struct SignalUiPlugin;
impl Plugin for SignalUiPlugin {
    fn build(&self, app: &mut App) {
        let tokens: Tokens =
            serde_json::from_str(include_str!("../../source-assets/ui-kit/tokens.json"))
                .expect("checked UI tokens");
        app.insert_resource(tokens)
            .init_resource::<SignalUi>()
            .add_systems(
                Startup,
                |mut commands: Commands, assets: Res<AssetServer>| {
                    commands.insert_resource(UiFont(assets.load(FONT)));
                },
            )
            .add_systems(RunFixedMainLoop, ui_input.in_set(UiInput))
            .add_systems(
                Update,
                (check_font, draw, style_controls, animate, scroll_detail).chain(),
            );
    }
}

fn check_font(font: Res<UiFont>, assets: Res<AssetServer>, mut exit: MessageWriter<AppExit>) {
    if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(font.0.id()) {
        error!("[ui/font] {FONT}: {error}");
        exit.write(AppExit::error());
    }
}

// Run before the free camera: the opening input must never move the world
fn ui_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    buttons: Query<(&Interaction, &Control), Changed<Interaction>>,
    mut wheel: MessageReader<MouseWheel>,
    scale: Res<UiScale>,
    mut ui: ResMut<SignalUi>,
) {
    ui.input_consumed = false;
    let pressed = |button| gamepads.iter().any(|pad| pad.just_pressed(button));
    let pad_input = gamepads
        .iter()
        .any(|pad| pad.get_just_pressed().next().is_some());
    if pad_input {
        ui.device = "gamepad";
    } else if keys.get_just_pressed().next().is_some() {
        ui.device = "keyboard_mouse";
    }

    if keys.any_just_pressed([KeyCode::Escape, KeyCode::Tab])
        || pressed(GamepadButton::East)
        || pressed(GamepadButton::Start)
    {
        ui.input_consumed = true;
        ui.back();
        return;
    }
    if !ui.is_open() {
        return;
    }
    ui.input_consumed = true;
    if keys.just_pressed(KeyCode::ArrowDown) || pressed(GamepadButton::DPadDown) {
        ui.focus = (ui.focus + 1) % ui.count();
    }
    if keys.just_pressed(KeyCode::ArrowUp) || pressed(GamepadButton::DPadUp) {
        ui.focus = (ui.focus + ui.count() - 1) % ui.count();
    }
    if keys.just_pressed(KeyCode::Enter) || pressed(GamepadButton::South) {
        ui.activate();
        return;
    }
    for (interaction, control) in &buttons {
        if *interaction == Interaction::Pressed && !control.disabled {
            ui.device = "keyboard_mouse";
            ui.focus = control.index;
            ui.activate();
            return;
        }
    }
    let mut scroll = 0.0;
    for event in wheel.read() {
        ui.device = "keyboard_mouse";
        scroll -= match event.unit {
            MouseScrollUnit::Line => event.y * 48.0,
            MouseScrollUnit::Pixel => event.y / scale.0,
        };
    }
    if keys.just_pressed(KeyCode::PageDown) || pressed(GamepadButton::RightTrigger) {
        scroll += 240.0;
    }
    if keys.just_pressed(KeyCode::PageUp) || pressed(GamepadButton::LeftTrigger) {
        scroll -= 240.0;
    }
    if scroll != 0.0 {
        ui.scroll = (ui.scroll + scroll).max(0.0);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Leaf text styling stays explicit without a separate builder"
)]
fn label(
    commands: &mut Commands,
    parent: Entity,
    value: impl Into<String>,
    font: &Handle<Font>,
    size: f32,
    bold: bool,
    ink: Color,
    line_height: f32,
) -> Entity {
    let entity = commands
        .spawn((
            Text::new(value),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(size),
                weight: FontWeight(if bold { 800 } else { 450 }),
                ..default()
            },
            LineHeight::RelativeToFont(line_height),
            TextColor(ink),
            Node {
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

fn panel(commands: &mut Commands, parent: Entity, tokens: &Tokens, node: Node) -> Entity {
    let id = commands
        .spawn((
            node,
            BackgroundColor(color(&tokens.colors.base)),
            BorderColor::all(color(&tokens.colors.raised)),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.65),
                px(6),
                px(8),
                px(0),
                px(0),
            ),
        ))
        .id();
    commands.entity(parent).add_child(id);
    id
}

fn control(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    text: &str,
    disabled: bool,
    font: &Handle<Font>,
    tokens: &Tokens,
) {
    let id = commands
        .spawn((
            Button,
            AccessibleLabel::new(text),
            Control {
                label: text.to_owned(),
                index,
                disabled,
            },
            Node {
                width: percent(100),
                min_height: px(76),
                padding: UiRect::axes(px(24), px(14)),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(tokens.button_radius)),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(color(&tokens.colors.raised)),
            BorderColor::all(color(&tokens.colors.secondary)),
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.8), px(0), px(5), px(0), px(0)),
        ))
        .id();
    commands.entity(parent).add_child(id);
    if disabled {
        commands.entity(id).insert(InteractionDisabled);
    }
    let text_entity = label(
        commands,
        id,
        text,
        font,
        tokens.body_size,
        true,
        color(&tokens.colors.text),
        1.3,
    );
    commands.entity(text_entity).insert(Node {
        width: percent(100),
        min_width: px(0),
        max_width: percent(100),
        flex_shrink: 0.0,
        ..default()
    });
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "UI tree helpers and Bevy system injection"
)]
fn draw(
    mut commands: Commands,
    ui: Res<SignalUi>,
    tokens: Res<Tokens>,
    font: Res<UiFont>,
    camera: Query<(Entity, &Camera), With<Camera3d>>,
    roots: Query<Entity, With<Root>>,
    mut scale: ResMut<UiScale>,
    mut prior: Local<Option<(Page, usize, u32, bool, &'static str, UVec2)>>,
) {
    let Ok((camera_id, camera)) = camera.single() else {
        return;
    };
    let Some(size) = camera.physical_viewport_size() else {
        return;
    };
    let key = (
        ui.page,
        ui.selected_record,
        ui.scale.to_bits(),
        ui.reduced_motion,
        ui.device,
        size,
    );
    if *prior == Some(key) {
        return;
    }
    *prior = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    // Fit the reference canvas; enlarged text reduces logical room, then flex/scroll handles it
    scale.0 = (size.y as f32 / 1080.0).min(size.x as f32 / 1440.0) * ui.scale;
    let narrow = size.x as f32 / scale.0 < 1450.0;
    let root = commands
        .spawn((
            Root,
            UiTargetCamera(camera_id),
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(40)),
                row_gap: px(20),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();
    let ink = color(&tokens.colors.text);
    let secondary = color(&tokens.colors.secondary);
    let focus = color(&tokens.colors.focus);
    let body = tokens.body_size;
    if ui.page == Page::World {
        let hud = panel(
            &mut commands,
            root,
            &tokens,
            Node {
                align_self: AlignSelf::FlexStart,
                padding: UiRect::axes(px(24), px(14)),
                border_radius: BorderRadius::all(px(16)),
                ..default()
            },
        );
        label(
            &mut commands,
            hud,
            ":  Null Site   /   街景预览     TAB / START  工作菜单",
            &font.0,
            body,
            true,
            ink,
            1.4,
        );
        return;
    }
    let header = commands
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    commands.entity(root).add_child(header);
    let badge = panel(
        &mut commands,
        header,
        &tokens,
        Node {
            padding: UiRect::axes(px(20), px(8)),
            border: UiRect {
                left: px(6),
                right: px(2),
                ..default()
            },
            ..default()
        },
    );
    commands.entity(badge).insert(BorderColor::all(focus));
    label(
        &mut commands,
        badge,
        "N:SIDE   /   NEIGHBORHOOD SIGNAL",
        &font.0,
        24.0,
        true,
        ink,
        1.2,
    );
    let stamp = panel(
        &mut commands,
        header,
        &tokens,
        Node {
            padding: UiRect::axes(px(18), px(8)),
            border: UiRect::bottom(px(3)),
            ..default()
        },
    );
    commands
        .entity(stamp)
        .insert(BorderColor::all(color(&tokens.colors.warm)));
    label(
        &mut commands,
        stamp,
        "UI 实验 · 内容样例",
        &font.0,
        24.0,
        true,
        color(&tokens.colors.warm),
        1.2,
    );
    let main = commands
        .spawn(Node {
            flex_grow: 1.0,
            min_height: px(0),
            column_gap: px(32),
            row_gap: px(20),
            flex_direction: if narrow && ui.page != Page::Menu {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            },
            ..default()
        })
        .id();
    commands.entity(root).add_child(main);

    if ui.page == Page::Menu {
        if !narrow {
            let brand = commands
                .spawn(Node {
                    flex_grow: 1.0,
                    justify_content: JustifyContent::FlexEnd,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                    padding: UiRect::bottom(px(42)),
                    ..default()
                })
                .id();
            commands.entity(main).add_child(brand);
            let title = panel(
                &mut commands,
                brand,
                &tokens,
                Node {
                    align_self: AlignSelf::FlexStart,
                    padding: UiRect::axes(px(32), px(12)),
                    ..default()
                },
            );
            commands.entity(title).insert(BoxShadow::new(
                color(&tokens.colors.warm),
                px(10),
                px(10),
                px(0),
                px(0),
            ));
            label(
                &mut commands,
                title,
                "N:SIDE",
                &font.0,
                132.0,
                true,
                focus,
                1.0,
            );
            let subtitle = panel(
                &mut commands,
                brand,
                &tokens,
                Node {
                    align_self: AlignSelf::FlexStart,
                    padding: UiRect::axes(px(24), px(10)),
                    ..default()
                },
            );
            label(
                &mut commands,
                subtitle,
                "街区信号  :  生活仍在继续",
                &font.0,
                32.0,
                true,
                ink,
                1.2,
            );
        }
        let menu = panel(
            &mut commands,
            main,
            &tokens,
            Node {
                width: if narrow { percent(100) } else { px(660) },
                padding: UiRect::all(px(32)),
                row_gap: px(16),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(tokens.panel_radius)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        );
        commands.entity(menu).insert((
            Detail,
            ScrollPosition(Vec2::new(0.0, ui.focus.saturating_sub(2) as f32 * 92.0)),
            EnterSlide(0.0),
        ));
        label(
            &mut commands,
            menu,
            "工作菜单",
            &font.0,
            tokens.title_size,
            true,
            ink,
            1.1,
        );
        label(
            &mut commands,
            menu,
            "Null Site / 月台杂货周边\n当前背景：真实城市灰盒",
            &font.0,
            body,
            false,
            secondary,
            1.5,
        );
        for (i, title) in [
            "01   继续游玩 · 返回街景",
            "02   委托  /  尚未接入",
            "03   线索  /  查看排版样例",
            "04   街区",
            "05   设置",
        ]
        .iter()
        .enumerate()
        {
            control(&mut commands, menu, i, title, i == 1, &font.0, &tokens);
        }
        label(
            &mut commands,
            menu,
            "委托不可用：任务运行时尚未接入\n本实验不创建剧情进度或存档",
            &font.0,
            24.0,
            false,
            secondary,
            1.5,
        );
    } else {
        let nav = panel(
            &mut commands,
            main,
            &tokens,
            Node {
                width: if narrow { percent(100) } else { px(430) },
                flex_shrink: 0.0,
                padding: UiRect::all(px(24)),
                display: if narrow { Display::Grid } else { Display::Flex },
                grid_template_columns: if narrow {
                    RepeatedGridTrack::flex(3, 1.0)
                } else {
                    vec![]
                },
                column_gap: px(16),
                row_gap: px(16),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(tokens.panel_radius)),
                ..default()
            },
        );
        let heading = match ui.page {
            Page::Records => "调查工作台",
            Page::Settings => "设置",
            _ => "街区",
        };
        let heading_id = label(
            &mut commands,
            nav,
            heading,
            &font.0,
            tokens.heading_size,
            true,
            ink,
            1.2,
        );
        if narrow {
            commands.entity(heading_id).insert(Node {
                grid_column: GridPlacement::span(3),
                ..default()
            });
        }
        match ui.page {
            Page::Records => {
                control(
                    &mut commands,
                    nav,
                    0,
                    if ui.selected_record == 0 {
                        "01   箱封的重复粘贴痕迹  · 当前"
                    } else {
                        "01   箱封的重复粘贴痕迹"
                    },
                    false,
                    &font.0,
                    &tokens,
                );
                control(
                    &mut commands,
                    nav,
                    1,
                    if ui.selected_record == 1 {
                        "02   梦中见闻 · 空记录  · 当前"
                    } else {
                        "02   梦中见闻 · 空记录"
                    },
                    false,
                    &font.0,
                    &tokens,
                );
                control(
                    &mut commands,
                    nav,
                    2,
                    "返回工作菜单",
                    false,
                    &font.0,
                    &tokens,
                );
            }
            Page::Settings => {
                control(
                    &mut commands,
                    nav,
                    0,
                    &format!("界面缩放   {}%", (ui.scale * 100.0) as u32),
                    false,
                    &font.0,
                    &tokens,
                );
                control(
                    &mut commands,
                    nav,
                    1,
                    if ui.reduced_motion {
                        "减少动态效果   开"
                    } else {
                        "减少动态效果   关"
                    },
                    false,
                    &font.0,
                    &tokens,
                );
                control(
                    &mut commands,
                    nav,
                    2,
                    "返回工作菜单",
                    false,
                    &font.0,
                    &tokens,
                );
            }
            _ => control(
                &mut commands,
                nav,
                0,
                "返回工作菜单",
                false,
                &font.0,
                &tokens,
            ),
        }
        let detail = panel(
            &mut commands,
            main,
            &tokens,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(px(36)),
                row_gap: px(22),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(tokens.panel_radius)),
                ..default()
            },
        );
        commands.entity(detail).insert((
            Detail,
            ScrollPosition(Vec2::new(0.0, ui.scroll)),
            EnterSlide(0.0),
        ));
        let lines: Vec<(String, f32, bool, Color)> = match ui.page {
            Page::Records if ui.selected_record == 0 => vec![
                ("来源 : 现场观察   /   核对状态 : 待核对".into(), 24.0, true, focus),
                ("箱封有重复粘贴的痕迹".into(), tokens.title_size, true, ink),
                ("排版样例  ·  非已获得的游戏线索".into(), body, true, color(&tokens.colors.warm)),
                ("获得位置\n本页只展示位置字段的排版；实际地点需由调查事件提供".into(), body, false, secondary),
                ("原始记录\n纸箱上有两层封口胶带，外层的一角翘起，下面留着旧胶带的边缘。箱面上的字仍能辨认，但仅凭这些痕迹，无法确认是谁打开过箱子，也无法确认物件何时被移动".into(), body, false, ink),
                ("已知关联\n目前没有可确认的关联。来源、观察时间与当事人的说法，需要在实际调查中分别记录；梦中见到的形象不会自动成为现实中的历史事实".into(), body, false, ink),
                ("尚待核对\n重复粘贴可能有多种原因。记录保留玩家看见的细节，不自动写出原因、可信度百分比或尚未获知的剧情答案".into(), body, false, ink),
                ("记录边界\n这段较长文字用于验证中文换行、缩放和滚动。打开或关闭工作台不会获得线索、推进委托，也不会修改世界状态。返回现场后，玩家仍应依据真实事件获得新的信息".into(), body, false, secondary),
            ],
            Page::Records => vec![("梦中见闻".into(), tokens.title_size, true, color(&tokens.colors.dream)), ("暂无记录".into(), tokens.heading_size, true, ink), ("当前是空状态样例\n潜梦及线索获取系统尚未接入，菜单不会编造梦中经历，也不会提前展示谜底".into(), body, false, secondary)],
            Page::Settings => vec![("让信息清楚可读".into(), tokens.title_size, true, ink), ("界面缩放".into(), tokens.heading_size, true, focus), ("100% / 125% 在本次运行中生效\n字体保持正常比例，内容换行并通过滚动读取；窄画幅将列表移到内容上方".into(), body, false, ink), ("减少动态效果".into(), tokens.heading_size, true, focus), ("开启后，页面装饰直接切换\n按键立即响应，内容不等待滑入动画。当前尚无 UI 音效，静音不影响任何信息".into(), body, false, ink), ("设置暂不保存到磁盘；重新启动恢复默认值".into(), body, false, secondary)],
            _ => vec![("街区地图尚未接入".into(), tokens.title_size, true, ink), ("返回街景可以继续查看真实城市".into(), tokens.heading_size, true, focus), ("玩家导航需要对应实际可通行空间。本轮没有人物碰撞与导航，因此不将开发总图作为玩家地图展示，也不提供虚构的目标路线".into(), body, false, secondary), ("当前位置标签是视觉样例，尚无玩家位置数据".into(), body, true, color(&tokens.colors.warning))],
        };
        for (text, size, bold, ink) in lines {
            label(
                &mut commands,
                detail,
                text,
                &font.0,
                size,
                bold,
                ink,
                tokens.line_height,
            );
        }
    }
    let footer = panel(
        &mut commands,
        root,
        &tokens,
        Node {
            padding: UiRect::axes(px(20), px(12)),
            flex_shrink: 0.0,
            ..default()
        },
    );
    label(
        &mut commands,
        footer,
        if ui.device == "gamepad" {
            "十字键 选择    A 确认    B 返回    RB/LB 滚动    START 菜单"
        } else {
            "↑ ↓ 选择    ENTER 确认    ESC 返回    滚轮 / PAGE UP · DOWN 滚动"
        },
        &font.0,
        24.0,
        true,
        ink,
        1.2,
    );
}

fn style_controls(
    ui: Res<SignalUi>,
    tokens: Res<Tokens>,
    mut focus: ResMut<InputFocus>,
    mut controls: Query<(
        Entity,
        &Control,
        &Interaction,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut text: Query<(&mut TextColor, &mut Text, &mut UiTransform)>,
) {
    if !ui.is_open() {
        focus.clear();
    }
    for (entity, control, interaction, children, mut bg, mut border) in &mut controls {
        let active = ui.focus == control.index;
        if active && focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
        let filled = active && !control.disabled;
        let background = color(if filled {
            &tokens.colors.focus
        } else {
            &tokens.colors.raised
        });
        if bg.0 != background {
            bg.0 = background;
        }
        let outline = BorderColor::all(color(if active {
            &tokens.colors.focus
        } else if *interaction == Interaction::Hovered {
            &tokens.colors.text
        } else {
            &tokens.colors.secondary
        }));
        if *border != outline {
            *border = outline;
        }
        for child in children {
            if let Ok((mut ink, mut text, mut transform)) = text.get_mut(*child) {
                let title = format!("{} {}", if active { ":" } else { " " }, control.label);
                if text.0 != title {
                    text.0 = title;
                }
                let offset = px(
                    if *interaction == Interaction::Pressed && !control.disabled {
                        3.0
                    } else {
                        0.0
                    },
                );
                if transform.translation.y != offset {
                    transform.translation.y = offset;
                }
                let foreground = color(if filled {
                    &tokens.colors.base
                } else if control.disabled {
                    &tokens.colors.secondary
                } else {
                    &tokens.colors.text
                });
                if ink.0 != foreground {
                    ink.0 = foreground;
                }
            }
        }
    }
}

fn animate(
    time: Res<Time>,
    ui: Res<SignalUi>,
    tokens: Res<Tokens>,
    mut panels: Query<(&mut EnterSlide, &mut UiTransform)>,
) {
    for (mut slide, mut transform) in &mut panels {
        if slide.0 >= 1.0 {
            continue;
        }
        slide.0 = if ui.reduced_motion {
            1.0
        } else {
            (slide.0 + time.delta_secs() * 1000.0 / tokens.panel_slide_ms).min(1.0)
        };
        transform.translation.x = px(24.0 * (1.0 - slide.0).powi(3));
    }
}

fn scroll_detail(
    mut ui: ResMut<SignalUi>,
    mut panels: Query<(&ComputedNode, &mut ScrollPosition), With<Detail>>,
    mut prior_focus: Local<Option<(Page, usize)>>,
) {
    if ui.page == Page::Menu && *prior_focus != Some((ui.page, ui.focus)) {
        ui.scroll = ui.focus.saturating_sub(2) as f32 * 92.0;
    }
    *prior_focus = Some((ui.page, ui.focus));
    for (computed, mut scroll) in &mut panels {
        if computed.size().y <= 0.0 {
            continue;
        }
        let max = ((computed.content_size().y - computed.size().y)
            * computed.inverse_scale_factor())
        .max(0.0);
        ui.scroll = ui.scroll.min(max);
        if scroll.y != ui.scroll {
            scroll.y = ui.scroll;
        }
    }
    if ui.page == Page::World {
        ui.scroll = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_flow_keeps_focus_and_sample_information_separate() {
        let mut ui = SignalUi {
            focus: 1,
            ..default()
        };
        ui.activate();
        assert_eq!(ui.page, Page::Menu);
        ui.focus = 2;
        ui.scroll = 184.0;
        ui.activate();
        assert_eq!(ui.page, Page::Records);
        assert_eq!(ui.scroll, 0.0);
        ui.scroll = 240.0;
        ui.back();
        ui.activate();
        assert_eq!(ui.scroll, 240.0);
        ui.focus = 1;
        ui.activate();
        assert_eq!(ui.selected_record, 1);
        ui.back();
        assert_eq!(ui.focus, 2);
        ui.activate();
        assert_eq!(ui.page, Page::Records);
        assert_eq!(ui.focus, 1);
        assert_eq!(ui.selected_record, 1);
        ui.activate();
        assert_eq!(ui.selected_record, 1);
        ui.back();
        ui.focus = 4;
        ui.activate();
        ui.activate();
        assert_eq!(ui.scale, 1.25);
        ui.focus = 1;
        ui.activate();
        assert!(ui.reduced_motion);
        ui.back();
        assert_eq!(ui.focus, 4);
        ui.back();
        assert_eq!(ui.page, Page::World);
        ui.back();
        assert_eq!(ui.focus, 4);
    }

    #[test]
    fn token_colors_and_text_contrast_are_valid() {
        let tokens: Tokens =
            serde_json::from_str(include_str!("../../source-assets/ui-kit/tokens.json")).unwrap();
        fn luminance(hex: &str) -> f32 {
            let c = color(hex).to_linear();
            c.red * 0.2126 + c.green * 0.7152 + c.blue * 0.0722
        }
        for (ink, background) in [
            (&tokens.colors.text, &tokens.colors.base),
            (&tokens.colors.secondary, &tokens.colors.raised),
            (&tokens.colors.base, &tokens.colors.focus),
        ] {
            let a = luminance(ink);
            let b = luminance(background);
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }
}
