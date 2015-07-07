use salem::state::State;
use ai::Ai;

struct DeclAi {
    useless: u64,
}

impl DeclAi {
    fn new () -> DeclAi {
        DeclAi {useless:0}
    }
}

impl Ai for DeclAi {
    fn update (&mut self, /*state*/_: &mut State) {
        println!("PRINT THIS AND DO NOTHING");
    }

    fn exec (&mut self, s: &str) {
        println!("EXEC: {}", s);
    }
    
    fn init (&mut self) {
        println!("INIT");
        self.useless = 42;
    }
    
    fn new () -> DeclAi {
        Self::new()
    }
}


