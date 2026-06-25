use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

use eframe::egui;
use pi_birthday_bench::date::validate_yyyymmdd;
use pi_birthday_bench::job::run_job;
use pi_birthday_bench::result::{BackendMode, BenchmarkResult, ProgressEvent, RunConfig, RunPhase};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "pi-birthday-bench",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::default()))),
    )
}

struct GuiApp {
    target: String,
    max_digits: String,
    chunk: String,
    selected_backend: BackendMode,
    benchmark_only: bool,
    verify: bool,
    phase: RunPhase,
    status: String,
    error_message: Option<String>,
    result: Option<BenchmarkResult>,
    receiver: Option<mpsc::Receiver<ProgressEvent>>,
    cancel_flag: Option<Arc<AtomicBool>>,
    worker: Option<thread::JoinHandle<()>>,
    started_at: Option<Instant>,
    elapsed_seconds: f64,
    digits_computed: usize,
    digits_per_second: f64,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            target: "19930628".to_owned(),
            max_digits: "1000000".to_owned(),
            chunk: "1000000".to_owned(),
            selected_backend: BackendMode::CpuSingle,
            benchmark_only: false,
            verify: false,
            phase: RunPhase::Idle,
            status: RunPhase::Idle.as_str().to_owned(),
            error_message: None,
            result: None,
            receiver: None,
            cancel_flag: None,
            worker: None,
            started_at: None,
            elapsed_seconds: 0.0,
            digits_computed: 0,
            digits_per_second: 0.0,
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();

        if self.is_running() {
            ctx.request_repaint();
            if let Some(started_at) = self.started_at {
                self.elapsed_seconds = started_at.elapsed().as_secs_f64();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("pi-birthday-bench");
            ui.add_space(8.0);

            self.render_inputs(ui);
            ui.separator();
            self.render_controls(ui);
            ui.separator();
            self.render_progress(ui);
            ui.separator();
            self.render_result(ui);
        });
    }
}

