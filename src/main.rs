use reversible_list::ReversibleList;

fn main() {
    let mut list: ReversibleList<u32> = ReversibleList::new();
    dbg!(&list);

    list.push_back(47);
    dbg!(list);
}
