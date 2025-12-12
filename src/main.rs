#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use core::f32;
use eframe::{
    egui::{
        self, FontData, FontDefinitions, FontFamily, FontId, Id, Modal, RichText, ScrollArea,
        TextEdit, Ui, Window,
    },
    epaint::text::FontInsert,
};
#[cfg(feature = "print")]
use log::{error, info};
use std::{
    ffi::OsString,
    path::PathBuf,
    process::exit,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};

use crate::font_info::dump;

macro_rules! s_error {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    // (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => (
        #[cfg(feature="print")]
        log::error!($($arg)+);
    )
}

macro_rules! s_info {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    // (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => {{
        #[cfg(feature="print")]
        log::info!($($arg)+);
    }};
    () => (

        #[cfg(feature="print")]
        log::info!("");
    )
}

/// 非打包情况下直接包含字节
#[cfg(not(feature = "pkg"))]
fn icon_data() -> Vec<u8> {
    let b = include_bytes!("../img/ico.png");
    b.to_vec()
}
/// bundle内执行方法
mod bundle {

    /// 检查是否在Bundle环境中运行
    pub(super) fn is_bundle_environment() -> bool {
        if let Ok(exe_path) = std::env::current_exe() {
            return exe_path.to_string_lossy().contains(".app");
        }
        false
    }
    /// 获取Bundle资源目录路径
    pub(super) fn get_bundle_resources_path() -> Option<std::path::PathBuf> {
        if let Ok(exe_path) = std::env::current_exe() {
            // 可执行文件位于: MyApp.app/Contents/MacOS/
            if let Some(contents_dir) = exe_path.parent() {
                s_info!("con = {}", contents_dir.display());
                if let Some(bundle_dir) = contents_dir.parent() {
                    s_info!("bun= {}", bundle_dir.display());
                    let resources_path = bundle_dir.join("Resources");
                    s_info!("res = {}", resources_path.display());
                    if resources_path.exists() {
                        return Some(resources_path);
                    }
                }
            }
        }
        None
    }
}
#[cfg(debug_assertions)]
mod custom_log {

    use std::{io::Write, time::Duration};
    /// 时间戳转换，从1970年开始
    pub(crate) fn time_display(value: u64) -> String {
        do_time_display(value, 1970, Duration::from_secs(8 * 60 * 60))
    }

