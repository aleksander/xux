//
// login & fell-trees
//
// wait-gameui
//     ...
//
// create-a-new-character
//      ...

use behavior_tree::{Node, Status, AlwaysRunning, boxed};

fn root () -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(login()),
        Box::new(fell_trees())
    ])
}

// login
//     wait-login-screen
//     &
//     login-existing-character | create-a-new-character
//
fn login () -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(wait_login_screen()),
        Box::new(boxed::Selector::new([
            Box::new(login_existing_character()),
            Box::new(create_a_new_character()),
        ]))
    ])
}

// wait-login-screen
//     wait-widget-1
//     &
//     wait-widget-2
//     &
//     wait-widget-3
//     &
//     wait-a-second
//
fn wait_login_screen () -> boxed::Sequence<2> {
    boxed::Sequence::new([
        Box::new(wait_widget_chain(&["ccnt","charlist"])),
        Box::new(wait_millis(300)),
    ])
}

// login-existing-character
//      have-any-characters
//      &
//      choose-a-character
//      &
//      wait-gameui
//
fn login_existing_character () -> boxed::Sequence<3> {
    Box::new(boxed::Sequence::new([
        Box::new(have_any_characters()),
        Box::new(choose_a_character()),
        Box::new(wait_gameui()),
    ]))
}

// choose-a-character {
//     send msg::focus
//     send msg::play("name")
// }
//
fn have_any_characters() -> {}
fn choose_a_character() -> {}
fn wait_gameui() -> {}

fn create_a_new_character () -> AlwaysFailed {
    AlwaysFailed
}

// fell-trees
//      avoid-hostiles
//      &
//      restore-stamina
//      &
//      cut-down-nearest-tree
fn fell_trees () -> boxed::Sequence {
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
