use behavior_tree::{Node, Status, AlwaysRunning, boxed, Wait, AlwaysFailure};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use crate::XuxState;
use xux::proto::List;
use log::info;
use std::rc::Rc;

//
// login & fell-trees
//
// wait-gameui
//     ...
//
// create-a-new-character
//      ...

pub fn root (state: Rc<RefCell<XuxState>>) -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(login(state)),
        Box::new(fell_trees())
    ])
}

// login
//     wait-login-screen
//     &
//     login-existing-character | create-a-new-character
//
fn login (state: Rc<RefCell<XuxState>>) -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(wait_login_screen(state.clone())),
        Box::new(boxed::Selector::new([
            Box::new(login_existing_character(state)),
            Box::new(create_a_new_character()),
        ]))
    ])
}

// wait-login-screen
//     wait-widget-chain
//     &
//     wait-a-second
//
fn wait_login_screen (state: Rc<RefCell<XuxState>>) -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(wait_widget_chain(state, vec!("ccnt","charlist"))),
        Box::new(wait_second()),
    ])
}

//
// wait-widget-chain
//
fn wait_widget_chain (state: Rc<RefCell<XuxState>>, chain: Vec<&'static str>) -> WaitWidgetChain {
    WaitWidgetChain::new(state, chain)
}

struct WaitWidgetChain {
    state: Rc<RefCell<XuxState>>,
    chain: Vec<&'static str>,
}

impl WaitWidgetChain {
    fn new (state: Rc<RefCell<XuxState>>, chain: Vec<&'static str>) -> WaitWidgetChain {
        WaitWidgetChain { state, chain }
    }
}

impl Node for WaitWidgetChain {
    fn tick(&mut self) -> Status {
        if self.state.borrow().widgets.find_chain(&self.chain).is_some() {
            Status::Success
        } else {
            Status::Running
        }
    }
}

//
// wait-a-second
//
fn wait_second () -> Wait {
    wait(Duration::from_secs(1))
}

fn wait (duration: Duration) -> Wait {
    Wait::new(duration)
}

// login-existing-character
//      have-any-characters
//      &
//      choose-a-character
//      &
//      wait-gameui
//
fn login_existing_character (state: Rc<RefCell<XuxState>>) -> boxed::Sequence<3> {
    boxed::Sequence::new([
        Box::new(have_any_characters(state.clone())),
        Box::new(choose_a_character("Клёцка", state.clone())),
        Box::new(wait_gameui(state)),
    ])
}

// choose-a-character {
//     send msg::focus
//     send msg::play("name")
// }
//
fn have_any_characters(state: Rc<RefCell<XuxState>>) -> HaveAnyCharacters {
    HaveAnyCharacters { state }
}

struct HaveAnyCharacters {
    state: Rc<RefCell<XuxState>>
}

impl HaveAnyCharacters {
    fn new (state: Rc<RefCell<XuxState>>) -> HaveAnyCharacters {
        HaveAnyCharacters { state }
    }
}

impl Node for HaveAnyCharacters {
    fn tick(&mut self) -> Status {
        if let Some(ref charlist) = self.state.borrow().widgets.find_chain(&["ccnt","charlist"]) {
            let chars = charlist.messages.iter().filter(|&&(ref name,_)|name == "add").count();
            info!("AI: found {} characters on account", chars);
            if chars > 0 {
                Status::Success
            } else {
                Status::Failure
            }
        } else {
            Status::Failure
        }
    }
}

fn choose_a_character(name: &'static str, state: Rc<RefCell<XuxState>>) -> ChooseACharacter {
    ChooseACharacter::new(name, state)
}

struct ChooseACharacter {
    name: &'static str,
    state: Rc<RefCell<XuxState>>,
    play_message_sent: bool,
}

impl ChooseACharacter {
    fn new (name: &'static str, state: Rc<RefCell<XuxState>>) -> ChooseACharacter {
        ChooseACharacter { name, state, play_message_sent: false }
    }
}

impl Node for ChooseACharacter {
    fn tick(&mut self) -> Status {
        if self.play_message_sent {
            return Status::Success;
        }
        let state = self.state.borrow();
        let charlist = state.widgets.find_chain(&["ccnt","charlist"]);
        let charlist = match charlist {
            Some(ref charlist) => charlist,
            None => return Status::Failure,
        };
        for (_,args) in charlist.messages.iter().filter(|&&(ref name,_)|name == "add") {
            if let Some(&List::Str(ref name)) = args.get(0) {
                if name == self.name {
                    //TODO send "focus" message
                    self.state.borrow().event_tx.send(xux::driver::Event::User(xux::driver::UserInput::Message(charlist.id, "play".into(), [List::Str(name.clone())].to_vec()))).expect("unable to send User::Message");
                    self.play_message_sent = true;
                    return Status::Success;
                }
            }
        }
        Status::Failure
    }
}

fn wait_gameui(state: Rc<RefCell<XuxState>>) -> WaitWidgetChain {
    WaitWidgetChain::new(state, vec!("gameui"))
}

fn create_a_new_character () -> AlwaysFailure {
    AlwaysFailure
}

// fell-trees
//      avoid-hostiles
//      &
//      restore-stamina
//      &
//      cut-down-nearest-tree
fn fell_trees () -> boxed::Sequence<1> {
    boxed::Sequence::new([
        Box::new(AlwaysRunning)
    ])
}

// avoid-hostiles
//      # ??? should we avoid nearest of predators AND players simultaneously?
//      avoid-predators
//      &
//      avoid-players
//
// avoid-predators
//      cant-see-any-predators | avoid-nearest-predator
//
// cant-see-any-predators
//      ...
//
// avoid-nearest-predator
//      # choose max available speed
//      # at the same time drink water if any in inventory
//      ...
//
// avoid-players
//      cant-see-any-players | avoid-nearest-player
//
// cant-see-any-players
//      ...
//
// avoid-nearest-player
//      # choose max available speed
//      # at the same time drink water if any in inventory
//      ...
//
// restore-stamina
//      dont-need-to-drink | drink
//
// dont-need-to-drink
//      if is_consuming {
//          if stamina > min_threshold {
//              SUCCESS
//          } else {
//              is_consuming = false
//              FAILURE
//          }
//      } else {
//          if stamina < max_threshold {
//              FAILURE
//          } else {
//              is_consuming = true
//              SUCCESS
//          }
//      }
//
// drink
//      have-cup-in-inventory | create-a-cup
//      &
//      have-a-cup-of-press-water | fill-a-cup-with-press-water
//      &
//      drink-from-a-cup
//
// cut-down-nearest-tree
//      have-an-axe-in-a-hand | take-an-axe-in-a-hand
//      &
//      goto-nearest-tree
//      &
//      chop-nearest-tree
//
// have-an-axe-in-a-hand
//      ...
//
// take-an-axe-in-a-hand
//      have-an-axe-in-inventory | create-an-axe
//      &
//      put-axe-from-inventory-to-hand
//
// have-an-axe-in-inventory
//      ...
//
// create-an-axe
//      ...
//
// put-axe-from-inventory-to-hand
//      ...
