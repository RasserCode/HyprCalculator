use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 180.0])
            .with_resizable(false)
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "HyprCalculator",
        options,
        Box::new(|_cc| Ok(Box::new(CalcApp::default()))),
    )
}

#[derive(Default)]
struct CalcApp {
    input: String,
    result: String,
    focused: bool,
}

impl eframe::App for CalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Simple Calculator");

                let text_edit = egui::TextEdit::singleline(&mut self.input)
                    .hint_text("Enter expression, e.g. (2+3)*4")
                    .desired_width(400.0)
                    .font(egui::FontId::proportional(28.0))
                    .id_source("calc_input");

                let response = ui.add(text_edit);

                // Focus handling
                if !self.focused {
                    response.request_focus();
                    self.focused = true;
                }
                if ctx.input(|i| i.focused) && !response.has_focus() {
                    response.request_focus();
                }

                // ESC: Clear
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.input.clear();
                    self.result.clear();
                }

                // ENTER: Calculate
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && response.has_focus() {
                    self.result = match eval(&self.input) {
                        Ok(r) => format!("= {}", r),
                        Err(e) => format!("Error: {}", e),
                    };
                }

                // Ctrl+C: Copy result
                if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
                    if self.result.starts_with("= ") {
                        let text = self.result[2..].to_string();
                        ui.output_mut(|o| o.copied_text = text);
                    }
                }

                // Result: huge orange
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(&self.result)
                        .size(36.0)
                        .color(egui::Color32::from_rgb(255, 140, 0)),
                );
            });
        });
    }
}

// === EVALUATOR WITH PARENTHESES (FULLY WORKING) ===
fn eval(expr: &str) -> Result<f64, String> {
    let expr = expr
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let (val, rest) = parse_add_sub(&expr)?;
    if !rest.is_empty() {
        Err("Extra characters".to_string())
    } else {
        Ok(val)
    }
}

fn parse_add_sub(s: &str) -> Result<(f64, &str), String> {
    let (mut val, mut rest) = parse_mul_div(s)?;
    while let Some(op) = rest.chars().next() {
        if op == '+' || op == '-' {
            rest = &rest[1..];
            let (rhs, new_rest) = parse_mul_div(rest)?;
            val = if op == '+' { val + rhs } else { val - rhs };
            rest = new_rest;
        } else {
            break;
        }
    }
    Ok((val, rest))
}

fn parse_mul_div(s: &str) -> Result<(f64, &str), String> {
    let (mut val, mut rest) = parse_primary(s)?;
    while let Some(op) = rest.chars().next() {
        if op == '*' || op == '/' || op == '×' || op == '÷' {
            rest = &rest[1..];
            let (rhs, new_rest) = parse_primary(rest)?;
            val = if op == '*' || op == '×' {
                val * rhs
            } else if rhs == 0.0 {
                return Err("Division by zero".to_string());
            } else {
                val / rhs
            };
            rest = new_rest;
        } else {
            break;
        }
    }
    Ok((val, rest))
}

fn parse_primary(s: &str) -> Result<(f64, &str), String> {
    if s.starts_with('(') {
        let inner = &s[1..];
        let (val, rest) = parse_add_sub(inner)?;
        if rest.starts_with(')') {
            Ok((val, &rest[1..]))
        } else {
            Err("Missing closing parenthesis".to_string())
        }
    } else {
        parse_number(s)
    }
}

fn parse_number(s: &str) -> Result<(f64, &str), String> {
    let mut i = 0;
    let mut has_dot = false;
    while i < s.len() {
        match s[i..].chars().next().unwrap() {
            '0'..='9' => i += 1,
            '.' => {
                if has_dot {
                    return Err("Multiple decimal points".to_string());
                }
                has_dot = true;
                i += 1;
            }
            _ => break,
        }
    }
    if i == 0 {
        Err("Expected number".to_string())
    } else {
        let num_str = &s[..i];
        let num = num_str
            .parse::<f64>()
            .map_err(|_| "Invalid number".to_string())?;
        Ok((num, &s[i..]))
    }
}
