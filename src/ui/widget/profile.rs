use super::*;

use crate::{leaderboard::Leaderboard, prelude::Assets, ui::layout::AreaOps};

pub struct ProfileWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
    pub offline: TextWidget,
    pub register: RegisterWidget,
    pub logged: LoggedWidget,
}

pub struct RegisterWidget {
    pub state: WidgetState,
    // pub username: InputWidget,
    // pub password: InputWidget,
    // pub login: ButtonWidget,
    // pub register: ButtonWidget,
    pub login_with: TextWidget,
    pub discord: IconButtonWidget,
}

pub struct LoggedWidget {
    pub state: WidgetState,
    pub username: TextWidget,
    pub logout: TextWidget,
}

impl ProfileWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            offline: TextWidget::new("Offline"),
            register: RegisterWidget {
                state: WidgetState::new(),
                // username: InputWidget::new("Username", false),
                // password: InputWidget::new("Password", true),
                // login: ButtonWidget::new("Login"),
                // register: ButtonWidget::new("Register"),
                login_with: TextWidget::new("Login with"),
                discord: IconButtonWidget::new_normal(&assets.sprites.discor),
            },
            logged: LoggedWidget {
                state: WidgetState::new(),
                username: TextWidget::new("<username>"),
                logout: TextWidget::new("Logout"),
            },
        }
    }
}

impl StatefulWidget for ProfileWidget {
    type State = Leaderboard;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let margin = context.layout_size * 0.5;
        let main = position.extend_uniform(-margin);

        let (off, reg, log) = match (state.is_online(), state.user.is_some()) {
            (false, _) => (true, false, false),
            (true, false) => (false, true, false),
            (true, true) => (false, false, true),
        };
        if off {
            self.offline.show();
            self.offline.update(main, context);
        } else {
            self.offline.hide();
        }
        if reg {
            self.register.show();
            self.register.update(main, context, state);
        } else {
            self.register.hide();
        }
        if log {
            self.logged.show();
            self.logged.update(main, context, state);
        } else {
            self.logged.hide();
        }
    }
}

impl StatefulWidget for RegisterWidget {
    type State = Leaderboard;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let mut main = position;

        // let rows = main.split_rows(3);
        // self.username.update(rows[0], context);
        // self.password.update(rows[1], context);

        // let cols = rows[2].split_columns(2);
        // self.login.update(cols[0], context);
        // self.register.update(cols[1], context);

        // let creds = Credentials {
        //     username: self.username.raw.clone(),
        //     password: self.password.raw.clone(),
        // };
        // if self.login.text.state.clicked {
        //     // state.login(creds);
        // } else if self.register.text.state.clicked {
        //     // state.register(creds);
        // }

        let login_with = main.cut_top(context.font_size);
        self.login_with.update(login_with, context);

        let with_options = [&mut self.discord];
        let size = vec2::splat(context.font_size * 1.2);
        let with = main.align_aabb(size, vec2(0.5, 0.5));
        let positions = with.stack_aligned(
            vec2(with.width() + context.layout_size, 0.0),
            with_options.len(),
            vec2(0.5, 0.5),
        );
        for (with, pos) in with_options.into_iter().zip(positions) {
            with.update(pos, context);
        }

        if self.discord.state.clicked {
            state.login_discord();
        }
    }
}

impl StatefulWidget for LoggedWidget {
    type State = Leaderboard;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        if let Some(user) = &state.user {
            self.username.text = user.name.clone();
        }

        let main = position;

        let rows = main.split_rows(2);
        self.username.update(rows[0], context);
        self.logout.update(rows[1], context);

        if self.logout.state.clicked {
            state.logout();
        }
    }
}
