use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke, LineCap};
use iced::{Color, Element, Length, Point, Rectangle, Theme, mouse};
use std::collections::VecDeque;

// === Ring Gauge (as a function returning Element) ===

struct RingProgram {
    percent: f32,
    color: Color,
    label: String,
}

impl canvas::Program<super::Message> for RingProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let frame_geo = canvas::Cache::new().draw(renderer, bounds.size(), |frame: &mut Frame| {
            let center = frame.center();
            let radius = (bounds.width.min(bounds.height) / 2.0) - 8.0;
            let stroke_width = 6.0;

            // Track
            let track = Path::circle(center, radius);
            frame.stroke(&track, Stroke {
                width: stroke_width,
                style: canvas::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.06)),
                line_cap: LineCap::Round,
                ..Default::default()
            });

            // Value arc
            if self.percent > 0.0 {
                let start_angle = -std::f32::consts::FRAC_PI_2;
                let sweep = 2.0 * std::f32::consts::PI * (self.percent / 100.0);
                let end_angle = start_angle + sweep;

                let arc = Path::new(|builder| {
                    builder.arc(canvas::path::Arc {
                        center,
                        radius,
                        start_angle: iced::Radians(start_angle),
                        end_angle: iced::Radians(end_angle),
                    });
                });

                frame.stroke(&arc, Stroke {
                    width: stroke_width,
                    style: canvas::Style::Solid(self.color),
                    line_cap: LineCap::Round,
                    ..Default::default()
                });
            }

            // Percentage text
            frame.fill_text(canvas::Text {
                content: format!("{:.0}%", self.percent),
                position: Point::new(center.x, center.y - 4.0),
                color: Color::WHITE,
                size: iced::Pixels(16.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            });

            // Label
            frame.fill_text(canvas::Text {
                content: self.label.clone(),
                position: Point::new(center.x, center.y + 12.0),
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.35),
                size: iced::Pixels(9.0),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        });

        vec![frame_geo]
    }
}

pub fn ring_gauge(percent: f32, color: Color, label: &str, size: f32) -> Element<'static, super::Message> {
    iced::widget::canvas(RingProgram {
        percent: percent.clamp(0.0, 100.0),
        color,
        label: label.to_string(),
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

// === Sparkline Chart ===

struct SparkProgram {
    data: Vec<f32>,
    color: Color,
    max_points: usize,
}

impl canvas::Program<super::Message> for SparkProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let frame_geo = canvas::Cache::new().draw(renderer, bounds.size(), |frame: &mut Frame| {
            let w = bounds.width;
            let h = bounds.height;

            if self.data.is_empty() { return; }

            let n = self.max_points.max(1);
            let step = w / n as f32;

            let points: Vec<Point> = self.data.iter().enumerate().map(|(i, &val)| {
                let x = i as f32 * step;
                let y = h - (val.clamp(0.0, 100.0) / 100.0 * (h - 2.0)) - 1.0;
                Point::new(x, y)
            }).collect();

            // Filled area
            let area = Path::new(|b| {
                b.move_to(Point::new(points[0].x, h));
                for p in &points { b.line_to(*p); }
                b.line_to(Point::new(points.last().unwrap().x, h));
                b.close();
            });
            frame.fill(&area, Color::from_rgba(self.color.r, self.color.g, self.color.b, 0.12));

            // Line
            let line = Path::new(|b| {
                b.move_to(points[0]);
                for p in points.iter().skip(1) { b.line_to(*p); }
            });
            frame.stroke(&line, Stroke {
                width: 1.5,
                style: canvas::Style::Solid(Color::from_rgba(self.color.r, self.color.g, self.color.b, 0.8)),
                line_cap: LineCap::Round,
                ..Default::default()
            });

            // Grid lines
            let grid_c = Color::from_rgba(1.0, 1.0, 1.0, 0.03);
            for pct in [25.0, 50.0, 75.0] {
                let y = h - (pct / 100.0 * h);
                frame.stroke(
                    &Path::line(Point::new(0.0, y), Point::new(w, y)),
                    Stroke { width: 0.5, style: canvas::Style::Solid(grid_c), ..Default::default() },
                );
            }
        });

        vec![frame_geo]
    }
}

pub fn sparkline(data: &VecDeque<f32>, color: Color, max_points: usize, height: f32) -> Element<'static, super::Message> {
    iced::widget::canvas(SparkProgram {
        data: data.iter().cloned().collect(),
        color,
        max_points,
    })
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .into()
}

// === Color utilities ===

pub fn usage_color(percent: f32, base: Color) -> Color {
    if percent > 90.0 {
        Color::from_rgb(0.94, 0.33, 0.31)
    } else if percent > 75.0 {
        Color::from_rgb(1.0, 0.44, 0.26)
    } else if percent > 50.0 {
        Color::from_rgb(1.0, 0.65, 0.15)
    } else {
        base
    }
}

pub const CPU_COLOR: Color = Color::from_rgb(0.31, 0.76, 0.97);
pub const RAM_COLOR: Color = Color::from_rgb(0.67, 0.28, 0.74);
pub const GPU_COLOR: Color = Color::from_rgb(0.40, 0.73, 0.42);
pub const VRAM_COLOR: Color = Color::from_rgb(1.0, 0.44, 0.26);
