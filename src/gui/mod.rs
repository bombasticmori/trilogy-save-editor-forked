use anyhow::Error;
use clap::ArgMatches;
use flume::{Receiver, Sender};
use imgui::{
    im_str, ChildWindow, ColorStackToken, Condition, ImString, PopupModal, ProgressBar, StyleColor,
    TabBar, TabItem, Ui, Window,
};
use std::path::PathBuf;

use crate::{
    databases,
    event_handler::{MainEvent, SaveGame},
    save_data::shared::appearance::{HasHeadMorph, HeadMorph},
};

mod backend;
mod changelog;
mod imgui_utils;
mod mass_effect_1;
mod mass_effect_1_le;
mod mass_effect_2;
mod mass_effect_3;
mod raw_ui;

static NOTIFICATION_TIME: f64 = 1.5; // seconde

// States
#[derive(Default)]
struct NotificationState {
    string: ImString,
    close_time: f64,
}

#[derive(Default)]
struct State {
    save_game: Option<SaveGame>,
    error: Option<Error>,
    notification: Option<NotificationState>,
}

// Events
pub enum UiEvent {
    Error(Error),
    Notification(&'static str),
    OpenedSave(SaveGame),
    ImportedHeadMorph(Box<HeadMorph>),
}

// UI
pub fn run(event_addr: Sender<MainEvent>, rx: Receiver<UiEvent>, args: ArgMatches) {
    let mut state = State::default();

    if let Err(err) = databases::initialize() {
        state.error = Some(err);
    }

    // UI
    let system = backend::init(
        &format!("Trilogy Save Editor - v{} by Karlitos", env!("CARGO_PKG_VERSION")),
        1000.0,
        670.0,
        &args,
    );

    // Open file from command line
    if let Some(path) = args.value_of("FILE") {
        let _ = event_addr.send(MainEvent::OpenSave(path.to_owned()));
    }

    system.main_loop(move |run, ui, dropped_file| {
        // Open file dropped into the window
        if let Some(path) = dropped_file.take() {
            let _ = event_addr.send(MainEvent::OpenSave(path));
        }

        // Manage events
        rx.try_iter().for_each(|ui_event| match ui_event {
            UiEvent::Error(err) => {
                state.error = Some(err);
            }
            UiEvent::Notification(string) => {
                state.notification = Some(NotificationState {
                    string: ImString::new(string),
                    close_time: ui.time() + NOTIFICATION_TIME,
                })
            }
            UiEvent::OpenedSave(opened_save_game) => {
                state.save_game = Some(opened_save_game);
            }
            UiEvent::ImportedHeadMorph(head_morph) => {
                let has_head_morph =
                    HasHeadMorph { has_head_morph: true, head_morph: Some(*head_morph) };
                match state.save_game.as_mut() {
                    Some(SaveGame::MassEffect1Le { save_game, .. }) => {
                        save_game.save_data.player.head_morph = has_head_morph
                    }
                    Some(SaveGame::MassEffect2 { save_game, .. }) => {
                        save_game.player.appearance.head_morph = has_head_morph
                    }
                    Some(SaveGame::MassEffect2Le { save_game, .. }) => {
                        save_game.player.appearance.head_morph = has_head_morph
                    }
                    Some(SaveGame::MassEffect3 { save_game, .. }) => {
                        save_game.player.appearance.head_morph = has_head_morph
                    }
                    Some(SaveGame::MassEffect1 { .. }) | None => unreachable!(),
                }
            }
        });

        let ui = Gui::new(ui, &event_addr);
        ui.draw(run, &mut state);
    });
}

pub struct Gui<'ui> {
    ui: &'ui Ui<'ui>,
    event_addr: Sender<MainEvent>,
}

impl<'ui> Gui<'ui> {
    fn new(ui: &'ui Ui<'ui>, event_addr: &Sender<MainEvent>) -> Self {
        Self { ui, event_addr: Sender::clone(event_addr) }
    }

