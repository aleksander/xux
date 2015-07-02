pub use salem::client::Client;

pub trait Ai {
    fn update (&mut self, client: &mut Client);
    fn exec (&mut self, s: &str);
}

