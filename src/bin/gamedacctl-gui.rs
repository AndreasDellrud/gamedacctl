use std::{cell::Cell, rc::Rc, str::FromStr, time::Duration};

use adw::prelude::*;
use gamedacctl::{
    Color, LightingPlan, Zone,
    profile::{MicrophoneLighting, Profile, ProfileBreatheMode, ProfileLighting, ProfileStore},
    transport::{HidTransport, Transport, TransportError},
};
use gtk::{gio, glib};

const APPLICATION_ID: &str = "io.github.andreasdellrud.gamedacctl";

#[derive(Clone)]
struct ColorInput {
    entry: gtk::Entry,
    picker: gtk::ColorDialogButton,
    widget: gtk::Box,
}

impl ColorInput {
    fn new(label: &str, initial: &str) -> Self {
        let entry = gtk::Entry::builder()
            .placeholder_text("#RRGGBB")
            .max_length(7)
            .width_chars(9)
            .max_width_chars(9)
            .build();
        entry.update_property(&[
            gtk::accessible::Property::Label(&format!("{label} hex color")),
            gtk::accessible::Property::Description("Exact six-digit RGB value"),
        ]);

        let dialog = gtk::ColorDialog::builder()
            .title(format!("Choose {label} color"))
            .modal(true)
            .with_alpha(false)
            .build();
        let picker = gtk::ColorDialogButton::new(Some(dialog));
        picker.set_tooltip_text(Some(&format!("Choose {label} color")));
        picker.update_property(&[gtk::accessible::Property::Label(&format!(
            "Choose {label} color"
        ))]);

        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        widget.append(&entry);
        widget.append(&picker);

        let input = Self {
            entry,
            picker,
            widget,
        };
        input.connect_sync();
        input.set_text(initial);
        input
    }

    fn connect_sync(&self) {
        self.entry.connect_changed({
            let picker = self.picker.clone();
            move |entry| match Color::from_str(entry.text().as_str()) {
                Ok(color) => {
                    entry.remove_css_class("error");
                    picker.set_rgba(&color_to_rgba(color));
                }
                Err(_) => entry.add_css_class("error"),
            }
        });
        self.picker.connect_rgba_notify({
            let entry = self.entry.clone();
            move |picker| {
                let text = rgba_to_color(picker.rgba()).to_string();
                if entry.text().as_str() != text {
                    entry.set_text(&text);
                }
            }
        });
    }

    fn set_text(&self, text: &str) {
        self.entry.set_text(text);
    }

    fn text(&self) -> String {
        self.entry.text().to_string()
    }

    fn parse(&self, label: &str) -> Result<Color, String> {
        Color::from_str(self.entry.text().as_str()).map_err(|error| format!("{label}: {error}"))
    }
}

fn color_to_rgba(color: Color) -> gtk::gdk::RGBA {
    let [red, green, blue] = color.bytes();
    gtk::gdk::RGBA::new(
        f32::from(red) / 255.0,
        f32::from(green) / 255.0,
        f32::from(blue) / 255.0,
        1.0,
    )
}

fn rgba_to_color(rgba: gtk::gdk::RGBA) -> Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::new(
        channel(rgba.red()),
        channel(rgba.green()),
        channel(rgba.blue()),
    )
}

#[derive(Clone)]
struct PaletteRow {
    widget: gtk::Box,
    controls: gtk::Box,
    color: ColorInput,
    move_up: gtk::Button,
    move_down: gtk::Button,
    remove: gtk::Button,
}

#[derive(Clone)]
struct PaletteEditor {
    widget: gtk::Box,
    rows: Rc<Vec<PaletteRow>>,
    count: Rc<Cell<usize>>,
    minimum: Rc<Cell<usize>>,
    maximum: Rc<Cell<usize>>,
    add: gtk::Button,
}

impl PaletteEditor {
    fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let mut rows = Vec::new();
        for index in 0..4 {
            let color = ColorInput::new(
                &format!("Animation color {}", index + 1),
                if index == 0 { "#FF0000" } else { "#0000FF" },
            );
            let move_up = icon_button("go-up-symbolic", &format!("Move color {} up", index + 1));
            let move_down = icon_button(
                "go-down-symbolic",
                &format!("Move color {} down", index + 1),
            );
            let remove = icon_button(
                "user-trash-symbolic",
                &format!("Remove color {}", index + 1),
            );
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            row.append(&color.widget);
            let controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            controls.append(&move_up);
            controls.append(&move_down);
            controls.append(&remove);
            row.append(&controls);
            widget.append(&row);
            rows.push(PaletteRow {
                widget: row,
                controls,
                color,
                move_up,
                move_down,
                remove,
            });
        }
        let add = gtk::Button::with_label("Add color");
        add.set_halign(gtk::Align::Start);
        add.set_tooltip_text(Some("Add another animation color"));
        widget.append(&add);

