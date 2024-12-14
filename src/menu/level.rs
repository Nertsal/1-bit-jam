mod ui;

pub use self::ui::*;

use super::*;

use crate::{
    game::PlayGroup,
    leaderboard::{Leaderboard, LeaderboardStatus, ScoreCategory, ScoreMeta},
    local::CachedMusic,
    render::{mask::MaskedRender, menu::MenuRender},
    ui::{
        widget::{ConfirmPopup, Widget},
        ShowTime, UiContext, WidgetRequest,
    },
};

#[derive(Debug)]
pub enum ConfirmAction {
    DeleteMusic(Id),
    DeleteGroup(Index),
    SyncDiscard,
    DownloadRecommended,
    SyncUpload,
}

pub struct LevelMenu {
    context: Context,
    transition: Option<geng::state::Transition>,

    render: MenuRender,
    util: UtilRender,
    dither: DitherRender,
    masked: MaskedRender,

    framebuffer_size: vec2<usize>,
    last_delta_time: Time,
    time: Time,

    ui: MenuUI,
    ui_focused: bool,
    ui_context: UiContext,

    camera: Camera2d,
    state: MenuState,
    play_button: HoverButton,
}

pub struct MenuState {
    pub context: Context,
    pub leaderboard: Leaderboard,
    pub player: Player,
    pub config: LevelConfig,

    pub confirm_popup: Option<ConfirmPopup<ConfirmAction>>,

    /// Currently showing music.
    pub selected_music: Option<ShowTime<Id>>,
    /// Currently showing group.
    pub selected_group: Option<ShowTime<Index>>,
    /// Currently showing level of the active group.
    pub selected_level: Option<ShowTime<usize>>,

    /// Switch to the music after current one finishes its animation.
    pub switch_music: Option<Id>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<Index>,
    /// Switch to the level of the active group after current one finishes its animation.
    pub switch_level: Option<usize>,

    /// Whether to open a (group, level) in the editor.
    pub edit_level: Option<(Index, Option<usize>)>,

    /// List of notifications to be consumed and transferred to UI.
    pub notifications: Vec<String>,
}

pub struct GroupEntry {
    pub meta: GroupInfo,
    pub logo: Option<ugli::Texture>,
}

impl Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupEntry")
            .field("meta", &self.meta)
            .field("logo", &self.logo.as_ref().map(|_| "<logo>"))
            .finish()
    }
}

impl MenuState {
    fn get_category(&self) -> ScoreCategory {
        let mods = self.config.modifiers.clone();
        let health = self.config.health.clone();
        ScoreCategory::new(mods, health)
    }

    fn update_board_meta(&mut self) {
        self.leaderboard.change_category(self.get_category());
    }

    fn select_music(&mut self, music: Id) {
        self.switch_music = Some(music);
        if self
            .selected_music
            .as_ref()
            .map_or(true, |selected| selected.data != music)
        {
            self.switch_group = None;
            self.switch_level = None;
        }
    }

    fn select_group(&mut self, group: Index) {
        self.switch_group = Some(group);
        if self
            .selected_group
            .as_ref()
            .map_or(true, |selected| selected.data != group)
        {
            self.switch_level = None;
        }
    }

    fn select_level(&mut self, level: usize) {
        self.switch_level = Some(level);
    }

    fn edit_level(&mut self, group: Index, level: Option<usize>) {
        self.edit_level = Some((group, level));
    }

    fn new_group(&mut self, music: Id) {
        self.switch_group = None; // Deselect group
        let local = &self.context.local;
        local.new_group(music);
    }

    /// Create a popup window with a message for the given action.
    pub fn popup_confirm(&mut self, action: ConfirmAction, message: impl Into<Name>) {
        self.confirm_popup = Some(ConfirmPopup {
            action,
            title: "Are you sure?".into(),
            message: message.into(),
        });
    }

    /// Create a popup window with a title and message for the given action.
    pub fn popup_confirm_custom(
        &mut self,
        action: ConfirmAction,
        title: impl Into<Name>,
        message: impl Into<Name>,
    ) {
        self.confirm_popup = Some(ConfirmPopup {
            action,
            title: title.into(),
            message: message.into(),
        });
    }

