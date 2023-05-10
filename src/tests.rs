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

    assert_eq!(snake.undistorted_iter().sum::<i32>(), snake.undistorted_iter().rev().sum());
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