    /// 时间戳转换，支持从不同年份开始计算
    pub(crate) fn do_time_display(value: u64, start_year: u64, timezone: Duration) -> String {
        // 先粗略定位到哪一年
        // 以 365 来计算，年通常只会相比正确值更晚，剩下的秒数也就更多，并且有可能出现需要往前一年的情况
        let value = value + timezone.as_secs();

        let per_year_sec = 365 * 24 * 60 * 60; // 平年的秒数

        let mut year = value / per_year_sec;
        // 剩下的秒数，如果这些秒数 不够填补闰年，比如粗略计算是 2024年，还有 86300秒，不足一天，那么中间有很多闰年，所以 年应该-1，只有-1，因为-2甚至更多 需要 last_sec > 365 * 86400，然而这是不可能的
        let last_sec = value - (year) * per_year_sec;
        year += start_year;

        let mut leap_year_sec = 0;
        // 计算中间有多少闰年，当前年是否是闰年不影响回退，只会影响后续具体月份计算
        for y in start_year..year {
            if is_leap(y) {
                // 出现了闰年
                leap_year_sec += 86400;
            }
        }
        if last_sec < leap_year_sec {
            // 不够填补闰年，年份应该-1
            year -= 1;
            // 上一年是闰年，所以需要补一天
            if is_leap(year) {
                leap_year_sec -= 86400;
            }
        }
        // 剩下的秒数
        let mut time = value - leap_year_sec - (year - start_year) * per_year_sec;

        // 平年的月份天数累加
        let mut day_of_year: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        // 找到了 计算日期
        let sec = time % 60;
        time /= 60;
        let min = time % 60;
        time /= 60;
        let hour = time % 24;
        time /= 24;

        // 计算是哪天，因为每个月不一样多，所以需要修改
        if is_leap(year) {
            day_of_year[1] += 1;
        }
        let mut month = 0;
        for (index, ele) in day_of_year.iter().enumerate() {
            if &time < ele {
                month = index + 1;
                time += 1; // 日期必须加一，否则 每年的 第 1 秒就成了第0天了
                break;
            }
            time -= ele;
        }

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, time, hour, min, sec
        )
    }
    //
    // 判断是否是闰年
    //
    fn is_leap(year: u64) -> bool {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    }
    ///
    /// 输出当前时间格式化
    ///
    /// 例如：
    /// 2023-09-28T09:32:24Z
    ///
    pub(crate) fn time_format() -> String {
        // 获取当前时间戳
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|v| v.as_secs())
            .unwrap_or(0);

        time_display(time)
    }
    struct Writer {
        console: std::io::Stdout,
        fs: Option<std::fs::File>,
    }
    impl Writer {
        pub fn new() -> Self {
            Writer {
                console: std::io::stdout(),
                fs: None,
            }
        }
    }
    impl Write for Writer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(fs) = &mut self.fs {
                self.console.write(buf)?;
                fs.write(buf)
            } else {
                self.console.write(buf)
            }
        }

        fn flush(&mut self) -> std::io::Result<()> {
            if let Some(fs) = &mut self.fs {
                self.console.flush()?;
                fs.flush()
            } else {
                self.console.flush()
            }
        }
    }
    pub(crate) fn init() -> Result<(), String> {
        // if opt.verbose {
        //     std::env::set_var("RUST_LOG", "debug");
        // } else {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }

        // }
        #[cfg(feature = "print")]
        {
            let mut s = env_logger::builder();
            s.default_format()
                .parse_default_env()
                .format(|buf, record| writeln!(buf, "{}: {}", time_format(), record.args()))
                .target(env_logger::Target::Pipe(Box::new(Writer::new())));

            s.init();
        }

        Ok(())
    }
}
/// 打包情况下手动读取文件
#[cfg(feature = "pkg")]
fn icon_data() -> Vec<u8> {
    if let Some(res) = bundle::get_bundle_resources_path() {
        let icon = res.join("img/ico.png");
        if icon.exists()
            && let Ok(v) = std::fs::read(icon)
        {
            s_info!("bytes = {:?}", &v[0..10]);
            return v;
        }
    }
    Vec::new()
}

fn main() -> eframe::Result {
    let arg = std::env::args().collect::<Vec<String>>();

    if arg.get(1).map(|v| v == "cli").unwrap_or(false) {
        // 命令行模式
        let mut pargs =
            pico_args::Arguments::from_vec(arg.iter().skip(2).map(OsString::from).collect());

        if pargs.contains(["-h", "--help"]) {
            let help: &str = "\
USAGE:
  fontview cli --input PATH --output PATH [OPTIONS]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --input PATH          Font File
  --output PATH         Output Path
  --text String         Used Text
  --file PATH           Read Used Text From File
";
            println!("{}", help);
        };

        let input: String = pargs.value_from_str("--input").expect("--input err");
        let output: String = pargs.value_from_str("--output").expect("--output err");

        let font = std::fs::read(input).expect("load font fail");

        let file = pargs
            .opt_value_from_str("--file")
            .expect("--file err")
            .and_then(|v: String| std::fs::read_to_string(v).ok())
            .or(pargs.opt_value_from_str("--text").expect("--text err"));
        match file {
            Some(text) => {
                let font_file = allsorts::binary::read::ReadScope::new(&font)
                    .read::<allsorts::font_data::FontData>()
                    .expect("load font fail");
                let provider = font_file.table_provider(0).unwrap();
                if let Some(n) = font_info::subset_text(
                    &provider,
                    text.as_str(),
                    &std::path::Path::new(&output).to_path_buf(),
                ) {
                    println!("{}", n);
                } else {
                    eprintln!("subset fail");
                    exit(101);
                }
            }
            None => {
                eprintln!("require --text or --file")
            }
        }

        return Ok(());
    }
    #[cfg(debug_assertions)]
    let _ = custom_log::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_icon(eframe::icon_data::from_png_bytes(&icon_data()).unwrap())
            .with_taskbar(true),
        ..Default::default()
    };

    // 消息

    eframe::run_native(
        "FontView",
        options,
        Box::new(|cc| {
            // This gives us image support:
            Ok(Box::new(FontViewApp::default(&cc.egui_ctx)))
        }),
    )
}
#[derive(Clone)]
struct FontInner {
    path: String,
    mock_name: String,
    font_name: String,
    file_name: String,
}

