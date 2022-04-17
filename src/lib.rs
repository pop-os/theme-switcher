#[macro_use]
extern crate gtk_extras;

mod gresource;

use gtk::prelude::*;
use gtk_extras::{
    settings::{GeditPreferencesEditor, GnomeDesktopInterface, MeldPreferencesEditor},
    ImageSelection, ImageSrc, SelectionVariant,
};

use std::cell::Cell;
use std::rc::Rc;
use std::ops::Deref;

#[derive(Clone, Copy, Debug)]
enum ThemeSelection {
    Light,
    Dark,
}

pub struct PopThemeSwitcher(gtk::Container);

impl PopThemeSwitcher {
    pub fn new() -> Self {
        gresource::init().expect("failed to init pop-theme-switcher gresource");

        let gdi = GnomeDesktopInterface::new();
        let gpe = GeditPreferencesEditor::new_checked();
        let mpe = MeldPreferencesEditor::new_checked();

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
            let (color_scheme, gtk_theme, gedit_scheme) = match event {
                ThemeSelection::Light => ("prefer-light", "Pop", "pop-light"),
                ThemeSelection::Dark => ("prefer-dark", "Pop-dark", "pop-dark"),
            };

            gdi.set_color_scheme(color_scheme);
            gdi.set_gtk_theme(gtk_theme);
            if let Some(gpe) = gpe.as_ref() {
                gpe.set_scheme(gedit_scheme);
            }
            if let Some(mpe) = mpe.as_ref() {
                mpe.set_style_scheme(gedit_scheme);
            }

            glib::Continue(true)
        });

        Self((*selection).clone().upcast::<gtk::Container>())
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
