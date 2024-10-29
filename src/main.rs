mod providers;

use anyhow::Result;
use eframe::egui::{self, Ui, mutex::Mutex};
use lazy_static::lazy_static;
use providers::{
    amazon::{self, query_products},
    bol, Product, Products, QueryHandles,
};
use regex::Regex;
use rust_xlsxwriter::Workbook;
use std::{
    io::{self, BufRead}, ops::DerefMut, sync::Arc
};

fn read_line(msg: &str) -> std::io::Result<String> {
    println!("{msg}:");
    let stdin = io::stdin().lock();
    stdin.lines().next().expect("input")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args();
    create_window().unwrap();
    std::process::exit(0);

    let subcommand = args.next().expect("a subcommand");

    // let url = read_line("Link naar amazon zoekresultaten")?;
    // let pages = read_line("Hoeveel paginas")?.parse().expect("Valid usize");

    // let products = match subcommand.to_lowercase().as_str() {
    //     "bol" => bol::query_products(&url, pages).await?,
    //     "amazon" => amazon::query_products(&url, pages).await?,
    //
    //     _ => anyhow::bail!("choose from: [bol, amazon]"),
    // };

    // let mut workbook = Workbook::new();
    // workbook.push_worksheet(products.as_worksheet()?);
    // workbook.save("products.xlsx")?;
    // println!("Done!");

    Ok(())
}

fn create_window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<State>::default())),
    )
}

#[derive(Debug)]
enum Platform {
    Amazon,
    Bol,
}

lazy_static! {
    static ref bol_url_regex: Regex = Regex::new(r"(?:http|https):\/\/bol\.com*.").unwrap();
    static ref amazon_url_regex: Regex = Regex::new(r"(?:http|https):\/\/amazon\.nl*.").unwrap();
}

// TODO: Pattern match by domain
impl Platform {
    fn from_url(value: &str) -> Option<Self> {
        if bol_url_regex.is_match(value) {
            Some(Self::Bol)
        } else if amazon_url_regex.is_match(value) {
            Some(Self::Amazon)
        } else {
            None
        }
    }

    async fn query_products(&self, url: &str, pages: usize) -> QueryHandles<Product> {
        match *self {
            Self::Amazon => amazon::query_products(url, pages).await,
            Self::Bol => amazon::query_products(url, pages).await,
        }
    }
}

struct State {
    url: String,
    path: Option<String>,
    platform: Option<Platform>,
    // handles: Arc<Mutex<Option<QueryHandles<Product>>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            url: "".to_owned(),
            path: None,
            platform: None,
            // handles: Arc::new(Mutex::new(None)),
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let label = ui.label("Search URL: ");
                ui.text_edit_singleline(&mut self.url).labelled_by(label.id);
            });

            self.platform = Platform::from_url(&self.url);

            if ui.button("Export products").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("products.xlsx")
                    .save_file()
                {
                    self.path = Some(path.display().to_string());

                    // let handles = self.handles.clone();
                    let url = self.url.to_owned();
                    tokio::spawn(async move {
                        // handles
                        // let mut handles = handles.lock();
                        let handles = query_products(&url, 1).await;

                        // if let Some(handles) = handles.deref_mut() {
                        //
                            // for (i,handle) in handles.iter().enumerate() {
                        //
                            // }
                        // }

                        // for handle in self.handles.clone().lock().await {}
                    });
                }
            }

            // if let Some(ref platform) = self.platform {
            //     if let Some(ref path) = self.path {
            //         platform.query_products(&self.url, 2).await;
            //     }
            // }

            debug_state(ui, self);
        });
    }
}

fn debug_state(ui: &mut Ui, state: &State) {
    ui.label(format!("url: {}", state.url));
    ui.label(format!("path: {:?}", state.path));
    ui.label(format!("platform: {:?}", state.platform));
    ui.label(format!("handles: {:?}", state.handles));
}