    fn draw(&self, _: &mut bool, state: &mut State) {
        let ui = self.ui;

        // Main window
        let window = Window::new(im_str!("###main"))
            .size(ui.io().display_size, Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .menu_bar(true)
            .bring_to_front_on_focus(false)
            .collapsible(false);

        // Pop on drop
        let _colors = self.style_colors(match state.save_game {
            None => Theme::MassEffect3,
            Some(SaveGame::MassEffect1 { .. }) | Some(SaveGame::MassEffect1Le { .. }) => {
                Theme::MassEffect1
            }
            Some(SaveGame::MassEffect2 { .. }) | Some(SaveGame::MassEffect2Le { .. }) => {
                Theme::MassEffect2
            }
            Some(SaveGame::MassEffect3 { .. }) => Theme::MassEffect3,
        });

        // Window
        if let Some(_t) = window.begin(ui) {
            // Main menu bar
            if let Some(_t) = ui.begin_menu_bar() {
                if ui.button(im_str!("Open")) {
                    self.open_dialog();
                }
                if let Some(save_game) = &state.save_game {
                    if ui.button(im_str!("Save")) {
                        self.save_dialog(save_game);
                    }
                }
                if let Some(_t) = ui.begin_menu(im_str!("About")) {
                    self.draw_about();
                }
            }

            // Error popup
            self.draw_error(&mut state.error);

            // Notification
            self.draw_notification_overlay(&mut state.notification);

            // Game
            match &mut state.save_game {
                None => self.draw_change_log(),
                Some(SaveGame::MassEffect1 { save_game, .. }) => self.draw_mass_effect_1(save_game),
                Some(SaveGame::MassEffect1Le { save_game, .. }) => {
                    self.draw_mass_effect_1_le(&mut save_game.save_data)
                }
                Some(SaveGame::MassEffect2 { save_game, .. }) => self.draw_mass_effect_2(save_game),
                Some(SaveGame::MassEffect2Le { save_game, .. }) => {
                    self.draw_mass_effect_2_le(save_game)
                }
                Some(SaveGame::MassEffect3 { save_game, .. }) => self.draw_mass_effect_3(save_game),
            };
        }
    }

    #[cfg(target_os = "windows")]
    fn get_document_dir() -> PathBuf {
        match dirs::document_dir() {
            Some(mut path) => {
                path.push("BioWare/");
                path
            }
            None => PathBuf::default(),
        }
    }

    // FIXME: Find some nicer way of finding where the game saves are.
    // Currently, this should be universal for everyone who has their
    // Mass Effect games installed in the default steam library, in
    // the user's home directory.
    #[cfg(target_os = "linux")]
    fn get_document_dir() -> PathBuf {
        match dirs::home_dir() {
            Some(mut path) => {
                path.push(".steam/root/steamapps/compatdata/1328670/pfx/drive_c/users/steamuser/My Documents/BioWare/");
                path
            }
            None => PathBuf::default(),
        }
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    fn get_document_dir() -> PathBuf {
        PathBuf::default()
    }

    fn open_dialog(&self) {
        let dir = Self::get_document_dir();

        let file = tinyfiledialogs::open_file_dialog(
            "",
            &dir.to_string_lossy(),
            Some((
                &["*.pcsav", "*.MassEffectSave"],
                "Mass Effect Trilogy Save (*.pcsav, *.MassEffectSave)",
            )),
        );

        if let Some(path) = file {
            let _ = self.event_addr.send(MainEvent::OpenSave(path));
        }
    }

    fn save_dialog(&self, save_game: &SaveGame) {
        let (file_path, description, extension) = match save_game {
            SaveGame::MassEffect1 { file_path, .. } => {
                (file_path, "Mass Effect 1 Save (*.MassEffectSave)", "*.MassEffectSave")
            }
            SaveGame::MassEffect1Le { file_path, .. } => {
                (file_path, "Mass Effect 1 Legendary Save (*.pcsav)", "*.pcsav")
            }
            SaveGame::MassEffect2 { file_path, .. } | SaveGame::MassEffect2Le { file_path, .. } => {
                (file_path, "Mass Effect 2 Save (*.pcsav)", "*.pcsav")
            }
            SaveGame::MassEffect3 { file_path, .. } => {
                (file_path, "Mass Effect 3 Save (*.pcsav)", "*.pcsav")
            }
        };

        let file =
            tinyfiledialogs::save_file_dialog_with_filter("", file_path, &[extension], description);

        if let Some(path) = file {
            let _ = self.event_addr.send(MainEvent::SaveSave(path, save_game.clone()));
        }
    }

    fn draw_about(&self) {
        let ui = self.ui;

        ui.separator();
        ui.text(im_str!("(C) 2021 Karlitos"));
        ui.separator();
        if let Some(_t) = ui.begin_menu(im_str!("License")) {
            if let Some(_t) = TabBar::new(im_str!("tabs")).begin(ui) {
                if let Some(_t) = TabItem::new(im_str!("English")).begin(ui) {
                    if let Some(_t) = ChildWindow::new("scroll").size([540.0, 500.0]).begin(ui) {
                        ui.text(include_str!("../../License_CeCILL_V2.1-en.txt"));
                    }
                }
                if let Some(_t) = TabItem::new(im_str!("French")).begin(ui) {
                    if let Some(_t) = ChildWindow::new("scroll").size([540.0, 500.0]).begin(ui) {
                        ui.text(include_str!("../../Licence_CeCILL_V2.1-fr.txt"));
                    }
                }
            }
        }
    }

    fn draw_error(&self, option_error: &mut Option<Error>) {
        let ui = self.ui;

        if let Some(error) = option_error {
            ui.open_popup(im_str!("Error###error"));

            if let Some(_t) =
                PopupModal::new(im_str!("Error###error")).always_auto_resize(true).begin_popup(ui)
            {
                ui.text(error.to_string());

                let chain = error.chain().skip(1);
                if chain.len() != 0 {
                    ui.separator();
                    for error in chain {
                        ui.text(error.to_string());
                    }
                }
                ui.separator();

                if ui.button_with_size(im_str!("OK"), [70.0, 0.0]) {
                    *option_error = None;
                    ui.close_current_popup();
                }
            }
        }
    }

    fn draw_notification_overlay(&self, notification: &mut Option<NotificationState>) {
        if let Some(NotificationState { string, close_time }) = notification {
            let ui = self.ui;
            let time = ui.time();

            let _style = ui.push_style_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 0.3]);
            let window = Window::new(im_str!("###notification"))
                .position([ui.io().display_size[0] / 2.0, 5.0], Condition::Always)
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .always_auto_resize(true);

            if let Some(_t) = window.begin(ui) {
                ui.text(&string);

                let remaining = (*close_time - time) / NOTIFICATION_TIME;
                ProgressBar::new(remaining as f32)
                    .overlay_text(im_str!("time_bar"))
                    .size([-0.000001, 2.0])
                    .build(ui);
            }

            if *close_time < time {
                *notification = None;
            }
        }
    }