enum Msg {
    Cancel,
    Dir(String),
    Font(Vec<FontInner>),
}

struct FontViewApp {
    dir: String,
    loading: bool,
    font: Vec<FontInner>,
    example: String,
    subset: SubsetModal,
    subset_open: bool,
    sx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl FontViewApp {
    fn default(cc: &egui::Context) -> Self {
        let (sx, rx) = std::sync::mpsc::channel();
        let res = Self {
            font: Vec::new(),
            loading: false,
            dir: String::new(),
            example: "测试文本".to_string(),
            subset: SubsetModal::default(),
            subset_open: false,
            sx,
            rx,
        };

        use rust_fontconfig::{FcFontCache, FcPattern};
        let fc = FcFontCache::build();

        let mut trace = Vec::new();
        #[cfg(target_os = "macos")]
        let re = fc.query(
            &FcPattern {
                family: Some("Heiti".to_string()),
                ..Default::default()
            },
            &mut trace,
        );
        #[cfg(target_os = "windows")]
        let re = fc.query(
            &FcPattern {
                family: Some("SimSun".to_string()),
                ..Default::default()
            },
            &mut trace,
        );
        #[cfg(target_os = "linux")]
        let re = fc.query(
            &FcPattern {
                family: Some("CJK".to_string()),
                ..Default::default()
            },
            &mut trace,
        );

        if let Some(result) = re
            && let Some(source) = fc.get_font_by_id(&result.id)
        {
            cc.add_font(FontInsert::new(
                "example",
                FontData::from_owned(match source {
                    rust_fontconfig::FontSource::Disk(path) => std::fs::read(&path.path).unwrap(),
                    rust_fontconfig::FontSource::Memory(b) => b.bytes.clone(),
                }),
                vec![
                    egui::epaint::text::InsertFontFamily {
                        family: egui::FontFamily::Proportional,
                        priority: egui::epaint::text::FontPriority::Highest,
                    },
                    egui::epaint::text::InsertFontFamily {
                        family: egui::FontFamily::Monospace,
                        priority: egui::epaint::text::FontPriority::Lowest,
                    },
                ],
            ));
        }

        res
    }
}

fn setup_fonts(ctx: &egui::Context, dir: &PathBuf) -> Vec<FontInner> {
    let before = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|f| f.as_secs())
        .unwrap_or_default();
    let mut font_data = Vec::new();
    if let Ok(d) = std::fs::read_dir(dir) {
        for ele in d {
            if let Ok(ele) = ele.map(|e| e.path())
                && ele.is_file()
            {
                let p = format!("{}", ele.display());
                if p.ends_with(".otf") || p.ends_with(".ttf") || p.ends_with(".ttc") {
                    font_data.push((format!("f_{}_{}", font_data.len(), before), p));
                }
            }
        }
    }

    let fonts = FontDefinitions::default();

