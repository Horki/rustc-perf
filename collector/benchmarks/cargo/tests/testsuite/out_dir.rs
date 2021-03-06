use std::path::Path;
use std::fs::{self, File};
use std::env;

use hamcrest::assert_that;

use cargotest::{process, sleep_ms, ChannelChanger};
use cargotest::support::{execs, project};

#[test]
fn binary_with_debug() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("src/main.rs", r#"fn main() { println!("Hello, World!") }"#)
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    check_dir_contents(
        &p.root().join("out"),
        &["foo"],
        &["foo", "foo.dSYM"],
        &["foo.exe", "foo.pdb"],
    );
}

#[test]
fn static_library_with_debug() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            crate-type = ["staticlib"]
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #[no_mangle]
            pub extern "C" fn foo() { println!("Hello, World!") }
        "#,
        )
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    check_dir_contents(
        &p.root().join("out"),
        &["libfoo.a"],
        &["libfoo.a"],
        &["foo.lib"],
    );
}

#[test]
fn dynamic_library_with_debug() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            crate-type = ["cdylib"]
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            #[no_mangle]
            pub extern "C" fn foo() { println!("Hello, World!") }
        "#,
        )
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    check_dir_contents(
        &p.root().join("out"),
        &["libfoo.so"],
        &["libfoo.dylib"],
        &["foo.dll", "foo.dll.lib"],
    );
}

#[test]
fn rlib_with_debug() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [lib]
            crate-type = ["rlib"]
        "#,
        )
        .file(
            "src/lib.rs",
            r#"
            pub fn foo() { println!("Hello, World!") }
        "#,
        )
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    check_dir_contents(
        &p.root().join("out"),
        &["libfoo.rlib"],
        &["libfoo.rlib"],
        &["libfoo.rlib"],
    );
}

#[test]
fn include_only_the_binary_from_the_current_package() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [workspace]

            [dependencies]
            utils = { path = "./utils" }
        "#,
        )
        .file("src/lib.rs", "extern crate utils;")
        .file(
            "src/main.rs",
            r#"
            extern crate foo;
            extern crate utils;
            fn main() {
                println!("Hello, World!")
            }
        "#,
        )
        .file(
            "utils/Cargo.toml",
            r#"
            [project]
            name = "utils"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("utils/src/lib.rs", "")
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --bin foo --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    check_dir_contents(
        &p.root().join("out"),
        &["foo"],
        &["foo", "foo.dSYM"],
        &["foo.exe", "foo.pdb"],
    );
}

#[test]
fn out_dir_is_a_file() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("src/main.rs", r#"fn main() { println!("Hello, World!") }"#)
        .build();
    File::create(p.root().join("out")).unwrap();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs()
            .with_status(101)
            .with_stderr_contains("[ERROR] failed to link or copy [..]"),
    );
}

#[test]
fn replaces_artifacts() {
    let p = project("foo")
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("src/main.rs", r#"fn main() { println!("foo") }"#)
        .build();

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    assert_that(
        process(&p.root()
            .join(&format!("out/foo{}", env::consts::EXE_SUFFIX))),
        execs().with_stdout("foo"),
    );

    sleep_ms(1000);
    p.change_file("src/main.rs", r#"fn main() { println!("bar") }"#);

    assert_that(
        p.cargo("build -Z unstable-options --out-dir out")
            .masquerade_as_nightly_cargo(),
        execs().with_status(0),
    );
    assert_that(
        process(&p.root()
            .join(&format!("out/foo{}", env::consts::EXE_SUFFIX))),
        execs().with_stdout("bar"),
    );
}

fn check_dir_contents(
    out_dir: &Path,
    expected_linux: &[&str],
    expected_mac: &[&str],
    expected_win: &[&str],
) {
    let expected = if cfg!(target_os = "windows") {
        expected_win
    } else if cfg!(target_os = "macos") {
        expected_mac
    } else {
        expected_linux
    };

    let actual = list_dir(out_dir);
    let mut expected = expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected);
}

fn list_dir(dir: &Path) -> Vec<String> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        res.push(entry.file_name().into_string().unwrap());
    }
    res.sort();
    res
}