    fn draw_help_marker(&self, desc: &str) {
        let ui = self.ui;

        ui.text_disabled(im_str!("(?)"));
        if ui.is_item_hovered() {
            let _t = ui.begin_tooltip();
            ui.text(desc);
        }
    }

    // Style
    fn style_colors(&self, game_theme: Theme) -> [ColorStackToken<'ui>; 23] {
        let ui = self.ui;
        let theme = match game_theme {
            Theme::MassEffect1 => ColorTheme {
                bg_color: [0.11, 0.32, 0.43, 1.0],
                color: [0.16, 0.42, 0.58, 1.0],
                active_color: [0.28, 0.55, 0.67, 1.0],
                hover_color: [0.83, 0.43, 0.17, 1.0],
            },
            Theme::MassEffect2 => ColorTheme {
                bg_color: [0.64, 0.32, 0.12, 1.0],
                color: [0.70, 0.37, 0.16, 1.0],
                active_color: [0.85, 0.49, 0.25, 1.0],
                hover_color: [0.22, 0.52, 0.23, 1.0],
            },
            Theme::MassEffect3 => ColorTheme {
                bg_color: [0.40, 0.0, 0.0, 1.0],
                color: [0.53, 0.0, 0.0, 1.0],
                active_color: [0.70, 0.0, 0.0, 1.0],
                hover_color: [0.02, 0.28, 0.43, 1.0],
            },
        };

        [
            ui.push_style_color(StyleColor::WindowBg, [0.05, 0.05, 0.05, 1.0]),
            ui.push_style_color(StyleColor::Border, [0.20, 0.20, 0.20, 1.0]),
            ui.push_style_color(StyleColor::Separator, [0.20, 0.20, 0.20, 1.0]),
            ui.push_style_color(StyleColor::PopupBg, [0.07, 0.07, 0.07, 0.99]),
            ui.push_style_color(StyleColor::TitleBgActive, theme.active_color),
            ui.push_style_color(StyleColor::FrameBg, theme.bg_color),
            ui.push_style_color(StyleColor::FrameBgActive, theme.active_color),
            ui.push_style_color(StyleColor::FrameBgHovered, theme.hover_color),
            ui.push_style_color(StyleColor::TextSelectedBg, theme.active_color),
            ui.push_style_color(StyleColor::Button, theme.bg_color),
            ui.push_style_color(StyleColor::ButtonActive, theme.active_color),
            ui.push_style_color(StyleColor::ButtonHovered, theme.hover_color),
            ui.push_style_color(StyleColor::Tab, theme.color),
            ui.push_style_color(StyleColor::TabActive, theme.active_color),
            ui.push_style_color(StyleColor::TabHovered, theme.hover_color),
            ui.push_style_color(StyleColor::Header, theme.bg_color),
            ui.push_style_color(StyleColor::HeaderActive, theme.active_color),
            ui.push_style_color(StyleColor::HeaderHovered, theme.hover_color),
            ui.push_style_color(StyleColor::CheckMark, [1.0, 1.0, 1.0, 1.0]),
            ui.push_style_color(StyleColor::PlotHistogram, [1.0, 1.0, 1.0, 1.0]),
            ui.push_style_color(StyleColor::TableRowBg, [0.07, 0.07, 0.07, 1.0]),
            ui.push_style_color(StyleColor::TableRowBgAlt, [0.1, 0.1, 0.1, 1.0]),
            ui.push_style_color(StyleColor::TableBorderStrong, [0.20, 0.20, 0.20, 1.0]),
        ]
    }
}

enum Theme {
    MassEffect1,
    MassEffect2,
    MassEffect3,
}

struct ColorTheme {
    bg_color: [f32; 4],
    color: [f32; 4],
    active_color: [f32; 4],
    hover_color: [f32; 4],
}
