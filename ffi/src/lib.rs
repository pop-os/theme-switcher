use glib::object::{Cast, ObjectType};
use gtk::WidgetExt;
use pop_theme_switcher::PopThemeSwitcher as Switcher;
use std::ptr;

#[no_mangle]
pub struct PopThemeSwitcher;

#[no_mangle]
pub extern "C" fn pop_theme_switcher_new() -> *mut PopThemeSwitcher {
    unsafe {
        gtk::set_initialized();
    }

    Box::into_raw(Box::new(Switcher::new())) as *mut PopThemeSwitcher
}

#[no_mangle]
pub extern "C" fn pop_theme_switcher_grab_focus(ptr: *const PopThemeSwitcher) {
    if let Some(switcher) = unsafe { (ptr as *const Switcher).as_ref() } {
        switcher.grab_focus();
    }
}

#[no_mangle]
pub extern "C" fn pop_theme_switcher_widget(
    ptr: *const PopThemeSwitcher,
) -> *mut gtk_sys::GtkWidget {
    let value = unsafe { (ptr as *const Switcher).as_ref() };
    value.map_or(ptr::null_mut(), |widget| {
        let widget: &gtk::Container = widget.as_ref();
        widget.upcast_ref::<gtk::Widget>().as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn pop_theme_switcher_free(widget: *mut PopThemeSwitcher) {
    unsafe { Box::from_raw(widget as *mut Switcher) };
}
