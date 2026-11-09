//! Menu inicial do Notarium (design original).

use egui::*;

const BG: Color32 = Color32::from_rgb(15, 18, 32);
const SIDEBAR: Color32 = Color32::from_rgb(21, 25, 50);
const CARD: Color32 = Color32::from_rgb(27, 32, 64);
const CARD_HOVER: Color32 = Color32::from_rgb(35, 41, 80);
const ACCENT: Color32 = Color32::from_rgb(232, 176, 74);
const TEXT: Color32 = Color32::from_rgb(233, 234, 242);
const MUTED: Color32 = Color32::from_rgb(141, 146, 173);
const PAPER: Color32 = Color32::from_rgb(246, 243, 235);
const INK: Color32 = Color32::from_rgb(60, 62, 80);

pub fn apply_theme(ctx: &Context) {
    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.override_text_color = Some(TEXT);
    v.widgets.noninteractive.bg_fill = CARD;
    v.widgets.inactive.bg_fill = CARD;
    v.widgets.hovered.bg_fill = CARD_HOVER;
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.widgets.inactive.rounding = Rounding::same(8.0);
    v.widgets.hovered.rounding = Rounding::same(8.0);
    v.widgets.active.rounding = Rounding::same(8.0);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = vec2(12.0, 12.0);
        s.spacing.button_padding = vec2(14.0, 8.0);
    });
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Scores,
    Plugins,
    Learn,
    Account,
}

impl Page {
    const ALL: [(Page, &'static str); 4] = [
        (Page::Scores, "Partituras"),
        (Page::Plugins, "Complementos"),
        (Page::Learn, "Aprender"),
        (Page::Account, "Conta"),
    ];
}

struct Recent {
    title: &'static str,
    info: &'static str,
}

pub struct Home {
    page: Page,
    search: String,
    status: String,
    pub quit: bool,
    recents: Vec<Recent>,
}

impl Default for Home {
    fn default() -> Self {
        Self {
            page: Page::Scores,
            search: String::new(),
            status: String::new(),
            quit: false,
            recents: vec![
                Recent { title: "Sonata em Lá menor", info: "Piano · 4 páginas" },
                Recent { title: "Coral a quatro vozes", info: "SATB · 2 páginas" },
                Recent { title: "Estudo para violão", info: "Violão · 1 página" },
                Recent { title: "Quarteto de cordas", info: "Cordas · 6 páginas" },
            ],
        }
    }
}

impl Home {
    pub fn show(&mut self, ctx: &Context) {
        SidePanel::left("nav")
            .exact_width(232.0)
            .resizable(false)
            .frame(Frame::none().fill(SIDEBAR).inner_margin(Margin::same(18.0)))
            .show(ctx, |ui| self.sidebar(ui));

        CentralPanel::default()
            .frame(Frame::none().fill(BG).inner_margin(Margin::same(36.0)))
            .show(ctx, |ui| match self.page {
                Page::Scores => self.scores(ui),
                _ => self.placeholder(ui),
            });
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
            paint_logo(ui.painter(), rect);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.label(RichText::new("Notarium").size(21.0).strong());
                ui.label(RichText::new("Editor de partituras").size(11.5).color(MUTED));
            });
        });
        ui.add_space(28.0);

        for (page, name) in Page::ALL {
            let selected = self.page == page;
            let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
            let hover = resp.hovered();
            if selected || hover {
                let fill = if selected { CARD_HOVER } else { CARD };
                ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
            }
            if selected {
                let bar = Rect::from_min_size(rect.left_center() - vec2(0.0, 11.0), vec2(4.0, 22.0));
                ui.painter().rect_filled(bar, Rounding::same(2.0), ACCENT);
            }
            ui.painter().text(
                rect.left_center() + vec2(20.0, 0.0),
                Align2::LEFT_CENTER,
                name,
                FontId::proportional(15.0),
                if selected { TEXT } else { MUTED },
            );
            if resp.clicked() {
                self.page = page;
            }
            ui.add_space(-6.0);
        }

        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.label(RichText::new("v0.1.0 · DirectX 11").size(11.5).color(MUTED));
            ui.add_space(6.0);
            if fancy_button(ui, "Sair", false).clicked() {
                self.quit = true;
            }
        });
    }

    fn scores(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Bem-vindo ao Notarium").size(30.0).strong());
        ui.label(RichText::new("Comece uma nova partitura ou continue de onde parou.").size(14.5).color(MUTED));
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if fancy_button(ui, "+  Nova partitura", true).clicked() {
                self.status = "Nova partitura: o editor chega nas próximas etapas.".into();
            }
            if fancy_button(ui, "Abrir arquivo…", false).clicked() {
                self.status = "Abrir arquivo: leitor de MusicXML/.mscx em desenvolvimento.".into();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_sized(
                    [260.0, 34.0],
                    TextEdit::singleline(&mut self.search).hint_text("Buscar partituras…"),
                );
            });
        });

        ui.add_space(18.0);
        ui.label(RichText::new("Recentes").size(18.0).strong());
        ui.add_space(2.0);

        let q = self.search.to_lowercase();
        let mut clicked: Option<&'static str> = None;
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(20.0, 20.0);
                let mut any = false;
                for r in self.recents.iter().filter(|r| r.title.to_lowercase().contains(&q)) {
                    any = true;
                    if score_card(ui, r).clicked() {
                        clicked = Some(r.title);
                    }
                }
                if !any {
                    ui.label(RichText::new("Nenhuma partitura encontrada.").color(MUTED));
                }
            });
        });
        if let Some(t) = clicked {
            self.status = format!("Abrir “{t}”: ainda não implementado.");
        }

        if !self.status.is_empty() {
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.label(RichText::new(&self.status).color(ACCENT).size(13.0));
            });
        }
    }

    fn placeholder(&mut self, ui: &mut Ui) {
        let name = Page::ALL.iter().find(|(p, _)| *p == self.page).map(|x| x.1).unwrap_or("");
        ui.label(RichText::new(name).size(30.0).strong());
        ui.add_space(8.0);
        ui.label(RichText::new("Esta seção chega em uma versão futura.").size(14.5).color(MUTED));
    }
}

