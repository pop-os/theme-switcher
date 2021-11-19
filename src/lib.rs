#[macro_use]
extern crate gtk_extras;

mod gresource;

use gtk::prelude::*;
use gtk_extras::{
    settings::{GeditPreferencesEditor, GnomeDesktopInterface},
    ImageSelection, ImageSrc, SelectionVariant, ToggleVariant, VariantToggler,
};

use std::cell::Cell;
use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone, Copy, Debug)]
enum ThemeVariant {
    Dark,
    Slim,
}

#[derive(Clone, Copy, Debug)]
enum ThemeSelection {
    Light,
    Dark,
}

pub const DARK: u8 = 0b01;
pub const SLIM: u8 = 0b10;

pub struct PopThemeSwitcher(gtk::Container);

impl PopThemeSwitcher {
    pub fn new() -> Self {
        gresource::init().expect("failed to init pop-theme-switcher gresource");

        let gpe = GeditPreferencesEditor::new_checked();
        let gdi = GnomeDesktopInterface::new();

        let variants = {
            let current_theme = gdi.gtk_theme();

            let dark_mode = current_theme.contains("dark");

            [
                SelectionVariant {
                    name:         "Light",
                    image:        Some(ImageSrc::Resource("/org/Pop-OS/ThemeSwitcher/light.svg")),
                    size_request: None,
                    active:       !dark_mode,
                    event:        ThemeSelection::Light,
                },
                SelectionVariant {
                    name:         "Dark",
                    image:        Some(ImageSrc::Resource("/org/Pop-OS/ThemeSwitcher/dark.svg")),
                    size_request: None,
                    active:       dark_mode,
                    event:        ThemeSelection::Dark,
                },
            ]
        };

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        //TODO: fix extra events from ImageSelection::new
        let selection_ready = Rc::new(Cell::new(false));
        let handler = {
            let selection_ready = selection_ready.clone();
            move |event| {
                if selection_ready.get() {
                    let _ = tx.send(event);
                }
            }
        };

        let selection = cascade! {
            ImageSelection::new(&variants, ImageSrc::Resource(""), handler);
            ..set_max_children_per_line(2);
            ..set_column_spacing(24);
            ..set_row_spacing(24);
            ..set_halign(gtk::Align::Center);
        };

        selection_ready.set(true);
        rx.attach(None, move |event| {
            let (gtk_theme, gedit_scheme) = match event {
                ThemeSelection::Light => ("Pop", "pop-light"),
                ThemeSelection::Dark => ("Pop-dark", "pop-dark"),
            };

            if let Some(gpe) = gpe.as_ref() {
                gpe.set_scheme(gedit_scheme);
            }
            gdi.set_gtk_theme(gtk_theme);

            glib::Continue(true)
        });

        Self((*selection).clone().upcast::<gtk::Container>())
    }

    pub fn dark_and_slim() -> Self {
        let gpe = GeditPreferencesEditor::new_checked();
        let gdi = GnomeDesktopInterface::new();

        let mut flags = 0;

        let variants = {
            let current_theme = gdi.gtk_theme();

            let dark_mode = current_theme.contains("dark");
            let slim_mode = current_theme.contains("slim");

            if dark_mode {
                flags |= DARK;
            }

            if slim_mode {
                flags |= SLIM;
            }

            [
                ToggleVariant {
                    name:        "Dark Mode",
                    description: "Changes your applications to a dark theme for easier viewing at \
                                  night.",
                    event:       ThemeVariant::Dark,
                    active:      dark_mode,
                },
                ToggleVariant {
                    name:        "Slim Mode",
                    description: "Reduces the height of application headers.",
                    event:       ThemeVariant::Slim,
                    active:      slim_mode,
                },
            ]
        };

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let theme_switcher = VariantToggler::new(&variants, move |event, active| {
            let _ = tx.send((event, active));
        });

        rx.attach(None, move |(event, active)| {
            match (event, active) {
                (ThemeVariant::Dark, true) => flags |= DARK,
                (ThemeVariant::Dark, false) => flags &= 255 ^ DARK,
                (ThemeVariant::Slim, true) => flags |= SLIM,
                (ThemeVariant::Slim, false) => flags &= 255 ^ SLIM,
            }

            let (gtk_theme, gedit_scheme) = if flags & (DARK | SLIM) == DARK | SLIM {
                ("Pop-slim-dark", "pop-dark")
            } else if flags & DARK != 0 {
                ("Pop-dark", "pop-dark")
            } else if flags & SLIM != 0 {
                ("Pop-slim", "pop-light")
            } else {
                ("Pop", "pop-light")
            };

            if let Some(gpe) = gpe.as_ref() {
                gpe.set_scheme(gedit_scheme);
            }
            gdi.set_gtk_theme(gtk_theme);

            glib::Continue(true)
        });

        Self(theme_switcher.into())
    }

    pub fn grab_focus(&self) {
        use gtk_extras::widgets::iter_from;

        for child in iter_from::<gtk::FlowBoxChild, gtk::Container>(&*self) {
            if let Some(inner) = child.child() {
                let inner = inner.downcast::<gtk::Container>().unwrap();
                if let Some(radio) = iter_from::<gtk::RadioButton, _>(&inner).next() {
                    if radio.is_active() {
                        child.grab_focus();
                    }
                }
            }
        }
    }
}

impl Deref for PopThemeSwitcher {
    type Target = gtk::Container;

    fn deref(&self) -> &Self::Target { &self.0 }
}