impl GuiApp {
    fn render_inputs(&mut self, ui: &mut egui::Ui) {
        let validation = self.validate_inputs();

        egui::Grid::new("inputs").num_columns(3).show(ui, |ui| {
            ui.label("Target date");
            ui.text_edit_singleline(&mut self.target);
            match validate_yyyymmdd(&self.target) {
                Ok(()) => ui.label("valid"),
                Err(_) => ui.colored_label(egui::Color32::RED, "invalid YYYYMMDD"),
            };
            ui.end_row();

            ui.label("Max digits");
            ui.text_edit_singleline(&mut self.max_digits);
            ui.label(number_status(validation.max_digits_ok));
            ui.end_row();

            ui.label("Chunk size");
            ui.text_edit_singleline(&mut self.chunk);
            ui.label(number_status(validation.chunk_ok));
            ui.end_row();

            ui.label("Backend");
            egui::ComboBox::from_id_salt("backend")
                .selected_text(self.selected_backend.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_backend,
                        BackendMode::CpuSingle,
                        "cpu-single",
                    );
                    for backend in [
                        "cpu-multi",
                        "cuda-search-only",
                        "cuda-compute",
                        "hip",
                        "opencl",
                        "vulkan",
                    ] {
                        ui.add_enabled(false, egui::Button::new(backend));
                    }
                });
            ui.label("cpu-single only");
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut self.benchmark_only, "Benchmark only"),
            );
            ui.add_enabled(false, egui::Checkbox::new(&mut self.verify, "Verify"));
        });
    }

    fn render_controls(&mut self, ui: &mut egui::Ui) {
        let validation = self.validate_inputs();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    validation.ok() && !self.is_running(),
                    egui::Button::new("Start"),
                )
                .clicked()
            {
                self.start_job(validation.config.unwrap());
            }

            if ui
                .add_enabled(self.is_running(), egui::Button::new("Cancel"))
                .clicked()
            {
                if let Some(cancel_flag) = &self.cancel_flag {
                    cancel_flag.store(true, Ordering::Relaxed);
                }
                self.status =
                    "cancelling; current calculation phase may finish before cancellation takes effect"
                        .to_owned();
            }

            if ui.button("Clear result").clicked() {
                self.result = None;
                self.error_message = None;
                if !self.is_running() {
                    self.phase = RunPhase::Idle;
                    self.status = RunPhase::Idle.as_str().to_owned();
                    self.elapsed_seconds = 0.0;
                    self.digits_computed = 0;
                    self.digits_per_second = 0.0;
                }
            }
        });
    }

    fn render_progress(&self, ui: &mut egui::Ui) {
        let max_digits = self.max_digits.parse::<usize>().unwrap_or(0);
        let progress = if max_digits > 0 {
            (self.digits_computed as f32 / max_digits as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        egui::Grid::new("progress").num_columns(2).show(ui, |ui| {
            ui.label("status");
            ui.label(&self.status);
            ui.end_row();

            ui.label("current phase");
            ui.label(self.phase.as_str());
            ui.end_row();

            ui.label("elapsed seconds");
            ui.label(format!("{:.2}", self.elapsed_seconds));
            ui.end_row();

            ui.label("digits computed");
            ui.label(self.digits_computed.to_string());
            ui.end_row();

            ui.label("speed digits/sec");
            ui.label(format!("{:.1}", self.digits_per_second));
            ui.end_row();

            ui.label("backend");
            ui.label(self.selected_backend.as_str());
            ui.end_row();
        });

        ui.add(egui::ProgressBar::new(progress).show_percentage());
        ui.label(
            "Cancellation during computing_pi may take effect only after that phase finishes.",
        );
    }

    fn render_result(&mut self, ui: &mut egui::Ui) {
        if let Some(error_message) = &self.error_message {
            ui.colored_label(egui::Color32::RED, error_message);
        }

        let Some(result) = &self.result else {
            return;
        };

        ui.monospace(result.as_text());
        ui.horizontal(|ui| {
            if ui.button("Copy result as text").clicked() {
                ui.ctx().copy_text(result.as_text());
            }
            if ui.button("Copy result as JSON").clicked() {
                ui.ctx().copy_text(result.as_json());
            }
        });
    }

    fn start_job(&mut self, config: RunConfig) {
        let (sender, receiver) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let worker_cancel_flag = Arc::clone(&cancel_flag);

        self.receiver = Some(receiver);
        self.cancel_flag = Some(cancel_flag);
        self.started_at = Some(Instant::now());
        self.elapsed_seconds = 0.0;
        self.digits_computed = 0;
        self.digits_per_second = 0.0;
        self.error_message = None;
        self.result = None;
        self.phase = RunPhase::Validating;
        self.status = RunPhase::Validating.as_str().to_owned();

        self.worker = Some(thread::spawn(move || {
            let sender_for_events = sender.clone();
            if let Err(error) = run_job(config, &worker_cancel_flag, |event| {
                let _ = sender_for_events.send(event);
            }) {
                if !worker_cancel_flag.load(Ordering::Relaxed) {
                    let _ = sender.send(ProgressEvent::Failed(error.to_string()));
                }
            }
        }));
    }

    fn drain_events(&mut self) {
        let mut events = Vec::new();
        if let Some(receiver) = &self.receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        for event in events {
            self.apply_event(event);
        }

        if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.is_finished())
        {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            self.receiver = None;
            self.cancel_flag = None;
        }
    }

    fn apply_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::Started { .. } => {
                self.phase = RunPhase::Validating;
                self.status = RunPhase::Validating.as_str().to_owned();
            }
            ProgressEvent::PhaseChanged { phase } => {
                self.phase = phase;
                self.status = phase.as_str().to_owned();
            }
            ProgressEvent::Progress {
                digits_computed,
                elapsed_seconds,
                digits_per_second,
            } => {
                self.digits_computed = digits_computed;
                self.elapsed_seconds = elapsed_seconds;
                self.digits_per_second = digits_per_second;
            }
            ProgressEvent::Completed(result) => {
                self.phase = RunPhase::Completed;
                self.status = RunPhase::Completed.as_str().to_owned();
                self.digits_computed = result.digits_computed;
                self.elapsed_seconds = result.elapsed_seconds;
                self.digits_per_second = result.digits_per_second;
                self.result = Some(result);
            }
            ProgressEvent::Cancelled => {
                self.phase = RunPhase::Cancelled;
                self.status = RunPhase::Cancelled.as_str().to_owned();
            }
            ProgressEvent::Failed(message) => {
                self.phase = RunPhase::Error;
                self.status = RunPhase::Error.as_str().to_owned();
                self.error_message = Some(message);
            }
        }
    }

    fn is_running(&self) -> bool {
        self.worker.is_some()
    }

    fn validate_inputs(&self) -> InputValidation {
        let target_ok = validate_yyyymmdd(&self.target).is_ok();
        let max_digits = self.max_digits.parse::<usize>().ok();
        let chunk = self.chunk.parse::<usize>().ok();

        InputValidation {
            target_ok,
            max_digits_ok: max_digits.is_some_and(|value| value > 0),
            chunk_ok: chunk.is_some_and(|value| value > 0),
            config: match (target_ok, max_digits, chunk) {
                (true, Some(max_digits), Some(chunk)) if max_digits > 0 && chunk > 0 => {
                    Some(RunConfig {
                        target: self.target.clone(),
                        max_digits,
                        chunk,
                        backend: self.selected_backend,
                    })
                }
                _ => None,
            },
        }
    }
}

struct InputValidation {
    target_ok: bool,
    max_digits_ok: bool,
    chunk_ok: bool,
    config: Option<RunConfig>,
}

impl InputValidation {
    fn ok(&self) -> bool {
        self.target_ok && self.max_digits_ok && self.chunk_ok
    }
}

fn number_status(ok: bool) -> egui::RichText {
    if ok {
        egui::RichText::new("valid")
    } else {
        egui::RichText::new("must be > 0").color(egui::Color32::RED)
    }
}
