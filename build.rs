use cc;

fn main() {
    cc::Build::new()
        .file("src/data_src/tornadoSrc.c")
        .compile("rusty_opengl_prog");
}