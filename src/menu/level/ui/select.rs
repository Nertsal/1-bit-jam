use crate::{local::CachedGroup, ui::UiWindow};

use super::*;

pub struct LevelSelectUI {
    // geng: Geng,
    assets: Rc<Assets>,
    pub tab_music: ToggleWidget,
    pub tab_groups: ToggleWidget,
    pub tab_levels: ToggleWidget,
    pub separator: WidgetState,

    pub add_music: TextWidget,
    pub grid_music: Vec<ItemMusicWidget>,
    pub add_group: AddItemWidget,
    pub grid_groups: Vec<ItemGroupWidget>,
    pub no_levels: TextWidget,
    pub grid_levels: Vec<ItemLevelWidget>,
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectAction {
    EditLevel(Index, usize),
    DeleteLevel(Index, usize),
    SyncGroup(Index),
    EditGroup(Index),
    DeleteGroup(Index),
    DeleteMusic(Id),
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectTab {
    Music,
    Group,
    Difficulty,
}

impl LevelSelectUI {
    pub fn new(_geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut ui = Self {
            // geng: geng.clone(),
            assets: assets.clone(),
            tab_music: ToggleWidget::new("Music"),
            tab_groups: ToggleWidget::new("Group"),
            tab_levels: ToggleWidget::new("Difficulty"),
            separator: WidgetState::new(),

            add_music: TextWidget::new("+"),
            grid_music: Vec::new(),
            add_group: AddItemWidget::new(assets),
            grid_groups: Vec::new(),
            no_levels: TextWidget::new("Create a Difficulty in the editor"),
            grid_levels: Vec::new(),
        };
        ui.tab_music.selected = true;
        ui.tab_groups.hide();
        ui.tab_levels.hide();
        ui
    }

    pub fn select_tab(&mut self, tab: LevelSelectTab) {
        for button in [
            &mut self.tab_music,
            &mut self.tab_groups,
            &mut self.tab_levels,
        ] {
            if !button.text.state.clicked {
                button.selected = false;
            }
        }

        let tab = match tab {
            LevelSelectTab::Music => &mut self.tab_music,
            LevelSelectTab::Group => {
                self.tab_music.show();
                &mut self.tab_groups
            }
            LevelSelectTab::Difficulty => {
                self.tab_music.show();
                self.tab_groups.show();
                &mut self.tab_levels
            }
        };
        tab.show();
        tab.selected = true;
    }

    pub fn update(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let mut main = main;
        main.cut_top(context.layout_size * 1.5);
        let bar = main.cut_top(context.font_size * 1.2);
        main.cut_top(context.layout_size * 1.0);

        self.tabs(bar, context);

        let mut action = None;
        if self.tab_music.selected {
            let act = self.grid_music(main, state, context);
            action = action.or(act);
        } else if self.tab_groups.selected {
            let act = self.grid_groups(main, state, context);
            action = action.or(act);
        } else if self.tab_levels.selected {
            let act = self.grid_levels(main, state, context);
            action = action.or(act);
        }

        action
    }

    fn tabs(&mut self, main: Aabb2<f32>, context: &mut UiContext) {
        let sep_size = vec2(main.width() * 0.9, 0.3 * context.layout_size);
        let sep = main.align_aabb(sep_size, vec2(0.5, 0.0));
        self.separator.update(sep, context);

        let buttons: Vec<_> = [
            Some(&mut self.tab_music),
            self.tab_groups
                .text
                .state
                .visible
                .then_some(&mut self.tab_groups),
            self.tab_levels
                .text
                .state
                .visible
                .then_some(&mut self.tab_levels),
        ]
        .into_iter()
        .flatten()
        .collect();

        let spacing = 1.0 * context.layout_size;
        let button_size = vec2(7.0 * context.layout_size, main.height());
        let button = Aabb2::point(main.center()).extend_symmetric(button_size / 2.0);

        let all_buttons = 3;
        let buttons_layout = button.stack_aligned(
            vec2(button_size.x + spacing, 0.0),
            all_buttons,
            vec2(0.5, 0.5),
        );

        let mut deselect = false;
        for (button, pos) in buttons.into_iter().zip(buttons_layout) {
            button.update(pos, context);
            if button.text.state.clicked {
                deselect = true;
            }
        }
        if deselect {
            for button in [
                &mut self.tab_music,
                &mut self.tab_groups,
                &mut self.tab_levels,
            ] {
                if !button.text.state.clicked {
                    button.selected = false;
                }
            }
        }
    }

    fn grid_music(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.clone();
        let music: Vec<_> = local
            .inner
            .borrow()
            .music
            .iter()
            .sorted_by_key(|(&k, _)| k)
            .map(|(_, music)| music.clone())
            .collect();

        // Synchronize vec length
        if self.grid_music.len() != music.len() {
            if let Some(cached) = music.first() {
                self.grid_music =
                    vec![ItemMusicWidget::new(&self.assets, "", cached.clone()); music.len()];
            } else {
                self.grid_music.clear();
            }
        }

        // Synchronize data
        for (widget, cache) in self.grid_music.iter_mut().zip(&music) {
            widget.music = cache.clone();
            widget.text.text = cache.meta.name.clone();
        }

        // Layout
        let columns = 3;
        let rows = self.grid_music.len() / columns + 1;
        let spacing = vec2(1.0, 2.0) * context.layout_size;
        let item_size = vec2(
            (main.width() - spacing.x * (columns as f32 - 1.0)) / columns as f32,
            1.3 * context.font_size,
        );

        let mut action = None;
        for row in 0..rows {
            let top_left =
                Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                    .extend_right(item_size.x)
                    .extend_down(item_size.y);
            let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);

            let mut row_items = 3;
            let mut skip = 0;
            if row == 0 {
                skip = 1;
                let pos = layout[0];
                self.add_music
                    .update(pos.extend_symmetric(-pos.size() * 0.1), context);
            }
            row_items -= skip;

            let i = if row == 0 { 0 } else { 2 + columns * (row - 1) };
            let range = (i + row_items).min(self.grid_music.len());

            let mut tab = None;
            for (widget, pos) in self.grid_music[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                // Check edited
                let edited = local
                    .inner
                    .borrow()
                    .groups
                    .iter()
                    .filter(|(_, group)| group.data.music == widget.music.meta.id)
                    .any(|(_, group)| {
                        group.data.id == 0
                            || group.origin.as_ref().map(|info| &info.hash) != Some(&group.hash)
                    });
                if edited {
                    widget.edited.show();
                } else {
                    widget.edited.hide();
                }

                // Update
                let act = widget.update(pos, context);
                action = action.or(act);
                if widget.state.clicked {
                    state.select_music(widget.music.meta.id);
                    tab = Some(LevelSelectTab::Group);

                    // If there is only 1 group for that music, automatically select it
                    let local = local.inner.borrow();
                    let mut groups = local
                        .groups
                        .iter()
                        .filter(|(_, group)| group.data.music == widget.music.meta.id);
                    if let Some((index, _)) = groups.next() {
                        if groups.next().is_none() {
                            drop(local);
                            state.select_group(index);
                            tab = Some(LevelSelectTab::Difficulty);
                        }
                    }
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }

        action
    }

    fn grid_groups(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
        let groups: Vec<_> = local
            .groups
            .iter()
            .filter(|(_, group)| Some(group.data.music) == state.switch_music)
            .sorted_by_key(|(_, group)| group.data.id)
            .collect();

        // Synchronize vec length
        if self.grid_groups.len() != groups.len() {
            self.grid_groups =
                vec![
                    ItemGroupWidget::new(&self.assets, "", Index::from_raw_parts(0, 0),);
                    groups.len()
                ];
        }

        // Synchronize data
        for (widget, &(group_id, cached)) in self.grid_groups.iter_mut().zip(&groups) {
            widget.sync(group_id, cached);
        }

        drop(local);

        // Layout
        let columns = 3;
        let rows = self.grid_groups.len() / columns + 1;
        let spacing = vec2(1.0, 2.0) * context.layout_size;
        let item_size = vec2(
            (main.width() - spacing.x * (columns as f32 - 1.0)) / columns as f32,
            1.3 * context.font_size,
        );

        let mut action = None;
        for row in 0..rows {
            let top_left =
                Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                    .extend_right(item_size.x)
                    .extend_down(item_size.y);
            let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);

            let mut row_items = 3;
            let mut skip = 0;
            if row == 0 {
                skip = 1;
                let pos = layout[0];
                self.add_group
                    .update(pos.extend_symmetric(-pos.size() * 0.1), context);
            }
            row_items -= skip;

            let i = if row == 0 { 0 } else { 2 + columns * (row - 1) };
            let range = (i + row_items).min(self.grid_groups.len());

            let mut tab = None;
            for (widget, pos) in self.grid_groups[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                let act = widget.update(pos, context);
                action = action.or(act);
                if widget.state.clicked {
                    state.select_group(widget.index);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }

        action
    }

    fn grid_levels(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
        let group_idx = state.switch_group;
        let levels: Vec<_> = group_idx
            .and_then(|group| local.groups.get(group))
            .map(|group| group.data.levels.clone())
            .into_iter()
            .flatten()
            .collect();

        // Synchronize vec length
        if self.grid_levels.len() != levels.len() {
            if let Some(cached) = levels.first() {
                self.grid_levels = vec![
                    ItemLevelWidget::new(
                        &self.assets,
                        "",
                        Index::from_raw_parts(0, 0),
                        0,
                        cached.clone(),
                    );
                    levels.len()
                ];
            } else {
                self.grid_levels.clear();
            }
        }

        let group_idx = group_idx?;
        let group = state.context.local.get_group(group_idx)?;

        // Synchronize data
        for (widget, (level_id, cached)) in
            self.grid_levels.iter_mut().zip(levels.iter().enumerate())
        {
            let origin_hash = group.origin.as_ref().and_then(|info| {
                info.levels
                    .iter()
                    .find(|level| level.id == cached.meta.id)
                    .map(|level| &level.hash)
            });
            let edited =
                origin_hash.map_or(false, |hash| Some(hash) != group.level_hashes.get(level_id));
            widget.sync(group_idx, level_id, cached, edited);
        }

        drop(local);

        if self.grid_levels.is_empty() {
            self.no_levels.show();
            let size = vec2(10.0, 1.2) * context.font_size;
            let pos = main.align_aabb(size, vec2(0.5, 0.5));
            self.no_levels.update(pos, context);
        } else {
            self.no_levels.hide();
        }

        // Layout
        let columns = 3;
        let rows = self.grid_levels.len() / columns + 1;
        let spacing = vec2(1.0, 2.0) * context.layout_size;
        let item_size = vec2(
            (main.width() - spacing.x * (columns as f32 - 1.0)) / columns as f32,
            1.3 * context.font_size,
        );

        let mut action = None;
        for row in 0..rows {
            let top_left =
                Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                    .extend_right(item_size.x)
                    .extend_down(item_size.y);
            let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);

            let row_items = 3;
            let skip = 0;

            let i = columns * row;
            let range = (i + row_items).min(self.grid_levels.len());

            let mut tab = None;
            for (widget, pos) in self.grid_levels[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                let act = widget.update(pos, context);
                action = action.or(act);
                if widget.state.clicked {
                    state.select_level(widget.index);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }

        action
    }
}

pub struct AddItemWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub menu: NewMenuWidget,
}

impl AddItemWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new("+"),
            menu: NewMenuWidget::new(assets),
        }
    }
}

impl Widget for AddItemWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.text.update(position, context);
        if self.state.clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.window.update(context.delta_time);
        if self.menu.window.show.time.is_min() {
            self.menu.hide();
        } else {
            self.menu.update(position, context);
        }
        if self.menu.state.hovered {
            context.can_focus = false;
        }
    }
}