        let editor = Self {
            widget,
            rows: Rc::new(rows),
            count: Rc::new(Cell::new(2)),
            minimum: Rc::new(Cell::new(2)),
            maximum: Rc::new(Cell::new(2)),
            add,
        };
        editor.connect_controls();
        editor.refresh();
        editor
    }

    fn connect_controls(&self) {
        self.add.connect_clicked({
            let editor = self.clone();
            move |_| {
                let count = editor.count.get();
                if count < editor.maximum.get() {
                    editor.count.set(count + 1);
                    editor.refresh();
                    editor.rows[count].color.entry.grab_focus();
                }
            }
        });
        for index in 0..self.rows.len() {
            self.rows[index].move_up.connect_clicked({
                let editor = self.clone();
                move |_| editor.swap(index, index.saturating_sub(1))
            });
            self.rows[index].move_down.connect_clicked({
                let editor = self.clone();
                move |_| editor.swap(index, index + 1)
            });
            self.rows[index].remove.connect_clicked({
                let editor = self.clone();
                move |_| editor.remove(index)
            });
        }
    }

    fn configure(&self, minimum: usize, maximum: usize) {
        self.minimum.set(minimum);
        self.maximum.set(maximum);
        self.count.set(self.count.get().clamp(minimum, maximum));
        self.refresh();
    }

    fn refresh(&self) {
        let count = self.count.get();
        for (index, row) in self.rows.iter().enumerate() {
            row.widget.set_visible(index < count);
            row.move_up.set_sensitive(index > 0 && index < count);
            row.move_down.set_sensitive(index + 1 < count);
            row.remove.set_sensitive(count > self.minimum.get());
        }
        self.add.set_visible(count < self.maximum.get());
    }

    fn swap(&self, first: usize, second: usize) {
        if first >= self.count.get() || second >= self.count.get() || first == second {
            return;
        }
        let first_text = self.rows[first].color.text();
        let second_text = self.rows[second].color.text();
        self.rows[first].color.set_text(&second_text);
        self.rows[second].color.set_text(&first_text);
    }

    fn remove(&self, index: usize) {
        let count = self.count.get();
        if count <= self.minimum.get() || index >= count {
            return;
        }
        for current in index..count - 1 {
            let next = self.rows[current + 1].color.text();
            self.rows[current].color.set_text(&next);
        }
        self.count.set(count - 1);
        self.refresh();
    }

    fn colors(&self) -> Result<Vec<Color>, String> {
        (0..self.count.get())
            .map(|index| {
                self.rows[index]
                    .color
                    .parse(&format!("Color {}", index + 1))
            })
            .collect()
    }

    fn set_colors(&self, colors: &[Color]) {
        let count = colors.len().clamp(self.minimum.get(), self.maximum.get());
        self.count.set(count);
        for (index, color) in colors.iter().take(count).enumerate() {
            self.rows[index].color.set_text(&color.to_string());
        }
        self.refresh();
    }

    fn set_narrow(&self, narrow: bool) {
        for row in self.rows.iter() {
            row.widget.set_orientation(if narrow {
                gtk::Orientation::Vertical
            } else {
                gtk::Orientation::Horizontal
            });
            row.controls.set_halign(if narrow {
                gtk::Align::End
            } else {
                gtk::Align::Fill
            });
        }
    }
}

fn icon_button(icon_name: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon_name);
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

#[derive(Clone)]
struct Editor {
    profile_name: gtk::Entry,
    profile_icon: gtk::Entry,
    profile_icon_widget: gtk::Box,
    style: gtk::DropDown,
    relationship: gtk::DropDown,
    left: ColorInput,
    right: ColorInput,
    microphone_live: ColorInput,
    microphone_muted: ColorInput,
    effect_colors: PaletteEditor,
    seconds: gtk::SpinButton,
    reverse: gtk::Switch,
    static_group: adw::PreferencesGroup,
    microphone_group: adw::PreferencesGroup,
    animation_group: adw::PreferencesGroup,
}

