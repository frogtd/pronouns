use futures::executor;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                if !(path
                    .iter()
                    .collect::<Vec<_>>()
                    .contains(&std::ffi::OsStr::new("min")))
                {
                    cb(&entry);
                }
            }
        }
    }
    Ok(())
}

fn minify_dir(dir: &str) {
    std::fs::create_dir(format!("{}/min", dir)).unwrap_or(());
    let kinda_working_offest = dir
        .chars()
        .filter(|a| return a == &'/' || a == &'\\')
        .count()
        + 1;
    visit_dirs(Path::new(dir), &|entry| {
        use minify_html::{copy, Cfg};
        use std::fs::DirBuilder;

        let mut builder = DirBuilder::new();
        builder.recursive(true);

        let path = entry.path();
        let mut out = path.iter().collect::<Vec<_>>();
        out.insert(kinda_working_offest, std::ffi::OsStr::new("min"));
        let out_path: std::path::PathBuf = out[0..(out.len() - 1)].iter().collect();
        builder.create(out_path).unwrap_or(());

        let out_path_file: std::path::PathBuf = out.iter().collect();
        if entry.path().extension() == Some(std::ffi::OsStr::new("html")) {
            std::fs::write(
                out_path_file,
                &copy(
                    &std::fs::read(entry.path()).unwrap(),
                    &Cfg {
                        minify_js: true,
                        minify_css: true,
                    },
                )
                .unwrap_or_else(|x| {
                    eprintln!("{:?}", x);
                    std::fs::read(entry.path()).unwrap()
                }),
            )
            .unwrap();
        } else if entry.path().extension() == Some(std::ffi::OsStr::new("css")) {
            let tranform_options = {
                let mut tranform_builder = esbuild_rs::TransformOptionsBuilder::new();
                tranform_builder.loader = esbuild_rs::Loader::CSS;
                tranform_builder.minify_whitespace = true;
                tranform_builder.build()
            };

            use std::sync::Arc;

            std::fs::write(
                out_path_file,
                executor::block_on(esbuild_rs::transform(
                    Arc::new(std::fs::read(entry.path()).unwrap()),
                    tranform_options,
                ))
                .code
                .as_str(),
            )
            .unwrap();
        } else {
            std::fs::write(out_path_file, std::fs::read(entry.path()).unwrap()).unwrap();
        }
    })
    .unwrap();
}

fn main() {
    minify_dir("./templates");
}