#[derive(Clone)]
pub struct ItemMusicWidget {
    pub state: WidgetState,
    pub menu: ItemMenuWidget,
    pub edited: IconWidget,
    pub text: TextWidget,
    pub music: Rc<CachedMusic>,
}

impl ItemMusicWidget {
    pub fn new(assets: &Rc<Assets>, text: impl Into<Name>, music: Rc<CachedMusic>) -> Self {
        let mut menu = ItemMenuWidget::new(assets);
        menu.sync.hide();
        menu.edit.hide();

        Self {
            state: WidgetState::new(),
            menu,
            edited: IconWidget::new(&assets.sprites.star),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            music,
        }
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.right_clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.can_focus = false;
        }

        self.state.update(position, context);

        let widgets = [&mut self.edited];
        if widgets.iter().any(|widget| widget.state.visible) {
            let icons = position
                .cut_left(position.height() / 2.0)
                .extend_left(-context.font_size * 0.2)
                .extend_symmetric(-vec2(0.0, context.font_size * 0.15));
            let positions = icons.split_rows(widgets.len());
            for (widget, pos) in widgets.into_iter().zip(positions) {
                widget.update(pos, context);
            }
        }

        self.text.update(position, &mut context.scale_font(0.9));

        let mut action = None;
        if self.menu.delete.state.clicked {
            action = Some(LevelSelectAction::DeleteMusic(self.music.meta.id));
        }
        action
    }
}