fn main() -> glib::ExitCode {
    let application = adw::Application::builder()
        .application_id(APPLICATION_ID)
        .flags(gio::ApplicationFlags::empty())
        .build();
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &adw::Application) {
    let (initial_store, initial_profile_error) = match ProfileStore::load() {
        Ok(store) => (store, None),
        Err(error) => {
            eprintln!("gamedacctl-gui: {error}");
            (ProfileStore::default(), Some(error))
        }
    };
    let store = Rc::new(std::cell::RefCell::new(initial_store));

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        "GameDAC Lighting",
        "Original GameDAC",
    )));
    let help = icon_button("help-about-symbolic", "How to use GameDAC Lighting");
    header.pack_end(&help);

    let page = adw::PreferencesPage::new();
    let status = build_device_group(&page);
    let toasts = adw::ToastOverlay::new();
    let profile_banner = adw::Banner::new("");

    let profile_names = gtk::StringList::new(&[]);
    let profile_picker = gtk::DropDown::new(Some(profile_names.clone()), gtk::Expression::NONE);
    let editor = build_editor();
    let master_lighting = gtk::Switch::new();
    master_lighting.set_valign(gtk::Align::Center);
    master_lighting.set_active(store.borrow().lighting_enabled);

    let profile_group = adw::PreferencesGroup::builder().title("Profiles").build();
    profile_group.add(&preference_row("Saved profile", None, &profile_picker));
    profile_group.add(&preference_row("Profile name", None, &editor.profile_name));
    profile_group.add(&preference_row(
        "Profile icon",
        Some("Optional emoji or symbol."),
        &editor.profile_icon_widget,
    ));
    let new_profile = gtk::Button::with_label("New");
    let delete_profile = gtk::Button::with_label("Delete");
    let profile_actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    profile_actions.set_homogeneous(true);
    profile_actions.append(&new_profile);
    profile_actions.append(&delete_profile);
    profile_group.add(&preference_row("Manage profiles", None, &profile_actions));
    page.add(&profile_group);

    let style_group = adw::PreferencesGroup::builder().title("Lighting").build();
    style_group.add(&preference_row(
        "Lighting enabled",
        Some("Keeps the selected profile when turned off."),
        &master_lighting,
    ));
    style_group.add(&preference_row("Lighting effect", None, &editor.style));
    page.add(&style_group);
    page.add(&editor.static_group);
    page.add(&editor.animation_group);
    page.add(&editor.microphone_group);

    let reconnect = gtk::Switch::new();
    reconnect.set_valign(gtk::Align::Center);
    reconnect.set_active(store.borrow().apply_on_reconnect);
    let behavior_group = adw::PreferencesGroup::builder().title("Behavior").build();
    behavior_group.add(&preference_row(
        "Apply after reconnect",
        Some("Restore the selected profile or off state."),
        &reconnect,
    ));
    page.add(&behavior_group);

    let save = gtk::Button::with_label("Save");
    let apply = gtk::Button::with_label("Apply");
    apply.add_css_class("suggested-action");
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    buttons.set_homogeneous(true);
    buttons.append(&save);
    buttons.append(&apply);
    let action_group = adw::PreferencesGroup::new();
    action_group.add(&preference_row("Apply lighting", None, &buttons));
    page.add(&action_group);

    refresh_profile_picker(&store.borrow(), &profile_names, &profile_picker);
    if let Some(profile) = store.borrow().selected() {
        populate_editor(&editor, profile);
    }
    update_effect_visibility(&editor);

    editor.style.connect_selected_notify({
        let editor = editor.clone();
        move |_| update_effect_visibility(&editor)
    });
    editor.relationship.connect_selected_notify({
        let editor = editor.clone();
        move |_| update_effect_visibility(&editor)
    });

    profile_picker.connect_selected_notify({
        let editor = editor.clone();
        let store = store.clone();
        let delete_profile = delete_profile.clone();
        move |picker| {
            if let Some(profile) = store.borrow().profiles.get(picker.selected() as usize) {
                populate_editor(&editor, profile);
                delete_profile.set_sensitive(true);
            } else {
                delete_profile.set_sensitive(false);
            }
        }
    });

    new_profile.connect_clicked({
        let editor = editor.clone();
        let profile_picker = profile_picker.clone();
        let delete_profile = delete_profile.clone();
        move |_| {
            profile_picker.set_selected(gtk::INVALID_LIST_POSITION);
            delete_profile.set_sensitive(false);
            reset_editor(&editor);
            editor.profile_name.grab_focus();
        }
    });

    save.connect_clicked({
        let editor = editor.clone();
        let store = store.clone();
        let profile_names = profile_names.clone();
        let profile_picker = profile_picker.clone();
        let reconnect = reconnect.clone();
        let toasts = toasts.clone();
        move |_| match profile_from_editor(&editor) {
            Ok(profile) => {
                let name = profile.name.clone();
                let apply_on_reconnect = reconnect.is_active();
                let result = ProfileStore::update(|latest| {
                    latest.upsert(profile)?;
                    latest.select(&name)?;
                    latest.apply_on_reconnect = apply_on_reconnect;
                    Ok(())
                });
                match result {
                    Ok(updated) => {
                        *store.borrow_mut() = updated;
                        refresh_profile_picker(&store.borrow(), &profile_names, &profile_picker);
                        show_toast(&toasts, "Profile saved");
                    }
                    Err(error) => {
                        show_toast(&toasts, &format!("Could not save profile: {error}"));
                    }
                }
            }
            Err(error) => show_toast(&toasts, &error),
        }
    });

    reconnect.connect_active_notify({
        let store = store.clone();
        let toasts = toasts.clone();
        move |toggle| {
            let desired = toggle.is_active();
            match ProfileStore::update(|latest| {
                latest.apply_on_reconnect = desired;
                Ok(())
            }) {
                Ok(updated) => *store.borrow_mut() = updated,
                Err(error) => {
                    show_toast(
                        &toasts,
                        &format!("Could not save reconnect policy: {error}"),
                    );
                }
            }
        }
    });

    master_lighting.connect_active_notify({
        let master_lighting = master_lighting.clone();
        let store = store.clone();
        let toasts = toasts.clone();
        move |toggle| {
            let desired = toggle.is_active();
            if desired == store.borrow().lighting_enabled {
                return;
            }
            let result = apply_master_state(&store.borrow(), desired).and_then(|()| {
                ProfileStore::update(|latest| {
                    latest.set_lighting_enabled(desired);
                    Ok(())
                })
                .map_err(|error| error.to_string())
            });
            match result {
                Ok(updated) => {
                    *store.borrow_mut() = updated;
                    show_toast(
                        &toasts,
                        if desired {
                            "Selected profile restored"
                        } else {
                            "Lighting turned off"
                        },
                    );
                }
                Err(error) => {
                    master_lighting.set_active(!desired);
                    show_toast(&toasts, &format!("Could not change lighting: {error}"));
                }
            }
        }
    });

    apply.connect_clicked({
        let editor = editor.clone();
        let master_lighting = master_lighting.clone();
        let store = store.clone();
        let toasts = toasts.clone();
        move |_| match profile_from_editor(&editor).and_then(apply_profile) {
            Ok(()) => match ProfileStore::update(|latest| {
                latest.set_lighting_enabled(true);
                Ok(())
            }) {
                Ok(updated) => {
                    *store.borrow_mut() = updated;
                    master_lighting.set_active(true);
                    show_toast(&toasts, "Lighting applied");
                }
                Err(error) => show_toast(
                    &toasts,
                    &format!("Lighting applied, but its state could not be saved: {error}"),
                ),
            },
            Err(error) => show_toast(&toasts, &error),
        }
    });

    let initial_device = match HidTransport::open() {
        Ok(transport) => {
            status.set_subtitle("Ready");
            Some(transport.path().to_owned())
        }
        Err(error) => {
            status.set_subtitle(&transport_status(&error));
            None
        }
    };
    if let Some(error) = initial_profile_error {
        profile_banner.set_title(&format!(
            "Profiles unavailable; existing file was left unchanged: {error}"
        ));
        profile_banner.set_revealed(true);
    }
    monitor_reconnect(store.clone(), status.clone(), initial_device);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_top_bar(&profile_banner);
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_width(320)
        .child(&page)
        .build();
    toolbar.set_content(Some(&scrolled));
    toasts.set_child(Some(&toolbar));

    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("GameDAC Lighting")
        .default_width(640)
        .default_height(760)
        .content(&toasts)
        .build();
    help.connect_clicked({
        let window = window.clone();
        move |_| show_help(&window)
    });
    let narrow = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("max-width: 500sp")
            .expect("the fixed narrow breakpoint must parse"),
    );
    narrow.connect_apply({
        let editor = editor.clone();
        let profile_actions = profile_actions.clone();
        let buttons = buttons.clone();
        move |_| {
            editor.effect_colors.set_narrow(true);
            profile_actions.set_orientation(gtk::Orientation::Vertical);
            buttons.set_orientation(gtk::Orientation::Vertical);
        }
    });
    narrow.connect_unapply({
        let editor = editor.clone();
        let profile_actions = profile_actions.clone();
        let buttons = buttons.clone();
        move |_| {
            editor.effect_colors.set_narrow(false);
            profile_actions.set_orientation(gtk::Orientation::Horizontal);
            buttons.set_orientation(gtk::Orientation::Horizontal);
        }
    });
    window.add_breakpoint(narrow);
    install_shortcuts(application, &window, &help, &new_profile, &save, &apply);
    delete_profile.connect_clicked({
        let window = window.clone();
        let store = store.clone();
        let profile_names = profile_names.clone();
        let profile_picker = profile_picker.clone();
        let editor = editor.clone();
        let toasts = toasts.clone();
        move |_| {
            let Some(name) = store
                .borrow()
                .profiles
                .get(profile_picker.selected() as usize)
                .map(|profile| profile.name.clone())
            else {
                return;
            };
            let dialog = adw::AlertDialog::new(
                Some("Delete saved profile?"),
                Some(&format!("“{name}” will be removed from this computer.")),
            );
            dialog.add_responses(&[("cancel", "Cancel"), ("delete", "Delete")]);
            dialog.set_close_response("cancel");
            dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
            dialog.choose(Some(&window), adw::gio::Cancellable::NONE, {
                let store = store.clone();
                let profile_names = profile_names.clone();
                let profile_picker = profile_picker.clone();
                let editor = editor.clone();
                let toasts = toasts.clone();
                move |response| {
                    if response != "delete" {
                        return;
                    }
                    match ProfileStore::update(|latest| latest.remove(&name)) {
                        Ok(updated) => {
                            *store.borrow_mut() = updated;
                            refresh_profile_picker(
                                &store.borrow(),
                                &profile_names,
                                &profile_picker,
                            );
                            reset_editor(&editor);
                            show_toast(&toasts, "Profile deleted");
                        }
                        Err(error) => {
                            show_toast(&toasts, &format!("Could not delete profile: {error}"))
                        }
                    }
                }
            });
        }
    });
    window.present();
}

