mod gui_utils;
use gui_utils::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn error_handler<T>(e: T) -> !
where
    T: std::fmt::Display + 'static,
{
    let app_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 100.0]),
        ..Default::default()
    };
    println!("{}", e);

    eframe::run_native(
        "Keithley 2230-30-1 Triple Channel DC Power Supply",
        app_options,
        Box::new(|_| Box::new(ErrorPopUp { error: e })),
    )
    .unwrap();
    std::process::exit(1);
}

fn main() {
    env_logger::init();
    let resource_manager = visa_api::DefaultRM::new().unwrap_or_else(|e| error_handler(e));
    let instrument = visa_api::Instrument::get_instrument(
        &resource_manager,
        keithley_2230_series::MANUFACTURER,
        keithley_2230_series::MODEL,
    )
    .unwrap_or_else(|e| error_handler(e));

    let instrument = instrument.unwrap_or_else(|| {
        error_handler(format!(
            "No {} {} found.",
            keithley_2230_series::MANUFACTURER,
            keithley_2230_series::MODEL
        ))
    });

    let keithley2230 = keithley_2230_series::Keithley2230::new(instrument);

    let app_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Keithley 2230-30-1 Triple Channel DC Power Supply",
        app_options,
        Box::new(|_| {
            Box::new(App {
                instrument: keithley2230,
                ch1: Channel::default(),
                ch2: Channel::default(),
                ch3: Channel::default(),
                ch_parallel: Channel::default(),
                ch_series: Channel::default(),
                time: InstrumentTime {
                    refresh_rate: Duration::from_secs_f32(1.0 / 5.0),
                    current: SystemTime::now(),
                    previous: SystemTime::now(),
                },
            })
        }),
    )
    .unwrap_or_else(|e| error_handler(e));
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        update_instrument(self);
        ctx.request_repaint();

        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Power Supply Controls");

            let ch_parallel_enabled = is_ch_enabled(&self.ch_parallel);
            let ch_series_enabled = is_ch_enabled(&self.ch_series);

            // Turn off CH1 and/or CH2 if Series/Parallel are enabled
            if ch_parallel_enabled || ch_series_enabled {
                if is_ch_enabled(&self.ch1) {
                    self.ch1.toggle_enable_button_state();
                }
                if is_ch_enabled(&self.ch2) {
                    self.ch2.toggle_enable_button_state();
                }
            }

            ui.add_enabled_ui(!(ch_parallel_enabled || ch_series_enabled), |ui| {
                channel(ui, &mut self.ch1, "1".to_string(), 0.0, 30.0, 0.0, 1.5);
                channel(ui, &mut self.ch2, "2".to_string(), 0.0, 30.0, 0.0, 1.5);
            });
            channel(ui, &mut self.ch3, "3".to_string(), 0.0, 6.0, 0.0, 5.0);
            ui.add_enabled_ui(!ch_parallel_enabled, |ui| {
                channel(
                    ui,
                    &mut self.ch_series,
                    "1/2 Series".to_string(),
                    0.0,
                    60.0,
                    0.0,
                    1.5,
                );
            });
            ui.add_enabled_ui(!ch_series_enabled, |ui| {
                channel(
                    ui,
                    &mut self.ch_parallel,
                    "1/2 Parallel".to_string(),
                    0.0,
                    30.0,
                    0.0,
                    3.0,
                );
            });
        });
    }
}

impl<T> eframe::App for ErrorPopUp<T>
where
    T: std::fmt::Display,
{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{}", self.error));
        });
    }
}

fn is_ch_enabled(ch: &Channel) -> bool {
    match ch.state {
        keithley_2230_series::ChannelOutputState::ON => true,
        keithley_2230_series::ChannelOutputState::OFF => false,
    }
}

fn update_instrument(app: &mut App) {
    app.time.current = SystemTime::now();

    if app.time.current.duration_since(app.time.previous).unwrap() > app.time.refresh_rate {
        app.time.previous = app.time.current;

        app.instrument
            .enable_channel(keithley_2230_series::Channel::CH1, app.ch1.state)
            .unwrap_or_else(|e| error_handler(e));

        app.instrument
            .enable_channel(keithley_2230_series::Channel::CH2, app.ch2.state)
            .unwrap_or_else(|e| error_handler(e));

        app.instrument
            .enable_channel(keithley_2230_series::Channel::CH3, app.ch3.state)
            .unwrap_or_else(|e| error_handler(e));

        app.instrument
            .set_channel(
                keithley_2230_series::Channel::CH1,
                app.ch1.set_values.voltage,
                app.ch1.set_values.current,
            )
            .unwrap_or_else(|e| error_handler(e));

        app.instrument
            .set_channel(
                keithley_2230_series::Channel::CH2,
                app.ch2.set_values.voltage,
                app.ch2.set_values.current,
            )
            .unwrap_or_else(|e| error_handler(e));

        app.instrument
            .set_channel(
                keithley_2230_series::Channel::CH3,
                app.ch3.set_values.voltage,
                app.ch3.set_values.current,
            )
            .unwrap_or_else(|e| error_handler(e));
    }
}