#[derive(Clone)]
pub struct ItemGroupWidget {
    pub state: WidgetState,
    pub edited: IconWidget,
    pub local: IconWidget,
    pub menu: ItemMenuWidget,
    pub text: TextWidget,
    pub index: Index,
}

impl ItemGroupWidget {
    pub fn new(assets: &Rc<Assets>, text: impl Into<Name>, index: Index) -> Self {
        let mut menu = ItemMenuWidget::new(assets);
        menu.sync.hide(); // NOTE: Disabled in itch demo
        Self {
            state: WidgetState::new(),
            edited: IconWidget::new(&assets.sprites.star),
            local: IconWidget::new(&assets.sprites.local),
            menu,
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            index,
        }
    }

    pub fn sync(&mut self, group_id: Index, cached: &CachedGroup) {
        self.index = group_id;
        self.text.text = cached.data.owner.name.clone();
        if cached.data.id == 0 {
            self.local.show();
            self.edited.hide();
        } else {
            self.local.hide();
            if cached
                .origin
                .as_ref()
                .map_or(false, |info| info.hash != cached.hash)
            {
                self.edited.show();
            } else {
                self.edited.hide();
            }
        }
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.right_clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.can_focus = false;
        }

        self.state.update(position, context);

        let widgets = [&mut self.edited, &mut self.local];
        if widgets.iter().any(|widget| widget.state.visible) {
            let icons = position
                .cut_left(position.height() / 2.0)
                .extend_left(-context.font_size * 0.2)
                .extend_symmetric(-vec2(0.0, context.font_size * 0.15));
            let positions = icons.split_rows(widgets.len());
            for (widget, pos) in widgets.into_iter().zip(positions) {
                widget.update(pos, context);
            }
        }