    // 这里需要替换为实际的中文字体文件路径
    // 假设我们有三种不同的中文字体
    // let font_data = [
    //     (
    //         "noto_sans",
    //         "/Users/inkbox/project/fontview/HarmonyOS_Sans_SC_Thin.ttf",
    //     ),
    //     (
    //         "source_han_serif",
    //         "/Users/inkbox/project/fontview/JingNanBoBoHei-Bold-2.ttf",
    //     ),
    //     (
    //         "siyuan_heiti",
    //         "/Users/inkbox/project/fontview/YouSheYuFeiTeJianKangTi-2.ttf",
    //     ),
    // ];
    let mut fm = Vec::new();
    for (font_name, font_path) in font_data.iter() {
        // 在实际使用中，需要通过 include_bytes! 或文件读取加载字体
        // 这里使用占位符演示结构
        // fonts.font_data.insert(
        //     font_name.to_string(),
        //     Arc::new(egui::FontData::from_owned(
        //         std::fs::read(font_path).expect("read fail"),
        //     )), // 替换为实际字体数据
        // );
        let fmn = font_name.to_string();
        // fonts.families.insert(
        //     FontFamily::Name(fmn.clone().into()),
        //     vec![font_name.to_string()],
        // );

        let cow: std::borrow::Cow<'_, [u8]> =
            std::borrow::Cow::Owned(std::fs::read(font_path).expect("read fail"));
        let font_name_real = dump(&cow.clone());
        if font_name_real.is_empty() {
            continue;
        }
        ctx.add_font(FontInsert::new(
            font_name,
            FontData {
                font: cow.clone(),
                index: 0,
                tweak: Default::default(),
            },
            // egui::FontData::from_owned(std::fs::read(font_path).expect("read fail")),
            vec![egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Name(fmn.clone().into()),
                priority: egui::epaint::text::FontPriority::Lowest,
            }],
        ));

        fm.push(FontInner {
            font_name: font_name_real,
            path: font_path.clone(),
            mock_name: font_name.into(),
            file_name: format!(
                "{:?}",
                std::path::Path::new(font_path)
                    .file_name()
                    .unwrap_or_default()
            ),
        });
    }

    fm
}
impl eframe::App for FontViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.text_edit_singleline(&mut self.example).w;
            ui.add(TextEdit::singleline(&mut self.example).desired_width(f32::INFINITY));
            ui.horizontal(|ui| {
                ui.heading("font file dir: ");
                ui.label(&self.dir);
                if ui.button("Dir").clicked() {
                    let cc = ctx.clone();
                    let sx = Arc::new(self.sx.clone());
                    thread::spawn(move || {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            let dir_s = format!("{:?}", dir.display()).replace("\"", "");

                            let _ = sx.send(Msg::Dir(dir_s));
                            cc.request_repaint();

                            let font = setup_fonts(&cc, &dir);
                            if !font.is_empty() {
                                loop {
                                    let r = font.iter().any(|ele| {
                                        !cc.fonts(|f| {
                                            f.lock()
                                                .fonts
                                                .definitions()
                                                .font_data
                                                .contains_key(ele.mock_name.as_str())
                                        })
                                    });
                                    if r {
                                        break;
                                    }
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                            }
                            let _ = sx.send(Msg::Font(font));
                            cc.request_repaint();
                        } else {
                            let _ = sx.send(Msg::Cancel);
                            cc.request_repaint();
                        }
                    });
                }
            });

            if let Ok(r) = self.rx.try_recv() {
                match r {
                    Msg::Dir(dir) => {
                        self.dir = dir;
                        self.loading = true;
                        // ctx.request_repaint();
                    }
                    Msg::Font(font) => {
                        self.font = font;
                        self.loading = false;
                        // ctx.request_repaint();
                    }
                    Msg::Cancel => {
                        self.loading = false;
                    }
                }
            }

            if !self.dir.is_empty() {
                if self.loading {
                    ui.label("loading....");
                } else if self.font.is_empty() {
                    ui.label("no font");
                } else {
                    ScrollArea::vertical()
                        .auto_shrink(false)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                        )
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                                |ui| {
                                    if !self.loading {
                                        for ele in self.font.iter() {
                                            view_panel(ui, &self.example, ele, |ui, font| {
                                                // self.subset.show(ui.ctx(), font);
                                                self.subset_open = true;
                                                self.subset.font = Some(font.clone());
                                                self.subset.text = self.example.clone();
                                            });
                                        }
                                    }
                                },
                            );
                        });
                }
            }
        });
        if self.subset_open {
            self.subset.show(ctx, &mut self.subset_open);
        }
    }
}

