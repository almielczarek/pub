use gotham::router::builder::*;

mod handlers;
mod pub_static;

use handlers::static_handler;

pub fn main() {
    gotham::start(
        "0.0.0.0:8080",
        build_simple_router(|route| {
            route.get("/").to(static_handler);
            route.get("*").to(static_handler);

            println!("Serving on 0.0.0.0:8080");
        }),
    );
}