fn show_toast(overlay: &adw::ToastOverlay, message: &str) {
    overlay.add_toast(adw::Toast::new(message));
}

fn show_help(window: &adw::ApplicationWindow) {
    let dialog = adw::AlertDialog::new(
        Some("How to use GameDAC Lighting"),
        Some(
            "1. Choose a saved profile or select New.\n\
             2. Pick a lighting effect and its colors.\n\
             3. Set the live and muted microphone colors.\n\
             4. Save keeps the profile; Apply sends it now.\n\n\
             Solid sets Left and Right independently. Color Flow transitions continuously between two colors. Color Pulse fades one to four colors through black. Together animates Left and Right in sync; Across alternates them and supports one color.\n\n\
             The Lighting enabled switch turns every zone off and restores the selected profile when turned back on.",
        ),
    );
    dialog.add_response("close", "Close");
    dialog.set_close_response("close");
    dialog.present(Some(window));
}

fn install_shortcuts(
    application: &adw::Application,
    window: &adw::ApplicationWindow,
    help: &gtk::Button,
    new_profile: &gtk::Button,
    save: &gtk::Button,
    apply: &gtk::Button,
) {
    for (name, accelerators, button) in [
        ("show-help", &["F1"][..], help),
        ("new-profile", &["<Primary>n"][..], new_profile),
        ("save-profile", &["<Primary>s"][..], save),
        ("apply-lighting", &["<Primary>Return"][..], apply),
    ] {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate({
            let button = button.clone();
            move |_, _| button.emit_clicked()
        });
        application.add_action(&action);
        application.set_accels_for_action(&format!("app.{name}"), accelerators);
    }
    let close = gio::SimpleAction::new("close", None);
    close.connect_activate({
        let window = window.clone();
        move |_, _| window.close()
    });
    application.add_action(&close);
    application.set_accels_for_action("app.close", &["<Primary>w"]);

    new_profile.update_property(&[gtk::accessible::Property::KeyShortcuts("Ctrl+N")]);
    help.update_property(&[gtk::accessible::Property::KeyShortcuts("F1")]);
    save.update_property(&[gtk::accessible::Property::KeyShortcuts("Ctrl+S")]);
    apply.update_property(&[gtk::accessible::Property::KeyShortcuts("Ctrl+Enter")]);
}

