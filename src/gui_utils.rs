#[derive(Debug, Clone, Default)]
pub struct Button {
    pub response: Option<egui::Response>,
    pub color: egui::Color32,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelSetValues {
    pub voltage: f32,
    pub current: f32,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelMeasurements {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TextBox {
    pub response: Option<egui::Response>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub enable_button: Button,
    pub submit_button: Button,
    pub voltage_text_box: TextBox,
    pub current_text_box: TextBox,
    pub set_values: ChannelSetValues,
    pub measurements: ChannelMeasurements,
    pub state: keithley_2230_series::ChannelOutputState,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            enable_button: Button {
                text: "OFF".to_string(),
                color: egui::Color32::RED,
                ..Default::default()
            },
            set_values: ChannelSetValues::default(),
            submit_button: Button::default(),
            measurements: ChannelMeasurements::default(),
            voltage_text_box: TextBox::default(),
            current_text_box: TextBox::default(),
            state: keithley_2230_series::ChannelOutputState::default(),
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub instrument: keithley_2230_series::Keithley2230,
    pub ch1: Channel,
    pub ch2: Channel,
    pub ch3: Channel,
    pub ch_series: Channel,
    pub ch_parallel: Channel,
    pub time: InstrumentTime,
}

#[derive(Debug)]
pub struct InstrumentTime {
    pub refresh_rate: std::time::Duration,
    pub current: std::time::SystemTime,
    pub previous: std::time::SystemTime,
}

#[derive(Debug, Default)]
pub struct ErrorPopUp<T>
where
    T: std::fmt::Display,
{
    pub error: T,
}

impl Channel {
    pub fn toggle_enable_button_state(&mut self) {
        match self.state {
            keithley_2230_series::ChannelOutputState::ON => {
                self.enable_button.color = egui::Color32::RED;
                self.enable_button.text = "OFF".to_string();
                self.state = keithley_2230_series::ChannelOutputState::OFF;
            }
            keithley_2230_series::ChannelOutputState::OFF => {
                self.enable_button.color = egui::Color32::GREEN;
                self.enable_button.text = "ON".to_string();
                self.state = keithley_2230_series::ChannelOutputState::ON;
            }
        }
    }
}

pub fn channel(
    ui: &mut egui::Ui,
    ch: &mut Channel,
    channel_number: String,
    min_voltage: f32,
    max_voltage: f32,
    min_current: f32,
    max_current: f32,
) {
    let enabled = match ch.state {
        keithley_2230_series::ChannelOutputState::OFF => false,
        keithley_2230_series::ChannelOutputState::ON => true,
    };

    ui.heading(format!("Channel {}", channel_number));
    ui.horizontal(|ui| {
        ui.label("State ");
        ch.enable_button.response = Some(
            ui.button(egui::RichText::new(&ch.enable_button.text).color(ch.enable_button.color)),
        );
        if ch.enable_button.response.as_ref().unwrap().clicked() {
            ch.toggle_enable_button_state();
        }
    });
    ui.horizontal(|ui| {
        ui.add_enabled_ui(!enabled, |ui| {
            text_box(ui, ch, min_voltage, max_voltage, min_current, max_current);
        });
    });
}

fn text_box(
    ui: &mut egui::Ui,
    ch: &mut Channel,
    min_voltage: f32,
    max_voltage: f32,
    min_current: f32,
    max_current: f32,
) {
    ui.label("Voltage (V) ");
    ch.voltage_text_box.response = Some(ui.add_sized(
        egui::vec2(50.0, 20.0),
        egui::TextEdit::singleline(&mut ch.voltage_text_box.text),
    ));

    ui.label("Current (A) ");
    ch.current_text_box.response = Some(ui.add_sized(
        egui::vec2(50.0, 20.0),
        egui::TextEdit::singleline(&mut ch.current_text_box.text),
    ));

    if (ch.voltage_text_box.response.as_ref().unwrap().lost_focus()
        || ch.current_text_box.response.as_ref().unwrap().lost_focus())
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
    {
        let value = ch.voltage_text_box.text.parse::<f32>().ok();
        if let Some(value) = value {
            if (min_voltage..=max_voltage).contains(&value) {
                ch.set_values.voltage = value;
            }
        }
        ch.voltage_text_box.text = String::new();

        let value = ch.current_text_box.text.parse::<f32>().ok();
        if let Some(value) = value {
            if (min_current..=max_current).contains(&value) {
                ch.set_values.current = value;
            }
        }
        ch.current_text_box.text = String::new();
    }
}
