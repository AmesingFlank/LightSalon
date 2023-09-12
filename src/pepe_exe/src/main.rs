mod app;

use app::App;

use pepe_core;

fn main() {
    let mut app = pollster::block_on(App::new());
    app.main();
}
