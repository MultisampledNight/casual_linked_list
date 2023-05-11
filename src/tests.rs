use crate::ReversibleList;

#[test]
fn casual_push_and_observe() {
    let mut list = ReversibleList::new();

    list.push_back("owo");
    assert_eq!(list.undistorted_iter().collect::<Vec<_>>(), vec![&"owo"]);

    list.push_front("uwu");
    list.push_front("kwk");
    list.push_back("xwx");
    list.push_front("-w-");
    list.push_back("qwq");

    assert_eq!(
        list.undistorted_iter().collect::<Vec<_>>(),
        vec![&"-w-", &"kwk", &"uwu", &"owo", &"xwx", &"qwq"]
    );
    assert_eq!(
        list.undistorted_iter().rev().collect::<Vec<_>>(),
        vec![&"qwq", &"xwx", &"owo", &"uwu", &"kwk", &"-w-"]
    );
}

#[test]
fn snake_and_reverse() {
    let mut snake = ReversibleList::new();

    snake.push_back(10);
    snake.push_front(-45);
    snake.push_front(-7);
    snake.push_back(1_000_000);
    snake.push_back(10);
    snake.push_front(-30);

    // nom
    assert_eq!(snake.pop_back(), Some(10));
    assert_eq!(snake.pop_front(), Some(-30));
    assert_eq!(snake.pop_front(), Some(-7));

    snake.push_front(1);

    assert_eq!(
        snake.undistorted_iter().sum::<i32>(),
        snake.undistorted_iter().rev().sum()
    );
    assert_eq!(
        snake.undistorted_iter().copied().collect::<Vec<_>>(),
        vec![1, -45, 10, 1_000_000]
    );

    // trying to pop an already empty list should not panic
    for _ in 0..10 {
        snake.pop_front();
        snake.pop_back();
    }
    assert_eq!(snake.pop_front(), None);
    assert_eq!(snake.pop_back(), None);
}

#[test]
fn curious_cursors() {
    let mut list = ReversibleList::new();
    list.push_back("rainbow-striped button");
    list.push_back("wall");
    list.push_back("the light switch");
    list.push_back("a few doors producing music");
    list.push_back("hyperbolic pillow");

    // then let's take a look around the room
    let mut player = list.undistorted_cursor_front();
    assert_eq!(player.current(), Some(&"rainbow-striped button"));
    player.move_next();
    assert_eq!(player.current(), Some(&"wall"));
    player.move_prev();
    assert_eq!(player.current(), Some(&"rainbow-striped button"));
    player.move_prev();
    assert_eq!(player.current(), Some(&"hyperbolic pillow"));
    player.move_prev();
    assert_eq!(player.current(), Some(&"a few doors producing music"));

    // and a bit faster
    player.move_next_n(4);
    assert_eq!(player.current(), Some(&"the light switch"));
    player.move_next_n(0);
    assert_eq!(player.current(), Some(&"the light switch"));
    player.move_prev_n(0);
    player.move_next_n(1);
    assert_eq!(player.current(), Some(&"a few doors producing music"));
    player.move_prev_n(4);
    assert_eq!(player.current(), Some(&"hyperbolic pillow"));

    // ok so how about moving to a specific position
    player.move_to(0);
    assert_eq!(player.current(), Some(&"rainbow-striped button"));
    player.move_to(4);
    assert_eq!(player.current(), Some(&"hyperbolic pillow"));
    player.move_to(2);
    assert_eq!(player.current(), Some(&"the light switch"));
    player.move_to(3);
    assert_eq!(player.current(), Some(&"a few doors producing music"));
    player.move_to(3);
    assert_eq!(player.current(), Some(&"a few doors producing music"));

    // now that we've looked around enough, let's modify
    let mut player = list.undistorted_cursor_front_mut();
    player.move_to(3);
    assert_eq!(player.remove_current(), Some("a few doors producing music")); // no more doors :(
    assert_eq!(player.remove_current(), Some("hyperbolic pillow"));
    player.insert_after("portable table");
    assert_eq!(player.remove_current(), Some("the light switch"));
    assert_eq!(player.remove_current(), Some("portable table"));

    player.insert_before("cookies");
    player.move_prev();
    assert_eq!(player.remove_current(), Some("cookies"));
    assert_eq!(player.index(), Some(1));

    player.remove_current().unwrap();
    player.remove_current().unwrap();
    assert_eq!(player.remove_current(), None);
    assert_eq!(player.index(), None);
}
