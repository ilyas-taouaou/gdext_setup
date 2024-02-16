use std::path::Path;
use std::process::Command;
use toml_edit::{value, Array, Document};

fn run(command: &str, args: &[&str], current_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    Command::new(command)
        .args(args)
        .current_dir(current_dir)
        .spawn()?
        .wait()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let plugin_name = args.next().unwrap_or("new_plugin".to_string());
    let mut path = std::env::current_dir().unwrap();
    let base_path = path.clone();
    path.push("addons/gdextensions");
    std::fs::create_dir_all(&path)?;
    run("cargo", &["new", "--lib", &plugin_name], &path)?;
    path.push(&plugin_name);
    run(
        "cargo",
        &[
            "add",
            "--git",
            "https://github.com/godot-rust/gdext",
            "--branch",
            "master",
            "godot",
        ],
        &path,
    )?;

    let mut cargo_toml = std::fs::read_to_string(path.join("Cargo.toml"))?.parse::<Document>()?;
    cargo_toml["lib"] = toml_edit::table();
    cargo_toml["lib"].as_table_mut().unwrap().set_position(1);
    let mut array = Array::default();
    array.push("cdylib");
    cargo_toml["lib"]["crate-type"] = value(array);
    std::fs::write(path.join("Cargo.toml"), cargo_toml.to_string())?;

    std::fs::write(path.join("src/lib.rs"), include_str!("res/lib.txt"))?;
    run("cargo", &["build"], &path)?;

    let gdextension_toml = include_str!("res/gdextension.toml");

    let re = regex::Regex::new(r"\{\{(.*?)\}\}").unwrap();
    let s = String::from("addons/gdextensions/") + &plugin_name;

    let x = re.replace_all(gdextension_toml, |caps: &regex::Captures| {
        let cap = caps.get(1).unwrap().as_str();
        match cap {
            "name" => &plugin_name,
            "path" => &s,
            _ => unreachable!(),
        }
        .to_string()
    });

    std::fs::write(
        base_path.join(format!("{}.gdextension", plugin_name)),
        x.to_string(),
    )?;

    std::fs::File::create(base_path.join("addons/gdextensions/.gdignore"))?;

    Ok(())
}