    /// Confirm the popup action and execute it.
    pub fn confirm_action(&mut self, ui: &mut MenuUI) {
        let Some(popup) = self.confirm_popup.take() else {
            return;
        };
        match popup.action {
            ConfirmAction::DeleteMusic(id) => self.context.local.delete_music(id),
            ConfirmAction::DeleteGroup(index) => self.context.local.delete_group(index),
            ConfirmAction::SyncDiscard => {
                if let Some(sync) = &mut ui.sync {
                    if let Some(client) = self.leaderboard.client.clone() {
                        sync.discard_changes(client);
                    }
                }
            }
            ConfirmAction::SyncUpload => {
                if let Some(sync) = &mut ui.sync {
                    if let Some(client) = self.leaderboard.client.clone() {
                        sync.upload(client);
                    }
                }
            }
            ConfirmAction::DownloadRecommended => {
                self.context.local.download_recommended();
                self.notifications
                    .push("Please wait while the levels are being downloaded".into());
            }
        }
    }
}

impl LevelMenu {
    pub fn new(context: Context, leaderboard: Leaderboard) -> Self {
        let player = Player::new(
            Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.1) }),
            r32(0.0),
        );

        let mut state = Self {
            render: MenuRender::new(context.clone()),
            util: UtilRender::new(&context.geng, &context.assets),
            dither: DitherRender::new(&context.geng, &context.assets),
            masked: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),

            framebuffer_size: vec2(1, 1),
            last_delta_time: Time::ONE,
            time: Time::ZERO,

            ui: MenuUI::new(context.clone()),
            ui_focused: false,
            ui_context: UiContext::new(context.clone()),

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            state: MenuState {
                context: context.clone(),
                leaderboard,
                player,
                config: LevelConfig::default(),

                confirm_popup: None,

                selected_music: None,
                selected_group: None,
                selected_level: None,

                switch_music: None,
                switch_group: None,
                switch_level: None,

                edit_level: None,

                notifications: Vec::new(),
            },
            play_button: HoverButton::new(
                Collider {
                    position: vec2(4.9, -0.5).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.8) },
                },
                1.5,
            ),

            context,
            transition: None,
        };

        if state.context.local.inner.borrow().groups.is_empty() {
            state.state.popup_confirm_custom(
                ConfirmAction::DownloadRecommended,
                "Download recommended levels?",
                "",
            );
        }

        state
    }

    fn get_active_level(&self) -> Option<(PlayGroup, usize, Rc<LevelFull>)> {
        let local = self.context.local.inner.borrow();

        let group = self.state.selected_group.as_ref()?;
        let group_index = group.data;
        let group = local.groups.get(group_index)?;

        let music = group.music.as_ref()?;

        let level = self.state.selected_level.as_ref()?;
        let level_index = level.data;
        let level = group.data.levels.get(level_index)?;

        let group = PlayGroup {
            music: Rc::clone(music),
            group_index,
            cached: Rc::clone(group),
        };

        Some((group, level_index, Rc::clone(level)))
    }

    fn play_level(&mut self) {
        let Some((group, level_index, level)) = self.get_active_level() else {
            log::error!("Trying to play a level, but there is no active level");
            return;
        };

        self.context.music.stop();
        self.ui_context.cursor.reset();
        self.play_button.hover_time.set(Time::ZERO);

        let future = {
            let context = self.context.clone();
            let leaderboard = self.state.leaderboard.clone();
            let options = self.state.context.get_options();
            let config = self.state.config.clone();

            async move {
                let level = crate::game::PlayLevel {
                    group,
                    level_index,
                    level: level.clone(),
                    config,
                    start_time: Time::ZERO,
                };
                crate::game::Game::new(context, options, level, leaderboard)
            }
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.context.geng,
                geng::EmptyLoadingScreen::new(&self.context.geng),
                future,
            ),
        )));
        // Queue leaderboard fetch when coming back
        self.state.leaderboard.status = LeaderboardStatus::None;
    }

    fn update_active_music(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_music) = &mut self.state.selected_music {
            if let Some(switch_music) = self.state.switch_music {
                if current_music.data != switch_music {
                    current_music.time.change(-delta_time);
                    current_music.going_up = false;

                    if current_music.time.is_min() {
                        // Switch
                        current_music.data = switch_music;
                    }
                } else {
                    current_music.time.change(delta_time);
                    current_music.going_up = true;
                }
            } else {
                current_music.time.change(-delta_time);
                current_music.going_up = false;

                if current_music.time.is_min() {
                    // Remove
                    self.state.selected_music = None;
                    self.ui.level_select.tab_groups.hide();
                    self.ui.level_select.tab_levels.hide();
                    self.ui.level_select.select_tab(LevelSelectTab::Music);
                }
            }
        } else if let Some(music) = self.state.switch_music {
            self.state.selected_music = Some(ShowTime {
                data: music,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn update_active_group(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_group) = &mut self.state.selected_group {
            if let Some(switch_group) = self.state.switch_group {
                if current_group.data != switch_group {
                    // Change level first
                    self.state.switch_level = None;
                    // if self.state.show_level.is_some() {
                    //     return;
                    // }

                    current_group.time.change(-delta_time);
                    current_group.going_up = false;

                    if current_group.time.is_min() {
                        // Switch
                        current_group.data = switch_group;
                    }
                } else {
                    current_group.time.change(delta_time);
                    current_group.going_up = true;
                }
            } else {
                current_group.time.change(-delta_time);
                current_group.going_up = false;

                if current_group.time.is_min() {
                    // Remove
                    self.state.selected_group = None;
                    self.state.selected_level = None;
                    self.ui.level_select.tab_levels.hide();
                }
            }
        } else if let Some(group) = self.state.switch_group {
            self.state.selected_group = Some(ShowTime {
                data: group,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn update_active_level(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_level) = &mut self.state.selected_level {
            if let Some(switch_level) = self.state.switch_level {
                if current_level.data != switch_level {
                    // self.state.show_leaderboard.going_up = false; // Hide leaderboard
                    current_level.time.change(-delta_time);
                    current_level.going_up = false;

                    if current_level.time.is_min() {
                        // Switch
                        current_level.data = switch_level;
                    }
                } else {
                    current_level.time.change(delta_time);
                    current_level.going_up = true;
                }
            } else {
                // self.state.level_up = false;
                current_level.time.change(-delta_time);
                current_level.going_up = false;

                if current_level.time.is_min() {
                    // Remove
                    self.state.selected_level = None;
                }
            }
        } else if let Some(level) = self.state.switch_level {
            self.state.selected_level = Some(ShowTime {
                data: level,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn fetch_leaderboard(&mut self) {
        let category = self.state.get_category();
        if let Some((_, _, level)) = self.get_active_level() {
            let meta = ScoreMeta {
                score: Score::new(category.mods.multiplier()),
                category,
            };
            self.state.leaderboard.submit(None, level.meta.id, meta);
        }
    }

    fn update_leaderboard(&mut self) {
        if let Some(req) = self.ui.leaderboard.window.last_request.take() {
            match req {
                WidgetRequest::Open | WidgetRequest::Reload => {
                    self.fetch_leaderboard();
                }
                WidgetRequest::Close => {}
            }
        }
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let transition = self.transition.take();
        if transition.is_some() {
            self.context.music.stop();
        }
        transition
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        let theme = self.context.get_options().theme;
        ugli::clear(framebuffer, Some(theme.dark), None, None);
        self.masked.update_size(framebuffer.size());

        self.dither.set_noise(1.0);
        let mut dither_buffer = self.dither.start();

        let fading = self.play_button.is_fading();

        if !fading || self.play_button.is_fading() {
            let play_time = r32(self
                .state
                .selected_level
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio()));
            let scale = crate::util::smoothstep(play_time);
            let mut button = self.play_button.clone();
            button.base_collider = button.base_collider.transformed(Transform::scale(scale));
            self.util.draw_button(
                &button,
                "PLAY",
                &crate::render::THEME,
                &self.camera,
                &mut dither_buffer,
            );
        }

        self.dither.finish(self.time, &theme);

        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);

        if !fading {
            let mut masked = self.masked.start();

            self.ui_focused = self.ui.layout(
                &mut self.state,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &mut self.ui_context,
            );
            self.render
                .draw_ui(&self.ui, &self.state, &mut masked.color);

            masked.mask_quad(self.ui.screen.position);

            let pixelated = false;

            if pixelated {
                self.dither.set_noise(0.0);
                let mut dither = self.dither.start();

                self.masked.draw(
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        ..default()
                    },
                    &mut dither,
                );

                // self.dither.finish(
                //     self.time,
                //     &Theme {
                //         dark: Color::TRANSPARENT_BLACK,
                //         ..self.state.options.theme
                //     },
                // );

                geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
                    .fit_screen(vec2(0.5, 0.5), framebuffer)
                    .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
            } else {
                self.masked.draw(
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        ..default()
                    },
                    framebuffer,
                );
            };
        }
        self.ui_context.frame_end();

        let mut dither_buffer = self.dither.start();
        self.util
            .draw_player(&self.state.player, &self.camera, &mut dither_buffer);
        self.dither.finish(self.time, &theme.transparent());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::F11,
            } => self.context.geng.window().toggle_fullscreen(),
            geng::Event::EditText(text) => {
                self.ui_context.text_edit.text = text;
            }
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => {
                if let Some(sync) = &mut self.ui.sync {
                    sync.window.request = Some(WidgetRequest::Close);
                } else if self.ui.explore.window.show.time.is_max() {
                    self.ui.explore.window.request = Some(WidgetRequest::Close);
                } else if self.ui.leaderboard.window.show.time.is_max() {
                    self.ui.leaderboard.window.request = Some(WidgetRequest::Close);
                } else if self.state.switch_level.take().is_some()
                    || self.state.switch_group.take().is_some()
                    || self.state.switch_music.take().is_some()
                {
                } else {
                    // Go to main menu
                    self.transition = Some(geng::state::Transition::Pop);
                }
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            _ => (),
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.state.player.update_tail(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::None);

        let options = self.context.get_options();

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());

        if self.ui.explore.state.visible {
            let t = if !self.ui.explore.window.show.going_up
                && self.context.music.current().map(|info| info.id)
                    != self.state.selected_music.as_ref().map(|show| show.data)
            {
                self.ui.explore.window.show.time.get_ratio()
            } else {
                1.0
            };
            self.context.music.set_volume(options.volume.music() * t);
        } else {
            // Music volume
            let t = (1.0 - self.play_button.hover_time.get_ratio().as_f32())
                .min(show_ratio(&self.state.selected_music).unwrap_or(0.0));
            self.context.music.set_volume(options.volume.music() * t);

            // Playing music
            if let Some(active) = self
                .state
                .selected_music
                .as_ref()
                .and_then(|music| self.state.context.local.get_music(music.data))
            {
                self.context.music.switch(&active); // TODO: rng start
            } else {
                self.context.music.stop();
            }
        }
        self.context.music.set_speed(1.0);

        let game_pos = geng_utils::layout::fit_aabb(
            self.dither.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = self.ui_context.cursor.position - game_pos.bottom_left();
        let cursor_world = self.camera.screen_to_world(game_pos.size(), pos);

        self.state.player.collider.position = cursor_world.as_r32();
        self.state.player.reset_distance();
        if !self.ui_focused && self.state.selected_level.is_some() {
            self.state
                .player
                .update_distance_simple(&self.play_button.base_collider);

            let hovering = self
                .play_button
                .base_collider
                .contains(cursor_world.as_r32());
            if hovering && self.ui_context.cursor.was_down {
                self.play_button.clicked = true;
            }
            self.play_button.update(hovering, delta_time);
        }
        if self.play_button.hover_time.is_max() {
            self.play_level();
        }

        self.state.leaderboard.poll();
        if let Some(player) = self.state.leaderboard.loaded.player {
            self.state.player.info.id = player;
        }

        self.update_active_music(delta_time);
        self.update_active_group(delta_time);
        self.update_active_level(delta_time);
        self.update_leaderboard();

        self.context.local.poll();
        self.state
            .notifications
            .extend(self.context.local.take_notifications());

        let edit_level = self
            .state
            .edit_level
            .take()
            .and_then(|(group_index, level_index)| {
                // TODO: warn if smth wrong
                let local = self.state.context.local.inner.borrow();
                local.groups.get(group_index).and_then(|group| {
                    group.music.as_ref().map(|music| {
                        let level = level_index.and_then(|idx| {
                            group.data.levels.get(idx).map(|level| (idx, level.clone()))
                        });
                        let group = PlayGroup {
                            group_index,
                            cached: Rc::clone(group),
                            music: Rc::clone(music),
                        };
                        (group, level)
                    })
                })
            });
        let context = self.context.clone();
        let manager = self.context.geng.asset_manager().clone();
        let assets_path = run_dir().join("assets");

        if let Some((group, level)) = edit_level {
            let future = async move {
                let config: crate::editor::EditorConfig =
                    geng::asset::Load::load(&manager, &assets_path.join("editor.ron"), &())
                        .await
                        .expect("failed to load editor config");

                if let Some((level_index, level)) = level {
                    let level = crate::game::PlayLevel {
                        group,
                        level_index,
                        level,
                        config: LevelConfig::default(),
                        start_time: Time::ZERO,
                    };
                    crate::editor::EditorState::new_level(context, config, level)
                } else {
                    crate::editor::EditorState::new_group(context, config, group)
                }
            };
            let state = geng::LoadingScreen::new(
                &self.context.geng,
                geng::EmptyLoadingScreen::new(&self.context.geng),
                future,
            );

            self.transition = Some(geng::state::Transition::Push(Box::new(state)));
        }

        self.last_delta_time = delta_time;
    }
}

fn show_ratio<T>(show: &Option<ShowTime<T>>) -> Option<f32> {
    show.as_ref().map(|show| show.time.get_ratio())
}
