use std::cell::Cell;
use std::cell::RefCell;

use super::*;

struct Publisher<'a> {
    events: &'a RefCell<Vec<&'static str>>,
    fail: bool,
}

impl FixedPointStage1Publisher for Publisher<'_> {
    fn publish_stage1(&self) -> Result<(), RunError> {
        self.events.borrow_mut().push("publish");
        if self.fail {
            return Err(internal("checkpoint publication failed".to_string()));
        }
        Ok(())
    }
}

#[test]
fn stage1_publication_finishes_before_stage2_continues() {
    let events = RefCell::new(Vec::new());
    let publisher = Publisher {
        events: &events,
        fail: false,
    };
    let outcome = publish_stage1_before_continuation(Some(&publisher), || {
        events.borrow_mut().push("stage2");
        Ok(())
    });
    assert!(outcome.is_ok());
    assert_eq!(*events.borrow(), vec!["publish", "stage2"]);
}

#[test]
fn failed_stage1_publication_prevents_stage2_effects() {
    let events = RefCell::new(Vec::new());
    let continued = Cell::new(false);
    let publisher = Publisher {
        events: &events,
        fail: true,
    };
    let outcome = publish_stage1_before_continuation(Some(&publisher), || {
        continued.set(true);
        Ok(())
    });
    assert!(outcome.is_err());
    assert!(!continued.get());
    assert_eq!(*events.borrow(), vec!["publish"]);
}

#[test]
fn absent_publisher_keeps_ordinary_continuation() {
    let continued = Cell::new(false);
    let outcome = publish_stage1_before_continuation(None, || {
        continued.set(true);
        Ok(())
    });
    assert!(outcome.is_ok());
    assert!(continued.get());
}
