#[cfg(target_os = "windows")]
extern crate winres;

fn main() {
  #[cfg(target_os = "windows")]
   {
    let mut res = winres::WindowsResource::new();
    res.set_icon("windows.ico");
    res.compile().unwrap();
  }
  let target= "allsorts/src/binary/read.rs";
  // 修改本地代码
  let s = std::fs::read_to_string(&target).expect("please run [git submodule update --remote]");

  std::fs::write(&target  , s.replace("    base: usize,", "    pub base: usize,")).expect("update read.rs fail");  


}