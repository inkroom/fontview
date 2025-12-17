use std::{error::Error, fmt::format, process::Command};

#[cfg(target_os = "windows")]
extern crate winres;

#[cfg(target_os = "macos")]
fn download_ffmpeg() -> Result<String, Box<dyn Error>> {
    let out = std::env::var("OUT_DIR")?;
    let ffmpeg = format!("{}/ffmpeg.zip", out);

    if std::fs::exists("ffmpeg-release-essentials.zip")? {
        std::fs::copy("ffmpeg-release-essentials.zip", &ffmpeg)?;
    } else if !std::fs::exists(ffmpeg.as_str())? {
        let mut resp = reqwest::blocking::Client::new()
            .get(if cfg!(target_os = "windows") {
                "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
            } else {
                "https://evermeet.cx/ffmpeg/ffmpeg-122122-g3b1214a897.zip"
            })
            .send()?;
        let mut fs = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&ffmpeg)?;
        resp.copy_to(&mut fs)?;
    }
    let file_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let mut z = zip::ZipArchive::new(std::fs::File::open(&ffmpeg)?)?;
    let ff = z
        .file_names()
        .find(|s| s.ends_with(file_name))
        .map(|f| f.to_string());
    if let Some(ff) = ff {
        let mut t = z.by_name(ff.as_str())?;
        let mut tt = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(format!("{}/{file_name}", out))?;
        std::io::copy(&mut t, &mut tt)?;
        println!("download ffmpeg success, location: {}/{file_name}", out);
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        let m = std::fs::metadata(format!("{}/{file_name}", out)).unwrap();
        let mut p = m.permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(format!("{}/{file_name}", out), p).unwrap();
    }
    Ok(format!("{}/{file_name}", out))
}

#[cfg(target_os="windows")]
fn download_ffmpeg() -> Result<String, Box<dyn Error>> {
    let out = std::env::var("OUT_DIR")?;
    let ffmpeg = format!("{}/ffmpeg.7z", out);

    if std::fs::exists("ffmpeg-7.1.1-full_build.7z")? {
        std::fs::copy("ffmpeg-7.1.1-full_build.7z", &ffmpeg)?;
    } else if !std::fs::exists(ffmpeg.as_str())? {
        let mut resp = reqwest::blocking::Client::new()
            .get("https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-7.1.1-full_build.7z")
            .send()?;
        let mut fs = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&ffmpeg)?;
        resp.copy_to(&mut fs)?;
    }
    let file_name = "ffmpeg-7.1.1-full_build\\bin\\ffmpeg.exe";
    sevenz_rust::decompress_file(&ffmpeg, out.as_str()).unwrap();

    Ok(format!("{}\\{file_name}", out))
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
    let out = std::env::var("OUT_DIR").unwrap();
    #[cfg(target_os = "windows")]
    {

        {
            let ff = download_ffmpeg().expect("download ffmpeg fail");


            let ico = format!("{out}\\ico.png");

            let mut c = Command::new(ff.as_str())
                .arg("-i")
                .arg("img\\ico-1024x1024.png")
                .arg("-vf")
                .arg("scale=256x256")
                .arg("-y")
                .arg(ico.as_str())
                .spawn()
                .unwrap();

            let ex = c.wait().unwrap();

            if !ex.success() {
                panic!("convert ico fail");
            }


            let ico = format!("{}\\windows.ico", out);

            let mut c = Command::new(ff.as_str())
                .arg("-i")
                .arg("img\\ico-1024x1024.png")
                .arg("-vf")
                .arg("scale=48:48")
                .arg("-y")
                .arg(ico.as_str())
                .spawn()
                .unwrap();

            let ex = c.wait().unwrap();

            if !ex.success() {
                panic!("convert windows ico fail");
            }

            let mut res = winres::WindowsResource::new();
            res.set_icon(ico.as_str());
            res.compile().expect("compile ico fail");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let t = format!("{}/ico.png", out);
        std::fs::copy("img/ico-1024x1024.png", t.as_str()).unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        let ff = download_ffmpeg().expect("download ffmpeg fail");

        let ico = format!("{}/ico.png", out);

        let mut c = Command::new(ff.as_str())
            .arg("-i")
            .arg("img/ico-1024x1024.png")
            .arg("-vf")
            .arg("scale=256x256")
            .arg("-y")
            .arg(ico.as_str())
            .spawn()
            .unwrap();

        let ex = c.wait().unwrap();

        if !ex.success() {
            panic!("convert ico fail");
        }

        // mac bundle使用
        std::fs::copy(ico.as_str(), "img/ico.png").unwrap();
    }
}
