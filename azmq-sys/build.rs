use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");

    #[cfg(feature = "draft-api")]
    zeromq_src::Build::new()
        .with_libsodium(None)
        .enable_draft(true)
        .build();

    #[cfg(not(feature = "draft-api"))]
    zeromq_src::Build::new().with_libsodium(None).build();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_dir.join("source").join("include");

    let builder = bindgen::Builder::default()
        .header(include_dir.join("zmq.h").to_string_lossy())
        .size_t_is_usize(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .derive_debug(true)
        .derive_hash(true)
        .use_core()
        .allowlist_function("^zmq_.*")
        .allowlist_type("^zmq_.*")
        .allowlist_var("^ZMQ_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    #[cfg(feature = "draft-api")]
    let builder = builder.clang_args(vec!["-DZMQ_BUILD_DRAFT_API=1"]);

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
