mod app;

use app::App;

use pepe_core;

fn main() {
    pepe_core::f();
    let mut app = App {};
    app.main()
}