        self.text.update(position, &mut context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.state.clicked {
            action = Some(LevelSelectAction::EditGroup(self.index));
        } else if self.menu.sync.state.clicked {
            action = Some(LevelSelectAction::SyncGroup(self.index));
        } else if self.menu.delete.state.clicked {
            action = Some(LevelSelectAction::DeleteGroup(self.index));
        }
        action
    }
}

#[derive(Clone)]
pub struct ItemLevelWidget {
    pub state: WidgetState,
    pub edited: IconWidget,
    pub local: IconWidget,
    pub text: TextWidget,
    pub group: Index,
    pub index: usize,
    pub level: Rc<LevelFull>,
    pub menu: ItemMenuWidget,
}

impl ItemLevelWidget {
    pub fn new(
        assets: &Rc<Assets>,
        text: impl Into<Name>,
        group: Index,
        index: usize,
        level: Rc<LevelFull>,
    ) -> Self {
        let mut menu = ItemMenuWidget::new(assets);
        menu.sync.hide();
        Self {
            state: WidgetState::new(),
            edited: IconWidget::new(&assets.sprites.star),
            local: IconWidget::new(&assets.sprites.local),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            group,
            index,
            level,
            menu,
        }
    }

    pub fn sync(
        &mut self,
        group_idx: Index,
        level_index: usize,
        cached: &Rc<LevelFull>,
        edited: bool,
    ) {
        self.index = level_index;
        self.group = group_idx;
        self.level = cached.clone();
        self.text.text = cached.meta.name.clone();
        if cached.meta.id == 0 {
            self.local.show();
            self.edited.hide();
        } else {
            self.local.hide();
            if edited {
                self.edited.show();
            } else {
                self.edited.hide();
            }
        }
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.right_clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.window.update(context.delta_time);
        if self.menu.window.show.time.is_min() {
            self.menu.hide();
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.can_focus = false;
        }

        self.state.update(position, context);

        let widgets = [&mut self.edited, &mut self.local];
        if widgets.iter().any(|widget| widget.state.visible) {
            let icons = position
                .cut_left(position.height() / 2.0)
                .extend_left(-context.font_size * 0.2)
                .extend_symmetric(-vec2(0.0, context.font_size * 0.15));
            let positions = icons.split_rows(widgets.len());
            for (widget, pos) in widgets.into_iter().zip(positions) {
                widget.update(pos, context);
            }
        }

        self.text.update(position, &mut context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.state.clicked {
            action = Some(LevelSelectAction::EditLevel(self.group, self.index));
        } else if self.menu.sync.state.clicked {
            action = Some(LevelSelectAction::SyncGroup(self.group));
        } else if self.menu.delete.state.clicked {
            action = Some(LevelSelectAction::DeleteLevel(self.group, self.index));
        }
        action
    }
}

pub struct NewMenuWidget {
    pub window: UiWindow<()>,
    pub state: WidgetState,
    pub create: TextWidget,
    pub browse: TextWidget,
}

impl NewMenuWidget {
    pub fn new(_assets: &Rc<Assets>) -> Self {
        Self {
            window: UiWindow::new((), 0.15),
            state: WidgetState::new(),
            create: TextWidget::new("create"),
            browse: TextWidget::new("browse"),
        }
    }
}

impl Widget for NewMenuWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        let position =
            position.translate(vec2(0.0, -position.height() + context.layout_size * 0.5));
        let size = vec2(position.width(), 2.5 * context.font_size);
        let position = position.align_aabb(size, vec2(0.0, 1.0));