fn build_device_group(page: &adw::PreferencesPage) -> adw::ActionRow {
    let group = adw::PreferencesGroup::builder().title("Device").build();
    let status = adw::ActionRow::builder()
        .title("Original GameDAC")
        .subtitle("Checking device…")
        .build();
    status.add_prefix(&gtk::Image::from_icon_name("audio-headphones-symbolic"));
    group.add(&status);
    page.add(&group);
    status
}

fn build_editor() -> Editor {
    let profile_name = entry("Everyday");
    let profile_icon = entry("");
    profile_icon.set_max_length(8);
    profile_icon.set_width_chars(8);
    profile_icon.set_max_width_chars(12);
    profile_icon.update_property(&[
        gtk::accessible::Property::Label("Profile icon"),
        gtk::accessible::Property::Description(
            "Optional emoji, symbol, or glyph containing up to eight characters",
        ),
    ]);
    let profile_icon_widget = profile_icon_input(&profile_icon);
    let style = gtk::DropDown::from_strings(&["Solid", "Color Flow", "Color Pulse"]);
    let relationship = gtk::DropDown::from_strings(&["Together", "Across"]);

    let left = ColorInput::new("Left", "#FF3700");
    let right = ColorInput::new("Right", "#0084FF");
    let microphone_live = ColorInput::new("Microphone live", "#00FF00");
    let microphone_muted = ColorInput::new("Microphone muted", "#FF0000");
    let static_group = adw::PreferencesGroup::builder()
        .title("Effect colors")
        .build();
    static_group.add(&preference_row("Left", None, &left.widget));
    static_group.add(&preference_row("Right", None, &right.widget));
    let microphone_group = adw::PreferencesGroup::builder()
        .title("Microphone colors")
        .build();
    microphone_group.add(&preference_row(
        "Microphone live",
        None,
        &microphone_live.widget,
    ));
    microphone_group.add(&preference_row(
        "Microphone muted",
        None,
        &microphone_muted.widget,
    ));

    let effect_colors = PaletteEditor::new();
    let seconds = gtk::SpinButton::with_range(1.0, 30.0, 1.0);
    seconds.set_value(10.0);
    let reverse = gtk::Switch::new();
    reverse.set_valign(gtk::Align::Center);
    let animation_group = adw::PreferencesGroup::builder()
        .title("Animated colors")
        .build();
    animation_group.add(&preference_row(
        "Color sequence",
        None,
        &effect_colors.widget,
    ));
    animation_group.add(&preference_row("Duration", Some("1–30 seconds."), &seconds));
    animation_group.add(&preference_row(
        "Timing",
        Some("Together or alternating."),
        &relationship,
    ));
    animation_group.add(&preference_row(
        "Reverse sequence flag",
        Some("Protocol option; its visible effect is subtle."),
        &reverse,
    ));

    Editor {
        profile_name,
        profile_icon,
        profile_icon_widget,
        style,
        relationship,
        left,
        right,
        microphone_live,
        microphone_muted,
        effect_colors,
        seconds,
        reverse,
        static_group,
        microphone_group,
        animation_group,
    }
}

