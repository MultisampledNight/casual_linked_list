use crate::ReversibleList;

#[test]
fn casual_push_and_observe() {
    let mut list = ReversibleList::new();

    list.push_back("owo");
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&"owo"]);

    list.push_front("uwu");
    list.push_front("kwk");
    list.push_back("xwx");
    list.push_front("-w-");
    list.push_back("qwq");

    assert_eq!(
        list.iter().collect::<Vec<_>>(),
        vec![&"-w-", &"kwk", &"uwu", &"owo", &"xwx", &"qwq"]
    );
    assert_eq!(
        list.iter().rev().collect::<Vec<_>>(),
        vec![&"qwq", &"xwx", &"owo", &"uwu", &"kwk", &"-w-"]
    );
}
