fn main() {
    // X11 is the core.
    println!("cargo:rustc-link-lib=X11");

    // Xss is the 'XScreenSaver' extension, used to get the idle time
    // of the user.
    println!("cargo:rustc-link-lib=Xss");
}
