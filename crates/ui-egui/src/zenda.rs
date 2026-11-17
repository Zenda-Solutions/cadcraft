//! Zenda patches for Work. Events carry metadata only; Work owns accounts and Theo.
use serde_json::{Value, json};

pub const ON: bool = cfg!(feature = "zenda");

/// Events kept for Work when it does not drain them; older ones are dropped first.
const OUTBOX_MAX: usize = 32;

/// Work's orange (#EA6C1E).
pub const ORANGE: egui::Color32 = egui::Color32::from_rgb(234, 108, 30);

/// Width the Theo button takes at the right end of the tool bar.
pub const BUTTON_WIDTH: f32 = 74.0;

/// What the person asked of Work, oldest first, until Work drains it (`work.events`).
#[derive(Default, Debug)]
pub struct Outbox {
    events: Vec<Value>,
    seq: u64,
}

impl Outbox {
    /// Queue `event` (`work.theo`, `work.share`) with its params.
    pub fn push(&mut self, event: &str, params: Value) {
        self.seq = self.seq.saturating_add(1);
        if self.events.len() == OUTBOX_MAX {
            self.events.remove(0);
        }
        self.events.push(json!({"seq": self.seq, "event": event, "params": params}));
    }

    /// Every queued event, oldest first; the outbox is empty afterwards.
    pub fn drain(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Where help was requested, without drawing content.
pub fn theo_context(app: &crate::CadApp) -> Value {
    json!({"from": "button", "hasDocument": app.session.state().is_ok() && !app.ui.start_tab})
}

/// The saved drawing, including whether its disk copy is stale.
pub fn share_params(app: &crate::CadApp) -> Value {
    let doc = app.session.state().ok();
    json!({"path": doc.and_then(|d| d.path.as_deref()), "dirty": doc.is_some_and(|d| d.is_dirty()), "name": doc.map(|d| d.title.as_str())})
}

/// Work's mascot and orange brand button, shared by the curated app patches.
pub fn theo_button(ui: &mut egui::Ui, label: &str, hint: &str) -> egui::Response {
    let key = egui::Id::new("zenda.theo.texture");
    let texture = ui.ctx().data_mut(|d| d.get_temp::<egui::TextureHandle>(key)).or_else(|| {
        let image = image::load_from_memory(include_bytes!("../../../assets/zenda/theo-icon-64.png")).ok()?.to_rgba8();
        let color = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
        let texture = ui.ctx().load_texture("Theo mascot", color, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(key, texture.clone()));
        Some(texture)
    });
    let text = egui::RichText::new("Theo").color(egui::Color32::BLACK).size(11.5);
    let button = if let Some(texture) = texture {
        egui::Button::image_and_text(egui::Image::new((texture.id(), egui::vec2(18.0, 18.0))), text)
    } else {
        egui::Button::new(text)
    };
    let response = ui.add(button.fill(ORANGE).corner_radius(11.0).min_size(egui::vec2(BUTTON_WIDTH, 24.0)));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    response.on_hover_text(hint)
}

/// The Theo button placed in `rect` of the tool bar; a press queues `work.theo`.
pub fn toolbar_theo(app: &mut crate::CadApp, ui: &mut egui::Ui, rect: egui::Rect) {
    let label = crate::tl!("Ask Theo for help");
    let hint = crate::tl!("Ask Theo about what is on screen");
    let pressed = ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| theo_button(ui, label, hint).clicked()).inner;
    if pressed {
        // With Work's chat flag on, the chat opens here instead (zenda_chat.rs).
        #[cfg(feature = "zenda")]
        if crate::zenda_chat::toggle(app) {
            return;
        }
        app.zenda.push("work.theo", theo_context(app));
    }
}

/// Share in the Zenda build: Work sends the file; the tool only says which one.
/// Returns whether the command was taken here.
pub fn intercept(app: &mut crate::CadApp, name: &str) -> bool {
    if !ON || !name.eq_ignore_ascii_case("share") {
        return false;
    }
    let params = share_params(app);
    app.zenda.push("work.share", params);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlRequest, Outcome};

    fn app() -> crate::CadApp {
        crate::CadApp::new(cadcraft_engine::Session::empty(), crate::Services::default())
    }

    #[test]
    fn zenda_events_drain_in_order_and_are_bounded() {
        let mut out = Outbox::default();
        for _ in 0..40 {
            out.push("work.theo", json!({"from": "button"}));
        }
        let got = out.drain();
        assert_eq!(got.len(), 32);
        assert_eq!(got[0]["seq"], 9);
        assert!(out.is_empty());
    }

    #[test]
    fn zenda_only_the_zenda_build_exposes_work_events() {
        let mut app = app();
        app.zenda.push("work.theo", theo_context(&app));
        let (req, _pending) = ControlRequest::new("work.events", json!({}));
        let Outcome::Done(reply) = crate::control::handle(&mut app, &egui::Context::default(), &req) else { panic!("immediate reply") };
        assert_eq!(reply["ok"], ON);
        if ON {
            assert_eq!(reply["result"]["events"][0]["event"], "work.theo");
            assert!(app.zenda.is_empty());
        }
    }

    #[test]
    fn zenda_share_goes_to_work_only_in_the_zenda_build() {
        let mut app = app();
        app.start("share");
        if ON {
            let got = app.zenda.drain();
            assert_eq!(got.len(), 1);
            assert_eq!(got[0]["event"], "work.share");
            assert!(got[0]["params"].get("dirty").is_some());
        } else {
            assert!(app.zenda.is_empty());
        }
    }

    #[test]
    fn zenda_theo_button_has_localized_accessible_names() {
        use egui_kittest::kittest::Queryable;
        for (language, label) in [("en", "Ask Theo for help"), ("es", "Pedile ayuda a Theo"), ("pt-BR", "Peça ajuda ao Theo")] {
            let mut h = egui_kittest::Harness::builder().build_ui_state(
                |ui, count: &mut usize| {
                    crate::i18n::with_language(language, || {
                        if theo_button(ui, crate::tl!("Ask Theo for help"), "Theo").clicked() {
                            *count += 1;
                        }
                    });
                },
                0usize,
            );
            h.get_by_label(label).click();
            h.run_steps(2);
            assert_eq!(*h.state(), 1, "{language}");
        }
        let image = image::load_from_memory(include_bytes!("../../../assets/zenda/theo-icon-64.png")).unwrap();
        assert_eq!((image.width(), image.height()), (64, 64));
    }

    #[test]
    fn zenda_tool_bar_theo_reaches_work() {
        use egui_kittest::kittest::Queryable;
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut crate::CadApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app(),
        );
        h.run_steps(8);
        if !ON {
            assert!(h.query_by_label("Ask Theo for help").is_none());
            return;
        }
        h.get_by_label("Ask Theo for help").click();
        h.run_steps(2);
        let events = h.state_mut().zenda.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event"], "work.theo");
    }

    #[test]
    fn zenda_about_names_the_base_app_without_artcraft_links() {
        let source = include_str!("dialogs.rs");
        assert!(source.contains("Based on CADCraft, by the ArtCraft team."));
    }

    #[test]
    fn zenda_theo_mascot_preview() {
        let Some(path) = std::env::var_os("ZENDA_THEO_PREVIEW") else { return };
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(160.0, 50.0)).with_pixels_per_point(3.0).wgpu().build_ui(|ui| {
            let _ = theo_button(ui, "Ask Theo for help", "Theo");
        });
        h.run_steps(2);
        h.render().unwrap().save(path).unwrap();
    }
}
