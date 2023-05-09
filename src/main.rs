use reversible_list::ReversibleList;

fn main() {
    let mut list: ReversibleList<u32> = ReversibleList::new();
    dbg!(&list);

    list.push_back(47);
    list.push_back(1000);
    list.push_front(4);
    list.push_back(10);
    list.push_front(84);

    dbg!(&list);
}
