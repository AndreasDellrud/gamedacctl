use std::{cell::Cell, rc::Rc, str::FromStr, time::Duration};

use adw::prelude::*;
use gamedacctl::{
    Color,
    profile::{Profile, ProfileBreatheMode, ProfileLighting, ProfileStore},
    transport::{HidTransport, Transport, TransportError},
};
use gtk::{gio, glib};

const APPLICATION_ID: &str = "io.github.andreasdellrud.gamedacctl";

#[derive(Clone)]
struct Editor {
    profile_name: gtk::Entry,
    profile_icon: gtk::Entry,
    style: gtk::DropDown,
    relationship: gtk::DropDown,
    left: gtk::Entry,
    right: gtk::Entry,
    microphone_live: gtk::Entry,
    microphone_muted: gtk::Entry,
    breathe_color: gtk::Entry,
    seconds: gtk::SpinButton,
    reverse: gtk::Switch,
    static_group: adw::PreferencesGroup,
    breathe_group: adw::PreferencesGroup,
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
    let store = Rc::new(std::cell::RefCell::new(
        ProfileStore::load().unwrap_or_else(|error| {
            eprintln!("gamedacctl-gui: {error}");
            ProfileStore::default()
        }),
    ));

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        "gamedacctl",
        "Original GameDAC lighting",
    )));

    let page = adw::PreferencesPage::new();
    let status = build_device_group(&page);

    let profile_names = gtk::StringList::new(&[]);
    let profile_picker = gtk::DropDown::new(Some(profile_names.clone()), gtk::Expression::NONE);
    let editor = build_editor();

    let profile_group = adw::PreferencesGroup::builder()
        .title("Profiles")
        .description("Save lighting configurations locally and recall them later.")
        .build();
    profile_group.add(&preference_row("Saved profile", None, &profile_picker));
    profile_group.add(&preference_row("Profile name", None, &editor.profile_name));
    profile_group.add(&preference_row(
        "Profile icon",
        Some("Optional emoji or glyph shown by integrations."),
        &editor.profile_icon,
    ));
    page.add(&profile_group);

    let style_group = adw::PreferencesGroup::builder()
        .title("Lighting")
        .description("Only effects verified against captured GameDAC traffic are available.")
        .build();
    style_group.add(&preference_row("Style", None, &editor.style));
    page.add(&style_group);
    page.add(&editor.static_group);
    page.add(&editor.breathe_group);

    let reconnect = gtk::Switch::new();
    reconnect.set_active(store.borrow().apply_on_reconnect);
    let behavior_group = adw::PreferencesGroup::builder().title("Behavior").build();
    behavior_group.add(&preference_row(
        "Apply after reconnect",
        Some("Opt in to restoring the last selected saved profile."),
        &reconnect,
    ));
    page.add(&behavior_group);

    let save = gtk::Button::with_label("Save Profile");
    let apply = gtk::Button::with_label("Apply to GameDAC");
    apply.add_css_class("suggested-action");
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    buttons.append(&save);
    buttons.append(&apply);
    let action_group = adw::PreferencesGroup::new();
    action_group.add(&preference_row(
        "Apply lighting",
        Some("Success is shown only after every HID write completes."),
        &buttons,
    ));
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
        move |picker| {
            if let Some(profile) = store.borrow().profiles.get(picker.selected() as usize) {
                populate_editor(&editor, profile);
            }
        }
    });

    save.connect_clicked({
        let editor = editor.clone();
        let store = store.clone();
        let profile_names = profile_names.clone();
        let profile_picker = profile_picker.clone();
        let reconnect = reconnect.clone();
        let status = status.clone();
        move |_| match profile_from_editor(&editor) {
            Ok(profile) => {
                let name = profile.name.clone();
                let result = (|| {
                    let mut store = store.borrow_mut();
                    store.upsert(profile)?;
                    store.last_selected = Some(name);
                    store.apply_on_reconnect = reconnect.is_active();
                    store.save()
                })();
                match result {
                    Ok(()) => {
                        refresh_profile_picker(&store.borrow(), &profile_names, &profile_picker);
                        status.set_subtitle("Profile saved");
                    }
                    Err(error) => {
                        status.set_subtitle(&format!("Could not save profile: {error}"));
                    }
                }
            }
            Err(error) => status.set_subtitle(&error),
        }
    });

    reconnect.connect_active_notify({
        let store = store.clone();
        let status = status.clone();
        move |toggle| {
            let result = {
                let mut store = store.borrow_mut();
                store.apply_on_reconnect = toggle.is_active();
                store.save()
            };
            if let Err(error) = result {
                status.set_subtitle(&format!("Could not save reconnect policy: {error}"));
            }
        }
    });

    apply.connect_clicked({
        let editor = editor.clone();
        let status = status.clone();
        move |_| match profile_from_editor(&editor).and_then(apply_profile) {
            Ok(()) => status.set_subtitle("All lighting reports sent"),
            Err(error) => status.set_subtitle(&error),
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
    monitor_reconnect(store.clone(), status.clone(), initial_device);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_width(480)
        .child(&page)
        .build();
    toolbar.set_content(Some(&scrolled));

    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("gamedacctl")
        .default_width(620)
        .default_height(760)
        .content(&toolbar)
        .build();
    window.present();
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
    let style = gtk::DropDown::from_strings(&["Static colors", "Breathe"]);
    let relationship = gtk::DropDown::from_strings(&["Synchronized", "Sweep"]);

    let left = entry("FF3700");
    let right = entry("0084FF");
    let microphone_live = entry("00FF00");
    let microphone_muted = entry("FF0000");
    let static_group = adw::PreferencesGroup::builder()
        .title("Static colors")
        .description("Set each earcup and microphone state independently. Use 000000 for off.")
        .build();
    static_group.add(&preference_row("Left earcup", None, &left));
    static_group.add(&preference_row("Right earcup", None, &right));
    static_group.add(&preference_row("Microphone live", None, &microphone_live));
    static_group.add(&preference_row("Microphone muted", None, &microphone_muted));

    let breathe_color = entry("7A21E6");
    let seconds = gtk::SpinButton::with_range(1.0, 30.0, 1.0);
    seconds.set_value(10.0);
    let reverse = gtk::Switch::new();
    let breathe_group = adw::PreferencesGroup::builder()
        .title("Breathe")
        .description("A verified single-color effect across both connected earcups.")
        .build();
    breathe_group.add(&preference_row("Color", None, &breathe_color));
    breathe_group.add(&preference_row(
        "Duration",
        Some("Whole seconds from 1 through 30."),
        &seconds,
    ));
    breathe_group.add(&preference_row(
        "Connected behavior",
        Some("Synchronized pulses together; Sweep alternates earcups."),
        &relationship,
    ));
    breathe_group.add(&preference_row(
        "Engine reverse flag",
        Some("Captured from GG; visible direction is still under investigation."),
        &reverse,
    ));

    Editor {
        profile_name,
        profile_icon,
        style,
        relationship,
        left,
        right,
        microphone_live,
        microphone_muted,
        breathe_color,
        seconds,
        reverse,
        static_group,
        breathe_group,
    }
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
    let is_static = editor.style.selected() == 0;
    editor.static_group.set_visible(is_static);
    editor.breathe_group.set_visible(!is_static);
    editor
        .reverse
        .set_sensitive(editor.relationship.selected() == 1);
    if editor.relationship.selected() != 1 {
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
            left: parse_color("Left earcup", &editor.left)?,
            right: parse_color("Right earcup", &editor.right)?,
            microphone_live: parse_color("Microphone live", &editor.microphone_live)?,
            microphone_muted: parse_color("Microphone muted", &editor.microphone_muted)?,
        },
        1 => ProfileLighting::Breathe {
            color: parse_color("Effect", &editor.breathe_color)?,
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
        lighting,
    };
    profile.plan().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn parse_color(label: &str, entry: &gtk::Entry) -> Result<Color, String> {
    Color::from_str(entry.text().as_str()).map_err(|error| format!("{label}: {error}"))
}

fn populate_editor(editor: &Editor, profile: &Profile) {
    editor.profile_name.set_text(&profile.name);
    editor
        .profile_icon
        .set_text(profile.icon.as_deref().unwrap_or(""));
    match profile.lighting {
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
            editor.style.set_selected(1);
            editor.relationship.set_selected(match mode {
                ProfileBreatheMode::Synchronized => 0,
                ProfileBreatheMode::Sweep => 1,
            });
            editor.breathe_color.set_text(&color.to_string());
            editor.seconds.set_value(f64::from(seconds));
            editor.reverse.set_active(reverse);
        }
    }
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
    }
}

fn apply_profile(profile: Profile) -> Result<(), String> {
    let plan = profile.plan().map_err(|error| error.to_string())?;
    let transport = HidTransport::open().map_err(|error| transport_status(&error))?;
    transport
        .execute(&plan)
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
                    if store.borrow().apply_on_reconnect && store.borrow().selected().is_some() {
                        status
                            .set_subtitle("Reconnected; waiting for the GameDAC to become ready…");
                        let store = store.clone();
                        let status = status.clone();
                        let reconnect_generation = reconnect_generation.clone();
                        glib::timeout_add_local_once(Duration::from_secs(4), move || {
                            if reconnect_generation.get() != generation {
                                return;
                            }
                            let profile = {
                                let store = store.borrow();
                                store
                                    .apply_on_reconnect
                                    .then(|| store.selected().cloned())
                                    .flatten()
                            };
                            match profile.map(apply_profile) {
                                Some(Ok(())) => {
                                    status
                                        .set_subtitle("Saved profile reports sent after reconnect");
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
