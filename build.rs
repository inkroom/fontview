use std::{error::Error, fmt::format, process::Command};

#[cfg(target_os = "windows")]
extern crate winres;

fn download_ffmpeg() -> Result<String, Box<dyn Error>> {
    let out = std::env::var("OUT_DIR")?;
    let ffmpeg = format!("{}/ffmpeg.zip", out);

    if std::fs::exists("ffmpeg-release-essentials.zip")? {
        std::fs::copy("ffmpeg-release-essentials.zip", &ffmpeg)?;
    } else {
        let mut resp = reqwest::blocking::Client::new()
            .get("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip")
            .send()?;
        let mut fs = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&ffmpeg)?;
        resp.copy_to(&mut fs)?;
    }
    //
    let mut z = zip::ZipArchive::new(std::fs::File::open(&ffmpeg)?)?;
    let ff = z
        .file_names()
        .find(|s| s.ends_with("ffmpeg.exe"))
        .map(|f| f.to_string());
    if let Some(ff) = ff {
        let mut t = z.by_name(ff.as_str())?;
        let mut tt = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(format!("{}/ffmpeg.exe", out))?;
        std::io::copy(&mut t, &mut tt)?;
        println!("download ffmpeg success, location: {}/ffmpeg.exe", out);
    }
    Ok(format!("{}/ffmpeg.exe", out))
}

fn main() {
    let target = "allsorts/src/binary/read.rs";
    // 修改本地代码
    let s = std::fs::read_to_string(&target).expect("please run [git submodule update --remote]");

    std::fs::write(
        &target,
        s.replace("    base: usize,", "    pub base: usize,"),
    )
    .expect("update read.rs fail");

    #[cfg(target_os = "windows")]
    {
        #[cfg(not(debug_assertions))]
        {
            let out = std::env::var("OUT_DIR").unwrap();
            let ff = download_ffmpeg().expect("download ffmpeg fail");

            let ico = format!("{}/windows.ico", out);

            let mut c = Command::new(ff.as_str())
                .arg("-i")
                .arg("img/ico.png")
                .arg("-vf")
                .arg("scale=48:48")
                .arg(ico.as_str())
                .spawn()
                .unwrap();

            let ex = c.wait().unwrap();

            if !ex.success() {
                panic!("convert windows ico fail");
            }
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico.as_str());
            res.compile().unwrap();
        }
    }
}
