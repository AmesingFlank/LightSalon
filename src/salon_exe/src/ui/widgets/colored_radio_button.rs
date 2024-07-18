use eframe::{
    egui::{Response, Sense, TextStyle, Ui, Widget, WidgetInfo, WidgetText, WidgetType},
    emath::NumExt,
    epaint::{self, pos2, vec2, Color32, Vec2},
};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ColoredRadioButton {
    checked: bool,
    text: WidgetText,
    base_color: Color32,
    checked_color: Color32,
}

impl ColoredRadioButton {
    pub fn new(
        checked: bool,
        text: impl Into<WidgetText>,
        base_color: Color32,
        checked_color: Color32,
    ) -> Self {
        Self {
            checked,
            text: text.into(),
            base_color,
            checked_color,
        }
    }
}

impl Widget for ColoredRadioButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let ColoredRadioButton {
            checked,
            text,
            base_color,
            checked_color,
        } = self;

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;
        let icon_spacing = spacing.icon_spacing;

        let (text, mut desired_size) = if text.is_empty() {
            (None, vec2(icon_width, 0.0))
        } else {
            let total_extra = vec2(icon_width + icon_spacing, 0.0);

            let wrap_width = ui.available_width() - total_extra.x;
            let text = text.into_galley(ui, None, wrap_width, TextStyle::Button);

            let mut desired_size = total_extra + text.size();
            desired_size = desired_size.at_least(spacing.interact_size);

            (Some(text), desired_size)
        };

        desired_size = desired_size.at_least(Vec2::splat(spacing.interact_size.y));
        desired_size.y = desired_size.y.max(icon_width);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::RadioButton,
                true,
                checked,
                text.as_ref().map_or("", |x| x.text()),
            )
        });

        if ui.is_rect_visible(rect) {
            // let visuals = ui.style().interact_selectable(&response, checked); // too colorful
            let visuals = ui.style().interact(&response);

            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);

            let painter = ui.painter();

            painter.add(epaint::CircleShape {
                center: big_icon_rect.center(),
                radius: big_icon_rect.width() / 2.0 + visuals.expansion,
                fill: base_color,
                stroke: visuals.bg_stroke,
            });

            if checked {
                painter.add(epaint::CircleShape {
                    center: small_icon_rect.center(),
                    radius: small_icon_rect.width() / 3.0,
                    fill: checked_color,
                    stroke: Default::default(),
                });
            }

            if let Some(galley) = text {
                let text_pos = pos2(
                    rect.min.x + icon_width + icon_spacing,
                    rect.center().y - 0.5 * galley.size().y,
                );
                ui.painter().galley(text_pos, galley, visuals.text_color());
            }
        }

        response
    }
}