fn view_panel(
    ui: &mut Ui,
    example: &str,
    fname: &FontInner,
    sub: impl FnOnce(&mut Ui, &FontInner),
) {
    egui::Frame::default()
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
        .show(ui, |ui| {
            ui.label(RichText::new(example).font(FontId::new(
                25.0,
                FontFamily::Name(fname.mock_name.to_string().into()),
            )));
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("子集化").clicked() {
                    sub(ui, fname);
                }
                if ui
                    .label(format!("[{}]", fname.font_name))
                    .on_hover_cursor(egui::CursorIcon::Copy)
                    .clicked()
                {
                    ui.ctx().copy_text(fname.font_name.clone());
                };
                ui.label(fname.file_name.as_str());
            });
        });
    ui.add_space(15.);
}

#[derive(Default)]
struct SubsetModal {
    text: String,
    font: Option<FontInner>,
    notify_modal: bool,
    result: Option<String>,
}

impl SubsetModal {
    fn notify_modal(&mut self, ui: &mut Ui) {
        let Self {
            notify_modal,
            result,
            ..
        } = self;
        if *notify_modal {
            let modal = Modal::new(Id::new(if result.is_some() { "Success" } else { "fail" }))
                .show(ui.ctx(), |ui| {
                    ui.set_width(200.0);
                    ui.heading(if result.is_some() { "Success" } else { "fail" });

                    ui.add_space(32.0);
                    if ui.button("Ok").clicked() {
                        ui.close();
                    }
                });

            if modal.should_close() {
                *notify_modal = false;
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.text_edit_multiline(&mut self.text);

        if ui.button("确认").clicked() {
            // 子集化
            if let Some(f) = &self.font
                && let Some(out) = rfd::FileDialog::new()
                    .set_file_name(f.file_name.as_str().replace("\"", ""))
                    .save_file()
                && !self.text.is_empty()
            {
                let buffer = std::fs::read(f.path.as_str()).unwrap();
                let font_file = allsorts::binary::read::ReadScope::new(&buffer)
                    .read::<allsorts::font_data::FontData>()
                    .unwrap();
                let provider = font_file.table_provider(0).unwrap();

                self.notify_modal = true;
                self.result = font_info::subset_text(&provider, &self.text, &out);
                // ui.close();
            }
        }

        self.notify_modal(ui);
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        let w = Window::new("subset").open(open).resizable(true);
        w.show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

mod font_info {
    use std::convert::TryFrom;
    use std::io::Write;
    use std::path::PathBuf;
    use std::str;

    use allsorts::gsub::{GlyphOrigin, RawGlyph, RawGlyphFlags};
    use allsorts::subset::SubsetProfile;

    use allsorts::binary::read::{ReadScope, ReadScopeOwned};
    use allsorts::error::ParseError;
    use allsorts::font_data::FontData;
    use allsorts::tables::{FontTableProvider, NameTable, OffsetTable, OpenTypeData, TTCHeader};
    use allsorts::tag::{self};
    use allsorts::woff::WoffFont;
    use allsorts::woff2::Woff2Font;

    pub type BoxError = Box<dyn std::error::Error>;
    ///
    /// 字体子集化
    ///
    pub(crate) fn subset_text<F: FontTableProvider>(
        font_provider: &F,
        text: &str,
        output_path: &PathBuf,
    ) -> Option<String> {
        let text = format!("{text}?◻"); // 添加两个占位符，用于字符不存在时渲染，避免完全不渲染的空白

        match do_subset_text(
            font_provider,
            remove_duplicate_chars(&text).as_str(),
            output_path,
        ) {
            Ok(v) => String::from_utf8(v).ok(),
            Err(e) => {
                s_error!("subset fail {:?}", e);
                None
            }
        }
    }
    /// 文本去重
    fn remove_duplicate_chars(input: &str) -> String {
        let mut seen = std::collections::HashSet::new();
        let mut result = String::new();

        for c in input.chars() {
            if !seen.contains(&c) {
                seen.insert(c);
                result.push(c);
            }
        }

        result
    }

    /// 随机数算法
    fn lcg(seed: u32) -> u32 {
        let a: u64 = 1664525;
        let c: u64 = 1013904223;
        let m: u64 = 1 << 32;
        ((a as u64 * seed as u64 + c) % m) as u32
    }

    fn do_subset_text<F: FontTableProvider>(
        font_provider: &F,
        text: &str,
        output_path: &PathBuf,
    ) -> Result<Vec<u8>, BoxError> {
        // Work out the glyphs we want to keep from the text
        let mut glyphs = chars_to_glyphs(font_provider, text).unwrap();
        let notdef = RawGlyph {
            unicodes: allsorts::tinyvec::tiny_vec![],
            glyph_index: 0,
            liga_component_pos: 0,
            glyph_origin: GlyphOrigin::Direct,
            flags: RawGlyphFlags::empty(),
            variation: None,
            extra_data: (),
        };
        glyphs.insert(0, Some(notdef));

        let mut glyphs: Vec<RawGlyph<()>> = glyphs.into_iter().flatten().collect();
        glyphs.sort_by(|a, b| a.glyph_index.cmp(&b.glyph_index));
        let mut glyph_ids = glyphs
            .iter()
            .map(|glyph| glyph.glyph_index)
            .collect::<Vec<_>>();
        glyph_ids.dedup();
        if glyph_ids.is_empty() {
            panic!("no glyphs left in font");
        }

        s_info!("Number of glyphs in new font: {}", glyph_ids.len());

        // Subset
        let mut new_font = allsorts::subset::subset(
            font_provider,
            &glyph_ids,
            &SubsetProfile::Minimal,
            allsorts::subset::CmapTarget::Unrestricted,
        )?;

        let name = do_dump(new_font.as_slice())?;
        let mut REP = Vec::new();
        if let Some(name) = name.1 {
            // 修改name
            let V = b"QWERTYUIOPASDFGHJKLMNBVCXZ";

            let mut seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u32;
            let len = name.scope().data().len();

            for i in 0..len {
                seed = lcg(seed);
                let s = seed % V.len() as u32;
                let t = V[s as usize];
                new_font[name.base + i] = t;
                REP.push(t);
            }
        }

        // Write out the new font
        let mut output = std::fs::File::create(output_path)?;
        output.write_all(&new_font)?;

        Ok(REP)
    }

    fn chars_to_glyphs<F: FontTableProvider>(
        font_provider: &F,
        text: &str,
    ) -> Result<Vec<Option<RawGlyph<()>>>, BoxError> {
        let cmap_data = font_provider.read_table_data(allsorts::tag::CMAP)?;
        let cmap = allsorts::binary::read::ReadScope::new(&cmap_data)
            .read::<allsorts::tables::cmap::Cmap>()?;
        let (_, cmap_subtable) = allsorts::font::read_cmap_subtable(&cmap)?.ok_or("fail")?;

        let glyphs = text
            .chars()
            .map(|ch| map(&cmap_subtable, ch, None))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(glyphs)
    }
    fn map(
        cmap_subtable: &allsorts::tables::cmap::CmapSubtable,
        ch: char,
        variation: Option<allsorts::unicode::VariationSelector>,
    ) -> Result<Option<RawGlyph<()>>, allsorts::error::ParseError> {
        if let Some(glyph_index) = cmap_subtable.map_glyph(ch as u32)? {
            let glyph = make(ch, glyph_index, variation);
            Ok(Some(glyph))
        } else {
            Ok(None)
        }
    }
    fn make(
        ch: char,
        glyph_index: u16,
        variation: Option<allsorts::unicode::VariationSelector>,
    ) -> RawGlyph<()> {
        RawGlyph {
            unicodes: allsorts::tinyvec::tiny_vec![[char; 1] => ch],
            glyph_index,
            liga_component_pos: 0,
            glyph_origin: GlyphOrigin::Char(ch),
            flags: RawGlyphFlags::empty(),
            variation,
            extra_data: (),
        }
    }
    pub(crate) fn dump(data: &[u8]) -> String {
        match do_dump(data) {
            Ok(v) => v.0,
            Err(e) => {
                s_error!("dump error {:?}", e);
                String::new()
            }
        }
    }
    fn do_dump(data: &[u8]) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        let scope = ReadScope::new(data);
        let font_file = scope.read::<FontData>()?;

        match &font_file {
            FontData::OpenType(font_file) => match &font_file.data {
                OpenTypeData::Single(ttf) => dump_ttf(&font_file.scope, ttf),
                OpenTypeData::Collection(ttc) => dump_ttc(&font_file.scope, ttc),
            },
            FontData::Woff(woff_file) => dump_woff(woff_file),
            FontData::Woff2(woff_file) => dump_woff2(woff_file, 0),
        }
    }

    fn dump_ttc<'a>(
        scope: &ReadScope<'a>,
        ttc: &TTCHeader<'a>,
    ) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        if let Some(offset_table_offset) = (&ttc.offset_tables).into_iter().next() {
            let offset_table_offset =
                usize::try_from(offset_table_offset).map_err(ParseError::from)?;
            let offset_table = scope.offset(offset_table_offset).read::<OffsetTable>()?;
            return dump_ttf(scope, &offset_table);
        }
        Ok((String::new(), None))
    }

    fn dump_ttf<'a>(
        scope: &ReadScope<'a>,
        ttf: &OffsetTable<'a>,
    ) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        if let Some(name_table_data) = ttf.read_table(scope, tag::NAME)? {
            let name_table = name_table_data.read::<NameTable>()?;
            return dump_name_table(&name_table);
        }

