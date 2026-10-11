#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod data;
mod pin;
mod probe;
use ferrite_design::prelude::*;
use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, KeyBinding, Render, Subscription, Task,
    Window, WindowControlArea, WindowHandle, WindowKind, div, px, size,
};
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Clock,
    Cpu,
    Playing,
}
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Clock => "Clock",
            Self::Cpu => "CPU",
            Self::Playing => "Now playing",
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::Clock => "relay-clock",
            Self::Cpu => "relay-cpu",
            Self::Playing => "relay-playing",
        }
    }
}
const KINDS: [Kind; 3] = [Kind::Clock, Kind::Cpu, Kind::Playing];
struct Widget {
    kind: Kind,
    cpu: data::Cpu,
    history: Vec<f32>,
    playing: Option<data::Playing>,
    _tick: Task<()>,
}
impl Widget {
    fn new(kind: Kind, cx: &mut Context<Self>) -> Self {
        let mut cpu = data::Cpu::new();
        cpu.sample();
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                if this
                    .update(cx, |s, cx| {
                        if s.kind == Kind::Cpu {
                            s.cpu.sample();
                            if let Some(usage) = s.cpu.usage {
                                s.history.push(usage);
                                if s.history.len() > 60 {
                                    s.history.remove(0);
                                }
                            }
                        }
                        if s.kind == Kind::Playing {
                            s.playing = data::playing();
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            kind,
            cpu,
            history: Vec::new(),
            playing: data::playing(),
            _tick: tick,
        }
    }
}
impl Render for Widget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = palette(cx);
        let heading = div()
            .flex()
            .items_center()
            .justify_between()
            .bg(hsla(p.surface))
            .pl_3()
            .child(
                div()
                    .flex_1()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .display(Scale::X1, window)
                    .child(self.kind.name().to_uppercase()),
            )
            .child(
                Button::new("close-widget")
                    .label("×")
                    .ghost()
                    .small()
                    .on_click(|_, w, _| w.remove_window()),
            );
        let body = match self.kind {
            Kind::Clock => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .display(Scale::X2, window)
                        .text_color(hsla(p.accent_text))
                        .child(chrono::Local::now().format("%H:%M:%S").to_string()),
                )
                .child(chrono::Local::now().format("%A · %d %B %Y").to_string()),
            Kind::Cpu => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .display(Scale::X2, window)
                        .text_color(hsla(p.accent_text))
                        .child(
                            self.cpu
                                .usage
                                .map(|v| format!("CPU {v:.0}%"))
                                .unwrap_or_else(|| "SAMPLING".into()),
                        ),
                )
                .child(
                    ascii_gauge(self.cpu.usage.unwrap_or(0.) / 100.)
                        .cells(24)
                        .label("LOAD"),
                )
                .child(line_chart("cpu-history", self.history.clone()).height(px(54.)))
                .child("System CPU · last 60 seconds"),
            Kind::Playing => {
                let (station, track) = self
                    .playing
                    .as_ref()
                    .map(|p| {
                        (
                            p.station.clone(),
                            if p.track.is_empty() {
                                "Live radio".into()
                            } else {
                                p.track.clone()
                            },
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            "WIRELESS INACTIVE".into(),
                            "Tune a station in Wireless to show its track here.".into(),
                        )
                    });
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .display(Scale::X1, window)
                            .text_color(hsla(p.accent_text))
                            .child(station.to_uppercase()),
                    )
                    .child(track)
            }
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(hsla(p.bg))
            .text_color(hsla(p.fg))
            .body(text::BASE)
            .border_1()
            .border_color(hsla(p.line_strong))
            .child(heading)
            .child(div().flex_1().min_h_0().overflow_hidden().p_3().child(body))
    }
}
fn open_widget(kind: Kind, cx: &mut App) -> Result<WindowHandle<Widget>, String> {
    let height = if kind == Kind::Cpu { 230. } else { 155. };
    let mut options = chrome::remembered_window_options(
        kind.key(),
        format!("Relay {}", kind.name()),
        size(px(400.), px(height)),
        cx,
    );
    options.titlebar = None;
    options.kind = WindowKind::PopUp;
    options.focus = false;
    options.is_resizable = false;
    options.window_min_size = Some(size(px(300.), px(130.)));
    cx.open_window(options, move |w, cx| {
        w.set_window_title(&format!("Relay {}", kind.name()));
        chrome::square_corners(w);
        pin::pin(w);
        chrome::remember_window(kind.key(), w, cx);
        cx.new(|cx| Widget::new(kind, cx))
    })
    .map_err(|e| e.to_string())
}
fn settings_path() -> std::path::PathBuf {
    std::env::var_os(if cfg!(windows) { "APPDATA" } else { "HOME" })
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("ferrite/relay-widgets.json")
}
fn saved() -> [bool; 3] {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or([false; 3])
}
fn save(state: [bool; 3]) {
    let path = settings_path();
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Ok(json) = serde_json::to_vec(&state) {
        let _ = std::fs::write(path, json);
    }
}
struct Relay {
    widgets: [Option<WindowHandle<Widget>>; 3],
    palette: Entity<CommandPalette>,
    _appearance: Subscription,
    _tick: Task<()>,
    status: String,
}
impl Relay {
    fn new(w: &mut Window, cx: &mut Context<Self>) -> Self {
        let palette = cx.new(|cx| CommandPalette::new(w, cx));
        let weak = cx.weak_entity();
        palette.update(cx, |p, cx| {
            let mut commands = Vec::new();
            for scheme in SCHEMES {
                commands.push(
                    command(format!("Scheme: {}", scheme.name))
                        .group("Theme")
                        .on_run(move |_, cx| theme::set_scheme(scheme, cx)),
                );
            }
            commands.push(
                command("Toggle light / dark")
                    .group("Theme")
                    .on_run(|w, cx| {
                        theme::set_appearance(
                            if theme::appearance(cx) == Appearance::Light {
                                Appearance::Dark
                            } else {
                                Appearance::Light
                            },
                            w,
                            cx,
                        )
                    }),
            );
            for (i, kind) in KINDS.iter().enumerate() {
                let weak = weak.clone();
                commands.push(
                    command(format!("Toggle {} panel", kind.name()))
                        .group("Relay")
                        .on_run(move |_, cx| {
                            let _ = weak.update(cx, |s, cx| s.toggle(i, cx));
                        }),
                );
            }
            p.set_commands(commands, cx);
        });
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                if this
                    .update(cx, |s, cx| {
                        let mut changed = false;
                        for widget in &mut s.widgets {
                            if widget.is_some_and(|h| h.update(cx, |_, _, _| ()).is_err()) {
                                *widget = None;
                                changed = true;
                            }
                        }
                        if changed {
                            save(s.widgets.map(|w| w.is_some()));
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let enabled = if std::env::args().any(|a| a == "--widgets") {
            [true; 3]
        } else {
            saved()
        };
        let mut widgets = [None; 3];
        for i in 0..3 {
            if enabled[i] {
                widgets[i] = open_widget(KINDS[i], cx).ok();
            }
        }
        Self{widgets,palette,_appearance:theme::follow_system(w),_tick:tick,status:"Drag a panel by its heading. Close it with ×. Positions and pinned panels are remembered.".into()}
    }
    fn toggle(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(handle) = self.widgets[i].take() {
            let _ = handle.update(cx, |_, w, _| w.remove_window());
        } else {
            match open_widget(KINDS[i], cx) {
                Ok(handle) => self.widgets[i] = Some(handle),
                Err(e) => self.status = e,
            };
        }
        save(self.widgets.map(|w| w.is_some()));
        cx.notify();
    }
}
impl Render for Relay {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = palette(cx);
        probe::record(format!(
            "Relay ready · {} pinned panels",
            self.widgets.iter().filter(|w| w.is_some()).count()
        ));
        let menu = Button::new("commands")
            .icon(Icon::Menu)
            .ghost()
            .small()
            .on_click(cx.listener(|s, _, w, cx| s.palette.update(cx, |p, cx| p.toggle(w, cx))));
        let mut panels = div().flex().flex_col().gap_3();
        for (i, kind) in KINDS.iter().enumerate() {
            panels=panels.child(panel(kind.name()).meta(if self.widgets[i].is_some(){"PINNED"}else{"CLOSED"})
            .child(match kind{Kind::Clock=>"Local time and date.",Kind::Cpu=>"Live system CPU load and a one-minute history.",Kind::Playing=>"Wireless station and track. Stale snapshots disappear after ten seconds."})
            .child(Button::new(("pin",i)).label(if self.widgets[i].is_some(){"Close panel"}else{"Pin panel"}).secondary().on_click(cx.listener(move|s,_,_,cx|s.toggle(i,cx)))));
        }
        window_frame().child(power_on_in(
            "power",
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(hsla(p.bg))
                .text_color(hsla(p.fg))
                .body(text::BASE)
                .on_action(cx.listener(|s, _: &TogglePalette, w, cx| {
                    s.palette.update(cx, |p, cx| p.toggle(w, cx))
                }))
                .child(self.palette.clone())
                .child(title_bar("Relay").child(menu))
                .child(
                    scroll_area("panels").flex_1().min_h_0().child(
                        div()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(div().display(Scale::X2, w).child("DESKTOP PANELS"))
                            .child(self.status.clone())
                            .child(panels),
                    ),
                )
                .child(
                    status_bar()
                        .left("FRAMELESS · ALWAYS ON TOP")
                        .right(theme::scheme(cx).name.to_uppercase()),
                ),
        ))
    }
}
fn main() {
    if std::env::args().any(|a| a == "--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    if std::env::args().any(|a| a == "--smoke") {
        let mut cpu = data::Cpu::new();
        cpu.sample();
        std::thread::sleep(std::time::Duration::from_millis(200));
        cpu.sample();
        if cfg!(windows) && cpu.usage.is_none() {
            eprintln!("CPU sample unavailable");
            std::process::exit(1);
        }
        println!("Relay CPU sample: {:?}", cpu.usage);
        return;
    }
    gpui_platform::application().run(|cx: &mut App| {
        ferrite_design::init(Appearance::Dark, cx);
        theme::apply_env(cx);
        cx.bind_keys([KeyBinding::new("ctrl-shift-p", TogglePalette, None)]);
        let options =
            chrome::remembered_window_options("relay", "Relay", size(px(700.), px(700.)), cx);
        cx.open_window(options, |w, cx| {
            chrome::square_corners(w);
            chrome::remember_window("relay", w, cx);
            chrome::power_off_on_close(w, cx);
            cx.new(|cx| Relay::new(w, cx))
        })
        .expect("opening Relay");
        cx.activate(true);
    });
}
