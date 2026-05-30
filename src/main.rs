use candybox_gfx::App;


fn main() {
    let mut app = App::new();
    
    match app.run() {
        Ok(()) => {},
        Err(error) => panic!("{}", error)
    }
}
