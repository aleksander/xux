pub use state::State;

pub trait Ai {
    fn update(&mut self, state: &mut State);
    fn exec(&mut self, s: &str);
    fn init(&mut self);
    fn new() -> Self;
}