        self.state.update(position, context);
        let rows = [&mut self.browse, &mut self.create];
        let spacing = context.layout_size * 0.3;
        let item_size = vec2(
            size.x - context.layout_size * 0.6,
            (size.y - spacing) / rows.len() as f32 - spacing,
        );
        let item = position
            .extend_up(-spacing)
            .align_aabb(item_size, vec2(0.5, 1.0));
        let positions = item.stack(vec2(0.0, -item.height() - spacing), rows.len());
        for (widget, pos) in rows.into_iter().zip(positions) {
            widget.update(pos, &mut context.scale_font(0.7));
        }

        context.update_focus(self.state.hovered);
    }
}

#[derive(Clone)]
pub struct ItemMenuWidget {
    pub window: UiWindow<()>,
    pub state: WidgetState,
    pub sync: IconButtonWidget,
    pub edit: IconButtonWidget,
    pub delete: IconButtonWidget,
}

impl ItemMenuWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            window: UiWindow::new((), 0.15),
            state: WidgetState::new(),
            sync: IconButtonWidget::new_normal(&assets.sprites.reset),
            edit: IconButtonWidget::new_normal(&assets.sprites.edit),
            delete: IconButtonWidget::new_danger(&assets.sprites.trash),
        }
    }
}

impl Widget for ItemMenuWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.window.update(context.delta_time);
        if self.window.show.time.is_min() {
            self.hide();
        }

        let mut columns: Vec<_> = [&mut self.delete, &mut self.edit, &mut self.sync]
            .into_iter()
            .filter(|widget| widget.state.visible)
            .collect();

        let position = position.translate(vec2(context.layout_size, -position.height()));
        let size = vec2(columns.len() as f32, 1.0) * 2.0 * context.layout_size;
        let position = position.align_aabb(size, vec2(0.0, 1.0));

        self.state.update(position, context);

        if !self.window.show.time.is_max() {
            return;
        }

        let positions = position.split_columns(columns.len());
        for (widget, pos) in columns.iter_mut().zip(positions) {
            widget.update(pos, context);
        }

        context.update_focus(self.state.hovered);
    }
}
