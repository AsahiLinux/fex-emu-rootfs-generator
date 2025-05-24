use std::collections::HashMap;
use std::fs::File;
use std::fs::{create_dir_all, read_dir};
use std::io::Write;
use std::path::Path;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

static LAYERS_DIR: &str = "/usr/share/fex-emu/layers";
static MOUNTS_DIR: &str = "/var/lib/fex-emu/layers";
static ROOTFS_DIR: &str = "/var/lib/fex-emu/rootfs";
static WORK_DIR: &str = "/var/lib/fex-emu/workdir";
static WRITEABLE_DIR: &str = "/var/lib/fex-emu/writable";

fn systemd_escape_path(name: &Path) -> String {
    return format!(
        "{}.mount",
        libsystemd::unit::escape_path(name.as_os_str().to_str().unwrap())
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging; we use SYSTEMD_LOG_LEVEL as that's the convention for
    // generators per systemd.generator(7)
    let logger_env = env_logger::Env::default().filter_or("SYSTEMD_LOG_LEVEL", "info");

    env_logger::Builder::from_env(logger_env)
        .format_level(false)
        .format_timestamp(None)
        .init();

    // /path/to/generator normal-dir [early-dir] [late-dir]
    let args: Vec<String> = std::env::args().collect();
    trace!("Args: {:?}", args);
    let dest_path = Path::new(&args[1]);
    let mut layers = HashMap::new();

    let layer_dirs = read_dir(LAYERS_DIR)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    for layer in layer_dirs.iter() {
        let stem = layer.file_stem().unwrap().to_str().unwrap();
        layers.insert(stem, layer);
    }

    trace!("layers = {:?}", layers);

    let mounts_path = Path::new(MOUNTS_DIR);

    let mut units = HashMap::new();
    for (stem, _) in &layers {
        units.insert(stem, systemd_escape_path(mounts_path.join(stem).as_path()));
    }
    trace!("units = {:?}", units);

    for (stem, unit) in &units {
        let unit_path = dest_path.join(&unit);
        let mut unit = File::create(&unit_path).unwrap();
        writeln!(
            unit,
            "[Unit]
Description=FEX RootFS layer for {}

[Mount]
What={}
Where={}",
            stem,
            layers[*stem].to_string_lossy(),
            mounts_path.join(stem).to_string_lossy()
        )?;
    }

    let mut stems: Vec<_> = layers.keys().collect::<Vec<_>>();
    stems.sort();
    trace!("stems = {:?}", stems);

    let rootfs_unit_name = systemd_escape_path(Path::new(ROOTFS_DIR));
    let rootfs_unit_path = dest_path.join(&rootfs_unit_name);
    let mut rootfs_unit = File::create(&rootfs_unit_path).unwrap();
    let rootfs_unit_deps: Vec<_> = stems.iter().map(|stem| units[stem].clone()).collect();
    trace!("rootfs_unit_deps = {:?}", rootfs_unit_deps);
    stems.reverse();
    let overlay_mounts: Vec<_> = stems
        .iter()
        .map(|stem| mounts_path.join(stem).to_string_lossy().to_string())
        .collect();
    trace!("overlay_mounts = {:?}", overlay_mounts);

    writeln!(
        rootfs_unit,
        "[Unit]
Description=FEX RootFS
BindsTo={}
After={}

[Mount]
What=overlay
Where={}
Type=overlay
Options=lowerdir={},upperdir={},workdir={}

[Install]
WantedBy=multi-user.target",
        rootfs_unit_deps.join(" "),
        rootfs_unit_deps.join(" "),
        ROOTFS_DIR,
        overlay_mounts.join(":"),
        WRITEABLE_DIR,
        WORK_DIR
    )?;

    let wanted_dir_path = dest_path.join("multi-user.target.wants");
    create_dir_all(&wanted_dir_path).unwrap();
    let link_path = wanted_dir_path.join(&rootfs_unit_name);
    let _ = std::os::unix::fs::symlink(Path::new("..").join(rootfs_unit_name), link_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_escape_path() {
        let path = Path::new("/foo/bar/fex-emu/rootfs");
        let escaped = systemd_escape_path(path);
        assert_eq!(escaped, "foo-bar-fex\\x2demu-rootfs.mount");
    }
}
