use state::State;
use ai::Ai;

#[derive(Debug)]
enum AiState {
    WaitForCharList,
    WaitForWorld,
    Walking,
    Walking1,
    Walking2,
    WaitForEnd,
}

#[derive(Debug)]
enum Step {
    A,
    B,
    C,
    D,
}

pub struct AiDecl {
    state: AiState,
    step: Step,
    cycle: usize,
}

impl AiDecl {
    pub fn new() -> AiDecl {
        AiDecl {
            state: AiState::WaitForCharList,
            step: Step::A,
            cycle: 0,
        }
    }
}

impl Ai for AiDecl {
    fn update(&mut self, state: &mut State) /*TODO -> Result<> */ {
        info!("AI: {:?}", self.state);
        match self.state {
            AiState::WaitForCharList => {
                if !state.charlist.is_empty() {
                    state.send_play(0).expect("ai waitforcharlist state.send_play");
                    self.state = AiState::WaitForWorld;
                }
            }
            AiState::WaitForWorld => {
                if state.widget_exists("mapview", None) && state.hero_exists() && state.hero_grid_exists() && !state.hero_is_moving() {
                    self.state = AiState::Walking;
                }
            }
            AiState::Walking => {
                match state.hero_xy() {
                    Some((x, y)) => {
                        let (dx, dy) = match self.step {
                            Step::A => (10, 0),
                            Step::B => (0, 10),
                            Step::C => (-100, 0),
                            Step::D => (0, -1000),
                            //Step::A => (100, 0),
                            //Step::B => (0, 100),
                            //Step::C => (-100, 0),
                            //Step::D => (0, -100),
                        };
                        if let Step::D = self.step {
                            self.cycle += 1;
                        }
                        self.step = match self.step {
                            Step::A => Step::B,
                            Step::B => Step::C,
                            Step::C => Step::D,
                            Step::D => Step::A,
                        };
                        state.go(x + dx, y + dy).expect("ai walking state.go");
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
                    if self.cycle < 400 {
                        self.state = AiState::Walking;
                    } else {
                        state.close().expect("ai walking2 state.close");
                        self.state = AiState::WaitForEnd;
                    }
                }
            }
            AiState::WaitForEnd => {}
        }
    }

    fn exec(&mut self, s: &str) {
        info!("AI: EXEC: {}", s);
    }

    fn init(&mut self) {
        info!("AI: INIT");
    }

    fn new() -> AiDecl {
        Self::new()
    }
}
