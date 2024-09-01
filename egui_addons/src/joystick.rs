use egui::{Color32, Response, Rounding, Sense, Shape, Stroke, Ui, Vec2, Widget};

pub struct JoystickStyle {
    pub markers_color: Color32,
    pub markers_width: f32,
    pub markers_dash_length: f32,
    pub joystick_radius: f32,
    pub joystick_color: Color32,
    pub joystick_stroke_width: f32,
    pub joystick_stroke_color: Color32,
    pub joystick_stroke_color_hover: Color32,
    pub motion_radius: f32,
}

impl JoystickStyle {
    pub fn new_with_desired_size(desired_size: Vec2) -> Self {
        let min_size = (desired_size.x.min(desired_size.y)) / 2.0;
        Self {
            markers_color: Color32::WHITE,
            markers_width: 1.0,
            markers_dash_length: 10.0,
            joystick_radius: 20.0,
            joystick_color: Color32::DARK_GRAY,
            joystick_stroke_width: 5.0,
            joystick_stroke_color: Color32::GRAY,
            joystick_stroke_color_hover: Color32::LIGHT_GRAY,
            motion_radius: min_size,
        }
    }
}

impl Default for JoystickStyle {
    fn default() -> Self {
        Self {
            markers_color: Color32::WHITE,
            markers_width: 1.0,
            markers_dash_length: 10.0,
            joystick_radius: 20.0,
            joystick_color: Color32::DARK_GRAY,
            joystick_stroke_width: 5.0,
            joystick_stroke_color: Color32::GRAY,
            joystick_stroke_color_hover: Color32::LIGHT_GRAY,
            motion_radius: 150.0,
        }
    }
}

pub struct Joystick<'a> {
    state: &'a mut Vec2,
    style: JoystickStyle,
}

impl<'a> Joystick<'a> {
    pub fn new(state: &'a mut Vec2, desired_size: Vec2) -> Self {
        Self {
            state,
            style: JoystickStyle::new_with_desired_size(desired_size),
        }
    }

    fn handle_response(self, response: &Response) {
        if response.is_pointer_button_down_on() {
            let pos = response.interact_pointer_pos().unwrap() - response.rect.center();
            let pos = Vec2 {
                x: pos.x / self.style.motion_radius,
                y: pos.y / self.style.motion_radius,
            };
            if pos.length() > 1.0 {
                *self.state = pos.normalized();
            } else {
                *self.state = pos;
            }
        }
    }
}

impl<'a> Widget for Joystick<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(
            Vec2::splat(self.style.joystick_radius * 2.0 + self.style.motion_radius * 2.0),
            Sense::click_and_drag(),
        );
        painter.rect_stroke(
            response.rect,
            Rounding::same(self.style.joystick_radius),
            Stroke::new(self.style.markers_width, self.style.markers_color),
        );

        [
            [
                response.rect.center(),
                response.rect.center() + Vec2::new(self.style.motion_radius, 0.0),
            ],
            [
                response.rect.center(),
                response.rect.center() + Vec2::new(0.0, self.style.motion_radius),
            ],
            [
                response.rect.center(),
                response.rect.center() + Vec2::new(-self.style.motion_radius, 0.0),
            ],
            [
                response.rect.center(),
                response.rect.center() + Vec2::new(0.0, -self.style.motion_radius),
            ],
        ]
        .iter()
        .for_each(|path| {
            let dashed_line = Shape::dashed_line(
                path,
                Stroke::new(self.style.markers_width, self.style.markers_color),
                self.style.markers_dash_length,
                self.style.markers_dash_length,
            );
            painter.add(dashed_line);
        });

        painter.circle_stroke(
            response.rect.center(),
            self.style.motion_radius,
            Stroke::new(self.style.markers_width, self.style.markers_color),
        );

        painter.circle(
            response.rect.center() + *self.state * self.style.motion_radius,
            self.style.joystick_radius,
            self.style.joystick_color,
            Stroke::new(
                self.style.joystick_stroke_width,
                if response.hovered() {
                    self.style.joystick_stroke_color_hover
                } else {
                    self.style.joystick_stroke_color
                },
            ),
        );

        self.handle_response(&response);
        response
    }
}