fn paint_logo(p: &Painter, rect: Rect) {
    p.rect_filled(rect, Rounding::same(11.0), ACCENT);
    let inner = rect.shrink(8.0);
    for i in 0..5 {
        let y = inner.top() + inner.height() * (i as f32) / 4.0;
        p.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.2_f32, Color32::from_rgb(70, 48, 8)));
    }
    let head = pos2(inner.center().x - 3.0, inner.bottom() - inner.height() * 0.25);
    p.circle_filled(head, 4.2, Color32::from_rgb(40, 26, 4));
    p.line_segment([head + vec2(3.8, 0.0), head + vec2(3.8, -inner.height() * 0.85)], Stroke::new(1.8_f32, Color32::from_rgb(40, 26, 4)));
}

fn fancy_button(ui: &mut Ui, text: &str, primary: bool) -> Response {
    let font = FontId::proportional(14.5);
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, Color32::WHITE);
    let size = vec2(galley.size().x + 36.0, 38.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hover = resp.hovered();
    let (fill, fg) = if primary {
        (if hover { ACCENT.gamma_multiply(1.12) } else { ACCENT }, Color32::from_rgb(35, 24, 4))
    } else {
        (if hover { CARD_HOVER } else { CARD }, TEXT)
    };
    ui.painter().rect_filled(rect, Rounding::same(10.0), fill);
    if !primary {
        ui.painter().rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.0_f32, Color32::from_white_alpha(18)));
    }
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(14.5), fg);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

fn score_card(ui: &mut Ui, r: &Recent) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(210.0, 250.0), Sense::click());
    let hover = resp.hovered();
    let t = ui.ctx().animate_bool_with_time(resp.id, hover, 0.12);
    let rect = rect.translate(vec2(0.0, -4.0 * t));
    let p = ui.painter();

    p.rect_filled(rect.translate(vec2(0.0, 6.0)), Rounding::same(14.0), Color32::from_black_alpha((40.0 + 40.0 * t) as u8));
    p.rect_filled(rect, Rounding::same(14.0), if hover { CARD_HOVER } else { CARD });
    if hover {
        p.rect_stroke(rect, Rounding::same(14.0), Stroke::new(1.5_f32, ACCENT));
    }

    // miniatura da "folha" com pautas
    let page = Rect::from_min_size(rect.min + vec2(18.0, 18.0), vec2(rect.width() - 36.0, 150.0));
    p.rect_filled(page, Rounding::same(6.0), PAPER);
    for s in 0..3 {
        let top = page.top() + 22.0 + s as f32 * 40.0;
        for l in 0..5 {
            let y = top + l as f32 * 4.5;
            p.line_segment([pos2(page.left() + 10.0, y), pos2(page.right() - 10.0, y)], Stroke::new(0.8_f32, INK));
        }
        p.line_segment([pos2(page.left() + 10.0, top), pos2(page.left() + 10.0, top + 18.0)], Stroke::new(1.2_f32, INK));
        for n in 0..5 {
            let x = page.left() + 34.0 + n as f32 * 24.0;
            let y = top + ((n * 7 + s * 3) % 5) as f32 * 4.5;
            p.circle_filled(pos2(x, y), 2.6, INK);
            p.line_segment([pos2(x + 2.4, y), pos2(x + 2.4, y - 13.0)], Stroke::new(0.9_f32, INK));
        }
    }

    p.text(rect.left_top() + vec2(18.0, 184.0), Align2::LEFT_TOP, r.title, FontId::proportional(15.5), TEXT);
    p.text(rect.left_top() + vec2(18.0, 208.0), Align2::LEFT_TOP, r.info, FontId::proportional(12.5), MUTED);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}
