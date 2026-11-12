// Pone el ícono de la app en el .exe de Windows. En otros sistemas no hace nada.
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("no se pudo incrustar el ícono de Windows");
    }
}