fn profile_icon_input(entry: &gtk::Entry) -> gtk::Box {
    let picker = gtk::MenuButton::builder()
        .icon_name("face-smile-symbolic")
        .tooltip_text("Choose a profile symbol")
        .build();
    picker.update_property(&[gtk::accessible::Property::Label("Choose a profile symbol")]);

    let popover = gtk::Popover::new();
    let grid = gtk::Grid::builder()
        .column_spacing(6)
        .row_spacing(6)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    for (index, symbol) in ["🎧", "🎮", "🌈", "🔥", "🌙", "☀️", "💜", "✨"]
        .into_iter()
        .enumerate()
    {
        let button = gtk::Button::with_label(symbol);
        button.update_property(&[gtk::accessible::Property::Label(&format!(
            "Use {symbol} as profile icon"
        ))]);
        button.connect_clicked({
            let entry = entry.clone();
            let popover = popover.clone();
            move |_| {
                entry.set_text(symbol);
                popover.popdown();
            }
        });
        grid.attach(&button, (index % 4) as i32, (index / 4) as i32, 1, 1);
    }
    let clear = gtk::Button::with_label("No icon");
    clear.set_hexpand(true);
    clear.connect_clicked({
        let entry = entry.clone();
        let popover = popover.clone();
        move |_| {
            entry.set_text("");
            popover.popdown();
        }
    });
    grid.attach(&clear, 0, 2, 4, 1);
    popover.set_child(Some(&grid));
    picker.set_popover(Some(&popover));

    let widget = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    widget.append(entry);
    widget.append(&picker);
    widget
}

fn entry(initial: &str) -> gtk::Entry {
    gtk::Entry::builder().text(initial).hexpand(true).build()
}

fn preference_row(
    title: &str,
    subtitle: Option<&str>,
    widget: &impl IsA<gtk::Widget>,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle.unwrap_or(""))
        .build();
    row.add_suffix(widget);
    row.set_activatable_widget(Some(widget));
    row
}

fn update_effect_visibility(editor: &Editor) {
    let is_steady = editor.style.selected() == 0;
    let is_multi_color_breathe = editor.style.selected() == 2;
    editor.static_group.set_visible(is_steady);
    editor.animation_group.set_visible(!is_steady);
    if editor.style.selected() == 1 {
        editor.effect_colors.configure(2, 2);
    } else if is_multi_color_breathe {
        editor.effect_colors.configure(1, 4);
    }
    editor.relationship.set_sensitive(is_multi_color_breathe);
    editor
        .reverse
        .set_sensitive(is_multi_color_breathe && editor.relationship.selected() == 1);
    if !is_multi_color_breathe || editor.relationship.selected() != 1 {
        editor.reverse.set_active(false);
    }
}

