use bevy::prelude::AppExit;

fn main() -> AppExit {
    match n_side::app::run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{error}");
            AppExit::error()
        }
    }
}
