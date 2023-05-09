use reversible_list::ReversibleList;

fn main() {
    let mut list: ReversibleList<u32> = ReversibleList::new();
    dbg!(&list);

    list.push_back(47);
    list.push_back(69);
    list.push_back(3);

    dbg!(&list);
}