fn profile_from_editor(editor: &Editor) -> Result<Profile, String> {
    let name = editor.profile_name.text().trim().to_owned();
    let icon = match editor.profile_icon.text().trim() {
        "" => None,
        icon => Some(icon.to_owned()),
    };
    let lighting = match editor.style.selected() {
        0 => ProfileLighting::Static {
            left: editor.left.parse("Left")?,
            right: editor.right.parse("Right")?,
            microphone_live: editor.microphone_live.parse("Microphone live")?,
            microphone_muted: editor.microphone_muted.parse("Microphone muted")?,
        },
        1 => ProfileLighting::ColorShift {
            colors: editor.effect_colors.colors()?,
            seconds: editor.seconds.value_as_int() as u16,
        },
        2 => ProfileLighting::MultiColorBreathe {
            colors: editor.effect_colors.colors()?,
            seconds: editor.seconds.value_as_int() as u16,
            mode: if editor.relationship.selected() == 0 {
                ProfileBreatheMode::Synchronized
            } else {
                ProfileBreatheMode::Sweep
            },
            reverse: editor.reverse.is_active(),
        },
        _ => return Err("Unsupported lighting style".to_owned()),
    };
    let profile = Profile {
        name,
        icon,
        microphone: Some(MicrophoneLighting {
            live: editor.microphone_live.parse("Microphone live")?,
            muted: editor.microphone_muted.parse("Microphone muted")?,
        }),
        lighting,
    };
    profile.plan().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn populate_editor(editor: &Editor, profile: &Profile) {
    editor.profile_name.set_text(&profile.name);
    editor
        .profile_icon
        .set_text(profile.icon.as_deref().unwrap_or(""));
    if let Some(microphone) = profile.microphone {
        editor
            .microphone_live
            .set_text(&microphone.live.to_string());
        editor
            .microphone_muted
            .set_text(&microphone.muted.to_string());
    } else {
        editor.microphone_live.set_text("#00FF00");
        editor.microphone_muted.set_text("#FF0000");
    }
    match &profile.lighting {
        ProfileLighting::Static {
            left,
            right,
            microphone_live,
            microphone_muted,
        } => {
            editor.style.set_selected(0);
            editor.left.set_text(&left.to_string());
            editor.right.set_text(&right.to_string());
            editor
                .microphone_live
                .set_text(&microphone_live.to_string());
            editor
                .microphone_muted
                .set_text(&microphone_muted.to_string());
        }
        ProfileLighting::Breathe {
            color,
            seconds,
            mode,
            reverse,
        } => {
            editor.style.set_selected(2);
            editor.effect_colors.configure(1, 4);
            editor.relationship.set_selected(match mode {
                ProfileBreatheMode::Synchronized => 0,
                ProfileBreatheMode::Sweep => 1,
            });
            editor.effect_colors.set_colors(&[*color]);
            editor.seconds.set_value(f64::from(*seconds));
            editor.reverse.set_active(*reverse);
        }
        ProfileLighting::ColorShift { colors, seconds } => {
            editor.style.set_selected(1);
            editor.effect_colors.configure(2, 2);
            editor.effect_colors.set_colors(colors);
            editor.seconds.set_value(f64::from(*seconds));
            editor.relationship.set_selected(0);
            editor.reverse.set_active(false);
        }
        ProfileLighting::MultiColorBreathe {
            colors,
            seconds,
            mode,
            reverse,
        } => {
            editor.style.set_selected(2);
            editor.effect_colors.configure(1, 4);
            editor.effect_colors.set_colors(colors);
            editor.relationship.set_selected(match mode {
                ProfileBreatheMode::Synchronized => 0,
                ProfileBreatheMode::Sweep => 1,
            });
            editor.seconds.set_value(f64::from(*seconds));
            editor.reverse.set_active(*reverse);
        }
    }
    update_effect_visibility(editor);
}

fn reset_editor(editor: &Editor) {
    editor.profile_name.set_text("");
    editor.profile_icon.set_text("");
    editor.style.set_selected(0);
    editor.left.set_text("#FF3700");
    editor.right.set_text("#0084FF");
    editor.microphone_live.set_text("#00FF00");
    editor.microphone_muted.set_text("#FF0000");
    update_effect_visibility(editor);
}

fn refresh_profile_picker(store: &ProfileStore, names: &gtk::StringList, picker: &gtk::DropDown) {
    names.splice(0, names.n_items(), &[]);
    for profile in &store.profiles {
        names.append(&profile.name);
    }
    if let Some(selected) = &store.last_selected
        && let Some(index) = store
            .profiles
            .iter()
            .position(|profile| &profile.name == selected)
    {
        picker.set_selected(index as u32);
    } else {
        picker.set_selected(gtk::INVALID_LIST_POSITION);
    }
}

fn apply_profile(profile: Profile) -> Result<(), String> {
    let plan = profile.plan().map_err(|error| error.to_string())?;
    apply_plan(&plan)
}

fn apply_master_state(store: &ProfileStore, enabled: bool) -> Result<(), String> {
    let plan = if enabled {
        store
            .selected()
            .ok_or_else(|| "Select and save a profile before turning lighting back on.".to_owned())?
            .plan()
            .map_err(|error| error.to_string())?
    } else {
        LightingPlan::steady([
            (Zone::Left, Color::BLACK),
            (Zone::Right, Color::BLACK),
            (Zone::MicrophoneLive, Color::BLACK),
            (Zone::MicrophoneMuted, Color::BLACK),
        ])
        .map_err(|error| error.to_string())?
    };
    apply_plan(&plan)
}

fn apply_plan(plan: &LightingPlan) -> Result<(), String> {
    let transport = HidTransport::open().map_err(|error| transport_status(&error))?;
    transport
        .execute(plan)
        .map_err(|error| transport_status(&error))
}

fn transport_status(error: &TransportError) -> String {
    match error {
        TransportError::NotFound => "Disconnected".to_owned(),
        TransportError::Open(_) => "Permission denied; check the scoped udev rule".to_owned(),
        TransportError::Initialization(_) => "Could not initialize HID access".to_owned(),
        TransportError::Feature(_) | TransportError::Output(_) | TransportError::Input(_) => {
            format!("Write failed: {error}")
        }
    }
}

fn monitor_reconnect(
    store: Rc<std::cell::RefCell<ProfileStore>>,
    status: adw::ActionRow,
    initial_device: Option<String>,
) {
    let previous_device = Rc::new(std::cell::RefCell::new(initial_device));
    let reconnect_generation = Rc::new(Cell::new(0_u64));
    let permission_failures = Rc::new(Cell::new(0_u8));
    glib::timeout_add_local(Duration::from_millis(500), move || {
        match HidTransport::open() {
            Ok(transport) => {
                permission_failures.set(0);
                let current_device = transport.path().to_owned();
                let reconnected = previous_device.borrow().as_deref() != Some(&current_device);
                *previous_device.borrow_mut() = Some(current_device);
                if reconnected {
                    let generation = reconnect_generation.get().wrapping_add(1);
                    reconnect_generation.set(generation);
                    if store.borrow().apply_on_reconnect
                        && (!store.borrow().lighting_enabled || store.borrow().selected().is_some())
                    {
                        status
                            .set_subtitle("Reconnected; waiting for the GameDAC to become ready…");
                        let store = store.clone();
                        let status = status.clone();
                        let reconnect_generation = reconnect_generation.clone();
                        glib::timeout_add_local_once(Duration::from_secs(4), move || {
                            if reconnect_generation.get() != generation {
                                return;
                            }
                            let reconnect_state = {
                                let store = store.borrow();
                                store.apply_on_reconnect.then(|| store.clone())
                            };
                            match reconnect_state
                                .as_ref()
                                .map(|store| apply_master_state(store, store.lighting_enabled))
                            {
                                Some(Ok(())) => {
                                    status.set_subtitle(
                                        if reconnect_state
                                            .as_ref()
                                            .is_some_and(|store| store.lighting_enabled)
                                        {
                                            "Saved profile reports sent after reconnect"
                                        } else {
                                            "Lighting-off state restored after reconnect"
                                        },
                                    );
                                }
                                Some(Err(error)) => {
                                    status
                                        .set_subtitle(&format!("Reconnect apply failed: {error}"));
                                }
                                None => status.set_subtitle("Ready"),
                            }
                        });
                    } else {
                        status.set_subtitle("Ready");
                    }
                }
            }
            Err(error) => {
                let was_connected = previous_device.borrow_mut().take().is_some();
                if was_connected {
                    reconnect_generation.set(reconnect_generation.get().wrapping_add(1));
                }
                match error {
                    TransportError::Open(_) => {
                        let failures = permission_failures.get().saturating_add(1);
                        permission_failures.set(failures);
                        if failures < 10 {
                            status.set_subtitle("Reconnecting; waiting for device access…");
                        } else {
                            status.set_subtitle(&transport_status(&error));
                        }
                    }
                    TransportError::NotFound => {
                        permission_failures.set(0);
                        status.set_subtitle("Disconnected");
                    }
                    _ => {
                        permission_failures.set(0);
                        status.set_subtitle(&transport_status(&error));
                    }
                }
            }
        }
        glib::ControlFlow::Continue
    });
}
