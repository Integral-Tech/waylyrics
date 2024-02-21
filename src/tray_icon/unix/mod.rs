use std::{env, process};

use async_channel::Sender;
use gettextrs::gettext;
use ksni::{Tray, TrayService};
use rust_decimal::prelude::Zero;
use strum::IntoEnumIterator;

use crate::app::actions::{UIAction, UI_ACTION};
use crate::sync::{PlayAction, PLAY_ACTION};

use crate::config::LyricDisplay;
use crate::sync::{list_player_names, PlayerId};

use crate::log::error;
use crate::{APP_ID, PACKAGE_NAME};

#[derive(Debug, Default)]
struct TrayIcon {}

impl Tray for TrayIcon {
    fn icon_name(&self) -> String {
        APP_ID.to_string()
    }
    fn id(&self) -> String {
        PACKAGE_NAME.into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let players = list_player_names();

        vec![
            SubMenu {
                label: gettext("Lyric Display Mode"),
                icon_name: "quickview".into(),
                submenu: LyricDisplay::iter()
                    .map(|display_mode| {
                        let label = gettext(display_mode.to_string());
                        StandardItem {
                            label,
                            activate: Box::new(move |_| {
                                let action = UIAction::SetDisplayMode(display_mode.to_string());
                                let _ = ui_action().send_blocking(action);
                            }),
                            ..Default::default()
                        }
                        .into()
                    })
                    .collect(),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Toggle Decoration"),
                icon_name: "window-new".into(),
                activate: Box::new(|_| {
                    let _ = ui_action().send_blocking(UIAction::SwitchDecoration);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Toggle Passthrough"),
                icon_name: "input-mouse".into(),
                activate: Box::new(|_| {
                    let _ = ui_action().send_blocking(UIAction::SwitchPassthrough);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Reload theme"),
                icon_name: "color-management".into(),
                activate: Box::new(|_| {
                    let _ = ui_action().send_blocking(UIAction::ReloadTheme);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            SubMenu {
                label: gettext("Select Player"),
                icon_name: "format-indent-more".into(),
                enabled: !players.len().is_zero(),
                submenu: players
                    .into_iter()
                    .map(
                        |PlayerId {
                             player_name,
                             inner_id,
                         }| {
                            StandardItem {
                                label: player_name,
                                activate: Box::new(move |_| {
                                    let _ = play_action()
                                        .send_blocking(PlayAction::Connect(inner_id.clone()));
                                }),
                                ..Default::default()
                            }
                            .into()
                        },
                    )
                    .collect(),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Search Lyric"),
                icon_name: "system-search".into(),
                activate: Box::new(|_| {
                    let _ = play_action().send_blocking(PlayAction::SearchLyric);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Refetch Lyric"),
                icon_name: "folder-download".into(),
                activate: Box::new(|_| {
                    let _ = play_action().send_blocking(PlayAction::RefetchLyric);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: gettext("Restart"),
                icon_name: "system-reboot".into(),
                activate: Box::new(|_| {
                    let my_path = env::args().nth(0).unwrap();
                    let Ok(_) = process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("sleep 1 && {my_path}"))
                        .spawn()
                    else {
                        error!("failed to run waylyrics");
                        return;
                    };
                    let _ = ui_action().send_blocking(UIAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Quit"),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| {
                    let _ = ui_action().send_blocking(UIAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn start_tray_service() -> Option<()> {
    let service = TrayService::new(TrayIcon::default());

    Some(service.spawn())
}

fn ui_action() -> Sender<UIAction> {
    let ui_action = UI_ACTION.get().unwrap().clone();
    ui_action
}
fn play_action() -> Sender<PlayAction> {
    let play_action = PLAY_ACTION.get().unwrap().clone();
    play_action
}
