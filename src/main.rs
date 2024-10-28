mod providers;

use anyhow::Result;
use providers::{amazon, bol};
use rust_xlsxwriter::Workbook;
use std::io::{self, BufRead};

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args();
    render_gui().unwrap();
    std::process::exit(0);

    let subcommand = args.next().expect("a subcommand");

    let url = read_line("Link naar amazon zoekresultaten")?;
    let pages = read_line("Hoeveel paginas")?.parse().expect("Valid usize");

    let products = match subcommand.to_lowercase().as_str() {
        "bol" => bol::query_products(&url, pages).await?,
        "amazon" => amazon::query_products(&url, pages).await?,

        _ => anyhow::bail!("choose from: [bol, amazon]"),
    };

    let mut workbook = Workbook::new();
    workbook.push_worksheet(products.as_worksheet()?);
    workbook.save("products.xlsx")?;
    println!("Done!");

    Ok(())
}

use eframe::egui;

fn render_gui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_| {
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