        Ok((String::new(), None))
    }

    fn dump_woff(woff: &WoffFont<'_>) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        if let Some(entry) = woff
            .table_directory
            .iter()
            .find(|entry| entry.tag == tag::NAME)
        {
            let table = entry.read_table(&woff.scope)?;
            let name_table = table.scope().read::<NameTable>()?;
            return dump_name_table(&name_table);
        }

        Ok((String::new(), None))
    }

    fn dump_woff2<'a>(
        woff: &Woff2Font<'a>,
        index: usize,
    ) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        if let Some(table) = woff.read_table(tag::NAME, index)? {
            s_info!();
            let name_table = table.scope().read::<NameTable>()?;
            return dump_name_table(&name_table);
        }

        Ok((String::new(), None))
    }
    fn dump_name_table(
        name_table: &allsorts::tables::NameTable,
    ) -> Result<(String, Option<ReadScopeOwned>), BoxError> {
        use encoding_rs::{MACINTOSH, UTF_16BE};
        for name_record in &name_table.name_records {
            let platform = name_record.platform_id;
            let encoding = name_record.encoding_id;
            let language = name_record.language_id;
            let offset = usize::from(name_record.offset);
            let length = usize::from(name_record.length);
            let name_scope = name_table.string_storage.offset_length(offset, length)?;
            let name_data = name_scope.data();

            // s_info!(
            //     "offset={}, length = {length},{:?}",
            //     name_table.string_storage.base + offset,
            //     name_data
            // );
            let name = match (platform, encoding) {
                (0, _) => decode(UTF_16BE, name_data),
                (1, 0) => decode(MACINTOSH, name_data),
                (3, 0) => decode(UTF_16BE, name_data),
                (3, 1) => decode(UTF_16BE, name_data),
                (3, 10) => decode(UTF_16BE, name_data),
                _ => format!(
                    "(unknown platform={} encoding={} language={})",
                    platform, encoding, language
                ),
            };
            if let NameTable::FULL_FONT_NAME = name_record.name_id {
                return Ok((name, Some(ReadScopeOwned::new(name_scope))));
            }
        }
        Ok((String::new(), None))
    }

    fn decode(encoding: &'static encoding_rs::Encoding, data: &[u8]) -> String {
        let mut decoder = encoding.new_decoder();
        if let Some(size) = decoder.max_utf8_buffer_length(data.len()) {
            let mut s = String::with_capacity(size);
            let (_res, _read, _repl) = decoder.decode_to_string(data, &mut s, true);
            s
        } else {
            String::new() // can only happen if buffer is enormous
        }
    }
}
