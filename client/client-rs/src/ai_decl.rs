use state::State;
use ai::Ai;

#[derive(Debug)]
enum AiState {
    WaitForCharList,
    WaitForWorld,
    Walking,
    Walking1,
    Walking2,
    WaitForEnd
}

#[derive(Debug)]
enum Step {
    A, B, C, D
}

pub struct DeclAi {
    state: AiState,
    step: Step,
}

impl DeclAi {
    fn new () -> DeclAi {
        DeclAi {
            state: AiState::WaitForCharList,
            step: Step::A
        }
    }
}

impl Ai for DeclAi {
    fn update (&mut self, state: &mut State) {
        println!("AI: {:?}", self.state);
        match self.state {
            AiState::WaitForCharList => {
                if state.charlist.len() > 0 {
                    state.send_play(0).unwrap();
                    self.state = AiState::WaitForWorld;
                }
            }
            AiState::WaitForWorld => {
                if state.widget_exists("mapview", None) &&
                   state.hero_exists() &&
                   state.hero_grid_exists() &&
                  !state.hero_is_moving() {
                    self.state = AiState::Walking;
                }
            }
            AiState::Walking => {
                match state.hero_xy() {
                    Some((x,y)) => {
                        let (dx,dy) = match self.step {
                            Step::A => (100,0),
                            Step::B => (0,100),
                            Step::C => (-100,0),
                            Step::D => (0,-100),
                        };
                        self.step = match self.step {
                            Step::A => Step::B,
                            Step::B => Step::C,
                            Step::C => Step::D,
                            Step::D => Step::A,
                        };
                        state.go(x + dx, y + dy);
                        self.state = AiState::Walking1;
                    }
                    None => {}
                }
            }
            AiState::Walking1 => {
                if state.hero_is_moving() {
                    self.state = AiState::Walking2;
                }
            }
            AiState::Walking2 => {
                if !state.hero_is_moving() {
                    self.state = AiState::Walking;
                }
            }
            AiState::WaitForEnd => {
            }
        }
    }

    fn exec (&mut self, s: &str) {
        println!("AI: EXEC: {}", s);
    }
    
    fn init (&mut self) {
        println!("AI: INIT");
    }
    
    fn new () -> DeclAi {
        Self::new()
    }
}

